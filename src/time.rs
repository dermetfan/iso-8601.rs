use {
    Valid,
    std::str::FromStr
};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct LocalTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct GlobalTime {
    pub local: LocalTime,
    /// Offset from UTC in minutes.
    pub timezone: i16
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum AnyTime {
    Global(GlobalTime),
    Local(LocalTime)
}

pub trait Time {
    fn hour(&self) -> u8;
    fn minute(&self) -> u8;
    fn second(&self) -> u8;
    fn nanos(&self) -> u32;
    fn timezone(&self) -> Option<i16>;
}

impl Time for LocalTime {
    fn hour(&self) -> u8 {
        self.hour
    }

    fn minute(&self) -> u8 {
        self.minute
    }

    fn second(&self) -> u8 {
        self.second
    }

    fn nanos(&self) -> u32 {
        self.nanos
    }

    fn timezone(&self) -> Option<i16> {
        None
    }
}

impl Time for GlobalTime {
    fn hour(&self) -> u8 {
        self.local.hour
    }

    fn minute(&self) -> u8 {
        self.local.minute
    }

    fn second(&self) -> u8 {
        self.local.second
    }

    fn nanos(&self) -> u32 {
        self.local.nanos
    }

    fn timezone(&self) -> Option<i16> {
        Some(self.timezone)
    }
}

impl Time for AnyTime {
    fn hour(&self) -> u8 {
        match self {
            AnyTime::Global(time) => time.hour(),
            AnyTime::Local (time) => time.hour()
        }
    }

    fn minute(&self) -> u8 {
        match self {
            AnyTime::Global(time) => time.minute(),
            AnyTime::Local (time) => time.minute()
        }
    }

    fn second(&self) -> u8 {
        match self {
            AnyTime::Global(time) => time.second(),
            AnyTime::Local (time) => time.second()
        }
    }

    fn nanos(&self) -> u32 {
        match self {
            AnyTime::Global(time) => time.nanos(),
            AnyTime::Local (time) => time.nanos()
        }
    }

    fn timezone(&self) -> Option<i16> {
        match self {
            AnyTime::Global(time) => time.timezone(),
            AnyTime::Local (time) => time.timezone()
        }
    }
}

impl_fromstr_parse!(LocalTime,  time_local );
impl_fromstr_parse!(GlobalTime, time_global);
impl_fromstr_parse!(AnyTime,    time       );

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

impl Valid for GlobalTime {
    fn is_valid(&self) -> bool {
        self.local.is_valid() &&
        self.timezone > -24 * 60 &&
        self.timezone <  24 * 60
    }
}

impl Valid for AnyTime {
    fn is_valid(&self) -> bool {
        match self {
            AnyTime::Global(time) => time.is_valid(),
            AnyTime::Local (time) => time.is_valid()
        }
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
    fn valid_time_global() {
        assert!(!GlobalTime {
            local: LocalTime {
                hour: 0,
                minute: 1,
                second: 0,
                nanos: 0
            },
            timezone: 24 * 60
        }.is_valid());
        assert!(!GlobalTime {
            local: LocalTime {
                hour: 0,
                minute: 1,
                second: 0,
                nanos: 0
            },
            timezone: -24 * 60
        }.is_valid());
    }
}
