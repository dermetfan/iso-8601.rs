use ::datetime::*;
use super::*;

named!(pub datetime <&[u8], DateTime>, do_parse!(
    date: date >>
    char!('T') >>
    time: time >>
    (DateTime { date, time })
));

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::*;
    use ::date::*;

    #[test]
    fn datetime() {
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
                timezone: 5 * 60
            }
        };
        assert_eq!(super::datetime(b"2007-08-31T16:47:22+05:00"), Ok((&[][..], value.clone())));
        assert_eq!(super::datetime(b"20070831T164722+05"),        Ok((&[][..], value        )));
    }
}
