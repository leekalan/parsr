use std::{io, ptr};

use super::{EOF_INDEX, Input, InvalidUtf8, ReadError, Result};

const BUFFER_SIZE: usize = 8192;

#[derive(Debug, Clone)]
pub struct ReaderInput<R: io::Read, const N: usize = BUFFER_SIZE> {
    // pub(super) for testing
    pub(super) reader: R,
    pub(super) buffer: Box<[u8; N]>,
    pub(super) index: usize,
    pub(super) cursor: usize,
    pub(super) char_boundary: usize,
    pub(super) filled: usize,
}

impl<R: io::Read, const N: usize> ReaderInput<R, N> {
    #[inline(always)]
    pub fn new(reader: R) -> Self {
        ReaderInput {
            reader,
            buffer: Box::new([0; N]),
            index: 0,
            cursor: 0,
            char_boundary: 0,
            filled: 0,
        }
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
}

impl<R: io::Read, const N: usize> Input for ReaderInput<R, N> {
    #[inline(always)]
    unsafe fn get_unchecked(&self, n: usize) -> &[u8] {
        unsafe { self.buffer.get_unchecked(self.cursor..self.cursor + n) }
    }

    #[inline(always)]
    fn index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    fn read(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.buffer.get_unchecked(self.cursor..self.char_boundary))
        }
    }

    fn buffer_at_least(&mut self, n: usize) -> Result<()> {
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

    /// # Safety
    /// `n` must be less than or equal to `self.filled - self.cursor`
    /// (less than the length of the slice returned by `read_at_least`)
    ///
    /// `n` must offset the input to a char boundary
    #[inline(always)]
    unsafe fn consume(&mut self, n: usize) {
        self.index += n;
        self.cursor += n;
    }

    #[inline(always)]
    fn set_eof(&mut self) {
        self.index = EOF_INDEX;
    }

    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.index == EOF_INDEX
    }

    #[inline(always)]
    fn trait_obj(&mut self) -> &mut dyn Input {
        self
    }
}
