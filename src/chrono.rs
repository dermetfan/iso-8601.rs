#![cfg(feature = "chrono")]

extern crate chrono;

use self::chrono::prelude::*;

impl From<::DateTime> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime) -> Self {
        let date: ::YmdDate = dt.date.into();

        FixedOffset::east((dt.time.timezone * 60).into())
            .ymd(
                date.year.into(),
                date.month.into(),
                date.day.into()
            ).and_hms_nano(
                dt.time.local.hour.into(),
                dt.time.local.minute.into(),
                dt.time.local.second.into(),
                dt.time.local.nanos
            )
    }
}

impl From<::DateTime> for DateTime<Utc> {
    fn from(dt: ::DateTime) -> Self {
        let dt: DateTime<FixedOffset> = dt.into();
        dt.with_timezone(&Utc)
    }
}

impl From<::DateTime> for DateTime<Local> {
    fn from(dt: ::DateTime) -> Self {
        let dt: DateTime<FixedOffset> = dt.into();
        dt.with_timezone(&Local)
    }
}

impl From<::DateTime<i16, ::LocalTime>> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime<i16, ::LocalTime>) -> Self {
        let dt: DateTime<Local> = dt.into();
        dt.with_timezone(&Utc.fix())
    }
}

impl From<::DateTime<i16, ::LocalTime>> for DateTime<Utc> {
    fn from(dt: ::DateTime<i16, ::LocalTime>) -> Self {
        let dt: DateTime<Local> = dt.into();
        dt.with_timezone(&Utc)
    }
}

impl From<::DateTime<i16, ::LocalTime>> for DateTime<Local> {
    fn from(dt: ::DateTime<i16, ::LocalTime>) -> Self {
        let date: ::YmdDate = dt.date.into();

        Local.from_local_datetime(
            &NaiveDate::from_ymd(
                date.year.into(),
                date.month.into(),
                date.day.into()
            ).and_hms_nano(
                dt.time.hour.into(),
                dt.time.minute.into(),
                dt.time.second.into(),
                dt.time.nanos
            )
        ).single().unwrap()
    }
}

impl From<::DateTime<i16, ::AnyTime>> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime<i16, ::AnyTime>) -> Self {
        let dt: DateTime<Local> = dt.into();
        dt.with_timezone(&Utc.fix())
    }
}

impl From<::DateTime<i16, ::AnyTime>> for DateTime<Utc> {
    fn from(dt: ::DateTime<i16, ::AnyTime>) -> Self {
        let dt: DateTime<Local> = dt.into();
        dt.with_timezone(&Utc)
    }
}

impl From<::DateTime<i16, ::AnyTime>> for DateTime<Local> {
    fn from(dt: ::DateTime<i16, ::AnyTime>) -> Self {
        match dt.time {
            ::AnyTime::Global(time) => {
                ::DateTime {
                    date: dt.date,
                    time: time
                }.into()
            }
            ::AnyTime::Local(time) => {
                ::DateTime {
                    date: dt.date,
                    time: time
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
    pub fn deserialize_DateTime<'de, D, Tz>(de: D) -> Result<DateTime<Tz>, D::Error>
    where
    	D: Deserializer<'de>,
    	Tz: TimeZone,
    	DateTime<Tz>: From<::DateTime<i16, ::AnyTime>>
    {
        Ok(
            ::parse::datetime(String::deserialize(de)?.as_bytes())
                .map_err(serde::de::Error::custom)?.1
                .into()
        )
    }
}
