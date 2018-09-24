use {
    parse,
    Valid,
    date::*,
    time::*,
    std::str::FromStr
};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct DateTime<Y: Year = i16> {
    pub date: Date<Y>,
    pub time: Time
}

impl FromStr for DateTime {
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
