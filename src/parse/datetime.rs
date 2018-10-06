use ::{
    datetime::*,
    date::*,
    time::*
};
use super::*;

macro_rules! datetime {
    (pub $name:ident, $date:ty, $date_parser:ident, $time:ty, $time_parser:ident) => {
        named!(pub $name <DateTime<$date, $time>>, do_parse!(
            date: $date_parser >>
            char!('T') >>
            peek!(not!(char!('T'))) >>
            time: $time_parser >>
            (DateTime { date, time })
        ));
    }
}
datetime!(pub datetime_global_hms,           Date,       date,        GlobalTime<HmsTime>, time_global_hms);
datetime!(pub datetime_global_hm,            Date,       date,        GlobalTime<HmTime>,  time_global_hm);
datetime!(pub datetime_global_h,             Date,       date,        GlobalTime<HTime>,   time_global_h);
datetime!(pub datetime_local_hms,            Date,       date,        LocalTime<HmsTime>,  time_local_hms);
datetime!(pub datetime_local_hm,             Date,       date,        LocalTime<HmTime>,   time_local_hm);
datetime!(pub datetime_local_h,              Date,       date,        LocalTime<HTime>,    time_local_h);
datetime!(pub datetime_any_hms,              Date,       date,        AnyTime<HmsTime>,    time_any_hms);
datetime!(pub datetime_any_hm,               Date,       date,        AnyTime<HmTime>,     time_any_hm);
datetime!(pub datetime_any_h,                Date,       date,        AnyTime<HTime>,      time_any_h);
datetime!(pub datetime_global_approx,        Date,       date,        ApproxGlobalTime,    time_global_approx);
datetime!(pub datetime_local_approx,         Date,       date,        ApproxLocalTime,     time_local_approx);
datetime!(pub datetime_any_approx,           Date,       date,        ApproxAnyTime,       time_any_approx);
datetime!(pub datetime_approx_global_hms,    ApproxDate, date_approx, GlobalTime<HmsTime>, time_global_hms);
datetime!(pub datetime_approx_global_hm,     ApproxDate, date_approx, GlobalTime<HmTime>,  time_global_hm);
datetime!(pub datetime_approx_global_h,      ApproxDate, date_approx, GlobalTime<HTime>,   time_global_h);
datetime!(pub datetime_approx_local_hms,     ApproxDate, date_approx, LocalTime<HmsTime>,  time_local_hms);
datetime!(pub datetime_approx_local_hm,      ApproxDate, date_approx, LocalTime<HmTime>,   time_local_hm);
datetime!(pub datetime_approx_local_h,       ApproxDate, date_approx, LocalTime<HTime>,    time_local_h);
datetime!(pub datetime_approx_any_hms,       ApproxDate, date_approx, AnyTime<HmsTime>,    time_any_hms);
datetime!(pub datetime_approx_any_hm,        ApproxDate, date_approx, AnyTime<HmTime>,     time_any_hm);
datetime!(pub datetime_approx_any_h,         ApproxDate, date_approx, AnyTime<HTime>,      time_any_h);
datetime!(pub datetime_approx_global_approx, ApproxDate, date_approx, ApproxGlobalTime,    time_global_approx);
datetime!(pub datetime_approx_local_approx,  ApproxDate, date_approx, ApproxLocalTime,     time_local_approx);
datetime!(pub datetime_approx_any_approx,    ApproxDate, date_approx, ApproxAnyTime,       time_any_approx);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn tt() {
        datetime_approx_any_approx(b"2018-08-02TT22:01:39Z").unwrap();
    }
}
