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
        if self.data.is_empty() {
            // We have reached the EOF
            self.index = EOF_INDEX;
            Err(ReadError::EOF)
        } else if n > self.data.len() {
            // EOF but we haven't reached it yet
            Err(ReadError::EOF)
        } else {
            // We have enough data
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
        self.data.chars().next().ok_or_else(|| {
            self.index = EOF_INDEX;
            ReadError::EOF
        })
    }

    #[inline(always)]
    fn trait_obj(&mut self) -> &mut dyn Input {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{core::trim::TrimWhitespace, trim::Trim};

    use super::*;

    #[test]
    pub fn whitespace() {
        let mut view = StrView::new(" \n ");

        assert_eq!(TrimWhitespace.trim(&mut view), Err(ReadError::EOF));
    }

    #[test]
    pub fn single_whitespace() {
        let mut view = StrView::new("\n");

        assert_eq!(TrimWhitespace.trim(&mut view), Err(ReadError::EOF));
    }

    #[test]
    pub fn empty() {
        let mut view = StrView::new("");

        assert_eq!(TrimWhitespace.trim(&mut view), Err(ReadError::EOF));
    }
}
