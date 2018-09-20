use std::ops::{AddAssign, MulAssign};
use {nom, Date, YmdDate, WeekDate, Time, DateTime};

fn buf_to_int<T>(buf: &[u8]) -> T
where T: AddAssign + MulAssign + From<u8> {
    let mut sum = T::from(0);
    for digit in buf {
        sum *= T::from(10);
        sum += T::from(*digit - b'0');
    }
    sum
}

/// Returns ".`buf`" as unit 10^(-(`precision` + 1)).
///
/// Panics on greater than the given precision
/// (`buf.chars().count() >= precision + 1`).
fn buf_to_frac_int(buf: &[u8], precision: u8) -> u64 {
    let mut nanos = 0;
    for (i, digit) in buf.iter().enumerate() {
        let digit = digit - b'0';
        nanos += digit as u64 * 10u64.pow((precision - i as u8) as u32);
    }
    nanos
}

macro_rules! frac_int {
    ($i:expr, $precision:expr) => {
        complete!($i, do_parse!(
            one_of!(",.") >>
            frac: alt_complete!(
                take_while1!(nom::is_digit) |
                take_rest!()
            ) >>
            (buf_to_frac_int(frac, $precision))
        ))
    }
}

/// Takes the rest of the input until EOF.
macro_rules! take_rest {
    ($i:expr,) => ({
        use nom::InputLength;

        take!($i, $i.input_len())
    })
}

named!(sign <&[u8], i8>, alt!(
    one_of!("-\u{2212}\u{2010}") => { |_| -1 } |
    char!('+')                   => { |_|  1 }
));

named!(positive_century <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

named!(century <&[u8], i8>, do_parse!(
    sign: opt!(sign) >>
    century: positive_century >>
    (sign.unwrap_or(1) * century as i8)
));

// TODO support expanded year
named!(positive_year <&[u8], u16>, map!(
    take_while_m_n!(4, 4, nom::is_digit),
    buf_to_int
));

named!(year <&[u8], i16>, do_parse!(
    sign: opt!(sign) >>
    year: positive_year >>
    (sign.unwrap_or(1) as i16 * year as i16)
));

named!(month <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |month| month >= 1 && month <= 12
));

/// Not verified since number of days
/// in a month depends on the month.
named!(day <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

/// Not fully verified since number of weeks
/// in a year depends on the year.
named!(year_week <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |week| week >= 1
));

named!(week_day <&[u8], u8>, verify!(
    map!(take!(1), buf_to_int),
    |day| day >= 1 && day <= 7
));

named!(pub date <&[u8], Date>, verify!(
    alt_complete!(
        do_parse!(
            year: year >>
            opt!(char!('-')) >>
            char!('W') >>
            week: year_week >>
            day: opt!(complete!(do_parse!(
                opt!(char!('-')) >>
                day: week_day >>
                (day)
            ))) >>
            (Date::Week(WeekDate {
                year, week,
                day: day.unwrap_or(1)
            }))
        ) |
        do_parse!(
            year: year >>
            month_day: opt!(complete!(do_parse!(
                opt!(char!('-')) >>
                month: month >>
                day: opt!(complete!(do_parse!(
                    opt!(char!('-')) >>
                    day: day >>
                    (day)
                ))) >>
                ((
                    month,
                    day.unwrap_or(1)
                ))
            ))) >>
            (Date::YMD(YmdDate {
                year,
                month: month_day.map(|x| x.0).unwrap_or(1),
                day:   month_day.map(|x| x.1).unwrap_or(1)
            }))
        ) |
        do_parse!(
            century: century >>
            (Date::YMD(YmdDate {
                year: century as i16 * 100,
                month: 1,
                day: 1
            }))
        )
        // TODO ordinal
    ),
    |date: Date| {
        use ::Year;

        match date {
            Date::YMD(YmdDate { year, month, day }) => day >= 1 && day <= match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11              => 30,
                2 => if year.is_leap() { 29 } else { 28 },
                _ => unreachable!()
            },
            Date::Week(WeekDate { year, week, .. }) => week <= year.num_weeks(),
            _ => unimplemented!()
        }
    }
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
    |minute| minute <= 59
));

