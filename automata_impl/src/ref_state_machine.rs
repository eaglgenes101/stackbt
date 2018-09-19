use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;
use std::mem::swap;

/// Transition trait for RefStateMachine. 
pub trait ReferenceTransition: Copy {
    type Input;
    type Action;
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

#[inline]
fn ref_state_transition<'k, C>(fn_ref: &mut Option<C>, input: &C::Input) 
    -> C::Action where
    C: ReferenceTransition + 'k
{
    let mut out_fn = Option::None;
    swap(fn_ref, &mut out_fn);
    let (action, new_fn) = out_fn.unwrap().step(&input);
    *fn_ref = Option::Some(new_fn);
    action
}

impl <'k, C> Automaton<'k> for RefStateMachine<'k, C> where
    C: ReferenceTransition + 'k
{
    type Input = C::Input;
    type Action = C::Action;
    #[inline]
    fn transition(&mut self, input: &C::Input) -> C::Action {
        ref_state_transition(&mut self.current_state, &input)
    }
    
    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&C::Input) -> C::Action + 't> where 
        'k: 't 
    {
        let mut state_fn_part = &mut self.current_state;
        Box::new(move |input: &C::Input| -> C::Action {
            ref_state_transition(state_fn_part, &input)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&C::Input) -> C::Action + 'k> {
        let mut state_fn_part = self.current_state;
        Box::new(move |input: &C::Input| -> C::Action {
            ref_state_transition(&mut state_fn_part, &input)
        })
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