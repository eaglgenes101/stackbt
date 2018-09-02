/// A three-valued logical value representing the current state of a behavior 
/// node, along with a corresponding decision value

use std::ops::Not;
use std::fmt::{Display, Formatter, Error};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BehaviorValue<S, R, F> {
    Success(S),
    Running(R),
    Failure(F)
}

impl <S, R, F> Not for BehaviorValue<S, R, F> {
    type Output = BehaviorValue<F, R, S>;
    fn not(self) -> BehaviorValue<F, R, S> {
        match self {
            BehaviorValue::Success(s) => BehaviorValue::Failure(s),
            BehaviorValue::Running(r) => BehaviorValue::Running(r),
            BehaviorValue::Failure(f) => BehaviorValue::Success(f)
        }
    }
}

impl <S, R, F> Display for BehaviorValue<S, R, F> where 
    S: Display, 
    R: Display,
    F: Display, 
{
    fn fmt(&self, fmter: &mut Formatter) -> Result<(), Error> {
        //Debug looks okay, so why not
        match self {
            BehaviorValue::Success(s) => {
                fmter.write_fmt(format_args!("Success({})", s))
            },
            BehaviorValue::Running(r) => {
                fmter.write_fmt(format_args!("Running({})", r))
            },
            BehaviorValue::Failure(f) => {
                fmter.write_fmt(format_args!("Failure({})", f))
            },
        }
    }
}