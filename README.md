# Fully Compliant ISO 8601 Parser

The go-to library for all your date and time parsing needs.

Any unimplemented notation the standard supports is considered a bug.

Chrono support is included.

## Roadmap

Version 1.0.0 will be reached when the entire standard is implemented.

- [x] calendar dates
- [x] week dates
- [x] ordinal dates
- [x] approximate dates
- [x] local time
- [x] global time / time zone offsets
- [x] time decimal fractions
- [x] approximate time
- [x] datetime
- [ ] durations
- [ ] intervals
- [ ] recurring intervals

Chrono support is very limited at the moment.
Contributions would very welcome, should be a low-hanging fruit.

## Examples

Basics:

```rust
use std::str::FromStr;
use iso_8601::*;

assert_eq!(
    Date::from_str("2018-08-02").unwrap(),
    Date::YMD(YmdDate {
        year: 2018,
        month: 8,
        day: 2,
    })
);

assert_eq!(
    LocalTime::from_str("13:42").unwrap(),
    LocalTime {
        naive: HmTime {
            hour: 13,
            minute: 42,
        },
        fraction: 0.,
    }
);
```

Chrono support:

```rust
extern crate chrono;

use std::str::FromStr;
use iso_8601::*;

fn main() {
    assert_eq!(
        chrono::DateTime::<chrono::Utc>::from(
            DateTime::<Date, GlobalTime>::from_str("2018-08-02T13:42:02Z").unwrap()
        ),
        chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd(2018, 8, 2),
                chrono::NaiveTime::from_hms(13, 42, 2)
            ),
            chrono::Utc
        )
    );
}
```
