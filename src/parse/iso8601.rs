use std::ops::{AddAssign, MulAssign};
use {nom, Date, Time, DateTime};

fn buf_to_int<T>(buf: &[u8]) -> T
where T: AddAssign + MulAssign + From<u8> {
    let mut sum = T::from(0);
    for digit in buf {
        sum *= T::from(10);
        sum += T::from(*digit - b'0');
    }
    sum
}

named!(sign <&[u8], i32>, alt!(
    tag!("-") => { |_| -1 } |
    tag!("+") => { |_|  1 }
));

named!(positive_year <&[u8], u32>, map!(
    take_while_m_n!(4, 4, nom::is_digit),
    buf_to_int
));

named!(year <&[u8], i32>, do_parse!(
    sign: opt!(sign) >>
    year: positive_year >>
    (sign.unwrap_or(1) * year as i32)
));

named!(month <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));
named!(day <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

named!(pub date <&[u8], Date>, do_parse!(
    year: year >>
    opt!(tag!("-")) >>
    month: month >>
    opt!(tag!("-")) >>
    day: day >>
    (Date { year, month, day })
));

named!(hour <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));
named!(minute <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));
named!(second <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

named!(pub time <&[u8], Time>, do_parse!(
    hour: hour >>
    opt!(tag!(":")) >>
    minute: minute >>
    second: opt!(complete!(do_parse!(
        opt!(tag!(":")) >>
        second: second >>
        (second)
    ))) >>
    tz_offset: opt!(complete!(timezone)) >>
    (Time {
        hour, minute,
        second: second.unwrap_or(0),
        tz_offset: tz_offset.unwrap_or(0)
    })
));

named!(timezone_utc <&[u8], i32>, map!(
    tag!("Z"), |_| 0
));

named!(timezone_hour <&[u8], i32>, do_parse!(
    sign: sign >>
    hour: hour >>
    minute: opt!(complete!(do_parse!(
        opt!(tag!(":")) >>
        minute: minute >>
        (minute)
    ))) >>
    (sign * (hour as i32 * 3600 + minute.unwrap_or(0) as i32 * 60))
));

named!(timezone <&[u8], i32>, alt!(timezone_utc | timezone_hour));

named!(pub datetime <&[u8], DateTime>, do_parse!(
    date: date >>
    tag!("T") >>
    time: time >>
    (DateTime { date, time })
));

#[cfg(test)]
mod tests {
    use nom::Context::Code;
    use nom::Err::{Error, Incomplete};
    use nom::ErrorKind::Alt;
    use nom::Needed::Size;
    use {Date, Time, DateTime};

    #[test]
    fn parse_sign() {
        use super::sign;

        assert_eq!(sign(b"-"), Ok((&[][..], -1)));
        assert_eq!(sign(b"+"), Ok((&[][..],  1)));
        assert_eq!(sign(b"" ), Err(Incomplete(Size(1))));
        assert_eq!(sign(b" "), Err(Error(Code(&b" "[..], Alt))));
    }

    #[test]
    fn parse_positive_year() {
        use super::positive_year;

        assert_eq!(positive_year(b"2018"), Ok((&[][..], 2018)));
    }

    #[test]
    fn parse_year() {
        use super::year;

        assert_eq!(year(b"2018" ), Ok((&[][..],  2018)));
        assert_eq!(year(b"+2018"), Ok((&[][..],  2018)));
        assert_eq!(year(b"-2018"), Ok((&[][..], -2018)));
    }

    #[test]
    fn parse_month() {
        use super::month;

        assert_eq!(month(b"06"), Ok((&[][..], 6)));
    }

    #[test]
    fn parse_day() {
        use super::day;

        assert_eq!(day(b"18"), Ok((&[][..], 18)));
    }

    #[test]
    fn parse_date() {
        use super::date;

        {
            let value = Date {
                year: 2015,
                month: 7,
                day: 16
            };
            assert_eq!(date(b"2015-07-16"), Ok((
                &[][..], value.clone()
            )));
            assert_eq!(date(b"20150716"), Ok((
                &[][..], value
            )));
        }
        {
            let value = Date {
                year: -333,
                month: 6,
                day: 11
            };
            assert_eq!(date(b"-0333-06-11"), Ok((
                &[][..], value.clone()
            )));
            assert_eq!(date(b"-03330611"), Ok((
                &[][..], value
            )));
        }
    }

    #[test]
    fn parse_time() {
        use super::time;

        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                tz_offset: 0
            };
            assert_eq!(time(b"16:43:52"), Ok((
                &[][..], value.clone()
            )));
            assert_eq!(time(b"164352"), Ok((
                &[][..], value
            )));
        }
        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 0,
                tz_offset: 0
            };
            assert_eq!(time(b"16:43"), Ok((
                &[][..], value.clone()
            )));
            assert_eq!(time(b"1643"), Ok((
                &[][..], value
            )));
        }
    }

    #[test]
    fn parse_time_with_timezone() {
        use super::time;

        assert_eq!(time(b"16:43:52Z"), Ok((
            &[][..], Time {
                hour: 16,
                minute: 43,
                second: 52,
                tz_offset: 0
            }
        )));
        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                tz_offset: 5 * 3600
            };
            assert_eq!(time(b"16:43:52+05"), Ok((
                &[][..], value.clone()
            )));
            assert_eq!(time(b"16:43:52+0500"), Ok((
                &[][..], value
            )));
        }
        assert_eq!(time(b"16:43-05:30"), Ok((
            &[][..], Time {
                hour: 16,
                minute: 43,
                second: 0,
                tz_offset: -(5 * 3600 + 30 * 60)
            }
        )));
    }

    #[test]
    fn parse_datetime() {
        use super::datetime;

        let value = DateTime {
            date: Date {
                year: 2007,
                month: 8,
                day: 31
            },
            time: Time {
                hour: 16,
                minute: 47,
                second: 22,
                tz_offset: 5 * 3600
            }
        };
        assert_eq!(datetime(b"2007-08-31T16:47:22+05:00"), Ok((
            &[][..], value.clone()
        )));
        assert_eq!(datetime(b"20070831T164722+05"), Ok((
            &[][..], value
        )));
    }
}
