mod date;
mod time;
mod datetime;

pub use self::{
    date::*,
    time::*,
    datetime::*
};

use {
    std::ops::{
        AddAssign,
        MulAssign
    },
    nom::{
        self,
        IResult,
        types::CompleteByteSlice
    }
};

fn buf_to_int<T>(buf: &[u8]) -> T
where T: AddAssign + MulAssign + From<u8> {
    let mut sum = T::from(0);
    for digit in buf {
        sum *= T::from(10);
        sum += T::from(*digit - b'0');
    }
    sum
}

named!(complete_float <CompleteByteSlice, f32>, flat_map!(
    nom::recognize_float, parse_to!(f32)
));

fn complete_float_bytes(i: &[u8]) -> IResult<&[u8], f32> {
    complete_float(CompleteByteSlice(i))
        .map(|(i, o)| (*i, o))
        .map_err(|e| {
            use nom::{
                Context::Code,
                Err::*
            };

            match e {
                Incomplete(n)       => Incomplete(n),
                Error  (Code(i, k)) => Error  (Code(*i, k)),
                Failure(Code(i, k)) => Failure(Code(*i, k))
            }
        })
}

named!(sign <i8>, alt!(
    one_of!("-\u{2212}\u{2010}") => { |_| -1 } |
    char!('+')                   => { |_|  1 }
));

named!(frac32 <f32>, do_parse!(
    peek!(char!('.')) >>
    fraction: complete_float_bytes >>
    (fraction)
));

#[cfg(test)]
mod tests {
    use nom::{
        Context::Code,
        Err::{
            Error,
            Incomplete
        },
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
