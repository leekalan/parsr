use std::{io, ptr, result};

const BUFFER_SIZE: usize = 8192;

const EOF_INDEX: usize = usize::MAX;

#[derive(Debug, Clone)]
pub struct Input<R: io::Read, const N: usize = BUFFER_SIZE> {
    reader: R,
    buffer: Box<[u8; N]>,
    index: usize,
    cursor: usize,
    char_boundary: usize,
    filled: usize,
}

pub type Result<T> = result::Result<T, ReadError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadError {
    EOF,
    InvalidUtf8(InvalidUtf8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidUtf8 {
    pub index: usize,
}

impl<R: io::Read, const N: usize> Input<R, N> {
    #[inline(always)]
    pub fn new(reader: R) -> Self {
        Input {
            reader,
            buffer: Box::new([0; N]),
            index: 0,
            cursor: 0,
            char_boundary: 0,
            filled: 0,
        }
    }

    #[inline(always)]
    pub fn read(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.buffer.get_unchecked(self.cursor..self.char_boundary))
        }
    }

    /// Will read at least n - 3 bytes of data depending on char boundaries
    #[inline(always)]
    pub fn read_at_least(&mut self, n: usize) -> Result<&str> {
        self.buffer_at_least(n).map(|_| self.read())
    }

    /// Will buffer at least n - 3 bytes of data depending on char boundaries
    #[inline]
    pub fn buffer_at_least(&mut self, n: usize) -> Result<()> {
        assert!(n <= N, "buffer overflow");

        if self.index == EOF_INDEX {
            return Err(ReadError::EOF);
        }

        // Filling the buffer if needed
        if self.cursor + n > self.filled {
            // Moving the data past the cursor to the start of the buffer
            if self.cursor + n > N {
                let src = unsafe { self.buffer.as_ptr().add(self.cursor) };
                let dst = self.buffer.as_mut_ptr();
                let len = self.filled - self.cursor;

                unsafe { ptr::copy(src, dst, len) };

                self.char_boundary -= self.cursor;
                self.cursor = 0;
                self.filled = len;
            }

            let mut is_empty = false;

            // Filling the buffer
            while self.cursor + n > self.filled {
                let result = self
                    .reader
                    .read(unsafe { self.buffer.get_unchecked_mut(self.filled..) })
                    .unwrap();

                if result == 0 {
                    is_empty = true;
                    break;
                }

                self.filled += result;
            }

            // Updating the char boundary
            match str::from_utf8(unsafe {
                self.buffer.get_unchecked(self.char_boundary..self.filled)
            }) {
                Ok(_) => {
                    self.char_boundary = self.filled;
                }
                Err(e) => {
                    if e.error_len().is_some() {
                        return Err(ReadError::InvalidUtf8(InvalidUtf8 {
                            index: self.index + e.valid_up_to(),
                        }));
                    }

                    self.char_boundary = self.cursor + e.valid_up_to();
                }
            }

            if is_empty {
                self.index = EOF_INDEX;
                return Err(ReadError::EOF);
            }
        }

        Ok(())
    }

    pub fn consume_until(&mut self, chunk_size: usize, func: impl Fn(char) -> bool) -> Result<()> {
        if self.index == EOF_INDEX {
            return Err(ReadError::EOF);
        }

        loop {
            let is_eof = match self.buffer_at_least(chunk_size) {
                Ok(()) => false,
                Err(ReadError::EOF) => true,
                err => return err,
            };

            let read = self.read();

            for (i, c) in read.char_indices() {
                if func(c) {
                    unsafe { self.consume(i) };
                    return Ok(());
                }
            }

            unsafe { self.consume(read.len()) };

            if is_eof {
                self.index = EOF_INDEX;
                return Err(ReadError::EOF);
            }
        }
    }

    pub fn read_until(&mut self, chunk_size: usize, func: impl Fn(char) -> bool) -> Result<&str> {
        if self.index == EOF_INDEX {
            return Err(ReadError::EOF);
        }

        let mut count = 0;

        loop {
            let is_eof = match self.buffer_at_least(chunk_size) {
                Ok(()) => false,
                Err(ReadError::EOF) => true,
                Err(err) => return Err(err),
            };

            let read = self.read();

            for (i, c) in read.char_indices() {
                if func(c) {
                    let s = unsafe {
                        str::from_utf8_unchecked(
                            self.buffer
                                .get_unchecked(self.cursor..self.cursor + count + i),
                        )
                    };

                    return Ok(s);
                }
            }

            count += read.len();

            if is_eof {
                self.index = EOF_INDEX;
                return Err(ReadError::EOF);
            }
        }
    }

    #[inline(always)]
    pub fn peek(&mut self) -> Result<char> {
        self.read_at_least(4)
            .map(|s| unsafe { s.chars().next().unwrap_unchecked() })
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    #[inline(always)]
    pub const fn char_boundary(&self) -> usize {
        self.char_boundary
    }

    #[inline(always)]
    pub const fn filled(&self) -> usize {
        self.filled
    }

    /// # Safety
    /// `n` must be less than or equal to `self.filled - self.cursor`
    /// (less than the length of the slice returned by `read_at_least`)
    ///
    /// `n` must offset the input to a char boundary
    #[inline(always)]
    pub const unsafe fn consume(&mut self, n: usize) {
        self.index += n;
        self.cursor += n;
    }

    pub const fn is_eof(&self) -> bool {
        self.index == EOF_INDEX
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StrView<'a> {
    data: &'a [u8],
}

impl<'a> StrView<'a> {
    #[inline(always)]
    pub const fn new(data: &'a str) -> Self {
        Self {
            data: data.as_bytes(),
        }
    }

    #[inline(always)]
    pub const fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }

    #[inline(always)]
    pub const fn bytes(&mut self) -> &mut &'a [u8] {
        &mut self.data
    }

    #[inline(always)]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.data
    }
}

#[cfg(test)]
pub mod tests {
    use std::cmp;

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

        let mut input = Input::<_, 128>::new(data);

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

        let mut input = Input::<_, 128>::new(data);

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

        let mut input = Input::<_, 12>::new(data);

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
