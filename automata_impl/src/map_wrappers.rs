//!
//! It sometimes happens that you have an automaton on hand that acts 
//! somewhat, but not exactly, like you want it to, or it works like you want 
//! it to but has the wrong type. In those cases, you can use a wrapper 
//! instead of writing a whole new automaton. 
//!
//! An example of the usage of wrappers: 
//! ```
//! use stackbt_automata_impl::map_wrappers::{InputMachineMap, 
//!     InputMappedMachine};
//! use stackbt_automata_impl::automaton::Automaton;
//! use std::convert::AsMut;
//! 
//! struct Inverter;
//! 
//! impl InputMachineMap for Inverter {
//!     type In = bool;
//!     type Out = bool;
//!     fn input_transform(&self, input: &bool) -> bool {
//!         !*input
//!     }
//! }
//! 
//! let mut count = 0;
//! let mut counter: Box<FnMut(&bool) -> i64> = Box::new(
//!     move |do_increment: &bool| {
//!         if *do_increment {
//!             count += 1;
//!         }
//!         count
//!     }
//! );
//! 
//! let mut modified_counter = InputMappedMachine::new(Inverter, counter);
//! 
//! assert_eq!(modified_counter.transition(&true), 0);
//! assert_eq!(modified_counter.transition(&false), 1);
//! assert_eq!(modified_counter.transition(&true), 1);
//! ```

use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Mapping between different input types. 
pub trait InputMachineMap {
    /// The input type for the input mapper, which is taken as the input type 
    /// of the wrapper. 
    type In;
    /// The output type for the input mapper, which is then fed into the 
    /// enclosed automaton. 
    type Out;
    /// Map between the input supplied and the output to feed into the 
    /// enclosed automaton. 
    fn input_transform(&self, &Self::In) -> Self::Out;
}

pub struct InputMapClosure<I, O, C> where C: Fn(&I) -> O {
    closure: C,
    _junk: PhantomData<(I, O)>
}

impl<I, O, C> InputMapClosure<I, O, C> where C: Fn(&I) -> O {
    fn new(closure: C) -> InputMapClosure<I, O, C> {
        InputMapClosure {
            closure: closure,
            _junk: PhantomData
        }
    }
}

impl<I, O, C> InputMachineMap for InputMapClosure<I, O, C> where 
    C: Fn(&I) -> O 
{
    type In = I;
    type Out = O;
    fn input_transform(&self, input: &I) -> O {
        (self.closure)(input)
    }

}

/// Wrapper for a automaton which converts between the provided input type
/// and the input type expected by the automaton. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct InputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: InputMachineMap<Out=M::Input>
{
    machine: M,
    wrapper: W,
    _lifetime_use: PhantomData<&'k W>
}

impl<'k, M, W> InputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: InputMachineMap<Out=M::Input>
{
    /// Create a new input mapped automaton. 
    pub fn new(wrapper: W, machine: M) -> InputMappedMachine<'k, M, W> {
        InputMappedMachine {
            machine,
            wrapper,
            _lifetime_use: PhantomData
        }
    }
}

impl<'k, M, W> Default for InputMappedMachine<'k, M, W> where
    M: Automaton<'k> + Default,
    W: InputMachineMap<Out=M::Input> + Default
{
    fn default() -> InputMappedMachine<'k, M, W> {
        InputMappedMachine::new(W::default(), M::default())
    }
}

impl<'k, M, W> Automaton<'k> for InputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: InputMachineMap<Out=M::Input>
{
    type Input = W::In;
    type Action = M::Action;

    #[inline]
    fn transition(&mut self, input: &W::In) -> M::Action {
        self.machine.transition(&self.wrapper.input_transform(input))
    }
}

impl<'k, M, W> FiniteStateAutomaton<'k> for InputMappedMachine<'k, M, W> where
    M: FiniteStateAutomaton<'k>,
    W: InputMachineMap<Out=M::Input> + Copy
{}

/// Mapping between different output types. 
pub trait OutputMachineMap {
    /// The input type for the output mapper, received from the enclosed 
    /// automaton. 
    type In;
    /// The output type for the output mapper, which is the type returned 
    /// by the wrapper. 
    type Out;
    /// Map between the input returned by the automaton and the output to
    /// return. 
    fn output_transform(&self, Self::In) -> Self::Out;
}

