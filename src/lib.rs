#[macro_use] extern crate nom;
extern crate regex;

mod date;
mod time;
mod datetime;
mod parse;
pub mod chrono;

pub use {
    date::*,
    time::*,
    datetime::*
};

pub trait Valid {
    fn is_valid(&self) -> bool;
}
