use std::ops::Deref;
use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// State machine implemented through a function to call and an encapsulated 
/// state. Each step, the referenced function is called with the input and 
/// current state, returning an action and possible modifying the state. 
/// 
/// To enforce that the state is self-contained, the internal state must 
/// be a Copy type, which is incompatible with references to non-static
/// memory. 

#[derive(Copy, Clone)]
pub struct FnMod<I, S, A>(fn(&I, &mut S) -> A);

impl<I, S, A> Deref for FnMod<I, S, A> {
    type Target=fn(&I, &mut S) -> A;
    fn deref(&self) -> &fn(&I, &mut S) -> A {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub struct InternalStateMachine<I, S, A> {
    transition_fn: FnMod<I, S, A>, 
    internal: S,
}

impl <I, S, A> InternalStateMachine<I, S, A> {
    pub fn new(calling_fn: FnMod<I, S, A>, init_state: S) -> InternalStateMachine<I, S, A> {
        InternalStateMachine {
            transition_fn: calling_fn,
            internal: init_state,
        }
    }

    #[doc(hidden)]
    fn step(&mut self, input: &I) -> A {
        (self.transition_fn)(&input, &mut self.internal)
    }
} 

impl<I, S, A> Automaton<I, A> for InternalStateMachine<I, S, A> {
    fn transition(&mut self, input: &I) -> A {
        self.step(input)
    }
}

impl<I, S, A> FiniteStateAutomaton<I, A> for InternalStateMachine<I, S, A> where S: Copy {}