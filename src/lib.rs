#![feature(never_type)]

pub mod input;
pub mod interner;

pub mod parse;
pub mod trim;

pub mod core;

pub use token_precedence as token;
