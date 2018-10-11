//!
//! It sometimes happens that you have an automaton on hand that acts 
//! somewhat, but not exactly, like you want it to, or it works like you want 
//! it to but has the wrong type. In those cases, you can use a wrapper 
//! instead of writing a whole new automaton. 
//!

use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;


pub struct MachineSeries<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Action>
{
    before: M,
    after: N,
    _bounds: PhantomData<&'k (M, N)>
}

impl<'k, M, N> Clone for MachineSeries<'k, M, N> where
    M: Automaton<'k> + Clone,
    N: Automaton<'k, Input=M::Action> + Clone
{
    fn clone(&self) -> Self {
        MachineSeries {
            before: self.before.clone(),
            after: self.after.clone(),
            _bounds: PhantomData
        }
    }
}

impl<'k, M, N> Copy for MachineSeries<'k, M, N> where
    M: Automaton<'k> + Copy,
    N: Automaton<'k, Input=M::Action> + Copy
{}

impl<'k, M, N> MachineSeries<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Action>
{
    pub fn new(before: M, after: N) -> Self {
        MachineSeries {
            before: before,
            after: after,
            _bounds: PhantomData
        }
    }
}

impl<'k, M, N> Automaton<'k> for MachineSeries<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Action>
{
    type Input = M::Input;
    type Action = N::Action;

    fn transition(&mut self, input: &M::Input) -> N::Action {
        let intermediate = self.before.transition(input);
        self.after.transition(&intermediate)
    }
}

impl<'k, M, N> FiniteStateAutomaton<'k> for MachineSeries<'k, M, N> where 
    M: Automaton<'k> + Copy,
    N: Automaton<'k, Input=M::Action> + Copy
{}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LazyConstructedInner<'k, M, C> where
    M: Automaton<'k>,
    C: Fn(&M::Input) -> M
{
    Machine(M, PhantomData<&'k M>),
    Pending(C),
}

/// Wrapper for for a machine, which defers initialization until the first 
/// input is supplied, after which the machine is constructed using this input
/// as a parameter. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: Fn(&M::Input) -> M
{
    internal: Option<LazyConstructedInner<'k, M, C>>
}

impl<'k, M, C> LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: Fn(&M::Input) -> M
{
    /// Create an new lazily constructed automaton. 
    pub fn new(construct: C) -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine {
            internal: Option::Some(LazyConstructedInner::Pending(construct))
        }
    }

    /// Wrap an existing automaton in the lazily constructed automaton 
    /// wrapper.
    pub fn from_existing(machine: M) -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine {
            internal: Option::Some(LazyConstructedInner::Machine(machine, PhantomData))
        }
    }
}

impl<'k, M, C> Default for LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: Fn(&M::Input) -> M + Default
{
    fn default() -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine::new(C::default())
    }
}

impl<'k, M, C> Automaton<'k> for LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: Fn(&M::Input) -> M
{
    type Input = M::Input;
    type Action = M::Action;

    #[inline]
    fn transition(&mut self, input: &M::Input) -> M::Action {
        let machine = match self.internal.take().unwrap() {
            LazyConstructedInner::Machine(m, _) => m,
            LazyConstructedInner::Pending(c) => c(input)
        };
        self.internal = Option::Some(
            LazyConstructedInner::Machine(machine, PhantomData)
        );
        match self.internal {
            Option::Some(LazyConstructedInner::Machine(ref mut m, _)) => {
                m.transition(input)
            },
            _ => unreachable!("Should be a machine at this point")
        }
    }
}

impl<'k, M, C> FiniteStateAutomaton<'k> for LazyConstructedMachine<'k, M, C> where
    M: FiniteStateAutomaton<'k>,
    C: Fn(&M::Input) -> M + Copy
{}


pub struct MachineTee<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Action>
{
    before: M,
    after: N,
    _bounds: PhantomData<&'k (M, N)>
}

impl<'k, M, N> Clone for MachineTee<'k, M, N> where
    M: Automaton<'k> + Clone,
    N: Automaton<'k, Input=M::Action> + Clone
{
    fn clone(&self) -> Self {
        MachineTee {
            before: self.before.clone(),
            after: self.after.clone(),
            _bounds: PhantomData
        }
    }
}

impl<'k, M, N> Copy for MachineTee<'k, M, N> where
    M: Automaton<'k> + Copy,
    N: Automaton<'k, Input=M::Action> + Copy
{}

impl<'k, M, N> MachineTee<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Action>
{
    pub fn new(before: M, after: N) -> Self {
        MachineTee {
            before: before,
            after: after,
            _bounds: PhantomData
        }
    }
}

impl<'k, M, N> Automaton<'k> for MachineTee<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Action>
{
    type Input = M::Input;
    type Action = (M::Action, N::Action);

