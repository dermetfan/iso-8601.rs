#![cfg(feature = "chrono")]

extern crate chrono;

use self::chrono::{
    DateTime,
    TimeZone,
    Utc,
    FixedOffset,
    Local
};

impl From<::DateTime> for DateTime<FixedOffset> {
    fn from(dt: ::DateTime) -> Self {
        FixedOffset::east(dt.time.tz_offset)
            .ymd(dt.date.year, dt.date.month.into(), dt.date.day.into())
            .and_hms(dt.time.hour.into(), dt.time.minute.into(), dt.time.second.into())
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
    pub fn deserialize_iso8601_DateTime<'de, D, Tz>(de: D) -> Result<DateTime<Tz>, D::Error>
    where
    	D: Deserializer<'de>,
    	Tz: TimeZone,
    	DateTime<Tz>: From<::DateTime>
    {
        Ok(::iso8601::datetime(String::deserialize(de)?.as_bytes())
            .map_err(serde::de::Error::custom)?.1
            .into()
        )
    }
}
