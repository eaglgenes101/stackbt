use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Transition trait for InternalStateMachine. 
pub trait InternalTransition: Copy {
    /// The input type taken by the state machine. 
    type Internal;
    /// The type of the internal state of the state machine. 
    type Input;
    /// The action type taken by the state machine. 
    type Action;
    /// Given references to the input and internal state, return the action 
    /// to return. 
    fn step(&Self::Input, &mut Self::Internal) -> Self::Action;
}

/// State machine implementation through a single, immutable function pointer 
/// called on an encapsualted state. Each step, the referenced function is 
/// called with the input and current state, returning an action and possibly 
/// modifying the state. 
/// 
/// To enforce that the state is self-contained, the internal state must 
/// be a Copy type, which is incompatible with safe references to non-static 
/// memory. 
#[derive(Copy, Clone)]
pub struct InternalStateMachine<'k, C> where 
    C: InternalTransition + 'k
{
    internal: C::Internal,
    _lifetime_check: PhantomData<&'k C>
}

impl<'k, C> InternalStateMachine<'k, C> where 
    C: InternalTransition + 'k
{
    /// Create a new internal state machine. 
    pub fn new(init_state: C::Internal) -> InternalStateMachine<'k, C> {
        InternalStateMachine {
            internal: init_state,
            _lifetime_check: PhantomData
        }
    }

    /// Create a new internal state machine, using a dummy object to supply 
    /// the type of the internal transition. 
    pub fn with(_calling_fn: C, init_state: C::Internal) -> InternalStateMachine<'k, C> {
        InternalStateMachine {
            internal: init_state,
            _lifetime_check: PhantomData
        }
    }
} 

impl<'k, C> Default for InternalStateMachine<'k, C> where 
    C: InternalTransition + 'k,
    C::Internal: Default
{
    fn default() -> InternalStateMachine<'k, C> {
        InternalStateMachine {
            internal: C::Internal::default(),
            _lifetime_check: PhantomData
        }
    }
} 

impl<'k, C> Automaton<'k> for InternalStateMachine<'k, C> where 
    C: InternalTransition + 'k
{
    type Input = C::Input;
    type Action = C::Action;
    #[inline]
    fn transition(&mut self, input: &C::Input) -> C::Action {
        C::step(&input, &mut self.internal)
    }
}

impl<'k, C> FiniteStateAutomaton<'k> for InternalStateMachine<'k, C> where 
    C: InternalTransition
{}

#[cfg(test)]
mod tests {
    use internal_state_machine::InternalTransition;

    #[derive(Copy, Clone)]
    struct ThingMachine;

    impl InternalTransition for ThingMachine {
        type Internal = i64;
        type Input = i64;
        type Action = i64;

        fn step(increment: &i64, accumulator: &mut i64) -> i64 {
            let orig_acc = *accumulator;
            *accumulator += increment;
            orig_acc
        }
    }

    #[test]
    fn check_def() {
        use internal_state_machine::InternalStateMachine;
        use automaton::Automaton;
        let mut x = InternalStateMachine::<ThingMachine>::new(0);
        assert_eq!(x.transition(&1), 0);
        assert_eq!(x.transition(&2), 1);
        assert_eq!(x.transition(&3), 3);
        assert_eq!(x.transition(&6), 6);
    }
}