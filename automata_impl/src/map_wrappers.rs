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
    fn input_transform(&Self::In) -> Self::Out;
}

/// Wrapper for a automaton which converts between the provided input type
/// and the input type expected by the automaton. 
pub struct InputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: InputMachineMap<Out=M::Input>
{
    machine: M,
    _exists_tuple: PhantomData<&'k W>
}

impl<'k, M, W> InputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: InputMachineMap<Out=M::Input>
{
    /// Create a new input mapped automaton. 
    pub fn new(machine: M) -> InputMappedMachine<'k, M, W> {
        InputMappedMachine {
            machine,
            _exists_tuple: PhantomData
        }
    }

    /// Create an new input mapped automaton, using a dummy object to supply
    /// the type of the wrapper. 
    pub fn with(_type_helper: W, machine: M) -> InputMappedMachine<'k, M, W> {
        InputMappedMachine {
            machine,
            _exists_tuple: PhantomData
        }
    }
}

impl<'k, M, W> Default for InputMappedMachine<'k, M, W> where
    M: Automaton<'k> + Default,
    W: InputMachineMap<Out=M::Input>
{
    fn default() -> InputMappedMachine<'k, M, W> {
        InputMappedMachine::new(M::default())
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
        self.machine.transition(&W::input_transform(input))
    }
}

impl<'k, M, W> FiniteStateAutomaton<'k> for InputMappedMachine<'k, M, W> where
    M: FiniteStateAutomaton<'k>,
    W: InputMachineMap<Out=M::Input>
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
    fn output_transform(Self::In) -> Self::Out;
}

/// Wrapper for an automaton which converts between the actions emitted by the
/// automaton and the ones exposed by the wrapper. 
pub struct OutputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: OutputMachineMap<In=M::Action>
{
    machine: M,
    _exists_tuple: PhantomData<&'k W>
}

impl<'k, M, W> OutputMappedMachine<'k, M, W> where
    M: Automaton<'k>,
    W: OutputMachineMap<In=M::Action>
{
    /// Create a new output mapped automaton. 
    pub fn new(machine: M) -> OutputMappedMachine<'k, M, W> {
        OutputMappedMachine {
            machine,
            _exists_tuple: PhantomData
        }
    }

    /// Create an new output mapped automaton, using a dummy object to supply
    /// the type of the wrapper. 
    pub fn with(_type_helper: W, machine: M) -> OutputMappedMachine<'k, M, W> {
        OutputMappedMachine {
            machine,
            _exists_tuple: PhantomData
        }
    }
}

impl<'k, M, W> Default for OutputMappedMachine<'k, M, W> where
    M: Automaton<'k> + Default,
    W: OutputMachineMap<In=M::Action>
{
    fn default() -> OutputMappedMachine<'k, M, W> {
        OutputMappedMachine::new(M::default())
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
        W::output_transform(self.machine.transition(input))
    }
}

impl<'k, M, W> FiniteStateAutomaton<'k> for OutputMappedMachine<'k, M, W> where
    M: FiniteStateAutomaton<'k>,
    W: OutputMachineMap<In=M::Action> 
{}


/// Lazy constructor for an automaton, depending on the first input. 
pub trait LazyConstructor<'k> {
    /// Type of the automaton to create. 
    type Creates: Automaton<'k>;
    /// Create a new automaton. 
    fn create(&<Self::Creates as Automaton<'k>>::Input) -> Self::Creates;
}

/// Wrapper for for a machine, which defers initialization until the first 
/// input is supplied, after which the machine is constructed using this input
/// as a parameter. 
pub struct LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: LazyConstructor<'k, Creates=M>
{
    machine: Option<M>,
    _exists_tuple: PhantomData<&'k C>
}

impl<'k, M, C> LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: LazyConstructor<'k, Creates=M>
{
    /// Create an new lazily constructed automaton. 
    pub fn new() -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine{
            machine: Option::None,
            _exists_tuple: PhantomData
        }
    }

    /// Create an new lazily constructed automaton, using a dummy object to 
    /// supply the type of the wrapper. 
    pub fn with(_type_assist: C) -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine{
            machine: Option::None,
            _exists_tuple: PhantomData
        }
    }

    /// Wrap an existing automaton in the lazily constructed automaton 
    /// wrapper.
    pub fn from_existing(machine: M) -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine {
            machine: Option::Some(machine),
            _exists_tuple: PhantomData
        }
    }
}

impl<'k, M, C> Default for LazyConstructedMachine<'k, M, C> where
    M: Automaton<'k>,
    C: LazyConstructor<'k, Creates=M>
{
    fn default() -> LazyConstructedMachine<'k, M, C> {
        LazyConstructedMachine::new()
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
        let maybe_machine = self.machine.take();
        match maybe_machine {
            Option::Some(mut m) => {
                let retval = m.transition(input);
                self.machine = Option::Some(m);
                retval
            },
            Option::None => {
                let mut new_machine = C::create(input);
                let retval = new_machine.transition(input);
                self.machine = Option::Some(new_machine);
                retval
            }
        }
    }
}

