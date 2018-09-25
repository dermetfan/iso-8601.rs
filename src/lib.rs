#[macro_use] extern crate nom;
extern crate regex;

macro_rules! impl_fromstr_parse {
    ($ty:ty, $func:ident) => {
        impl FromStr for $ty {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                ::parse::$func(s.as_bytes())
                    .map(|x| x.1)
                    .or(Err(()))
            }
        }
    }
}

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
