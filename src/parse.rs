use std::ops::{AddAssign, MulAssign};
use {nom, Date, YmdDate, WeekDate, OrdinalDate, LocalTime, Time, DateTime};

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

named!(month <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

named!(day <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

named!(year_week <&[u8], u8>, map!(
    take_while_m_n!(2, 2, nom::is_digit),
    buf_to_int
));

named!(year_day <&[u8], u8>, map!(
    take_while_m_n!(3, 3, nom::is_digit),
    buf_to_int
));

named!(week_day <&[u8], u8>, map!(
    take!(1),
    buf_to_int
));

named!(date_ymd <&[u8], YmdDate>, alt_complete!(
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
        (YmdDate {
            year,
            month: month_day.map(|x| x.0).unwrap_or(1),
            day:   month_day.map(|x| x.1).unwrap_or(1)
        })
    ) |
    do_parse!(
        century: century >>
        (YmdDate {
            year: century as i16 * 100,
            month: 1,
            day: 1
        })
    )
));

named!(date_week <&[u8], WeekDate>, do_parse!(
    year: year >>
    opt!(char!('-')) >>
    char!('W') >>
    week: year_week >>
    day: opt!(complete!(do_parse!(
        opt!(char!('-')) >>
        day: week_day >>
        (day)
    ))) >>
    (WeekDate {
        year, week,
        day: day.unwrap_or(1)
    })
));

named!(date_ordinal <&[u8], OrdinalDate>, do_parse!(
    year: year >>
    opt!(char!('-')) >>
    day: year_day >>
    (OrdinalDate {
        year,
        day: day.into()
    })
));

named!(pub date <&[u8], Date>, alt_complete!(
    do_parse!(
        date: date_week >>
        (Date::Week(date))
    ) |
    do_parse!(
        peek!(re_bytes_match!(r"^\d{4}-?\d{3}$")) >>
        date: date_ordinal >>
        (Date::Ordinal(date))
    ) |
    do_parse!(
        date: date_ymd >>
        (Date::YMD(date))
    )
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

named!(time_naive <&[u8], LocalTime>, do_parse!(
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
    (LocalTime {
        hour,
        minute: minute_second_nanos.0,
        second: minute_second_nanos.1,
        nanos:  minute_second_nanos.2 as u32
    })
));

named!(pub time_local <&[u8], LocalTime>, do_parse!(
    opt!(char!('T')) >>
    time: time_naive >>
    (time)
));

named!(pub time <&[u8], Time>, do_parse!(
    local: time_naive >>
    tz_offset: complete!(timezone) >>
    (Time {
        local: local,
        tz_offset: tz_offset
    })
));

named!(timezone_utc <&[u8], i16>, map!(
    char!('Z'), |_| 0
));

named!(timezone_fixed <&[u8], i16>, do_parse!(
    sign: sign >>
    hour: hour >>
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
    use nom::ErrorKind::{Alt, Char};
    use nom::Needed::Size;
    use {Date, YmdDate, WeekDate, OrdinalDate, LocalTime, Time, DateTime};

    #[test]
    fn sign() {
        assert_eq!(super::sign(b"-"), Ok((&[][..], -1)));
        assert_eq!(super::sign(b"+"), Ok((&[][..],  1)));
        assert_eq!(super::sign(b"" ), Err(Incomplete(Size(1))));
        assert_eq!(super::sign(b" "), Err(Error(Code(&b" "[..], Alt))));
    }

    #[test]
    fn positive_year() {
        assert_eq!(super::positive_year(b"2018"), Ok((&[][..], 2018)));
    }

    #[test]
    fn year() {
        assert_eq!(super::year(b"2018" ), Ok((&[][..],  2018)));
        assert_eq!(super::year(b"+2018"), Ok((&[][..],  2018)));
        assert_eq!(super::year(b"-2018"), Ok((&[][..], -2018)));
    }

    #[test]
    fn month() {
        assert_eq!(super::month(b"06"), Ok((&[][..],  6)));
        assert_eq!(super::month(b"12"), Ok((&[][..], 12)));
    }

    #[test]
    fn year_week() {
        assert_eq!(super::year_week(b"01"), Ok((&[][..], 1)));
    }

    #[test]
    fn year_day() {
        assert_eq!(super::year_day(b"001"),  Ok((&[][..],     1)));
        assert_eq!(super::year_day(b"011"),  Ok((&[][..],    11)));
        assert_eq!(super::year_day(b"111"),  Ok((&[][..],   111)));
        assert_eq!(super::year_day(b"1111"), Ok((&b"1"[..], 111)));
    }

    #[test]
    fn day() {
        assert_eq!(super::day(b"18"), Ok((&[][..], 18)));
    }

    #[test]
    fn week_day() {
        assert_eq!(super::week_day(b"1"), Ok((&[][..], 1)));
        assert_eq!(super::week_day(b"2"), Ok((&[][..], 2)));
        assert_eq!(super::week_day(b"3"), Ok((&[][..], 3)));
        assert_eq!(super::week_day(b"4"), Ok((&[][..], 4)));
        assert_eq!(super::week_day(b"5"), Ok((&[][..], 5)));
        assert_eq!(super::week_day(b"6"), Ok((&[][..], 6)));
        assert_eq!(super::week_day(b"7"), Ok((&[][..], 7)));
    }

    #[test]
    fn date_ymd() {
        {
            let value = YmdDate {
                year: 2015,
                month: 7,
                day: 16
            };
            assert_eq!(super::date_ymd(b"2015-07-16"), Ok((&[][..], value.clone())));
            assert_eq!(super::date_ymd(b"20150716"),   Ok((&[][..], value        )));
        }
        {
            let value = YmdDate {
                year: -333,
                month: 6,
                day: 11
            };
            assert_eq!(super::date_ymd(b"-0333-06-11"), Ok((&[][..], value.clone())));
            assert_eq!(super::date_ymd(b"-03330611"),   Ok((&[][..], value        )));
        }
        assert_eq!(super::date_ymd(b"2016-02-29"), Ok((&[][..], YmdDate {
            year: 2016,
            month: 2,
            day: 29
        })));
        assert_eq!(super::date_ymd(b"2016-02"), Ok((&[][..], YmdDate {
            year: 2016,
            month: 2,
            day: 1
        })));
        assert_eq!(super::date_ymd(b"2016"), Ok((&[][..], YmdDate {
            year: 2016,
            month: 1,
            day: 1
        })));
        assert_eq!(super::date_ymd(b"20"), Ok((&[][..], YmdDate {
            year: 2000,
            month: 1,
            day: 1
        })));
    }

    #[test]
    fn date_week() {
        assert_eq!(super::date_week(b"2018-W01-1"), Ok((&[][..], WeekDate {
            year: 2018,
            week: 1,
            day: 1
        })));
        assert_eq!(super::date_week(b"2018-W52-7"), Ok((&[][..], WeekDate {
            year: 2018,
            week: 52,
            day: 7
        })));
        assert_eq!(super::date_week(b"2018W223"), Ok((&[][..], WeekDate {
            year: 2018,
            week: 22,
            day: 3
        })));
        assert_eq!(super::date_week(b"2018W22"), Ok((&[][..], WeekDate {
            year: 2018,
            week: 22,
            day: 1
        })));
        assert_eq!(super::date_week(b"2020-W53"), Ok((&[][..], WeekDate {
            year: 2020,
            week: 53,
            day: 1
        })));
    }

    #[test]
    fn date_ordinal() {
        let value = OrdinalDate {
            year: 1985,
            day: 102
        };
        assert_eq!(super::date_ordinal(b"1985-102"), Ok((&[][..], value.clone())));
        assert_eq!(super::date_ordinal(b"1985102"),  Ok((&[][..], value        )));
    }

    #[test]
    fn date() {
        assert_eq!(super::date(b"2018-02-12"), Ok((&[][..], Date::YMD(YmdDate {
            year: 2018,
            month: 2,
            day: 12
        }))));

        assert_eq!(super::date(b"2018-W02-2"), Ok((&[][..], Date::Week(WeekDate {
            year: 2018,
            week: 2,
            day: 2
        }))));

        assert_eq!(super::date(b"2018-102"), Ok((&[][..], Date::Ordinal(OrdinalDate {
            year: 2018,
            day: 102
        }))));
    }

    #[test]
    fn hour() {
        assert_eq!(super::hour(b"02"), Ok((&[][..],  2)));
        assert_eq!(super::hour(b"24"), Ok((&[][..], 24)));
    }

    #[test]
    fn minute() {
        assert_eq!(super::minute(b"02"), Ok((&[][..],  2)));
        assert_eq!(super::minute(b"59"), Ok((&[][..], 59)));
    }

    #[test]
    fn second() {
        assert_eq!(super::second(b"02"), Ok((&[][..],  2)));
        assert_eq!(super::second(b"60"), Ok((&[][..], 60)));
    }

    #[test]
    fn timezone_fixed() {
        assert_eq!(super::timezone_fixed(b"+23:59"), Ok((&[][..], 23 * 60 + 59)));
        assert_eq!(super::timezone_fixed(b"+2359"),  Ok((&[][..], 23 * 60 + 59)));
        assert_eq!(super::timezone_fixed(b"+23"),    Ok((&[][..], 23 * 60     )));
    }

    #[test]
    fn timezone_utc() {
        assert_eq!(super::timezone_utc(b"Z"), Ok((&[][..], 0)));
        assert_eq!(super::timezone_utc(b"z"), Err(Error(Code(&b"z"[..], Char))));
    }

    #[test]
    fn time_naive() {
        {
            let value = LocalTime {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0
            };
            assert_eq!(super::time_naive(b"16:43:52"), Ok((&[][..], value.clone())));
            assert_eq!(super::time_naive(b"164352"),   Ok((&[][..], value        )));
        }
        {
            let value = LocalTime {
                hour: 16,
                minute: 43,
                second: 0,
                nanos: 0
            };
            assert_eq!(super::time_naive(b"16:43"), Ok((&[][..], value.clone())));
            assert_eq!(super::time_naive(b"1643"),  Ok((&[][..], value        )));
        }
        assert_eq!(super::time_naive(b"16"), Ok((&[][..], LocalTime {
            hour: 16,
            minute: 0,
            second: 0,
            nanos: 0
        })));
    }

    #[test]
    fn time_naive_precision() {
        {
            let value = LocalTime {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0
            };
            assert_eq!(super::time_naive(b"16:43:52.1"), Ok((&[][..], LocalTime {
                nanos: 100_000_000,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52,01"), Ok((&[][..], LocalTime {
                nanos: 10_000_000,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52.001"), Ok((&[][..], LocalTime {
                nanos: 1_000_000,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52,0001"), Ok((&[][..], LocalTime {
                nanos: 100_000,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52.00001"), Ok((&[][..], LocalTime {
                nanos: 10_000,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52,000001"), Ok((&[][..], LocalTime {
                nanos: 1_000,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52.0000001"), Ok((&[][..], LocalTime {
                nanos: 100,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52,00000001"), Ok((&[][..], LocalTime {
                nanos: 10,
                ..value
            })));
            assert_eq!(super::time_naive(b"16:43:52.000000001"), Ok((&[][..], LocalTime {
                nanos: 1,
                ..value
            })));
        }
        assert_eq!(super::time_naive(b"16:43.1234567891"), Ok((&[][..], LocalTime {
            hour: 16,
            minute: 43,
            second: 7,
            nanos: 407_407_346
        })));
        assert_eq!(super::time_naive(b"16.12345678901"), Ok((&[][..], LocalTime {
            hour: 16,
            minute: 7,
            second: 24,
            nanos: 444_440_436
        })));
    }

    #[test]
    #[should_panic]
    fn time_naive_precision_panic() {
        super::time_naive(b"16:43:52.0000000001").unwrap();
    }

    #[test]
    fn time_local() {
        let value = LocalTime {
            hour: 2,
            minute: 22,
            second: 22,
            nanos: 0
        };
        assert_eq!(super::time_local(b"T02:22:22"), Ok((&[][..], value.clone())));
        assert_eq!(super::time_local(b"02:22:22"),  Ok((&[][..], value.clone())));
        assert_eq!(super::time_local(b"T022222"),   Ok((&[][..], value.clone())));
        assert_eq!(super::time_local(b"022222"),    Ok((&[][..], value        )));
    }

    #[test]
    fn time() {
        assert_eq!(super::time(b"16:43:52Z"), Ok((&[][..], Time {
            local: LocalTime {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0
            },
            tz_offset: 0
        })));
        assert_eq!(super::time(b"16:43:52.1Z"), Ok((&[][..], Time {
            local: LocalTime {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 100_000_000
            },
            tz_offset: 0
        })));
        {
            let value = Time {
                local: LocalTime {
                    hour: 16,
                    minute: 43,
                    second: 52,
                    nanos: 0
                },
                tz_offset: 5 * 60
            };
            assert_eq!(super::time(b"16:43:52+05"),   Ok((&[][..], value.clone())));
            assert_eq!(super::time(b"16:43:52+0500"), Ok((&[][..], value        )));
        }
        assert_eq!(super::time(b"16:43-05:32"), Ok((&[][..], Time {
            local: LocalTime {
                hour: 16,
                minute: 43,
                second: 0,
                nanos: 0
            },
            tz_offset: -(5 * 60 + 32)
        })));
        assert_eq!(super::time(b"16:43+23:59"), Ok((&[][..], Time {
            local: LocalTime {
                hour: 16,
                minute: 43,
                second: 0,
                nanos: 0
            },
            tz_offset: 23 * 60 + 59
        })));
    }

    #[test]
    fn datetime() {
        let value = DateTime {
            date: Date::YMD(YmdDate {
                year: 2007,
                month: 8,
                day: 31
            }),
            time: Time {
                local: LocalTime {
                    hour: 16,
                    minute: 47,
                    second: 22,
                    nanos: 0
                },
                tz_offset: 5 * 60
            }
        };
        assert_eq!(super::datetime(b"2007-08-31T16:47:22+05:00"), Ok((&[][..], value.clone())));
        assert_eq!(super::datetime(b"20070831T164722+05"),        Ok((&[][..], value        )));
    }
}
