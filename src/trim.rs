use crate::input::{self, Input};

pub trait Trim {
    fn trim<I: ?Sized + Input>(self, input: &mut I) -> input::Result<()>;
}

impl Trim for () {
    fn trim<I: ?Sized + Input>(self, _input: &mut I) -> input::Result<()> {
        Ok(())
    }
}
