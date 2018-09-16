mod parse;
pub mod chrono;

#[macro_use] extern crate nom;

pub use parse::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Date {
    YMD {
        year: i16,
        month: u8,
        day: u8
    },
    Week {
        year: i16,
        week: u8,
        day: u8
    },
    Ordinal {
        year: i16,
        day: u16
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32,
    /// minutes
    pub tz_offset: i16
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct DateTime {
    pub date: Date,
    pub time: Time
}
