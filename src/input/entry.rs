use super::{Input, Result};

/// Removes the need for unsafe code by using an entry system
pub trait InputExt: Input {
    #[inline(always)]
    fn peek_entry<'a>(&'a mut self) -> Result<CharEntry<'a, Self>> {
        self.peek().map(|c| CharEntry {
            input: self,
            character: c,
        })
    }

    #[inline(always)]
    fn match_str_entry<'a>(&'a mut self, other: &str) -> Result<Option<Entry<'a, Self>>> {
        let slice = self.read_at_least(other.len())?;

        Ok(if slice.starts_with(other) {
            Some(Entry {
                input: self,
                size: other.len(),
            })
        } else {
            None
        })
    }

    #[inline(always)]
    fn read_until_entry<'a>(
        &'a mut self,
        chunk_size: usize,
        func: impl Fn(char) -> bool,
    ) -> Result<Entry<'a, Self>> {
        let len = self.read_until(chunk_size, func)?.len();

        Ok(Entry {
            input: self,
            size: len,
        })
    }
}
impl<I: ?Sized + Input> InputExt for I {}

pub struct CharEntry<'a, I: ?Sized + Input> {
    input: &'a mut I,
    character: char,
}

impl<'a, I: ?Sized + Input> CharEntry<'a, I> {
    #[inline(always)]
    pub const fn input(&self) -> &I {
        self.input
    }

    #[inline(always)]
    pub const fn get(&mut self) -> char {
        self.character
    }

    #[inline(always)]
    pub fn consume(self) {
        unsafe { self.input.consume(self.character.len_utf8()) };
    }

    #[inline(always)]
    pub const fn discard(self) {}
}

pub struct Entry<'a, I: ?Sized + Input> {
    input: &'a mut I,
    size: usize,
}

impl<'a, I: ?Sized + Input> Entry<'a, I> {
    #[inline(always)]
    pub const fn input(&self) -> &I {
        self.input
    }

    #[inline(always)]
    pub fn get(&mut self) -> &str {
        unsafe { str::from_utf8_unchecked(self.input.get_unchecked(self.size)) }
    }

    #[inline(always)]
    pub fn consume(self) {
        unsafe { self.input.consume(self.size) };
    }

    #[inline(always)]
    pub const fn discard(self) {}
}
