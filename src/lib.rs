#[macro_use] extern crate nom;

mod parse;
pub mod chrono;

pub use parse::*;

use std::convert::From;
use std::str::FromStr;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Date<Y: Year = i16> {
    YMD(YmdDate<Y>),
    Week(WeekDate<Y>),
    Ordinal(OrdinalDate<Y>)
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct YmdDate<Y: Year = i16> {
    year: Y,
    month: u8,
    day: u8
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct WeekDate<Y: Year = i16> {
    year: Y,
    week: u8,
    day: u8
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct OrdinalDate<Y: Year = i16> {
    year: Y,
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
pub struct DateTime<Y: Year = i16> {
    pub date: Date<Y>,
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

    fn num_days(&self) -> u16 {
        if self.is_leap() { 366 } else { 365 }
    }
}

macro_rules! impl_year {
    ($ty:ty) => {
        impl Year for $ty {
            fn is_leap(&self) -> bool {
                let factor = |x| self % x == 0;
                factor(4) && (!factor(100) || factor(400))
            }

            fn num_weeks(&self) -> u8 {
                // https://en.wikipedia.org/wiki/ISO_week_date#Weeks_per_year
                let p = |x| (x + x / 4 - x / 100 + x / 400) % 7;
                if p(*self) == 4 || p(self - 1) == 3 { 53 } else { 52 }
            }
        }
    }
}

impl_year!(i16);
impl_year!(i32);
impl_year!(i64);
impl_year!(u16);
impl_year!(u32);
impl_year!(u64);

impl From<Date> for YmdDate {
    fn from(date: Date) -> Self {
        match date {
            Date::YMD    (date) => date,
            Date::Week   (date) => date.into(),
            Date::Ordinal(date) => date.into()
        }
    }
}

impl From<Date> for WeekDate {
    fn from(date: Date) -> Self {
        match date {
            Date::YMD    (date) => date.into(),
            Date::Week   (date) => date,
            Date::Ordinal(date) => date.into()
        }
    }
}

impl From<Date> for OrdinalDate {
    fn from(date: Date) -> Self {
        match date {
            Date::YMD    (date) => date.into(),
            Date::Week   (date) => date.into(),
            Date::Ordinal(date) => date
        }
    }
}

impl From<WeekDate> for YmdDate {
    fn from(date: WeekDate) -> Self {
        OrdinalDate::from(date).into()
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

impl From<YmdDate> for WeekDate {
    fn from(date: YmdDate) -> Self {
        OrdinalDate::from(date).into()
    }
}

impl From<OrdinalDate> for WeekDate {
    fn from(date: OrdinalDate) -> Self {
        // https://en.wikipedia.org/wiki/ISO_week_date#Calculating_the_week_number_of_a_given_date
        let y = date.year % 100 % 28;
        let cc = (date.year / 100) % 4;
        let mut c = ((y + (y - 1) / 4 + 5 * cc - 1) % 7) as i16;
        if c > 3 {
            c -= 7;
        }
        let dc = date.day as i16 + c;
        Self {
            year: date.year,
            week: (dc as f32 / 7.0).ceil() as u8,
            day: (dc % 7) as u8
        }
    }
}

impl From<YmdDate> for OrdinalDate {
    fn from(date: YmdDate) -> Self {
        let leap = date.year.is_leap();
        Self {
            year: date.year,
            day: match date.month {
                 1         =>   0,
                 2         =>  31,
                 3 if leap =>  60,
                 3         =>  59,
                 4 if leap =>  91,
                 4         =>  90,
                 5 if leap => 121,
                 5         => 120,
                 6 if leap => 152,
                 6         => 151,
                 7 if leap => 182,
                 7         => 181,
                 8 if leap => 213,
                 8         => 212,
                 9 if leap => 244,
                 9         => 243,
                10 if leap => 274,
                10         => 273,
                11 if leap => 305,
                11         => 304,
                12 if leap => 335,
                12         => 334,
                _ => unreachable!()
            } + date.day as u16
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn week_from_ymd() {
        assert_eq!(
            WeekDate::from(YmdDate {
                year: 1985,
                month: 4,
                day: 12
            }),
            WeekDate {
                year: 1985,
                week: 15,
                day: 5
            }
        );
        assert_eq!(
            WeekDate::from(YmdDate {
                year: 2023,
                month: 2,
                day: 27
            }),
            WeekDate {
                year: 2023,
                week: 9,
                day: 1
            }
        );
    }

    #[test]
    fn week_from_ordinal() {
        assert_eq!(
            WeekDate::from(OrdinalDate {
                year: 1985,
                day: 102
            }),
            WeekDate {
                year: 1985,
                week: 15,
                day: 5
            }
        );
    }

    #[test]
    fn ordinal_from_ymd() {
        assert_eq!(
            OrdinalDate::from(YmdDate {
                year: 1985,
                month: 4,
                day: 12
            }),
            OrdinalDate {
                year: 1985,
                day: 102
            }
        );
    }

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
}
