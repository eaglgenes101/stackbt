use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// State machine which combines the changing functions of ref_state_machine 
/// with the internal state of internal_state_machine. This is the most 
/// general state machine form in this framework, but the other two are 
/// usually easier to work with. 

#[derive(Clone)]
pub struct DualStateMachine<I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> (A, C)> + Clone
{
    state_fn: C, 
    internal: S,
    _i_exists: PhantomData<I>,
    _a_exists: PhantomData<A>
}

impl<I, S, A, C> DualStateMachine<I, S, A, C> where
    C: Into<fn(&I, &mut S) -> (A, C)> + Clone
{
    pub fn new(calling_fn: C, init_state: S) -> DualStateMachine<I, S, A, C> {
        DualStateMachine {
            state_fn: calling_fn,
            internal: init_state,
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
}

impl<I, S, A, C> Automaton<I, A> for DualStateMachine<I, S, A, C>  where 
    C: Into<fn(&I, &mut S) -> (A, C)> + Clone
{
    fn transition(&mut self, input: &I) -> A {
        let (action, new_fn) = (self.state_fn.clone().into())(&input, &mut self.internal);
        self.state_fn = new_fn;
        action
    }
}

impl<I, S, A, C> FiniteStateAutomaton<I, A> for DualStateMachine<I, S, A, C> where 
    S: Copy,
    C: Into<fn(&I, &mut S) -> (A, C)> + Copy 
{}