use nom::combinator::map;
use nom::Parser;

use crate::error::{Error, ParseError};

use super::{noerr, IResult, Input};

pub trait Tuple<O>: Sized {
    fn parse<'a>(&mut self, input: Input<'a>) -> IResult<'a, O>;
}

impl Tuple<()> for () {
    fn parse<'a>(&mut self, input: Input<'a>) -> IResult<'a, ()> {
        let (input, res) = nom::sequence::Tuple::parse(self, input)?;
        Ok((input, (res, vec![])))
    }
}

impl<T, PT> Tuple<(T,)> for (PT,)
where
    PT: for<'a> Parser<Input<'a>, (T, Vec<Error>), ParseError>,
{
    fn parse<'a>(&mut self, input: Input<'a>) -> IResult<'a, (T,)> {
        map(|input|nom::sequence::Tuple::parse(self, input), |((t1, e1),)| ((t1,), e1))(input)
    }
}

macro_rules! impl_tuple {
    ($t:ident $tp:ident: $e:ident, $($ti:ident $tpi:ident: $ei:ident),*) => {
       #[allow(non_snake_case)]
       impl<$($ti,)* $t, $($tpi,)* $tp> Tuple<($($ti,)* $t)> for ($($tpi,)* $tp)
       where
           $($tpi: for<'a> Parser<Input<'a>, ($ti, Vec<Error>), ParseError>,)*
           $tp: for<'a> Parser<Input<'a>, ($t, Vec<Error>), ParseError>,
       {
           fn parse<'a>(&mut self, input: Input<'a>) -> IResult<'a, ($($ti,)* $t)> {
               map(|input| nom::sequence::Tuple::parse(self, input), |($(($ti, $ei),)* ($t, $e))| {
                   let err = [$($ei,)* $e].concat();
                   (($($ti,)* $t), err)
               })(input)
           }
       }

       impl_tuple!($($ti $tpi: $ei),*);
    };
    ($t:ident $tp:ident: $e:ident) => {};
}

impl_tuple!(T1 TP1: e1, T2 TP2: e2, T3 TP3: e3, T4 TP4: e4, T5 TP5: e5, T6 TP6: e6, T7 TP7: e7, T8 TP8: e8, T9 TP9: e9, T10 TP10: e10);

pub fn tuplee<'a, O>(mut parsers: impl Tuple<O>) -> impl FnMut(Input<'a>) -> IResult<O> {    
    move |input| parsers.parse(input)
}

pub trait CollectErrTuple {
    type Output;

    fn collect_err(self) -> Self::Output;
}

impl CollectErrTuple for () {
    type Output = (Vec<Error>,);

    fn collect_err(self) -> Self::Output {
        (vec![],)
    }
}   

impl<T> CollectErrTuple for ((T, Vec<Error>),) {
    type Output = (T, Vec<Error>);

    fn collect_err(self) -> Self::Output {
        let ((t, e),) = self;
        (t, e)
    }   
}

macro_rules! impl_collect_err {
    ($v:ident, $e:ident: $t:ident) => {};
    ($v:ident, $e:ident: $t:ident, $($vs:ident, $es:ident: $ts:ident),*) => {
        impl<$t, $($ts),*> CollectErrTuple for (($t, Vec<Error>), $(($ts, Vec<Error>)),*)
        {
            type Output = ($t, $($ts,)* Vec<Error>);

            fn collect_err(self) -> Self::Output {
                let (($v, $e), $(($vs, $es)),*) = self;
                ($v, $($vs,)* [$e, $($es,)*].concat())
            }
        }

        impl_collect_err!($($vs, $es: $ts),*);
    }
}

impl_collect_err!(v1, e1: T1, v2, e2: T2, v3, e3: T3, v4, e4: T4, v5, e5: T5, v6, e6: T6, v7, e7: T7, v8, e8: T8, v9, e9: T9, v10, e10: T10);

pub fn collect_err<T: CollectErrTuple>(t: T) -> T::Output {
    t.collect_err()
}