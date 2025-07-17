use crate::input::{Input, ReadError};

mod parse_iter;
mod parse_mut_iter;
pub use parse_iter::{ParseIter, ParseIterError};
pub use parse_mut_iter::ParseMutIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseError<E> {
    ReadError(ReadError),
    Error(E),
}

impl<E> From<ReadError> for ParseError<E> {
    #[inline(always)]
    fn from(value: ReadError) -> Self {
        ParseError::ReadError(value)
    }
}

impl<E> ParseError<E> {
    #[inline(always)]
    pub const fn new(err: E) -> Self {
        ParseError::Error(err)
    }
}

pub trait IsParse<'a> {
    type Output;
    type Error;

    fn __parse<I: ?Sized + Input>(
        self,
        input: &'a mut I,
    ) -> Result<Self::Output, ParseError<Self::Error>>;
}

pub trait Parse: for<'a> IsParse<'a> + Sized {
    #[inline(always)]
    fn parse<'a, I: ?Sized + Input>(
        self,
        input: &'a mut I,
    ) -> Result<ParseOutput<'a, Self>, ParseError<ParseErrorOutput<'a, Self>>> {
        self.__parse(input)
    }
}
impl<P: for<'a> IsParse<'a>> Parse for P {}

pub type ParseOutput<'a, P> = <P as IsParse<'a>>::Output;
pub type ParseErrorOutput<'a, P> = <P as IsParse<'a>>::Error;

pub trait ParseExt: Parse {
    #[inline(always)]
    fn mapped<M: Mapping<Self>>(self, mapping: M) -> MappedParse<Self, M> {
        MappedParse {
            parse: self,
            mapping,
        }
    }

    #[inline(always)]
    fn mapped_mut<M: MappingMut<Self>>(self, mapping: M) -> MappedMutParse<Self, M> {
        MappedMutParse {
            parse: self,
            mapping,
        }
    }
}
impl<P: Parse> ParseExt for P {}

pub trait IsMapping<'a, P: IsParse<'a>> {
    type Output;

    fn __map(self, output: ParseOutput<'a, P>) -> Self::Output;
}

impl<'a, P: IsParse<'a>, F: FnOnce(ParseOutput<'a, P>) -> O, O> IsMapping<'a, P> for F {
    type Output = O;

    #[inline(always)]
    fn __map(self, output: ParseOutput<'a, P>) -> Self::Output {
        self(output)
    }
}

pub trait Mapping<P: Parse>: for<'a> IsMapping<'a, P> + Sized {
    #[inline(always)]
    fn map<'a>(self, output: ParseOutput<'a, P>) -> MappingOutput<'a, P, Self> {
        self.__map(output)
    }
}
impl<P: Parse, M: for<'a> IsMapping<'a, P>> Mapping<P> for M {}

pub type MappingOutput<'a, P, M> = <M as IsMapping<'a, P>>::Output;

#[derive(Debug, Clone, Copy)]
pub struct MappedParse<P: Parse, M: Mapping<P>> {
    pub parse: P,
    pub mapping: M,
}

impl<'a, P: Parse, M: Mapping<P>> IsParse<'a> for MappedParse<P, M> {
    type Output = MappingOutput<'a, P, M>;
    type Error = ParseErrorOutput<'a, P>;

    #[inline(always)]
    fn __parse<I: ?Sized + Input>(
        self,
        input: &'a mut I,
    ) -> Result<Self::Output, ParseError<Self::Error>> {
        self.parse
            .parse(input)
            .map(|output| self.mapping.map(output))
    }
}

pub trait IsMappingMut<'a, P: IsParse<'a>> {
    type Output;

    fn __map_mut(&mut self, output: ParseOutput<'a, P>) -> Self::Output;
}

impl<'a, P: IsParse<'a>, F: FnMut(ParseOutput<'a, P>) -> O, O> IsMappingMut<'a, P> for F {
    type Output = O;

    #[inline(always)]
    fn __map_mut(&mut self, output: ParseOutput<'a, P>) -> Self::Output {
        self(output)
    }
}

pub trait MappingMut<P: Parse>: for<'a> IsMappingMut<'a, P> + Sized {
    #[inline(always)]
    fn map_mut<'a>(&mut self, output: ParseOutput<'a, P>) -> MappingMutOutput<'a, P, Self> {
        self.__map_mut(output)
    }
}
impl<P: Parse, M: for<'a> IsMappingMut<'a, P>> MappingMut<P> for M {}

pub type MappingMutOutput<'a, P, M> = <M as IsMappingMut<'a, P>>::Output;

#[derive(Debug)]
pub struct MappedMutParse<P: Parse, M: MappingMut<P>> {
    pub parse: P,
    pub mapping: M,
}

impl<'a, P: Parse, M: MappingMut<P>> IsParse<'a> for MappedMutParse<P, M> {
    type Output = MappingMutOutput<'a, P, M>;
    type Error = ParseErrorOutput<'a, P>;

    #[inline(always)]
    fn __parse<I: ?Sized + Input>(
        mut self,
        input: &'a mut I,
    ) -> Result<Self::Output, ParseError<Self::Error>> {
        self.parse
            .parse(input)
            .map(|output| self.mapping.map_mut(output))
    }
}

impl<'a, P: Parse + Clone, M: MappingMut<P>> IsParse<'a> for &mut MappedMutParse<P, M> {
    type Output = MappingMutOutput<'a, P, M>;
    type Error = ParseErrorOutput<'a, P>;

    #[inline(always)]
    fn __parse<I: ?Sized + Input>(
        self,
        input: &'a mut I,
    ) -> Result<Self::Output, ParseError<Self::Error>> {
        self.parse
            .clone()
            .parse(input)
            .map(|output| self.mapping.map_mut(output))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{parse::SplitUpTo, trim::TrimWhitespace},
        input::{Entry, Input},
        parse::{ParseExt, ParseIter, ParseMutIter},
    };

    #[allow(unused)]
    fn test_iter_typing<I: ?Sized + Input>(input: &mut I) {
        let mapped = SplitUpTo::new(|c| !char::is_whitespace(c)).mapped(|mut entry: Entry| {
            let ret = entry.get().to_string();
            entry.consume();
            ret
        });

        for i in ParseIter::new(input, TrimWhitespace, mapped).unwrap() {
            println!("{}", i.unwrap());
        }
    }

    #[allow(unused)]
    fn test_iter_mut_typing<I: ?Sized + Input>(input: &mut I) {
        let mut total_len = 0u32;

        let parser = SplitUpTo::new(|c| !char::is_whitespace(c));

        let mut mapped = parser.mapped_mut(|mut entry: Entry| {
            total_len += entry.get().len() as u32;
            let ret = entry.get().contains('!');
            entry.consume();
            ret
        });

        for i in ParseMutIter::new(input, TrimWhitespace, &mut mapped).unwrap() {
            println!("{}", i.unwrap());
        }
    }
}
