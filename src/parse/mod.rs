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

mod date;
mod time;
mod datetime;

pub use self::{
    date::*,
    time::*,
    datetime::*
};

use std::ops::{AddAssign, MulAssign};

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

named!(sign <&[u8], i8>, alt!(
    one_of!("-\u{2212}\u{2010}") => { |_| -1 } |
    char!('+')                   => { |_|  1 }
));

#[cfg(test)]
mod tests {
    use nom::{
        Context::Code,
        Err::{Error, Incomplete},
        ErrorKind::Alt,
        Needed::Size
    };

    #[test]
    fn sign() {
        assert_eq!(super::sign(b"-"), Ok((&[][..], -1)));
        assert_eq!(super::sign(b"+"), Ok((&[][..],  1)));
        assert_eq!(super::sign(b"" ), Err(Incomplete(Size(1))));
        assert_eq!(super::sign(b" "), Err(Error(Code(&b" "[..], Alt))));
    }
}
