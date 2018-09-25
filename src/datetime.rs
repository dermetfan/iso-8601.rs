use {
    parse,
    Valid,
    date::*,
    time::*,
    std::str::FromStr
};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct DateTime<Y = i16, T = GlobalTime>
where Y: Year, T: Time {
    pub date: Date<Y>,
    pub time: T
}

impl FromStr for DateTime {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::datetime_global(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for DateTime<i16, LocalTime> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::datetime_local(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl FromStr for DateTime<i16, AnyTime> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::datetime(s.as_bytes())
            .map(|x| x.1)
            .or(Err(()))
    }
}

impl Valid for DateTime {
    fn is_valid(&self) -> bool {
        self.date.is_valid() &&
        self.time.is_valid()
    }
}