impl<'k, M, C> FiniteStateAutomaton<'k> for LazyConstructedMachine<'k, M, C> where
    M: FiniteStateAutomaton<'k>,
    C: LazyConstructor<'k, Creates=M>
{}

/// Eager constructor for an automaton. 
pub trait CustomConstructor<'k> {
    /// Type of the automaton to create. 
    type Creates: Automaton<'k>;
    /// Create a new automaton. 
    fn create() -> Self::Creates;
}

/// Wrapper for an automaton which designates a default constructor for that
/// automaton, constructing it from the constructor. 
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
    pub fn new() -> CustomConstructedMachine<'k, M, C> {
        CustomConstructedMachine {
            machine: C::create(),
            _exists_tuple: PhantomData
        }
    }

    /// Create an new custom constructed automaton, using a dummy object to 
    /// supply the type of the wrapper. 
    pub fn with(_type_assist: C) -> CustomConstructedMachine<'k, M, C> {
        CustomConstructedMachine {
            machine: C::create(),
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
    C: CustomConstructor<'k, Creates=M>
{
    fn default() -> CustomConstructedMachine<'k, M, C> {
        CustomConstructedMachine::new()
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
    C: CustomConstructor<'k, Creates=M>
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

        fn step(input: &i64, state: &mut ()) -> i64 {
            *input
        }
    }

    struct InMap;

    impl InputMachineMap for InMap {
        type In = i64;
        type Out = i64;
        fn input_transform(input: &i64) -> i64 {
            -input
        }
    }

    #[test]
    fn input_map_test() {
        use map_wrappers::InputMappedMachine;
        let base_node = InternalStateMachine::with(Echoer, ());
        let mut wrapped_machine = InputMappedMachine::with(InMap, base_node);
        assert_eq!(wrapped_machine.transition(&-5), 5);
        assert_eq!(wrapped_machine.transition(&4), -4);
        assert_eq!(wrapped_machine.transition(&-7), 7);
        assert_eq!(wrapped_machine.transition(&12), -12);
    }

    struct OutMap;

    impl OutputMachineMap for OutMap {
        type In = i64;
        type Out = i64;

        fn output_transform(val: i64) -> i64 {
            val + 1
        }
    }

    #[test]
    fn output_map_test() {
        use map_wrappers::OutputMappedMachine;
        let base_node = InternalStateMachine::with(Echoer, ());
        let mut wrapped_machine = OutputMappedMachine::with(OutMap, base_node);
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

        fn step(input: &i64, state: &mut i64) -> i64{
            *state
        }
    }

    struct LazyWrapper;

    impl LazyConstructor<'static> for LazyWrapper {
        type Creates = InternalStateMachine<'static, IndefinitePlayback>;

        fn create(input: &i64) -> Self::Creates {
            InternalStateMachine::with(IndefinitePlayback, *input)
        }
    }

    #[test]
    fn lazy_constructor_test() {
        use map_wrappers::LazyConstructedMachine;
        let mut new_machine = LazyConstructedMachine::with(LazyWrapper);
        assert_eq!(new_machine.transition(&2), 2);
        assert_eq!(new_machine.transition(&5), 2);
        assert_eq!(new_machine.transition(&-3), 2);
        assert_eq!(new_machine.transition(&2), 2);
        assert_eq!(new_machine.transition(&4), 2);


        let mut new_machine_1 = LazyConstructedMachine::with(LazyWrapper);
        assert_eq!(new_machine_1.transition(&5), 5);
        assert_eq!(new_machine_1.transition(&5), 5);
        assert_eq!(new_machine_1.transition(&-3), 5);
        assert_eq!(new_machine_1.transition(&-4), 5);
        assert_eq!(new_machine_1.transition(&-5), 5);
    }

    struct FixedWrapper; 

    impl CustomConstructor<'static> for FixedWrapper {
        type Creates = InternalStateMachine<'static, IndefinitePlayback>;
        fn create() -> Self::Creates {
            InternalStateMachine::with(IndefinitePlayback, 12)
        }
    }
    #[test]
    fn custom_constructor_test() {
        use map_wrappers::CustomConstructedMachine;
        let mut new_machine = CustomConstructedMachine::with(FixedWrapper);
        assert_eq!(new_machine.transition(&4), 12);
        assert_eq!(new_machine.transition(&-5), 12);
        assert_eq!(new_machine.transition(&2), 12);
        assert_eq!(new_machine.transition(&-2), 12);
        assert_eq!(new_machine.transition(&4), 12);

        let mut new_machine_1 = CustomConstructedMachine::with(FixedWrapper);
        assert_eq!(new_machine_1.transition(&-3), 12);
        assert_eq!(new_machine_1.transition(&-3), 12);
        assert_eq!(new_machine_1.transition(&6), 12);
        assert_eq!(new_machine_1.transition(&-8), 12);
        assert_eq!(new_machine_1.transition(&-1), 12);
    }
}