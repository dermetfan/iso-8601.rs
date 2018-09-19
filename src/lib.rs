#[macro_use] extern crate nom;

mod parse;
pub mod chrono;

pub use parse::*;

use std::convert::From;
use std::str::FromStr;

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

impl FromStr for Date {
    type Err = (); // XXX

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::date(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for Time {
    type Err = (); // XXX

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::time(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for DateTime {
    type Err = (); // XXX

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::datetime(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

pub trait Year {
    fn is_leap(&self) -> bool;
    fn num_weeks(&self) -> u8;
    fn num_days(&self) -> u16;
}

impl Year for i16 {
    fn is_leap(&self) -> bool {
        let factor = |x| self % x == 0;
        factor(4) && (!factor(100) || factor(400))
    }

    fn num_weeks(&self) -> u8 {
        // https://en.wikipedia.org/wiki/ISO_week_date#Weeks_per_year
        let p = |x| (x + x / 4 - x / 100 + x / 400) % 7;
        if p(*self) == 4 || p(self - 1) == 3 { 53 } else { 52 }
    }

    fn num_days(&self) -> u16 {
        if self.is_leap() { 366 } else { 365 }
    }
}

impl From<Date> for YmdDate {
    fn from(date: Date) -> Self {
        match date {
            Date::YMD    (date) => date,
            Date::Week   (date) => date.into(),
            Date::Ordinal(date) => date.into()
        }
    }
}

impl From<WeekDate> for OrdinalDate {
    fn from(date: WeekDate) -> Self {
        // https://en.wikipedia.org/wiki/ISO_week_date#Calculating_a_date_given_the_year,_week_number_and_weekday

        fn weekday_jan4(year: i16) -> u8 {
            fn weekday_jan1(year: i16) -> u8 {
                // https://en.wikipedia.org/wiki/Determination_of_the_day_of_the_week#Gauss's_algorithm
                let y = year - 1;
                ((1 + 5 * (y % 4) + 4 * (y % 100) + 6 * (y % 400)) % 7) as u8
            }

            (weekday_jan1(year) + 3) % 7
        }

        let mut day = (date.week * 7 + date.day - (weekday_jan4(date.year) + 3)) as u16;
        if day < 1 {
            day += (date.year - 1).num_days();
        }
        if day > date.year.num_days() {
            day -= date.year.num_days();
        }

        Self {
            year: date.year,
            day
        }
    }
}

impl From<OrdinalDate> for YmdDate {
    fn from(date: OrdinalDate) -> Self {
        let leap = date.year.is_leap();
        let (month, day) = match date.day {
              1 ...  31         => ( 1, date.day -   0),
             32 ...  60 if leap => ( 2, date.day -  31),
             32 ...  59         => ( 2, date.day -  31),
             61 ...  91 if leap => ( 3, date.day -  60),
             60 ...  90         => ( 3, date.day -  59),
             92 ... 121 if leap => ( 4, date.day -  91),
             91 ... 120         => ( 4, date.day -  90),
            122 ... 152 if leap => ( 5, date.day - 121),
            121 ... 151         => ( 5, date.day - 120),
            153 ... 182 if leap => ( 6, date.day - 152),
            152 ... 181         => ( 6, date.day - 151),
            183 ... 213 if leap => ( 7, date.day - 182),
            182 ... 212         => ( 7, date.day - 181),
            214 ... 244 if leap => ( 8, date.day - 213),
            213 ... 243         => ( 8, date.day - 212),
            245 ... 274 if leap => ( 9, date.day - 244),
            244 ... 273         => ( 9, date.day - 243),
            275 ... 305 if leap => (10, date.day - 274),
            274 ... 304         => (10, date.day - 273),
            306 ... 335 if leap => (11, date.day - 305),
            305 ... 334         => (11, date.day - 304),
            336 ... 366 if leap => (12, date.day - 335),
            335 ... 365         => (12, date.day - 334),
            _ => unreachable!()
        };

        Self {
            year: date.year,
            month,
            day: day as u8
        }
    }
}

impl From<WeekDate> for YmdDate {
    fn from(date: WeekDate) -> Self {
        let date: OrdinalDate = date.into();
        date.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinal_from_week() {
        assert_eq!(
            OrdinalDate::from(WeekDate {
                year: 1985,
                week: 15,
                day: 5
            }),
            OrdinalDate {
                year: 1985,
                day: 102
            }
        );
    }

    #[test]
    fn ymd_from_ordinal() {
        assert_eq!(
            YmdDate::from(OrdinalDate {
                year: 1985,
                day: 102
            }),
            YmdDate {
                year: 1985,
                month: 4,
                day: 12
            }
        );
    }

    #[test]
    fn ymd_from_week() {
        assert_eq!(
            YmdDate::from(WeekDate {
                year: 1985,
                week: 15,
                day: 5
            }),
            YmdDate {
                year: 1985,
                month: 4,
                day: 12
            }
        );
    }
}
