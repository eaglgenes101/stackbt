use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Transition trait for RefStateMachine. 
pub trait ReferenceTransition: Copy {
    /// The input type taken by the state machine. 
    type Input;
    /// The type of the internal state of the state machine. 
    type Action;
    /// Given a reference to the input, consume self, returning the action to
    /// return and the instance of Self used to reconstitute the 
    /// RefStateMachine. 
    fn step(self, &Self::Input) -> (Self::Action, Self);
}

/// State machine implemented through a closure reference wrapper struct. 
/// Each step, the currently referenced closure is called, returning an 
/// action and a reference to the closure to call for the next step. 
#[derive(Copy, Clone)]
pub struct RefStateMachine<'k, C> where 
    C: ReferenceTransition + 'k
{
    current_state: Option<C>,
    _lifetime_check: PhantomData<&'k C>
}

impl <'k, C> RefStateMachine<'k, C> where 
    C: ReferenceTransition + 'k
{
    /// Create a new reference state machine. 
    pub fn new(init_state: C) -> RefStateMachine<'k, C> {
        RefStateMachine {
            current_state: Option::Some(init_state),
            _lifetime_check: PhantomData
        }
    }
}

impl <'k, C> Default for RefStateMachine<'k, C> where 
    C: ReferenceTransition + Default + 'k
{
    fn default() -> RefStateMachine<'k, C> {
        RefStateMachine::new(C::default())
    }
}

impl <'k, C> Automaton<'k> for RefStateMachine<'k, C> where
    C: ReferenceTransition + 'k
{
    type Input = C::Input;
    type Action = C::Action;
    #[inline]
    fn transition(&mut self, input: &C::Input) -> C::Action {
        let (action, new_fn) = self.current_state
            .take()
            .expect("State machine was poisoned")
            .step(&input);
        self.current_state = Option::Some(new_fn);
        action
    }
}

impl <'k, C> FiniteStateAutomaton<'k> for RefStateMachine<'k, C> where 
    C: ReferenceTransition + 'k
{}

#[cfg(test)]
mod tests {
    use ref_state_machine::ReferenceTransition;

    #[derive(Copy, Clone)]
    enum ThingBob {
        XorSwap0,
        XorSwap1
    }

    impl ReferenceTransition for ThingBob {
        type Input = bool;
        type Action = bool;

        fn step(self, input: &bool) -> (bool, ThingBob) {
            match self {
                ThingBob::XorSwap0 => {
                    if *input {
                        (false, ThingBob::XorSwap1)
                    } else {
                        (false, ThingBob::XorSwap0)
                    }
                },
                ThingBob::XorSwap1 => {
                    if *input {
                        (true, ThingBob::XorSwap0)
                    } else {
                        (true, ThingBob::XorSwap1)
                    }
                }
            }
        }
    }

    #[test]
    fn check_def() {
        use ref_state_machine::RefStateMachine;
        use automaton::Automaton;
        let mut x = RefStateMachine::new(ThingBob::XorSwap0);
        assert!(!x.transition(&true));
        assert!(x.transition(&false));
        assert!(x.transition(&true));
        assert!(!x.transition(&false));
        assert!(!x.transition(&true));
    }
}