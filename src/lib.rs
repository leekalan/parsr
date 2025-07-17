#![feature(never_type, coerce_unsized)]

pub mod input;
pub mod interner;

pub mod parse;
pub mod trim;

pub mod core;

pub use token_precedence as token;
