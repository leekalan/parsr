use super::{EOF_INDEX, Input, ReadError, Result};

pub struct StrView<'a> {
    data: &'a str,
    index: usize,
}

impl<'a> StrView<'a> {
    #[inline(always)]
    pub const fn new(data: &'a str) -> Self {
        StrView { data, index: 0 }
    }
}

impl<'a> Input for StrView<'a> {
    #[inline(always)]
    unsafe fn get_unchecked(&self, n: usize) -> &[u8] {
        unsafe { self.data.as_bytes().get_unchecked(..n) }
    }

    #[inline(always)]
    fn index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    fn read(&self) -> &str {
        self.data
    }

    #[inline(always)]
    fn buffer_at_least(&mut self, n: usize) -> Result<()> {
        if n > self.data.len() {
            Err(ReadError::EOF)
        } else {
            Ok(())
        }
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
    unsafe fn consume(&mut self, n: usize) {
        self.data = unsafe { self.data.get_unchecked(n..) };
        self.index += n;
    }

    #[inline(always)]
    fn peek(&mut self) -> Result<char> {
        self.data.chars().next().ok_or(ReadError::EOF)
    }

    #[inline(always)]
    fn trait_obj(&mut self) -> &mut dyn Input {
        self
    }
}
