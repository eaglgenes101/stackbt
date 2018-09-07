use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;
use std::convert::Into;
use std::mem::swap;

/// State machine implemented through a closure reference wrapper struct. 
/// Each step, the currently referenced closure is called, returning an 
/// action and a reference to the closure to call for the next step. 
#[derive(Copy, Clone)]
pub struct RefStateMachine<'k, I, A, C> where 
    C: Into<fn(&I)->(A, C)> + 'k,
    I: 'k
{
    current_state: Option<C>,
    _i_exists: PhantomData<&'k I>,
    _a_exists: PhantomData<A>
}

impl <'k, I, A, C> RefStateMachine<'k, I, A, C> where 
    C: Into<fn(&I)->(A, C)> + 'k
{
    pub fn new(init_state: C) -> RefStateMachine<'k, I, A, C> {
        RefStateMachine {
            current_state: Option::Some(init_state),
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
}

#[inline]
fn ref_state_transition<'k, I, C, A>(fn_ref: &mut Option<C>, input: &I) 
    -> A where
    C: Into<fn(&I) -> (A, C)> + 'k
{
    let mut out_fn = Option::None;
    swap(fn_ref, &mut out_fn);
    let (action, new_fn) = (out_fn.unwrap().into())(&input);
    *fn_ref = Option::Some(new_fn);
    action
}

impl <'k, I, A, C> Automaton<'k> for RefStateMachine<'k, I, A, C> where
    C: Into<fn(&I)->(A, C)> + 'k
{
    type Input = I;
    type Action = A;
    #[inline]
    fn transition(&mut self, input: &I) -> A {
        ref_state_transition(&mut self.current_state, &input)
    }
    
    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&I) -> A + 't> where 'k: 't {
        let mut state_fn_part = &mut self.current_state;
        Box::new(move |input: &I| -> A {
            ref_state_transition(state_fn_part, &input)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&I) -> A + 'k> {
        let mut state_fn_part = self.current_state;
        Box::new(move |input: &I| -> A {
            ref_state_transition(&mut state_fn_part, &input)
        })
    }
}

impl <'k, I, A, C> FiniteStateAutomaton<'k> for RefStateMachine<'k, I, A, C> where 
    C: Into<fn(&I)->(A, C)> + 'k
{}

mod tests {
    #[derive(Copy, Clone)]
    struct ThingBob {
        fn_ref: fn(&bool) -> (bool, ThingBob)
    }

    impl From<ThingBob> for fn(&bool) -> (bool, ThingBob) {
        fn from(fn_box: ThingBob) -> fn(&bool) -> (bool, ThingBob) {
            fn_box.fn_ref
        }
    }

    fn xor_swap_0 (other: &bool) -> (bool, ThingBob) {
        if *other {
            (false, ThingBob{fn_ref: xor_swap_1})
        } else {
            (false, ThingBob{fn_ref: xor_swap_0})
        }
    }

    fn xor_swap_1 (other: &bool) -> (bool, ThingBob) {
        if *other {
            (true, ThingBob{fn_ref: xor_swap_0})
        } else {
            (true, ThingBob{fn_ref: xor_swap_1})
        }
    }

    #[test]
    fn check_def() {
        use ref_state_machine::RefStateMachine;
        use automaton::Automaton;
        let init: fn(&bool) -> (bool, ThingBob) = xor_swap_0;
        let mut x = RefStateMachine::new(ThingBob { fn_ref: init });
        assert!(!x.transition(&true));
        assert!(x.transition(&false));
        assert!(x.transition(&true));
        assert!(!x.transition(&false));
        assert!(!x.transition(&true));
    }
}