use ::{
    datetime::*,
    date::*,
    time::*
};
use super::*;

macro_rules! datetime {
    ($name:ident, $date:ty, $date_parser:ident, $time:ty, $time_parser:ident) => {
        named!(pub $name <&[u8], DateTime<$date, $time>>, do_parse!(
            date: $date_parser >>
            char!('T') >>
            time: $time_parser >>
            (DateTime { date, time })
        ));
    }
}
datetime!(datetime_local,         Date,       date,        LocalTime,  time_local);
datetime!(datetime_global,        Date,       date,        GlobalTime, time_global);
datetime!(datetime,               Date,       date,        AnyTime,    time);
datetime!(datetime_approx_local,  ApproxDate, date_approx, LocalTime,  time_local);
datetime!(datetime_approx_global, ApproxDate, date_approx, GlobalTime, time_global);
datetime!(datetime_approx,        ApproxDate, date_approx, AnyTime,    time);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_local() {
        let value = DateTime {
            date: Date::YMD(YmdDate {
                year: 2007,
                month: 8,
                day: 31
            }),
            time: LocalTime {
                hour: 16,
                minute: 47,
                second: 22,
                nanos: 0
            }
        };
        assert_eq!(super::datetime_local(b"2007-08-31T16:47:22"), Ok((&[][..], value)));
    }

    #[test]
    fn datetime_global() {
        let value = DateTime {
            date: Date::YMD(YmdDate {
                year: 2007,
                month: 8,
                day: 31
            }),
            time: GlobalTime {
                local: LocalTime {
                    hour: 16,
                    minute: 47,
                    second: 22,
                    nanos: 0
                },
                timezone: 5 * 60
            }
        };
        assert_eq!(super::datetime_global(b"2007-08-31T16:47:22+05:00"), Ok((&[][..], value.clone())));
        assert_eq!(super::datetime_global(b"20070831T164722+05"),        Ok((&[][..], value        )));
    }

    #[test]
    fn datetime() {
        let date = Date::YMD(YmdDate {
            year: 2007,
            month: 8,
            day: 31
        });
        let time_local = LocalTime {
            hour: 16,
            minute: 47,
            second: 22,
            nanos: 0
        };
        assert_eq!(super::datetime(b"2007-08-31T16:47:22"), Ok((&[][..], DateTime {
            date: date.clone(),
            time: AnyTime::Local(time_local.clone())
        })));
        assert_eq!(super::datetime(b"2007-08-31T16:47:22Z"), Ok((&[][..], DateTime {
            date: date,
            time: AnyTime::Global(GlobalTime {
                local: time_local,
                timezone: 0
            })
        })));
    }
}
