mod parse;

#[macro_use] extern crate nom;

#[derive(Eq, PartialEq, Debug)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8
}

#[derive(Eq, PartialEq, Debug)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
    tz_offset: i32
}

#[derive(Eq, PartialEq, Debug)]
pub struct DateTime {
    date: Date,
    time: Time
}
