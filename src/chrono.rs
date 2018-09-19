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
        let date: ::YmdDate = dt.date.into();

        FixedOffset::east((dt.time.tz_offset * 60) as i32)
            .ymd(
                date.year as i32,
                date.month.into(),
                date.day.into()
            )
            .and_hms_nano(
                dt.time.hour.into(),
                dt.time.minute.into(),
                dt.time.second.into(),
                dt.time.nanos
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
    	DateTime<Tz>: From<::DateTime>
    {
        Ok(::parse::datetime(String::deserialize(de)?.as_bytes())
            .map_err(serde::de::Error::custom)?.1
            .into()
        )
    }
}
