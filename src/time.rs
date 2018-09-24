use {
    parse,
    Valid,
    std::str::FromStr
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
