#[macro_use] extern crate nom;

mod parse;
pub mod chrono;

pub use parse::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Date {
    YMD(YmdDate),
    Week(WeekDate),
    Ordinal(OrdinalDate)
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct YmdDate {
    year: i16,
    month: u8,
    day: u8
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct WeekDate {
    year: i16,
    week: u8,
    day: u8
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct OrdinalDate {
    year: i16,
    day: u16
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
