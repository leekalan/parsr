use std::result;

mod reader_input;
mod str_view;

pub use reader_input::ReaderInput;
pub use str_view::StrView;

const EOF_INDEX: usize = usize::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadError {
    EOF,
    InvalidUtf8(InvalidUtf8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidUtf8 {
    pub index: usize,
}

pub type Result<T> = result::Result<T, ReadError>;

pub trait Input {
    /// # Safety
    /// n must not exceed the length of currently buffered data
    /// (this is more default trait function and should not be used)
    unsafe fn get_unchecked(&self, n: usize) -> &[u8];
    fn index(&self) -> usize;
    fn read(&self) -> &str;
    /// Will buffer at least n - 3 bytes of data depending on char boundaries
    fn buffer_at_least(&mut self, n: usize) -> Result<()>;
    fn set_eof(&mut self);
    fn is_eof(&self) -> bool;
    /// # Safety
    /// n must not exceed the length of currently buffered data
    /// and must offset the input to an existing char boundary
    unsafe fn consume(&mut self, n: usize);

    /// Will read at least n - 3 bytes of data depending on char boundaries
    #[inline(always)]
    fn read_at_least(&mut self, n: usize) -> Result<&str> {
        self.buffer_at_least(n).map(|_| self.read())
    }

    fn consume_until(&mut self, chunk_size: usize, func: impl Fn(char) -> bool) -> Result<()> {
        loop {
            if self.is_eof() {
                return Err(ReadError::EOF);
            }

            if let Result::Err(ReadError::InvalidUtf8(err)) = self.buffer_at_least(chunk_size) {
                return Err(ReadError::InvalidUtf8(err));
            };

            let read = self.read();

            for (i, c) in read.char_indices() {
                if func(c) {
                    unsafe { self.consume(i) };
                    return Ok(());
                }
            }

            unsafe { self.consume(read.len()) };
        }
    }

    fn read_until(&mut self, chunk_size: usize, func: impl Fn(char) -> bool) -> Result<&str> {
        let mut count = 0;

        loop {
            if self.is_eof() {
                return Err(ReadError::EOF);
            }

            if let Result::Err(ReadError::InvalidUtf8(err)) = self.buffer_at_least(chunk_size) {
                return Err(ReadError::InvalidUtf8(err));
            };

            let read = self.read();

            for (i, c) in read.char_indices() {
                if func(c) {
                    let s = unsafe { str::from_utf8_unchecked(self.get_unchecked(count + i)) };

                    return Ok(s);
                }
            }

            count += read.len();
        }
    }

    #[inline(always)]
    fn peek(&mut self) -> Result<char> {
        self.read_at_least(4)
            .map(|s| unsafe { s.chars().next().unwrap_unchecked() })
    }
}

#[cfg(test)]
pub mod tests {
    use std::{cmp, io, ptr};

    use super::*;

    struct ReadOneAtATime<'a> {
        data: &'a [u8],
        index: usize,
    }

    impl<'a> ReadOneAtATime<'a> {
        fn new(data: &'a [u8]) -> Self {
            Self { data, index: 0 }
        }
    }

    impl<'a> io::Read for ReadOneAtATime<'a> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.index >= self.data.len() {
                Ok(0)
            } else {
                buf[0] = self.data[self.index];
                self.index += 1;
                Ok(1)
            }
        }
    }

    #[test]
    fn simple_test() {
        let data = ReadOneAtATime::new("hello world!".as_bytes());

        let mut input = ReaderInput::<_, 128>::new(data);

        assert_eq!(input.read_at_least(5), Ok("hello"));

        assert_eq!(input.index, 0);
        assert_eq!(input.cursor, 0);
        assert_eq!(input.char_boundary, 5);
        assert_eq!(input.filled, 5);

        unsafe { input.consume("hello".len()) };

        assert_eq!(input.index, 5);
        assert_eq!(input.cursor, 5);
        assert_eq!(input.char_boundary, 5);
        assert_eq!(input.filled, 5);

        assert_eq!(input.read_at_least(6), Ok(" world"));

        assert_eq!(input.index, 5);
        assert_eq!(input.cursor, 5);
        assert_eq!(input.char_boundary, 11);
        assert_eq!(input.filled, 11);

        unsafe { input.consume(" ".len()) };

        assert_eq!(input.read_at_least(6), Ok("world!"));

        assert_eq!(input.read_at_least(1), Ok("world!"));
    }

    #[test]
    fn deal_with_utf8() {
        let data = ReadOneAtATime::new("party 🎉 🎉!".as_bytes());

        let mut input = ReaderInput::<_, 128>::new(data);

        assert_eq!(input.read_at_least(5), Ok("party"));
        unsafe { input.consume("party".len()) };

        assert_eq!(input.read_at_least(3), Ok(" "));

        assert_eq!(input.index, 5);
        assert_eq!(input.cursor, 5);
        assert_eq!(input.char_boundary, 6);
        assert_eq!(input.filled, 8);

        unsafe { input.consume(" ".len()) };

        assert_eq!(input.index, 6);
        assert_eq!(input.cursor, 6);
        assert_eq!(input.char_boundary, 6);
        assert_eq!(input.filled, 8);

        assert_eq!(input.read_at_least(4), Ok("🎉"));

        assert_eq!(input.index, 6);
        assert_eq!(input.cursor, 6);
        assert_eq!(input.char_boundary, 10);
        assert_eq!(input.filled, 10);

        unsafe { input.consume("🎉".len()) };

        assert_eq!(input.index, 10);
        assert_eq!(input.cursor, 10);
        assert_eq!(input.char_boundary, 10);
        assert_eq!(input.filled, 10);

        assert_eq!(input.read_at_least(5), Ok(" 🎉"));
        unsafe { input.consume(" ".len()) };

        assert_eq!(input.read_at_least(5), Ok("🎉!"));
    }

    struct ReadEightAtATime<'a> {
        data: &'a [u8],
        index: usize,
    }

    impl<'a> ReadEightAtATime<'a> {
        fn new(data: &'a [u8]) -> Self {
            Self { data, index: 0 }
        }
    }

    impl<'a> io::Read for ReadEightAtATime<'a> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.index >= self.data.len() {
                Ok(0)
            } else {
                let size = cmp::min(buf.len(), cmp::min(self.data.len() - self.index, 8));

                unsafe { ptr::copy(&self.data[self.index], buf.as_mut_ptr(), size) };
                self.index += size;
                Ok(size)
            }
        }
    }

    #[test]
    fn deal_with_wrapping() {
        let data = ReadEightAtATime::new("hello! \nworld!\n".as_bytes());

        let mut input = ReaderInput::<_, 12>::new(data);

        assert_eq!(input.read_at_least(8), Ok("hello! \n"));
        unsafe { input.consume("hello! \n".len()) };

        assert_eq!(input.read_at_least(4), Ok("worl"));

        assert_eq!(input.index, 8);
        assert_eq!(input.cursor, 8);
        assert_eq!(input.char_boundary, 12);
        assert_eq!(input.filled, 12);

        assert_eq!(input.read_at_least(5), Ok("world!\n"));

        assert_eq!(input.index, 8);
        assert_eq!(input.cursor, 0);
        assert_eq!(input.char_boundary, 7);
        assert_eq!(input.filled, 7);

        assert_eq!(input.read_at_least(1), Ok("world!\n"));

        unsafe { input.consume("world!".len()) };

        assert_eq!(input.index, 14);
        assert_eq!(input.cursor, 6);
        assert_eq!(input.char_boundary, 7);
        assert_eq!(input.filled, 7);

        assert_eq!(input.read_at_least(1), Ok("\n"));
    }
}