named!(second <&[u8], u8>, verify!(
    map!(
        take_while_m_n!(2, 2, nom::is_digit),
        buf_to_int
    ),
    |second| second <= 60
));

named!(pub time <&[u8], Time>, do_parse!(
    hour: hour >>
    minute_second_nanos: alt_complete!(
        do_parse!(
            nanos: frac_int!(10) >>
            ({
                let nanos = nanos * 6u64.pow(2);
                ((
                    (nanos / 60_000_000_000) as u8,
                    (nanos % 60_000_000_000 / 1_000_000_000) as u8,
                    nanos % 1_000_000_000
                ))
            })
        ) |
        do_parse!(
            minute: opt!(complete!(do_parse!(
                opt!(char!(':')) >>
                minute: minute >>
                (minute)
            ))) >>
            second_nanos: opt!(alt_complete!(
                do_parse!(
                    nanos: frac_int!(9) >>
                    ({
                        let nanos = nanos * 6;
                        ((
                            (nanos / 1_000_000_000) as u8,
                            nanos % 1_000_000_000
                        ))
                    })
                ) |
                do_parse!(
                    opt!(char!(':')) >>
                    second: second >>
                    nanos: opt!(frac_int!(8)) >>
                    ((
                        second,
                        nanos.unwrap_or(0)
                    ))
                )
            )) >>
            ((
                minute.unwrap_or(0),
                second_nanos.map(|x| x.0).unwrap_or(0),
                second_nanos.map(|x| x.1).unwrap_or(0)
            ))
        )
    ) >>
    tz_offset: opt!(complete!(timezone)) >>
    (Time {
        hour,
        minute: minute_second_nanos.0,
        second: minute_second_nanos.1,
        nanos:  minute_second_nanos.2 as u32,
        tz_offset: tz_offset.unwrap_or(0)
    })
));

named!(timezone_utc <&[u8], i16>, map!(
    char!('Z'), |_| 0
));

named!(timezone_fixed <&[u8], i16>, do_parse!(
    sign: sign >>
    hour: verify!(hour, |hour| hour < 24) >>
    minute: opt!(complete!(do_parse!(
        opt!(char!(':')) >>
        minute: minute >>
        (minute)
    ))) >>
    (sign as i16 * (hour as i16 * 60 + minute.unwrap_or(0) as i16))
));

