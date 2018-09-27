use {
    Valid,
    date::*,
    time::*
};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct DateTime<D = YmdDate, T = GlobalTime>
where D: Datelike, T: Time {
    pub date: D,
    pub time: T
}

impl_fromstr_parse!(DateTime<Date,       GlobalTime>, datetime_global);
impl_fromstr_parse!(DateTime<Date,       LocalTime>,  datetime_local);
impl_fromstr_parse!(DateTime<Date,       AnyTime>,    datetime);
impl_fromstr_parse!(DateTime<ApproxDate, GlobalTime>, datetime_approx_global);
impl_fromstr_parse!(DateTime<ApproxDate, LocalTime>,  datetime_approx_local);
impl_fromstr_parse!(DateTime<ApproxDate, AnyTime>,    datetime_approx);

impl<D, T> Valid for DateTime<D, T>
where
    D: Datelike + Valid,
    T: Time     + Valid
{
    fn is_valid(&self) -> bool {
        self.date.is_valid() &&
        self.time.is_valid()
    }
}
