use crate::{
    input::Input,
    parse::{IsParse, ParseError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplitUpTo<F: Fn(char) -> bool> {
    func: F,
}

impl<F: Fn(char) -> bool> SplitUpTo<F> {
    #[inline(always)]
    pub const fn new(func: F) -> Self {
        Self { func }
    }
}

impl<'a, F: Fn(char) -> bool> IsParse<'a> for SplitUpTo<F> {
    type Output = &'a str;
    type Error = !;

    fn __parse<I: Input>(self, input: &'a mut I) -> Result<Self::Output, ParseError<Self::Error>> {
        input
            .read_until(8, self.func)
            .map_err(ParseError::ReadError)
    }
}