named!(timezone <&[u8], i16>, alt!(timezone_utc | timezone_fixed));

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
    use nom::ErrorKind::{Alt, Char, Verify};
    use nom::Needed::Size;
    use {Date, YmdDate, WeekDate, Time, DateTime};

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
        assert_eq!(month(b"13"), Err(Error(Code(&b"13"[..], Verify))));
        assert_eq!(month(b"00"), Err(Error(Code(&b"00"[..], Verify))));
    }

    #[test]
    fn parse_year_week() {
        use super::year_week;

        assert_eq!(year_week(b"01"), Ok((&[][..], 1)));
        assert_eq!(year_week(b"00"), Err(Error(Code(&b"00"[..], Verify))));
    }

    #[test]
    fn parse_day() {
        use super::day;

        assert_eq!(day(b"18"), Ok((&[][..], 18)));
    }

    #[test]
    fn parse_week_day() {
        use super::week_day;

        assert_eq!(week_day(b"1"), Ok((&[][..], 1)));
        assert_eq!(week_day(b"2"), Ok((&[][..], 2)));
        assert_eq!(week_day(b"3"), Ok((&[][..], 3)));
        assert_eq!(week_day(b"4"), Ok((&[][..], 4)));
        assert_eq!(week_day(b"5"), Ok((&[][..], 5)));
        assert_eq!(week_day(b"6"), Ok((&[][..], 6)));
        assert_eq!(week_day(b"7"), Ok((&[][..], 7)));
        assert_eq!(week_day(b"0"), Err(Error(Code(&b"0"[..], Verify))));
        assert_eq!(week_day(b"8"), Err(Error(Code(&b"8"[..], Verify))));
    }

    #[test]
    fn parse_date() {
        use super::date;

        {
            let value = Date::YMD(YmdDate {
                year: 2015,
                month: 7,
                day: 16
            });
            assert_eq!(date(b"2015-07-16"), Ok((&[][..], value.clone())));
            assert_eq!(date(b"20150716"),   Ok((&[][..], value        )));
        }
        {
            let value = Date::YMD(YmdDate {
                year: -333,
                month: 6,
                day: 11
            });
            assert_eq!(date(b"-0333-06-11"), Ok((&[][..], value.clone())));
            assert_eq!(date(b"-03330611"),   Ok((&[][..], value        )));
        }
        assert_eq!(date(b"2018-02-29"), Err(Error(Code(&b"2018-02-29"[..], Verify))));
        assert_eq!(date(b"2016-02-29"), Ok((&[][..], Date::YMD(YmdDate {
            year: 2016,
            month: 2,
            day: 29
        }))));
        assert_eq!(date(b"2016-02"), Ok((&[][..], Date::YMD(YmdDate {
            year: 2016,
            month: 2,
            day: 1
        }))));
        assert_eq!(date(b"2016"), Ok((&[][..], Date::YMD(YmdDate {
            year: 2016,
            month: 1,
            day: 1
        }))));
        assert_eq!(date(b"20"), Ok((&[][..], Date::YMD(YmdDate {
            year: 2000,
            month: 1,
            day: 1
        }))));
    }

    #[test]
    fn parse_date_week() {
        use super::date;

        assert_eq!(date(b"2018-W01-1"), Ok((&[][..], Date::Week(WeekDate {
            year: 2018,
            week: 1,
            day: 1
        }))));
        assert_eq!(date(b"2018-W52-7"), Ok((&[][..], Date::Week(WeekDate {
            year: 2018,
            week: 52,
            day: 7
        }))));
        assert_eq!(date(b"2018W223"), Ok((&[][..], Date::Week(WeekDate {
            year: 2018,
            week: 22,
            day: 3
        }))));
        assert_eq!(date(b"2018W22"), Ok((&[][..], Date::Week(WeekDate {
            year: 2018,
            week: 22,
            day: 1
        }))));
        assert_eq!(date(b"2020-W53"), Ok((&[][..], Date::Week(WeekDate {
            year: 2020,
            week: 53,
            day: 1
        }))));
        assert_eq!(date(b"2018-W53"), Err(Error(Code(&b"2018-W53"[..], Verify))));
    }

    #[test]
    fn parse_hour() {
        use super::hour;

        assert_eq!(hour(b"02"), Ok((&[][..],  2)));
        assert_eq!(hour(b"24"), Ok((&[][..], 24)));
        assert_eq!(hour(b"25"), Err(Error(Code(&b"25"[..], Verify))));
    }

    #[test]
    fn parse_minute() {
        use super::minute;

        assert_eq!(minute(b"02"), Ok((&[][..],  2)));
        assert_eq!(minute(b"59"), Ok((&[][..], 59)));
        assert_eq!(minute(b"60"), Err(Error(Code(&b"60"[..], Verify))));
    }

    #[test]
    fn parse_second() {
        use super::second;

        assert_eq!(second(b"02"), Ok((&[][..],  2)));
        assert_eq!(second(b"60"), Ok((&[][..], 60)));
        assert_eq!(second(b"61"), Err(Error(Code(&b"61"[..], Verify))));
    }

    #[test]
    fn parse_timezone_fixed() {
        use super::timezone_fixed;

        assert_eq!(timezone_fixed(b"+23:59"), Ok((&[][..], 23 * 60 + 59)));
        assert_eq!(timezone_fixed(b"+2359"),  Ok((&[][..], 23 * 60 + 59)));
        assert_eq!(timezone_fixed(b"+23"),    Ok((&[][..], 23 * 60     )));
        assert_eq!(timezone_fixed(b"+24:00"), Err(Error(Code(&b"24:00"[..], Verify))));
        assert_eq!(timezone_fixed(b"-24:00"), Err(Error(Code(&b"24:00"[..], Verify))));
    }

    #[test]
    fn parse_timezone_utc() {
        use super::timezone_utc;

        assert_eq!(timezone_utc(b"Z"), Ok((&[][..], 0)));
        assert_eq!(timezone_utc(b"z"), Err(Error(Code(&b"z"[..], Char))));
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
        assert_eq!(time(b"16"), Ok((&[][..], Time {
            hour: 16,
            minute: 0,
            second: 0,
            nanos: 0,
            tz_offset: 0
        })));
    }

    #[test]
    fn parse_time_precision() {
        use super::time;

        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                tz_offset: 0
            };
            assert_eq!(time(b"16:43:52.1"), Ok((&[][..], Time {
                nanos: 100_000_000,
                ..value
            })));
            assert_eq!(time(b"16:43:52,01"), Ok((&[][..], Time {
                nanos: 10_000_000,
                ..value
            })));
            assert_eq!(time(b"16:43:52.001"), Ok((&[][..], Time {
                nanos: 1_000_000,
                ..value
            })));
            assert_eq!(time(b"16:43:52,0001"), Ok((&[][..], Time {
                nanos: 100_000,
                ..value
            })));
            assert_eq!(time(b"16:43:52.00001"), Ok((&[][..], Time {
                nanos: 10_000,
                ..value
            })));
            assert_eq!(time(b"16:43:52,000001"), Ok((&[][..], Time {
                nanos: 1_000,
                ..value
            })));
            assert_eq!(time(b"16:43:52.0000001"), Ok((&[][..], Time {
                nanos: 100,
                ..value
            })));
            assert_eq!(time(b"16:43:52,00000001"), Ok((&[][..], Time {
                nanos: 10,
                ..value
            })));
            assert_eq!(time(b"16:43:52.000000001"), Ok((&[][..], Time {
                nanos: 1,
                ..value
            })));
        }
        assert_eq!(time(b"16:43.1234567891"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 7,
            nanos: 407_407_346,
            tz_offset: 0
        })));
        assert_eq!(time(b"16.12345678901"), Ok((&[][..], Time {
            hour: 16,
            minute: 7,
            second: 24,
            nanos: 444_440_436,
            tz_offset: 0
        })));
    }

    #[test]
    #[should_panic]
    fn parse_time_precision_panic() {
        super::time(b"16:43:52.0000000001").unwrap();
    }

    #[test]
    fn parse_time_with_timezone() {
        use super::time;

        assert_eq!(time(b"16:43:52Z"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 52,
            nanos: 0,
            tz_offset: 0
        })));
        assert_eq!(time(b"16:43:52.1Z"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 52,
            nanos: 100_000_000,
            tz_offset: 0
        })));
        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                tz_offset: 5 * 60
            };
            assert_eq!(time(b"16:43:52+05"),   Ok((&[][..], value.clone())));
            assert_eq!(time(b"16:43:52+0500"), Ok((&[][..], value        )));
        }
        assert_eq!(time(b"16:43-05:32"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 0,
            nanos: 0,
            tz_offset: -(5 * 60 + 32)
        })));
        assert_eq!(time(b"16:43+23:59"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 0,
            nanos: 0,
            tz_offset: 23 * 60 + 59
        })));
    }

    #[test]
    fn parse_datetime() {
        use super::datetime;

        let value = DateTime {
            date: Date::YMD(YmdDate {
                year: 2007,
                month: 8,
                day: 31
            }),
            time: Time {
                hour: 16,
                minute: 47,
                second: 22,
                nanos: 0,
                tz_offset: 5 * 60
            }
        };
        assert_eq!(datetime(b"2007-08-31T16:47:22+05:00"), Ok((&[][..], value.clone())));
        assert_eq!(datetime(b"20070831T164722+05"),        Ok((&[][..], value        )));
    }
}
