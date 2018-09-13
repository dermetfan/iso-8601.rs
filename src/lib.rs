mod parse;
pub mod chrono;

#[macro_use] extern crate nom;

pub use parse::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32,
    /// seconds
    pub tz_offset: i32
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct DateTime {
    pub date: Date,
    pub time: Time
}
