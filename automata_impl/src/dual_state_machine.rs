use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Transition trait for DualStateMachine. 
pub trait DualTransition {
    /// The input type taken by the state machine. 
    type Input;
    /// The type of the internal state of the state machine. 
    type Internal;
    /// The action type taken by the state machine. 
    type Action;
    /// Given references to the input and internal state, consume self, 
    /// returning the action to return and the instance of Self used to 
    /// reconstitute the DualStateMachine. 
    fn step(self, &Self::Input, &mut Self::Internal) -> (Self::Action, Self);
}

/// Type which exists to make utilizing closures with internal state machines
/// that much more possible. 
pub struct DualTransClosure<I, N, A, C> where 
    C: FnMut(&I, &mut N) -> A
{
    closure: C,
    _junk: PhantomData<(I, N, A)>
}

impl<I, N, A, C> DualTransClosure<I, N, A, C> where 
    C: FnMut(&I, &mut N) -> A
{
    fn new(closure: C) -> DualTransClosure<I, N, A, C> {
        DualTransClosure {
            closure: closure,
            _junk: PhantomData
        }
    }
}

impl<I, N, A, C> DualTransition for DualTransClosure<I, N, A, C> where 
    C: FnMut(&I, &mut N) -> A
{
    type Input = I;
    type Internal = N;
    type Action = A;
    fn step(self, input: &I, internal: &mut N) -> (A, Self) {
        let mut mut_self = self;
        ((mut_self.closure)(input, internal), mut_self)
    }
}

/// State machine implementation which combines the changing functions of 
/// RefStateMachine with the internal mutable state of InternalStateMachine. 
/// This is the most general state machine form in this crate, but the other 
/// two are generally easier to work with. 
/// 
/// # Example
/// ```
/// use stackbt_automata_impl::automaton::Automaton;
/// use stackbt_automata_impl::dual_state_machine::{DualTransition, 
///     DualStateMachine};
/// 
/// enum EvenTickCounter {
///     Step,
///     Pass
/// }
/// 
/// impl DualTransition for EvenTickCounter {
///     type Input = bool;
///     type Internal = i64;
///     type Action = i64;
///     fn step(self, do_incr: &bool, state: &mut i64) -> (i64, Self) {
///         match (self, *do_incr) {
///             (EvenTickCounter::Step, true) => {
///                 *state += 1;
///                 (*state, EvenTickCounter::Pass)
///             },
///             (EvenTickCounter::Step, false) => (*state, EvenTickCounter::Pass),
///             _ => (*state, EvenTickCounter::Step)
///         }
///     }
/// }
/// 
/// let mut counter = DualStateMachine::new(EvenTickCounter::Step, 0);
/// assert_eq!(counter.transition(&false), 0);
/// assert_eq!(counter.transition(&true), 0);
/// assert_eq!(counter.transition(&true), 1);
/// assert_eq!(counter.transition(&false), 1);
/// assert_eq!(counter.transition(&false), 1);
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
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
    /// Create a new dual state machine. 
    pub fn new(calling_fn: C, init_state: C::Internal) -> DualStateMachine<'k, C> {
        DualStateMachine {
            state_fn: Option::Some(calling_fn),
            internal: init_state,
            _lifetime_check: PhantomData
        }
    }
}

impl<'k, I, N, A, C> DualStateMachine<'k, DualTransClosure<I, N, A, C>> where 
    C: FnMut(&I, &mut N) -> A
{
    /// Create a new internal state machine from a closure. 
    pub fn with(init: C, init_state: N) -> DualStateMachine<'k, 
        DualTransClosure<I, N, A, C>> 
    {
        DualStateMachine::new(
            DualTransClosure::new(init),
            init_state
        )
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
    C: DualTransition + Copy,
    C::Internal: Copy
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