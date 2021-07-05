// https://github.com/rust-lang/cargo/issues/383#issuecomment-720873790
#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern {}
        }
    }

    external_doc_test!(include_str!("../README.md"));
}

#[macro_use] extern crate nom;

macro_rules! impl_fromstr_parse {
    ($ty:ty, $func:ident) => {
        impl ::std::str::FromStr for $ty {
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