impl<I, O> OutputMachineMap for Fn(I) -> O {
    type In = I;
    type Out = O;
    fn output_transform(&self, input: I) -> O {
        self(input)
    }
}

/// Wrapper for an automaton which converts between the actions emitted by the
/// automaton and the ones exposed by the wrapper. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct OutputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: OutputMachineMap<In=M::Action>
{
    machine: M,
    wrapper: W,
    _lifetime_use: PhantomData<&'k W>
}

impl<'k, M, W> OutputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: OutputMachineMap<In=M::Action>
{
    /// Create a new output mapped automaton. 
    pub fn new(outer: W, machine: M) -> OutputMappedMachine<'k, M, W> {
        OutputMappedMachine {
            machine,
            wrapper: outer,
            _lifetime_use: PhantomData
        }
    }
}

impl<'k, M, W> Default for OutputMappedMachine<'k, M, W> where
    M: Automaton<'k> + Default,
    W: OutputMachineMap<In=M::Action> + Default
{
    fn default() -> OutputMappedMachine<'k, M, W> {
        OutputMappedMachine::new(W::default(), M::default())
    }
}

impl<'k, M, W> Automaton<'k> for OutputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: OutputMachineMap<In=M::Action>
{
    type Input = M::Input;
    type Action = W::Out;

    #[inline]
    fn transition(&mut self, input: &M::Input) -> W::Out {
        self.wrapper.output_transform(self.machine.transition(input))
    }
}

impl<'k, M, W> FiniteStateAutomaton<'k> for OutputMappedMachine<'k, M, W> where
    M: FiniteStateAutomaton<'k>,
    W: OutputMachineMap<In=M::Action> + Copy
{}


/// Lazy constructor for an automaton, depending on the first input. 
pub trait LazyConstructor<'k> {
    /// Type of the automaton to create. 
    type Creates: Automaton<'k>;
    /// Create a new automaton. 
    fn create(&self, &<Self::Creates as Automaton<'k>>::Input) -> Self::Creates;
}

impl<'k, S> LazyConstructor<'k> for Fn(&S::Input) -> S where 
    S: Automaton<'k>
{
    type Creates = S;

    fn create(&self, input: &S::Input) -> S {
        self(input)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LazyConstructedInner<'k, M, C> where
    M: Automaton<'k>,
    C: LazyConstructor<'k, Creates=M>
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
    C: LazyConstructor<'k, Creates=M>
{
    internal: Option<LazyConstructedInner<'k, M, C>>
}

impl<'k, M, C> LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: LazyConstructor<'k, Creates=M>
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
    C: LazyConstructor<'k, Creates=M> + Default
{
    fn default() -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine::new(C::default())
    }
}

impl<'k, M, C> Automaton<'k> for LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: LazyConstructor<'k, Creates=M>
{
    type Input = M::Input;
    type Action = M::Action;

    #[inline]
    fn transition(&mut self, input: &M::Input) -> M::Action {
        let machine = match self.internal.take().unwrap() {
            LazyConstructedInner::Machine(m, _) => m,
            LazyConstructedInner::Pending(c) => c.create(input)
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
    C: LazyConstructor<'k, Creates=M> + Copy
{}

/// Eager constructor for an automaton. 
pub trait CustomConstructor<'k> {
    /// Type of the automaton to create. 
    type Creates: Automaton<'k>;
    /// Create a new automaton. 
    fn create(&self) -> Self::Creates;
}

/// Wrapper for an automaton which designates a default constructor for that
/// automaton, constructing it from the constructor. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CustomConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: CustomConstructor<'k, Creates=M>
{
    machine: M, 
    _exists_tuple: PhantomData<&'k C>
}

impl<'k, M, C> CustomConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: CustomConstructor<'k, Creates=M>
{
    /// Create an new custom constructed automaton. 
    pub fn new(constructor: &C) -> CustomConstructedMachine<'k, M, C> {
        CustomConstructedMachine {
            machine: constructor.create(),
            _exists_tuple: PhantomData
        }
    }

    /// Wrap an existing automaton in the custom constructed automaton 
    /// wrapper.
    pub fn from_existing(machine: M) -> CustomConstructedMachine<'k, M, C> {
        CustomConstructedMachine {
            machine: machine,
            _exists_tuple: PhantomData
        }
    }
}

impl<'k, M, C> Default for CustomConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: CustomConstructor<'k, Creates=M> + Default
{
    fn default() -> CustomConstructedMachine<'k, M, C> {
        CustomConstructedMachine::new(&C::default())
    }
}

