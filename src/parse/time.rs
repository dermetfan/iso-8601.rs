use ::time::*;
use super::*;
use nom;

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
        nanos:  minute_second_nanos.2 as u32,
        timezone: ()
    })
));

named!(pub time_local <&[u8], LocalTime>, do_parse!(
    opt!(char!('T')) >>
    time: time_naive >>
    (time)
));

named!(pub time <&[u8], Time>, do_parse!(
    time: time_naive >>
    timezone: complete!(timezone) >>
    (Time {
        hour: time.hour,
        minute: time.minute,
        second: time.second,
        nanos: time.nanos,
        timezone: timezone
    })
));

named!(pub time_any <&[u8], AnyTime>, alt!(
    do_parse!(
        time: time >>
        (AnyTime::Global(time))
    ) |
    do_parse!(
        time: time_local >>
        (AnyTime::Local(time))
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        Context::Code,
        Err::Error,
        ErrorKind::Char
    };

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
                nanos: 0,
                timezone: ()
            };
            assert_eq!(super::time_naive(b"16:43:52"), Ok((&[][..], value.clone())));
            assert_eq!(super::time_naive(b"164352"),   Ok((&[][..], value        )));
        }
        {
            let value = LocalTime {
                hour: 16,
                minute: 43,
                second: 0,
                nanos: 0,
                timezone: ()
            };
            assert_eq!(super::time_naive(b"16:43"), Ok((&[][..], value.clone())));
            assert_eq!(super::time_naive(b"1643"),  Ok((&[][..], value        )));
        }
        assert_eq!(super::time_naive(b"16"), Ok((&[][..], LocalTime {
            hour: 16,
            minute: 0,
            second: 0,
            nanos: 0,
            timezone: ()
        })));
    }

    #[test]
    fn time_naive_precision() {
        {
            let value = LocalTime {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                timezone: ()
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
            nanos: 407_407_346,
            timezone: ()
        })));
        assert_eq!(super::time_naive(b"16.12345678901"), Ok((&[][..], LocalTime {
            hour: 16,
            minute: 7,
            second: 24,
            nanos: 444_440_436,
            timezone: ()
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
            nanos: 0,
            timezone: ()
        };
        assert_eq!(super::time_local(b"T02:22:22"), Ok((&[][..], value.clone())));
        assert_eq!(super::time_local(b"02:22:22"),  Ok((&[][..], value.clone())));
        assert_eq!(super::time_local(b"T022222"),   Ok((&[][..], value.clone())));
        assert_eq!(super::time_local(b"022222"),    Ok((&[][..], value        )));
    }

    #[test]
    fn time() {
        assert_eq!(super::time(b"16:43:52Z"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 52,
            nanos: 0,
            timezone: 0
        })));
        assert_eq!(super::time(b"16:43:52.1Z"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 52,
            nanos: 100_000_000,
            timezone: 0
        })));
        {
            let value = Time {
                hour: 16,
                minute: 43,
                second: 52,
                nanos: 0,
                timezone: 5 * 60
            };
            assert_eq!(super::time(b"16:43:52+05"),   Ok((&[][..], value.clone())));
            assert_eq!(super::time(b"16:43:52+0500"), Ok((&[][..], value        )));
        }
        assert_eq!(super::time(b"16:43-05:32"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 0,
            nanos: 0,
            timezone: -(5 * 60 + 32)
        })));
        assert_eq!(super::time(b"16:43+23:59"), Ok((&[][..], Time {
            hour: 16,
            minute: 43,
            second: 0,
            nanos: 0,
            timezone: 23 * 60 + 59
        })));
    }

    #[test]
    fn time_any() {
        assert_eq!(super::time_any(b"16:43"), Ok((&[][..], AnyTime::Local(LocalTime {
            hour: 16,
            minute: 43,
            second: 0,
            nanos: 0,
            timezone: ()
        }))));
        assert_eq!(super::time_any(b"T16:43"), Ok((&[][..], AnyTime::Local(LocalTime {
            hour: 16,
            minute: 43,
            second: 0,
            nanos: 0,
            timezone: ()
        }))));

        assert_eq!(super::time_any(b"02:03Z"), Ok((&[][..], AnyTime::Global(Time {
            hour: 2,
            minute: 3,
            second: 0,
            nanos: 0,
            timezone: 0
        }))));
        assert_eq!(super::time_any(b"02:03-01"), Ok((&[][..], AnyTime::Global(Time {
            hour: 2,
            minute: 3,
            second: 0,
            nanos: 0,
            timezone: -1 * 60
        }))));

        assert_eq!(super::time_any(b"T12:23Z"), Ok((&b"Z"[..], AnyTime::Local(LocalTime {
            hour: 12,
            minute: 23,
            second: 0,
            nanos: 0,
            timezone: ()
        }))));
    }
}
