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

/// Panics on greater than nanosecond precision (length > 9).
fn sec_frac_buf_to_nanos(buf: &[u8]) -> u32 {
    let mut nanos = 0;
    for (i, digit) in buf.iter().enumerate() {
        let digit = digit - b'0';
        nanos += digit as u32 * 10u32.pow(8 - i as u32);
    }
    nanos
}

/// Takes the rest of the input until EOF.
macro_rules! take_rest(
    ($i:expr,) => ({
        use nom::InputLength;

        take!($i, $i.input_len())
    })
);

named!(sign <&[u8], i32>, alt!(
    char!('-') => { |_| -1 } |
    char!('+') => { |_|  1 }
));

// TODO support expanded year
named!(positive_year <&[u8], u32>, map!(
    take_while_m_n!(4, 4, nom::is_digit),
    buf_to_int
));

named!(year <&[u8], i32>, do_parse!(
    sign: opt!(sign) >>
    year: positive_year >>
    (sign.unwrap_or(1) * year as i32)
));

named!(month <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |month| month <= 12
));

/// Not verified since number of days
/// in a month depends on the month.
named!(day <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

// TODO verify!() date validity
named!(pub date <&[u8], Date>, do_parse!(
    year: year >>
    opt!(char!('-')) >>
    month: month >>
    opt!(char!('-')) >>
    day: day >>
    (Date { year, month, day })
));

named!(hour <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |hour| hour <= 24
));

named!(minute <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |minute| minute <= 60
));

named!(second <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |second| second <= 60
));

// TODO verify!() time validity
named!(pub time <&[u8], Time>, do_parse!(
    hour: hour >>
    opt!(char!(':')) >>
    minute: minute >>
    seconds: opt!(complete!(do_parse!(
        opt!(char!(':')) >>
        second: second >>
        (second)
    ))) >>
    nanos: opt!(complete!(do_parse!(
        char!('.') >>
        sec_frac: take_rest!() >>
        (sec_frac_buf_to_nanos(sec_frac))
    ))) >>
    tz_offset: opt!(complete!(timezone)) >>
    (Time {
        hour, minute,
        second: seconds.unwrap_or(0),
        nanos: nanos.unwrap_or(0),
        tz_offset: tz_offset.unwrap_or(0)
    })
));

named!(timezone_utc <&[u8], i32>, map!(
    char!('Z'), |_| 0
));

named!(timezone_fixed <&[u8], i32>, do_parse!(
    sign: sign >>
    hour: hour >>
    minute: opt!(complete!(do_parse!(
        opt!(char!(':')) >>
        minute: minute >>
        (minute)
    ))) >>
    (sign * (hour as i32 * 3600 + minute.unwrap_or(0) as i32 * 60))
));

named!(timezone <&[u8], i32>, alt!(timezone_utc | timezone_fixed));

named!(pub datetime <&[u8], DateTime>, do_parse!(
    date: date >>
    char!('T') >>
    time: time >>
    (DateTime { date, time })
));

#[cfg(test)]
mod tests {
    use nom::Context::Code;
    use nom::Err::{Error, Incomplete};
    use nom::ErrorKind::{Alt, Verify};
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

        assert_eq!(month(b"06"), Ok((&[][..],  6)));
        assert_eq!(month(b"12"), Ok((&[][..], 12)));
        assert_eq!(month(b"13"), Err(
            Error(Code(&b"13"[..], Verify))
        ));
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
            assert_eq!(date(b"2015-07-16"), Ok((&[][..], value.clone())));
            assert_eq!(date(b"20150716"),   Ok((&[][..], value        )));
        }
        {
            let value = Date {
                year: -333,
                month: 6,
                day: 11
            };
            assert_eq!(date(b"-0333-06-11"), Ok((&[][..], value.clone())));
            assert_eq!(date(b"-03330611"),   Ok((&[][..], value        )));
        }
    }

    #[test]
    fn parse_hour() {
        use super::hour;

        assert_eq!(hour(b"02"), Ok((&[][..],  2)));
        assert_eq!(hour(b"24"), Ok((&[][..], 24)));
        assert_eq!(hour(b"25"), Err(
            Error(Code(&b"25"[..], Verify))
        ));
    }

    #[test]
    fn parse_minute() {
        use super::minute;

        assert_eq!(minute(b"02"), Ok((&[][..],  2)));
        assert_eq!(minute(b"60"), Ok((&[][..], 60)));
        assert_eq!(minute(b"61"), Err(
            Error(Code(&b"61"[..], Verify))
        ));
    }

    #[test]
    fn parse_second() {
        use super::second;

        assert_eq!(second(b"02"), Ok((&[][..],  2)));
        assert_eq!(second(b"60"), Ok((&[][..], 60)));
        assert_eq!(second(b"61"), Err(
            Error(Code(&b"61"[..], Verify))
        ));
    }

    #[test]
    fn parse_time() {
        use super::time;

        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                tz_offset: 0
            };
            assert_eq!(time(b"16:43:52"), Ok((&[][..], value.clone())));
            assert_eq!(time(b"164352"),   Ok((&[][..], value        )));
        }
        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 0,
                nanos: 0,
                tz_offset: 0
            };
            assert_eq!(time(b"16:43"), Ok((&[][..], value.clone())));
            assert_eq!(time(b"1643"),  Ok((&[][..], value        )));
        }
    }

    #[test]
    fn parse_time_precision() {
        use super::time;

        let value = Time {
            hour: 16,
            minute: 43,
            second: 52,
            nanos: 0,
            tz_offset: 0
        };
        assert_eq!(time(b"16:43:52.1"), Ok((
            &[][..], Time {
                nanos: 100_000_000,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.01"), Ok((
            &[][..], Time {
                nanos: 10_000_000,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.001"), Ok((
            &[][..], Time {
                nanos: 1_000_000,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.0001"), Ok((
            &[][..], Time {
                nanos: 100_000,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.00001"), Ok((
            &[][..], Time {
                nanos: 10_000,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.000001"), Ok((
            &[][..], Time {
                nanos: 1_000,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.0000001"), Ok((
            &[][..], Time {
                nanos: 100,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.00000001"), Ok((
            &[][..], Time {
                nanos: 10,
                ..value
            }
        )));
        assert_eq!(time(b"16:43:52.000000001"), Ok((
            &[][..], Time {
                nanos: 1,
                ..value
            }
        )));
    }

    #[test]
    #[should_panic]
    fn parse_time_precision_panic() {
        super::time(b"16:43:52.0000000001").unwrap();
    }

    #[test]
    fn parse_time_with_timezone() {
        use super::time;

        assert_eq!(time(b"16:43:52Z"), Ok((
            &[][..], Time {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                tz_offset: 0
            }
        )));
        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                tz_offset: 5 * 3600
            };
            assert_eq!(time(b"16:43:52+05"),   Ok((&[][..], value.clone())));
            assert_eq!(time(b"16:43:52+0500"), Ok((&[][..], value        )));
        }
        assert_eq!(time(b"16:43-05:30"), Ok((
            &[][..], Time {
                hour: 16,
                minute: 43,
                second: 0,
                nanos: 0,
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
                nanos: 0,
                tz_offset: 5 * 3600
            }
        };
        assert_eq!(datetime(b"2007-08-31T16:47:22+05:00"), Ok((&[][..], value.clone())));
        assert_eq!(datetime(b"20070831T164722+05"),        Ok((&[][..], value        )));
    }
}