impl<'k, M, C> Automaton<'k> for CustomConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: CustomConstructor<'k, Creates=M>
{
    type Input = M::Input;
    type Action = M::Action;

    #[inline]
    fn transition(&mut self, input: &M::Input) -> M::Action {
        self.machine.transition(input)
    }
}

impl<'k, M, C> FiniteStateAutomaton<'k> for CustomConstructedMachine<'k, M, C> where
    M: FiniteStateAutomaton<'k>,
    C: CustomConstructor<'k, Creates=M> + Copy
{}

#[cfg(test)]
mod tests {
    use map_wrappers::{InputMachineMap, OutputMachineMap, LazyConstructor, 
        CustomConstructor};
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

    struct InMap;

    impl InputMachineMap for InMap {
        type In = i64;
        type Out = i64;
        fn input_transform(&self, input: &i64) -> i64 {
            -input
        }
    }

    #[test]
    fn input_map_test() {
        use map_wrappers::InputMappedMachine;
        let base_node = InternalStateMachine::new(Echoer, ());
        let mut wrapped_machine = InputMappedMachine::new(InMap, base_node);
        assert_eq!(wrapped_machine.transition(&-5), 5);
        assert_eq!(wrapped_machine.transition(&4), -4);
        assert_eq!(wrapped_machine.transition(&-7), 7);
        assert_eq!(wrapped_machine.transition(&12), -12);
    }

    struct OutMap;

    impl OutputMachineMap for OutMap {
        type In = i64;
        type Out = i64;

        fn output_transform(&self, val: i64) -> i64 {
            val + 1
        }
    }

    #[test]
    fn output_map_test() {
        use map_wrappers::OutputMappedMachine;
        let base_node = InternalStateMachine::new(Echoer, ());
        let mut wrapped_machine = OutputMappedMachine::new(OutMap, base_node);
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

    struct LazyWrapper;

    impl LazyConstructor<'static> for LazyWrapper {
        type Creates = InternalStateMachine<'static, IndefinitePlayback>;

        fn create(&self, input: &i64) -> Self::Creates {
            InternalStateMachine::new(IndefinitePlayback, *input)
        }
    }

    #[test]
    fn lazy_constructor_test() {
        use map_wrappers::LazyConstructedMachine;
        let mut new_machine = LazyConstructedMachine::new(LazyWrapper);
        assert_eq!(new_machine.transition(&2), 2);
        assert_eq!(new_machine.transition(&5), 2);
        assert_eq!(new_machine.transition(&-3), 2);
        assert_eq!(new_machine.transition(&2), 2);
        assert_eq!(new_machine.transition(&4), 2);


        let mut new_machine_1 = LazyConstructedMachine::new(LazyWrapper);
        assert_eq!(new_machine_1.transition(&5), 5);
        assert_eq!(new_machine_1.transition(&5), 5);
        assert_eq!(new_machine_1.transition(&-3), 5);
        assert_eq!(new_machine_1.transition(&-4), 5);
        assert_eq!(new_machine_1.transition(&-5), 5);
    }

    struct FixedWrapper; 

    impl CustomConstructor<'static> for FixedWrapper {
        type Creates = InternalStateMachine<'static, IndefinitePlayback>;
        fn create(&self) -> Self::Creates {
            InternalStateMachine::new(IndefinitePlayback, 12)
        }
    }
    #[test]
    fn custom_constructor_test() {
        use map_wrappers::CustomConstructedMachine;
        let mut new_machine = CustomConstructedMachine::new(&FixedWrapper);
        assert_eq!(new_machine.transition(&4), 12);
        assert_eq!(new_machine.transition(&-5), 12);
        assert_eq!(new_machine.transition(&2), 12);
        assert_eq!(new_machine.transition(&-2), 12);
        assert_eq!(new_machine.transition(&4), 12);

        let mut new_machine_1 = CustomConstructedMachine::new(&FixedWrapper);
        assert_eq!(new_machine_1.transition(&-3), 12);
        assert_eq!(new_machine_1.transition(&-3), 12);
        assert_eq!(new_machine_1.transition(&6), 12);
        assert_eq!(new_machine_1.transition(&-8), 12);
        assert_eq!(new_machine_1.transition(&-1), 12);
    }
}