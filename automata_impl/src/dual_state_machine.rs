use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;
use std::mem::swap;

/// State machine implementation which combines the changing functions of 
/// ref_state_machine with the internal mutable state of 
/// internal_state_machine. This is the most general state machine form in 
/// this crate, but the other two are generally easier to work with. 
#[derive(Copy, Clone)]
pub struct DualStateMachine<'k, I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> (A, C)> + 'k,
    S: 'k,
    I: 'k
{
    state_fn: Option<C>, 
    internal: S,
    _i_exists: PhantomData<&'k I>,
    _a_exists: PhantomData<A>
}

impl<'k, I, S, A, C> DualStateMachine<'k, I, S, A, C> where
    C: Into<fn(&I, &mut S) -> (A, C)> + 'k,
    S: 'k
{
    pub fn new(calling_fn: C, init_state: S) -> DualStateMachine<'k, I, S, A, C> {
        DualStateMachine {
            state_fn: Option::Some(calling_fn),
            internal: init_state,
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
}

#[inline]
fn dual_state_transition<'k, I, S, C, A>(fn_r: &mut Option<C>, st_r: &mut S, in_r: &I) 
    -> A where
    C: Into<fn(&I, &mut S) -> (A, C)> + 'k,
    S: 'k
{
    let mut out_fn = Option::None;
    swap(fn_r, &mut out_fn);
    let (action, new_fn) = (out_fn.unwrap().into())(&in_r, st_r);
    *fn_r = Option::Some(new_fn);
    action
}

impl<'k, I, S, A, C> Automaton<'k> for DualStateMachine<'k, I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> (A, C)> + 'k,
    S: 'k
{
    type Input = I;
    type Action = A;
    #[inline]
    fn transition(&mut self, input: &I) -> A{
        dual_state_transition(&mut self.state_fn, &mut self.internal, &input)
    }
    
    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&I) -> A + 't> where 'k: 't {
        let mut state_fn_part = &mut self.state_fn;
        let mut internal_part = &mut self.internal;
        Box::new(move |input: &I| -> A {
            dual_state_transition(state_fn_part, internal_part, &input)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&I) -> A + 'k> {
        let mut state_fn_part = self.state_fn;
        let mut internal_part = self.internal;
        Box::new(move |input: &I| -> A {
            dual_state_transition(&mut state_fn_part, &mut internal_part, &input)
        })
    }
}

impl<'k, I, S, A, C> FiniteStateAutomaton<'k> for DualStateMachine<'k, I, S, A, C> where 
    S: Copy + 'k,
    C: Into<fn(&I, &mut S) -> (A, C)> + Copy + 'k
{}

mod tests {
    #[derive(Copy, Clone)]
    struct ThingBob {
        fn_ref: fn(&i64, &mut i64) -> (i64, ThingBob)
    }

    impl From<ThingBob> for fn(&i64, &mut i64) -> (i64, ThingBob) {
        fn from(fn_box: ThingBob) -> fn(&i64, &mut i64) -> (i64, ThingBob) {
            fn_box.fn_ref
        }
    }

    fn add_fn(add: &i64, sum: &mut i64) -> (i64, ThingBob) {
        if *add == 0 {
            (*sum, ThingBob { fn_ref: sub_fn })
        } else {
            *sum += add;
            (*sum, ThingBob { fn_ref: add_fn })
        }
    }

    fn sub_fn(sub: &i64, sum: &mut i64) -> (i64, ThingBob) {
        if *sub == 0 {
            (*sum, ThingBob { fn_ref: add_fn })
        } else {
            *sum -= sub;
            (*sum, ThingBob { fn_ref: sub_fn })
        }
    }

    #[test]
    fn check_def() {
        use dual_state_machine::DualStateMachine;
        use automaton::Automaton;
        let init_fn: fn(&i64, &mut i64) -> (i64, ThingBob) = add_fn;
        let mut x = DualStateMachine::new(ThingBob { fn_ref: init_fn }, 0);
        assert!(x.transition(&2) == 2);
        assert!(x.transition(&0) == 2);
        assert!(x.transition(&4) == -2);
        assert!(x.transition(&0) == -2);
        assert!(x.transition(&10) == 8);
    }
}