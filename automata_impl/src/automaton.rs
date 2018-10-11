use std::ops::FnMut;
use std::iter::Iterator;
use automata_combinators::{MachineSeries, MachineTee, ParallelMachines};

/// The automaton trait is used to represent agents which, at a regular rate, 
/// take input, process it, and return an action. Most of them also change 
/// their internal state each transition. 
/// 
/// # Example
/// ```
/// use stackbt_automata_impl::automaton::Automaton;
/// use stackbt_automata_impl::internal_state_machine::InternalStateMachine;
/// let mut counter = InternalStateMachine::with(
///     |do_increment: &bool, count: &mut i64| {
///         if *do_increment {
///             *count += 1;
///         }
///         *count
///     }, 0
/// );
/// 
/// assert_eq!(counter.transition(&false), 0);
/// assert_eq!(counter.transition(&true), 1);
/// assert_eq!(counter.transition(&false), 1);
/// ```
pub trait Automaton<'k> {
    /// The input type taken by the automaton. 
    type Input: 'k;
    /// The action type returned by the automaton. 
    type Action;

    /// Take an input by reference, and change state and output an action 
    /// based on the state. 
    fn transition(&mut self, input: &Self::Input) -> Self::Action;

    /// Temporarily use the automaton as an FnMut. 
    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&Self::Input)->Self::Action+'t> where 
        'k: 't
    {
        let mut this = self;
        Box::new(move |input: &Self::Input| {
            this.transition(input)
        })
    }

    #[cfg(not(feature = "unsized_locals"))]
    /// Turn a automaton value into an FnMut. 
    fn into_fnmut(self) -> Box<FnMut(&Self::Input) -> Self::Action + 'k> where 
        Self: Sized + 'k
    {
        let mut this = self;
        Box::new(move |input: &Self::Input| {
            this.transition(input)
        })
    }

    #[cfg(feature = "unsized_locals")]
    /// Turn a automaton value into an FnMut. 
    fn into_fnmut(self) -> Box<FnMut(&Self::Input) -> Self::Action + 'k> where 
        Self: 'k
    {
        let mut this = self;
        Box::new(move |input: &Self::Input| {
            this.transition(input)
        })
    }

    /// Turn the boxed automaton into an fnmut. 
    fn boxed_into_fnmut(self: Box<Self>) -> Box<FnMut(&Self::Input) -> 
        Self::Action + 'k> where 
        Self: 'k
    {
        let mut this = self;
        Box::new(move |input: &Self::Input| {
            this.transition(input)
        })
    }

    fn then<N>(self, after: N) -> MachineSeries<'k, Self, N> where
        N: Automaton<'k, Input=Self::Action>,
        Self: Sized + 'k
    {
        MachineSeries::new(self, after)
    }

    fn tee<N>(self, after: N) -> MachineTee<'k, Self, N> where
        N: Automaton<'k, Input=Self::Action>,
        Self: Sized + 'k
    {
        MachineTee::new(self, after)
    }

    fn alongside<N>(self, other: N) -> ParallelMachines<'k, Self, N> where 
        N: Automaton<'k, Input=Self::Input>,
        Self: Sized + 'k
    {
        ParallelMachines::new(self, other)
    }
}

impl<'k, P> Automaton<'k> for Box<P> where 
    P: Automaton<'k> + ?Sized,
    P::Input: 'k
{
    type Input = P::Input;
    type Action = P::Action;

    fn transition(&mut self, input: &P::Input) -> P::Action {
        self.as_mut().transition(input)
    }
}

impl<'k, M> Automaton<'k> for [M] where 
    M: Automaton<'k>
{
    type Input = M::Input;
    type Action = Box<[M::Action]>;

    fn transition(&mut self, input: &M::Input) -> Self::Action {
        let items = self.iter_mut()
            .map(|mach| mach.transition(input))
            .collect::<Vec<_>>();
        items.into_boxed_slice()
    }
}

