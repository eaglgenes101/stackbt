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
pub struct InternalStateMachine<I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> A> + Clone 
{
    transition_fn: C, 
    internal: S,
    _i_exists: PhantomData<I>,
    _a_exists: PhantomData<A>
}

impl <I, S, A, C> InternalStateMachine<I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> A> + Clone
{
    pub fn new(calling_fn: C, init_state: S) -> InternalStateMachine<I, S, A, C> {
        InternalStateMachine {
            transition_fn: calling_fn,
            internal: init_state,
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
} 

impl<I, S, A, C> Automaton<I, A> for InternalStateMachine<I, S, A, C>  where 
    C: Into<fn(&I, &mut S) -> A> + Clone
{
    fn transition(&mut self, input: &I) -> A {
        (self.transition_fn.clone().into())(&input, &mut self.internal)
    }
}

impl<I, S, A, C> FiniteStateAutomaton<I, A> for InternalStateMachine<I, S, A, C> where 
    S: Copy,
    C: Into<fn(&I, &mut S) -> A> + Copy 
{}

#[cfg(test)]
mod tests {

    fn increment_swap(increment: &i64, accumulator: &mut i64) -> i64 {
        let orig_acc = *accumulator;
        *accumulator += increment;
        orig_acc
    }

    #[test]
    fn check_def() {
        use internal_state_machine::InternalStateMachine;
        use automaton::Automaton;
        let incr: fn(&i64, &mut i64) -> i64 = increment_swap;
        let mut x = InternalStateMachine::new(incr, 0);
        assert!(x.transition(&1) == 0);
        assert!(x.transition(&2) == 1);
        assert!(x.transition(&3) == 3);
        assert!(x.transition(&6) == 6);
    }
}