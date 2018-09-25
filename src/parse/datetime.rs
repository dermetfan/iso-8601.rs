use ::{
    datetime::*,
    time::*
};
use super::*;

named!(pub datetime_local <&[u8], DateTime<i16, LocalTime>>, do_parse!(
    date: date >>
    char!('T') >>
    time: time_local >>
    (DateTime { date, time })
));

named!(pub datetime_global <&[u8], DateTime>, do_parse!(
    date: date >>
    char!('T') >>
    time: time_global >>
    (DateTime { date, time })
));

named!(pub datetime <&[u8], DateTime<i16, AnyTime>>, do_parse!(
    date: date >>
    char!('T') >>
    time: time >>
    (DateTime { date, time })
));

#[cfg(test)]
mod tests {
    use super::*;
    use ::date::*;

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
