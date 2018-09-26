use {
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

impl_fromstr_parse!(DateTime,                 datetime_global);
impl_fromstr_parse!(DateTime<i16, LocalTime>, datetime_local );
impl_fromstr_parse!(DateTime<i16, AnyTime>,   datetime       );

impl<Y, T> Valid for DateTime<Y, T>
where
    Y: Year + Clone,
    T: Time + Valid
{
    fn is_valid(&self) -> bool {
        self.date.is_valid() &&
        self.time.is_valid()
    }
}
