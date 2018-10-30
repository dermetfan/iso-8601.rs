#![cfg(feature = "chrono")]

extern crate chrono;

use self::chrono::prelude::*;

impl From<::DateTime<::Date, ::GlobalTime>> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime<::Date, ::GlobalTime>) -> Self {
        let date: ::YmdDate = dt.date.into();

        FixedOffset::east((dt.time.timezone * 60).into())
            .ymd(
                date.year.into(),
                date.month.into(),
                date.day.into()
            ).and_hms_nano(
                dt.time.local.naive.hour.into(),
                dt.time.local.naive.minute.into(),
                dt.time.local.naive.second.into(),
                dt.time.local.nanosecond()
            )
    }
}

impl From<::DateTime<::Date, ::GlobalTime>> for DateTime<Utc> {
    fn from(dt: ::DateTime<::Date, ::GlobalTime>) -> Self {
        DateTime::<FixedOffset>::from(dt)
            .with_timezone(&Utc)
    }
}

impl From<::DateTime<::Date, ::GlobalTime>> for DateTime<Local> {
    fn from(dt: ::DateTime<::Date, ::GlobalTime>) -> Self {
        DateTime::<FixedOffset>::from(dt)
            .with_timezone(&Local)
    }
}

impl From<::DateTime<::Date, ::LocalTime>> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime<::Date, ::LocalTime>) -> Self {
        DateTime::<Local>::from(dt)
            .with_timezone(&Utc.fix())
    }
}

impl From<::DateTime<::Date, ::LocalTime>> for DateTime<Utc> {
    fn from(dt: ::DateTime<::Date, ::LocalTime>) -> Self {
        DateTime::<Local>::from(dt)
            .with_timezone(&Utc)
    }
}

impl From<::DateTime<::Date, ::LocalTime>> for DateTime<Local> {
    fn from(dt: ::DateTime<::Date, ::LocalTime>) -> Self {
        let date: ::YmdDate = dt.date.into();

        Local.from_local_datetime(
            &NaiveDate::from_ymd(
                date.year.into(),
                date.month.into(),
                date.day.into()
            ).and_hms_nano(
                dt.time.naive.hour.into(),
                dt.time.naive.minute.into(),
                dt.time.naive.second.into(),
                dt.time.nanosecond()
            )
        ).single().unwrap()
    }
}

impl From<::DateTime<::Date, ::AnyTime>> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime<::Date, ::AnyTime>) -> Self {
        DateTime::<Local>::from(dt)
            .with_timezone(&Utc.fix())
    }
}

impl From<::DateTime<::Date, ::AnyTime>> for DateTime<Utc> {
    fn from(dt: ::DateTime<::Date, ::AnyTime>) -> Self {
        DateTime::<Local>::from(dt)
            .with_timezone(&Utc)
    }
}

impl From<::DateTime<::Date, ::AnyTime>> for DateTime<Local> {
    fn from(dt: ::DateTime<::Date, ::AnyTime>) -> Self {
        match dt.time {
            ::AnyTime::Global(time) => {
                ::DateTime {
                    date: dt.date,
                    time
                }.into()
            }
            ::AnyTime::Local(time) => {
                ::DateTime {
                    date: dt.date,
                    time
                }.into()
            }
        }
    }
}

#[cfg(feature = "chrono-serde")]
pub mod serde {
    extern crate serde;

    use self::serde::{
        Deserialize,
        Deserializer
    };
    use super::{
        DateTime,
        TimeZone
    };

    #[allow(non_snake_case)]
    pub fn deserialize_DateTime<'de, D, Tz>(de: D) -> Result<DateTime<Tz>, D::Error> where
    	D: Deserializer<'de>,
    	Tz: TimeZone,
    	DateTime<Tz>: From<::DateTime<::ApproxDate, ::ApproxAnyTime>>
    {
        Ok(
            ::parse::datetime_approx_any_approx(String::deserialize(de)?.as_bytes())
                .map_err(serde::de::Error::custom)?.1
                .into()
        )
    }
}
