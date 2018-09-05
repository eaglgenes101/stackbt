use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;
use std::convert::Into;

/// State machine implemented through a closure reference wrapper struct. 
/// Each step, the currently referenced closure is called, returning an 
/// action and a reference to the closure to call for the next step. 
#[derive(Copy, Clone)]
pub struct RefStateMachine<I, A, C> where 
    C: Into<fn(&I)->(A, C)> + Copy
{
    current_state: C,
    _i_exists: PhantomData<I>,
    _a_exists: PhantomData<A>
}

impl <I, A, C> RefStateMachine<I, A, C> where 
    C: Into<fn(&I)->(A, C)> + Copy
{
    pub fn new(init_state: C) -> RefStateMachine<I, A, C> {
        RefStateMachine {
            current_state: init_state,
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
}

impl <I, A, C> Automaton<I, A> for RefStateMachine<I, A, C> where
    C: Into<fn(&I)->(A, C)> + Copy
{
    fn transition(&mut self, input: &I) -> A {
        let (action, next_state) = (self.current_state.into())(input);
        self.current_state = next_state;
        action
    }
}

impl <I, A, C> FiniteStateAutomaton<I, A> for RefStateMachine<I, A, C> where 
    C: Into<fn(&I)->(A, C)> + Copy
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