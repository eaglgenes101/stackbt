use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// State machine implementation through a single, immutable function pointer 
/// called on an encapsualted state. Each step, the referenced function is 
/// called with the input and current state, returning an action and possibly 
/// modifying the state. 
/// 
/// To enforce that the state is self-contained, the internal state must 
/// be a Copy type, which is incompatible with safe references to non-static 
/// memory. 
#[derive(Copy, Clone)]
pub struct InternalStateMachine<'k, I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> A> + Clone + 'k,
    S: 'k,
    I: 'k
{
    transition_fn: C, 
    internal: S,
    _i_exists: PhantomData<&'k I>,
    _a_exists: PhantomData<A>
}

impl <'k, I, S, A, C> InternalStateMachine<'k, I, S, A, C> where 
    C: Into<fn(&I, &mut S) -> A> + Clone + 'k,
    S: 'k
{
    pub fn new(calling_fn: C, init_state: S) -> InternalStateMachine<'k, I, S, A, C> {
        InternalStateMachine {
            transition_fn: calling_fn,
            internal: init_state,
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
} 

impl<'k, I, S, A, C> Automaton<'k> for InternalStateMachine<'k, I, S, A, C>  where 
    C: Into<fn(&I, &mut S) -> A> + Clone + 'k,
    S: 'k
{
    type Input = I;
    type Action = A;
    #[inline]
    fn transition(&mut self, input: &I) -> A {
        (self.transition_fn.clone().into())(&input, &mut self.internal)
    }

    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&I) -> A + 't> where 'k: 't {
        let transition_fn_part = &self.transition_fn;
        let mut internal_part = &mut self.internal;
        Box::new(move |input: &I| -> A {
            (transition_fn_part.clone().into())(&input, &mut internal_part)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&I) -> A + 'k> {
        let transition_fn_part = self.transition_fn;
        let mut internal_part = self.internal;
        Box::new(move |input: &I| -> A {
            (transition_fn_part.clone().into())(&input, &mut internal_part)
        })
    }
}

impl<'k, I, S, A, C> FiniteStateAutomaton<'k> for 
    InternalStateMachine<'k, I, S, A, C> where 
    S: Copy + 'k,
    C: Into<fn(&I, &mut S) -> A> + Copy + 'k
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