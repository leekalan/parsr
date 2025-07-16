use crate::{
    input::{Input, InvalidUtf8, ReadError},
    parse::{IsParse, Parse, ParseError},
    trim::Trim,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseIterError<E> {
    InvalidUtf8(InvalidUtf8),
    Error(E),
}

impl<E> From<InvalidUtf8> for ParseIterError<E> {
    #[inline(always)]
    fn from(value: InvalidUtf8) -> Self {
        ParseIterError::InvalidUtf8(value)
    }
}

impl<E> ParseIterError<E> {
    #[inline(always)]
    pub const fn new(err: E) -> Self {
        ParseIterError::Error(err)
    }
}

#[derive(Debug)]
pub struct ParseIter<'a, I: Input, T: Trim, P: Parse> {
    input: &'a mut I,
    trimmer: T,
    parser: P,
}

impl<'a, I: Input, T: Trim + Clone, P: Parse> ParseIter<'a, I, T, P> {
    #[inline(always)]
    pub fn new(input: &'a mut I, trimmer: T, parser: P) -> Result<Self, InvalidUtf8> {
        if let Err(ReadError::InvalidUtf8(err)) = trimmer.clone().trim(input) {
            return Err(err);
        }

        Ok(Self {
            input,
            trimmer,
            parser,
        })
    }
}

impl<'a, I: Input, T: Trim + Clone, P: for<'s> IsParse<'s, Output = O, Error = E> + Clone, O, E>
    Iterator for ParseIter<'a, I, T, P>
{
    type Item = Result<O, ParseIterError<E>>;

    fn next(&mut self) -> Option<Result<O, ParseIterError<E>>> {
        if self.input.is_eof() {
            return None;
        }

        let output = match self.parser.clone().parse(self.input) {
            Ok(output) => output,
            Err(ParseError::ReadError(ReadError::EOF)) => return None,
            Err(ParseError::ReadError(ReadError::InvalidUtf8(err))) => {
                return Some(Err(ParseIterError::InvalidUtf8(err)));
            }
            Err(ParseError::Error(err)) => return Some(Err(ParseIterError::Error(err))),
        };

        if let Err(ReadError::InvalidUtf8(err)) = self.trimmer.clone().trim(self.input) {
            return Some(Err(ParseIterError::InvalidUtf8(err)));
        }

        Some(Ok(output))
    }
}
