#[macro_use] extern crate nom;
extern crate regex;

mod parse;
pub mod chrono;

use std::convert::From;
use std::str::FromStr;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Date<Y: Year = i16> {
    YMD(YmdDate<Y>),
    Week(WeekDate<Y>),
    Ordinal(OrdinalDate<Y>)
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct YmdDate<Y: Year = i16> {
    pub year: Y,
    pub month: u8,
    pub day: u8
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct WeekDate<Y: Year = i16> {
    pub year: Y,
    pub week: u8,
    pub day: u8
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct OrdinalDate<Y: Year = i16> {
    pub year: Y,
    pub day: u16
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Time {
    pub local: LocalTime,
    /// minutes
    pub tz_offset: i16
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct LocalTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct DateTime<Y: Year = i16> {
    pub date: Date<Y>,
    pub time: Time
}

impl FromStr for Date {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::date(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for LocalTime {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::time_local(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for Time {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::time(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for DateTime {
    type Err = ();

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

pub trait Valid {
    fn is_valid(&self) -> bool;
}

impl Valid for Date {
    fn is_valid(&self) -> bool {
        match self {
            Date::YMD    (date) => date.is_valid(),
            Date::Week   (date) => date.is_valid(),
            Date::Ordinal(date) => date.is_valid()
        }
    }
}

impl Valid for YmdDate {
    fn is_valid(&self) -> bool {
        self.day >= 1 && self.day <= match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11              => 30,
            2 => if self.year.is_leap() { 29 } else { 28 },
            _ => return false
        }
    }
}

impl Valid for WeekDate {
    fn is_valid(&self) -> bool {
        self.week >= 1 && self.week <= self.year.num_weeks() &&
        self.day >= 1 && self.day <= 7
    }
}

impl Valid for OrdinalDate {
    fn is_valid(&self) -> bool {
        self.day >= 1 && self.day <= self.year.num_days()
    }
}

impl Valid for LocalTime {
    /// Accepts leap seconds on any day
    /// since they are not predictable.
    fn is_valid(&self) -> bool {
        self.hour <= 24 &&
        self.minute <= 59 &&
        self.second <= 60 &&
        self.nanos < 1_000_000_000
    }
}

impl Valid for Time {
    fn is_valid(&self) -> bool {
        self.local.is_valid() &&
        self.tz_offset < 24 * 60 && self.tz_offset > -24 * 60
    }
}

impl Valid for DateTime {
    fn is_valid(&self) -> bool {
        self.date.is_valid() &&
        self.time.is_valid()
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

    #[test]
    fn valid_date_ymd() {
        assert!(!YmdDate {
            year: 0,
            month: 13,
            day: 1
        }.is_valid());
        assert!(!YmdDate {
            year: 0,
            month: 0,
            day: 1
        }.is_valid());

        assert!(!YmdDate {
            year: 2018,
            month: 2,
            day: 29
        }.is_valid());
    }

    #[test]
    fn valid_date_week() {
        assert!(!WeekDate {
            year: 0,
            week: 0,
            day: 1
        }.is_valid());
        assert!(!WeekDate {
            year: 2018,
            week: 53,
            day: 1
        }.is_valid());

        assert!(!WeekDate {
            year: 0,
            week: 1,
            day: 0
        }.is_valid());
        assert!(!WeekDate {
            year: 0,
            week: 1,
            day: 8
        }.is_valid());
    }

    #[test]
    fn valid_date_ordinal() {
        assert!(!OrdinalDate {
            year: 2018,
            day: 366
        }.is_valid());
        assert!(OrdinalDate {
            year: 2020,
            day: 366
        }.is_valid());
    }

    #[test]
    fn valid_time_local() {
        assert!(!LocalTime {
            hour: 25,
            minute: 0,
            second: 0,
            nanos: 0
        }.is_valid());

        assert!(!LocalTime {
            hour: 0,
            minute: 60,
            second: 0,
            nanos: 0
        }.is_valid());

        assert!(!LocalTime {
            hour: 0,
            minute: 1,
            second: 61,
            nanos: 0
        }.is_valid());

        assert!(!LocalTime {
            hour: 0,
            minute: 1,
            second: 0,
            nanos: 1_000_000_000
        }.is_valid());
    }

    #[test]
    fn valid_time() {
        assert!(!Time {
            local: LocalTime {
                hour: 0,
                minute: 1,
                second: 0,
                nanos: 0
            },
            tz_offset: 24 * 60
        }.is_valid());
        assert!(!Time {
            local: LocalTime {
                hour: 0,
                minute: 1,
                second: 0,
                nanos: 0
            },
            tz_offset: -24 * 60
        }.is_valid());
    }
}
