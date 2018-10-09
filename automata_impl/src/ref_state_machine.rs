use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Transition trait for RefStateMachine. 
pub trait ReferenceTransition {
    /// The input type taken by the state machine. 
    type Input;
    /// The type of the internal state of the state machine. 
    type Action;
    /// Given a reference to the input, consume self, returning the action to
    /// return and the instance of Self used to reconstitute the 
    /// RefStateMachine. 
    fn step(self, &Self::Input) -> (Self::Action, Self);
}

/// Type which exists to make utilizing closures with internal state machines
/// that much more possible. 
pub struct ReferenceTransClosure<I, A, C> where 
    C: FnMut(&I) -> A
{
    closure: C,
    _junk: PhantomData<(I, A)>
}

impl<I, A, C> ReferenceTransClosure<I, A, C> where 
    C: FnMut(&I) -> A
{
    fn new(closure: C) -> ReferenceTransClosure<I, A, C> {
        ReferenceTransClosure {
            closure: closure,
            _junk: PhantomData
        }
    }
}

impl<I, A, C> ReferenceTransition for ReferenceTransClosure<I, A, C> where 
    C: FnMut(&I) -> A
{
    type Input = I;
    type Action = A;
    fn step(self, input: &I) -> (A, Self) {
        let mut mut_self = self;
        ((mut_self.closure)(input), mut_self)
    }
}

/// State machine implemented through a self-contained callable type. Each 
/// step, the currently referenced callable is called, returning an action 
/// and the new value to call for the next step. 
/// 
/// # Example
/// ```
/// use stackbt_automata_impl::automaton::Automaton;
/// use stackbt_automata_impl::ref_state_machine::{ReferenceTransition, 
///     RefStateMachine};
/// 
/// enum SRLatch {
///     Low, 
///     High
/// }
/// 
/// impl ReferenceTransition for SRLatch {
///     type Input = (bool, bool);
///     type Action = bool;
///     fn step(self, input: &(bool, bool)) -> (bool, Self) {
///         match self {
///             SRLatch::Low => match *input {
///                 (_, true) => (false, SRLatch::High),
///                 _ => (false, SRLatch::Low)
///             },
///             SRLatch::High => match *input {
///                 (true, _) => (true, SRLatch::Low),
///                 _ => (true, SRLatch::High)
///             }
///         }
///     }
/// }
/// 
/// let mut latch = RefStateMachine::new(SRLatch::Low);
/// assert!(!latch.transition(&(true, false)));
/// assert!(!latch.transition(&(false, true)));
/// assert!(latch.transition(&(false, false)));
/// assert!(latch.transition(&(true, true)));
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
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

impl<'k, I, A, C> RefStateMachine<'k, ReferenceTransClosure<I, A, C>> where 
    C: FnMut(&I) -> A
{
    /// Create a new internal state machine from a closure. 
    pub fn with(init: C) -> RefStateMachine<'k, ReferenceTransClosure<I, A, C>> {
        RefStateMachine::new(ReferenceTransClosure::new(init))
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
    C: ReferenceTransition + Copy + 'k
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