    fn transition(&mut self, input: &M::Input) -> (M::Action, N::Action) {
        let intermediate = self.before.transition(input);
        let reaction = self.after.transition(&intermediate);
        (intermediate, reaction)
    }
}

impl<'k, M, N> FiniteStateAutomaton<'k> for MachineTee<'k, M, N> where 
    M: Automaton<'k> + Copy,
    N: Automaton<'k, Input=M::Action> + Copy
{}

#[derive(PartialEq, Debug)]
pub struct ParallelMachines<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Input>
{
    first: M,
    second: N,
    _bounds: PhantomData<&'k (M, N)>
}

impl<'k, M, N> Clone for ParallelMachines<'k, M, N> where 
    M: Automaton<'k> + Clone,
    N: Automaton<'k, Input=M::Input> + Clone
{
    fn clone(&self) -> Self {
        ParallelMachines {
            first: self.first.clone(),
            second: self.second.clone(),
            _bounds: PhantomData
        }
    }
}

impl<'k, M, N> Copy for ParallelMachines<'k, M, N> where 
    M: Automaton<'k> + Copy,
    N: Automaton<'k, Input=M::Input> + Copy
{}

impl<'k, M, N> ParallelMachines<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Input>
{
    pub fn new(first: M, second: N) -> Self {
        ParallelMachines {
            first: first,
            second: second,
            _bounds: PhantomData
        }
    }
}

impl<'k, M, N> Automaton<'k> for ParallelMachines<'k, M, N> where 
    M: Automaton<'k>,
    N: Automaton<'k, Input=M::Input>
{
    type Input = M::Input;
    type Action = (M::Action, N::Action);

    fn transition(&mut self, input: &M::Input) -> Self::Action {
        (self.first.transition(input), self.second.transition(input))
    }
}

impl<'k, M, N> FiniteStateAutomaton<'k> for ParallelMachines<'k, M, N> where 
    M: Automaton<'k> + Copy,
    N: Automaton<'k, Input=M::Input> + Copy
{}

#[cfg(test)]
mod tests {
    use internal_state_machine::{InternalTransition, 
        InternalStateMachine};
    use automaton::Automaton;

    #[derive(Copy, Clone)]
    struct Echoer;

    impl InternalTransition for Echoer {
        type Input = i64;
        type Internal = ();
        type Action = i64;

        fn step(&self, input: &i64, state: &mut ()) -> i64 {
            *input
        }
    }

    #[test]
    fn input_map_test() {
        use stateless_mapper::StatelessMapper;
        use automata_combinators::MachineSeries;
        let base_node = InternalStateMachine::new(Echoer, ());
        let mut wrapped_machine = MachineSeries::new(
            StatelessMapper::new(|input: &i64| -input), base_node);
        assert_eq!(wrapped_machine.transition(&-5), 5);
        assert_eq!(wrapped_machine.transition(&4), -4);
        assert_eq!(wrapped_machine.transition(&-7), 7);
        assert_eq!(wrapped_machine.transition(&12), -12);
    }

    #[test]
    fn output_map_test() {
        use stateless_mapper::StatelessMapper;
        use automata_combinators::MachineSeries;
        let base_node = InternalStateMachine::new(Echoer, ());
        let mut wrapped_machine = MachineSeries::new(
            base_node,
            StatelessMapper::new(|&val| val+1)
        );
        assert_eq!(wrapped_machine.transition(&-5), -4);
        assert_eq!(wrapped_machine.transition(&4), 5);
        assert_eq!(wrapped_machine.transition(&-7), -6);
        assert_eq!(wrapped_machine.transition(&12), 13);
    }

    #[derive(Copy, Clone, Default)]
    struct IndefinitePlayback;

    impl InternalTransition for IndefinitePlayback {
        type Input = i64;
        type Internal = i64;
        type Action = i64;

        fn step(&self, input: &i64, state: &mut i64) -> i64{
            *state
        }
    }

    #[test]
    fn lazy_constructor_test() {
        use automata_combinators::LazyConstructedMachine;
        let creation_closure = |input: &i64| {
            InternalStateMachine::new(IndefinitePlayback, *input)
        };
        let mut new_machine = LazyConstructedMachine::new(creation_closure);
        assert_eq!(new_machine.transition(&2), 2);
        assert_eq!(new_machine.transition(&5), 2);
        assert_eq!(new_machine.transition(&-3), 2);
        assert_eq!(new_machine.transition(&2), 2);
        assert_eq!(new_machine.transition(&4), 2);

        let mut new_machine_1 = LazyConstructedMachine::new(creation_closure);
        assert_eq!(new_machine_1.transition(&5), 5);
        assert_eq!(new_machine_1.transition(&5), 5);
        assert_eq!(new_machine_1.transition(&-3), 5);
        assert_eq!(new_machine_1.transition(&-4), 5);
        assert_eq!(new_machine_1.transition(&-5), 5);
    }
}