use super::{Input, ReadError, Result};

#[inline(always)]
pub(super) fn default_read_at_least<I: ?Sized + Input>(input: &mut I, n: usize) -> Result<&str> {
    input.buffer_at_least(n).map(|_| input.read())
}

pub(super) fn default_consume_until<I: ?Sized + Input>(
    input: &mut I,
    chunk_size: usize,
    func: impl Fn(char) -> bool,
) -> Result<()> {
    loop {
        if input.is_eof() {
            return Err(ReadError::EOF);
        }

        if let Result::Err(ReadError::InvalidUtf8(err)) = input.buffer_at_least(chunk_size) {
            return Err(ReadError::InvalidUtf8(err));
        };

        let read = input.read();

        for (i, c) in read.char_indices() {
            if func(c) {
                unsafe { input.consume(i) };
                return Ok(());
            }
        }

        unsafe { input.consume(read.len()) };
    }
}

pub(super) fn default_read_until<I: ?Sized + Input>(
    input: &mut I,
    chunk_size: usize,
    func: impl Fn(char) -> bool,
) -> Result<&str> {
    let mut count = 0;

    loop {
        if input.is_eof() {
            return Err(ReadError::EOF);
        }

        if let Result::Err(ReadError::InvalidUtf8(err)) = input.buffer_at_least(chunk_size) {
            return Err(ReadError::InvalidUtf8(err));
        };

        let read = input.read();

        for (i, c) in read.char_indices() {
            if func(c) {
                let s = unsafe { str::from_utf8_unchecked(input.get_unchecked(count + i)) };

                return Ok(s);
            }
        }

        count += read.len();
    }
}

#[inline(always)]
pub(super) fn default_peek<I: ?Sized + Input>(input: &mut I) -> Result<char> {
    input
        .read_at_least(4)
        .map(|s| unsafe { s.chars().next().unwrap_unchecked() })
}
