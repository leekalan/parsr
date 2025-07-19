#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    #[inline(always)]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub const fn from_self_to_other(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }

    #[inline(always)]
    pub const fn over<T>(self, inner: T) -> Spanned<T> {
        Spanned { inner, span: self }
    }
}

impl Default for Span {
    #[inline(always)]
    fn default() -> Self {
        Self { start: 0, end: 0 }
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub const fn default_span(inner: T) -> Self {
        Self {
            inner,
            span: Span { start: 0, end: 0 },
        }
    }

    #[inline(always)]
    pub const fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }
}
