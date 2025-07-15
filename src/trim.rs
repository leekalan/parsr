use std::io;

use crate::input::{self, Input};

pub trait Trim {
    fn trim<R: io::Read>(self, input: &mut Input<R>) -> input::Result<()>;
}

impl Trim for () {
    fn trim<R: io::Read>(self, _input: &mut Input<R>) -> input::Result<()> {
        Ok(())
    }
}
