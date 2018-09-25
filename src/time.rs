use {
    parse,
    Valid,
    std::str::FromStr
};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Time<Tz: TimeZone = i16> {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32,
    pub timezone: Tz
}

pub type LocalTime = Time<()>;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum AnyTime {
    Global(Time),
    Local(LocalTime)
}

impl FromStr for Time {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::time(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

fn is_valid_local<Tz>(time: &Time<Tz>) -> bool
where Tz: TimeZone {
    time.hour <= 24 &&
    time.minute <= 59 &&
    time.second <= 60 &&
    time.nanos < 1_000_000_000
}

impl Valid for Time {
    /// Accepts leap seconds on any day
    /// since they are not predictable.
    fn is_valid(&self) -> bool {
        is_valid_local(self) &&
        self.timezone > -24 * 60 &&
        self.timezone <  24 * 60
    }
}

impl Valid for LocalTime {
    /// Accepts leap seconds on any day
    /// since they are not predictable.
    fn is_valid(&self) -> bool {
        is_valid_local(self)
    }
}

pub trait TimeZone {}

/// Offset from UTC in minutes.
impl TimeZone for i16 {}

/// Local time.
impl TimeZone for () {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_time_local() {
        assert!(!LocalTime {
            hour: 25,
            minute: 0,
            second: 0,
            nanos: 0,
            timezone: ()
        }.is_valid());

        assert!(!LocalTime {
            hour: 0,
            minute: 60,
            second: 0,
            nanos: 0,
            timezone: ()
        }.is_valid());

        assert!(!LocalTime {
            hour: 0,
            minute: 1,
            second: 61,
            nanos: 0,
            timezone: ()
        }.is_valid());

        assert!(!LocalTime {
            hour: 0,
            minute: 1,
            second: 0,
            nanos: 1_000_000_000,
            timezone: ()
        }.is_valid());
    }

    #[test]
    fn valid_time() {
        assert!(!Time {
            hour: 0,
            minute: 1,
            second: 0,
            nanos: 0,
            timezone: 24 * 60
        }.is_valid());
        assert!(!Time {
            hour: 0,
            minute: 1,
            second: 0,
            nanos: 0,
            timezone: -24 * 60
        }.is_valid());
    }
}
