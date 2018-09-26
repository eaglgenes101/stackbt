use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Transition trait for DualStateMachine. 
pub trait DualTransition: Copy {
    type Internal;
    type Input;
    type Action;
    fn step(self, &Self::Input, &mut Self::Internal) -> (Self::Action, Self);
}

/// State machine implementation which combines the changing functions of 
/// RefStateMachine with the internal mutable state of InternalStateMachine. 
/// This is the most general state machine form in this crate, but the other 
/// two are generally easier to work with. 
#[derive(Copy, Clone)]
pub struct DualStateMachine<'k, C> where 
    C: DualTransition + 'k
{
    state_fn: Option<C>, 
    internal: C::Internal,
    _lifetime_check: PhantomData<&'k C>
}

impl<'k, C> DualStateMachine<'k, C> where
    C: DualTransition + 'k
{
    pub fn new(calling_fn: C, init_state: C::Internal) -> DualStateMachine<'k, C> {
        DualStateMachine {
            state_fn: Option::Some(calling_fn),
            internal: init_state,
            _lifetime_check: PhantomData
        }
    }
}

impl<'k, C> Default for DualStateMachine<'k, C> where
    C: DualTransition + Default + 'k,
    C::Internal: Default
{
    fn default() -> DualStateMachine<'k, C> {
        DualStateMachine::new(C::default(), C::Internal::default())
    }
}

impl<'k, C> Automaton<'k> for DualStateMachine<'k, C> where 
    C: DualTransition + 'k
{
    type Input = C::Input;
    type Action = C::Action;
    
    #[inline]
    fn transition(&mut self, input: &C::Input) -> C::Action {
        let (action, new_fn) = self.state_fn
            .take()
            .expect("State machine was poisoned")
            .step(input, &mut self.internal);
        self.state_fn = Option::Some(new_fn);
        action
    }
}

impl<'k, C> FiniteStateAutomaton<'k> for DualStateMachine<'k, C> where 
    C: DualTransition
{}

#[cfg(test)]
mod tests {
    use dual_state_machine::DualTransition;

    #[derive(Copy, Clone)]
    enum ThingMachine{
        Add,
        Subtract
    }

    impl DualTransition for ThingMachine {
        type Internal = i64;
        type Input = i64;
        type Action = i64;
        fn step(self, input: &i64, state: &mut i64) -> (i64, ThingMachine) {
            match self {
                ThingMachine::Add => {
                    if *input == 0 {
                        (*state, ThingMachine::Subtract)
                    } else {
                        *state += input;
                        (*state, ThingMachine::Add)
                    }
                },
                ThingMachine::Subtract => {
                    if *input == 0 {
                        (*state, ThingMachine::Add)
                    } else {
                        *state -= input;
                        (*state, ThingMachine::Subtract)
                    }
                }
            }
        }
    }

    #[test]
    fn check_def() {
        use dual_state_machine::DualStateMachine;
        use automaton::Automaton;
        let mut x = DualStateMachine::new(ThingMachine::Add, 0);
        assert_eq!(x.transition(&2), 2);
        assert_eq!(x.transition(&0), 2);
        assert_eq!(x.transition(&4), -2);
        assert_eq!(x.transition(&0), -2);
        assert_eq!(x.transition(&10), 8);
    }
}