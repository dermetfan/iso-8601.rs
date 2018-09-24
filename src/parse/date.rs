use ::date::*;
use super::*;
use nom;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
