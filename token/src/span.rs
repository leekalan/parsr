#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub fn from_self_to_other(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
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
    pub fn default_span(inner: T) -> Self {
        Self {
            inner,
            span: Span::default(),
        }
    }

    #[inline(always)]
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }
}
