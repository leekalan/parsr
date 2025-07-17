use crate::{
    input::{Input, InvalidUtf8, ReadError},
    parse::{IsParse, Parse, ParseError, ParseIterError},
    trim::Trim,
};

#[derive(Debug)]
pub struct ParseMutBorrowedIter<'a, 'p, I: ?Sized + Input, T: Trim, P>
where
    for<'s> &'s mut P: Parse,
{
    input: &'a mut I,
    trimmer: T,
    parser: &'p mut P,
}

impl<'a, 'p, I: ?Sized + Input, T: Trim + Clone, P> ParseMutBorrowedIter<'a, 'p, I, T, P>
where
    for<'s> &'s mut P: Parse,
{
    #[inline(always)]
    pub fn new(input: &'a mut I, trimmer: T, parser: &'p mut P) -> Result<Self, InvalidUtf8> {
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

impl<'a, 'p, I: ?Sized + Input, T: Trim + Clone, P, O, E> Iterator
    for ParseMutBorrowedIter<'a, 'p, I, T, P>
where
    for<'s, 'k> &'s mut P: IsParse<'k, Output = O, Error = E>,
{
    type Item = Result<O, ParseIterError<E>>;

    fn next(&mut self) -> Option<Result<O, ParseIterError<E>>> {
        if self.input.is_eof() {
            return None;
        }

        let output = match self.parser.parse(self.input) {
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
