use std::ops::Deref;
use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;
use std::convert::Into;

#[derive(Copy, Clone)]
pub struct FnStep<T, A, C> where C: Into<fn(&T)->(A, FnStep<T, A, C>)> {
    fn_ref: fn(&T)->(A, FnStep<T, A, C>),
    _c_exists: PhantomData<C>
}

/// State machine implemented through a closure reference wrapper struct. 
/// Each step, the currently referenced closure is called, returning an 
/// action and a reference to the closure to call for the next step. 
#[derive(Copy, Clone)]
pub struct RefStateMachine<I, A, C> where C: Into<fn(&I)->(A, FnStep<I, A, C>)> {
    current_state: FnStep<I, A, C>
}

impl <I, A, C: Into<fn(&I)->(A, FnStep<I, A, C>)>> RefStateMachine<I, A, C>
{
    pub fn new(init_state: FnStep<I, A, C>) -> RefStateMachine<I, A, C> {
        RefStateMachine {
            current_state: init_state
        }
    }

    #[doc(hidden)]
    fn step(&mut self, input: &I) -> A {
        let (action, next_state) = (self.current_state.fn_ref)(input);
        self.current_state = next_state;
        action
    }
}

impl <I, A, C> Automaton<I, A> for RefStateMachine<I, A, C> where
    C: Into<fn(&I)->(A, FnStep<I, A, C>)>
{
    fn transition(&mut self, input: &I) -> A {
        self.step(input)
    }
}

impl <I, A, C> FiniteStateAutomaton<I, A> for RefStateMachine<I, A, C> where 
    C: Into<fn(&I)->(A, FnStep<I, A, C>)>
{}