impl<'k, I, A> Automaton<'k> for [&'k mut dyn Automaton<'k, Input=I, Action=A>] {
    type Input = I;
    type Action = Box<[A]>;

    fn transition(&mut self, input: &I) -> Box<[A]> {
        let items = self.iter_mut()
            .map(|mach| mach.transition(input))
            .collect::<Vec<_>>();
        items.into_boxed_slice()
    }
}

/// Marker trait for Finite State Automata, which are a restricted class of 
/// automata that are quite well behaved. In particular, they occupy fixed 
/// memory, and thus do not need extra allocation to operate, and instances 
/// with known type can be copied around freely. 
pub trait FiniteStateAutomaton<'k>: Automaton<'k> + Copy {}

#[cfg(test)]
mod tests {
    use internal_state_machine::InternalTransition;

    #[derive(Copy, Clone)]
    struct ThingMachine;

    impl InternalTransition for ThingMachine {
        type Internal = i64;
        type Input = i64;
        type Action = i64;

        fn step(&self, increment: &i64, accumulator: &mut i64) -> i64 {
            let orig_acc = *accumulator;
            *accumulator += increment;
            orig_acc
        }
    }

    #[test]
    fn as_fnmut_test() {
        use internal_state_machine::InternalStateMachine;
        use automaton::Automaton;
        let zero_inf = 0..8;
        let mut machine = InternalStateMachine::new(ThingMachine, 0);
        let machine_fn = machine.as_fnmut();
        let mut scanner = zero_inf.scan(machine_fn, |mach, input| {
            Option::Some(mach(&input))
        });
        assert_eq!(scanner.next().unwrap(), 0);
        assert_eq!(scanner.next().unwrap(), 0);
        assert_eq!(scanner.next().unwrap(), 1);
        assert_eq!(scanner.next().unwrap(), 3);
        assert_eq!(scanner.next().unwrap(), 6);
        assert_eq!(scanner.next().unwrap(), 10);
        assert_eq!(scanner.next().unwrap(), 15);
        assert_eq!(scanner.next().unwrap(), 21);
        assert!(scanner.next().is_none());
    }

    #[test]
    fn into_fnmut_test() {
        use internal_state_machine::InternalStateMachine;
        use automaton::Automaton;
        let zero_inf = 0..8;
        let machine = InternalStateMachine::new(ThingMachine, 0);
        let machine_fn = machine.into_fnmut();
        let mut scanner = zero_inf.scan(machine_fn, |mach, input| {
            Option::Some(mach(&input))
        });
        assert_eq!(scanner.next().unwrap(), 0);
        assert_eq!(scanner.next().unwrap(), 0);
        assert_eq!(scanner.next().unwrap(), 1);
        assert_eq!(scanner.next().unwrap(), 3);
        assert_eq!(scanner.next().unwrap(), 6);
        assert_eq!(scanner.next().unwrap(), 10);
        assert_eq!(scanner.next().unwrap(), 15);
        assert_eq!(scanner.next().unwrap(), 21);
        assert!(scanner.next().is_none());
    }

    #[test]
    fn box_into_fnmut_test() {
        use internal_state_machine::InternalStateMachine;
        use automaton::Automaton;
        let zero_inf = 0..8;
        let machine = InternalStateMachine::new(ThingMachine, 0);
        let machine_fn = Box::new(machine).boxed_into_fnmut();
        let mut scanner = zero_inf.scan(machine_fn, |mach, input| {
            Option::Some(mach(&input))
        });
        assert_eq!(scanner.next().unwrap(), 0);
        assert_eq!(scanner.next().unwrap(), 0);
        assert_eq!(scanner.next().unwrap(), 1);
        assert_eq!(scanner.next().unwrap(), 3);
        assert_eq!(scanner.next().unwrap(), 6);
        assert_eq!(scanner.next().unwrap(), 10);
        assert_eq!(scanner.next().unwrap(), 15);
        assert_eq!(scanner.next().unwrap(), 21);
        assert!(scanner.next().is_none());
    }
}