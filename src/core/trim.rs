use crate::{
    input::{self, Input, InputExt},
    trim::Trim,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrimUntil<F: Fn(char) -> bool> {
    func: F,
}

impl<F: Fn(char) -> bool> TrimUntil<F> {
    #[inline(always)]
    pub const fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F: Fn(char) -> bool> Trim for TrimUntil<F> {
    #[inline(always)]
    fn trim<I: ?Sized + Input>(self, input: &mut I) -> input::Result<()> {
        input.consume_until(8, self.func)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrimWhitespace;
impl Trim for TrimWhitespace {
    #[inline(always)]
    fn trim<I: ?Sized + Input>(self, input: &mut I) -> input::Result<()> {
        input.consume_until(8, |c| !c.is_whitespace())
    }
}
