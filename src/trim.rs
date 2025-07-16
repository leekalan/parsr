use crate::input::{self, Input};

pub trait Trim {
    fn trim<I: Input>(self, input: &mut I) -> input::Result<()>;
}

impl Trim for () {
    fn trim<I: Input>(self, _input: &mut I) -> input::Result<()> {
        Ok(())
    }
}
