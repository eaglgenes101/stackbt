use behavior_tree_node::Statepoint;
use serial_node::{Enumerable, SerialDecider, NontermDecision, TermDecision};
use parallel_node::ParallelDecider;
use std::marker::PhantomData;

/// Runs all nodes in sequence, one at a time, regardless of how they resolve 
/// in the end. 
pub struct SerialRunner<E, N, T> where E: Enumerable {
    _who_cares: PhantomData<(E, N, T)>
}

impl<E, N, T> SerialDecider for SerialRunner<E, N, T> where 
    E: Enumerable
{
    type Enum = E;
    type Nonterm = N;
    type Term = T;
    type Exit = ();

    fn on_nonterminal(_ordinal: E, statept: N) -> NontermDecision<E, N, ()> {
        NontermDecision::Step(statept)
    }

    fn on_terminal(ordinal: E, statept: T) -> TermDecision<E, T, ()> {
        match ordinal.successor() {
            Option::Some(e) => {
                TermDecision::Trans(e, statept)
            },
            Option::None => TermDecision::Exit(())
        }
    }
}

/// Runs nodes in sequence until one resolves into an Option::Some, which 
/// depending on context may be either success or failure. 
pub struct SerialSelector<E, N, T> where E: Enumerable {
    _who_cares: PhantomData<(E, N, T)>
}

impl<E, N, T> SerialDecider for SerialSelector<E, N, T> where 
    E: Enumerable
{
    type Enum = E;
    type Nonterm = N;
    type Term = Option<T>;
    type Exit = Option<(E, T)>;

    fn on_nonterminal(_ord: E, statept: N) -> NontermDecision<E, N, Option<(E, T)>> {
        NontermDecision::Step(statept)
    }

    fn on_terminal(ord: E, statept: Option<T>) -> TermDecision<E, Option<T>, Option<(E, T)>> {
        match statept {
            Option::Some(t) => TermDecision::Exit(Option::Some((ord, t))),
            Option::None => match ord.successor() {
                Option::Some(e) => TermDecision::Trans(e, Option::None),
                Option::None => TermDecision::Exit(Option::None)
            }
        }
    }
}

/// Runs nodes in parallel until at some point, they all terminate or 
/// enter a trap state indicated by returning a statepoint terminal 
/// as the nonterminal. 
pub struct ParallelRunner<I, N, R, T> {
    _who_cares: PhantomData<(I, N, R, T)>
}

impl<I, N, R, T> ParallelDecider for ParallelRunner<I, N, R, T> where 
    I: 'static,
    N: 'static,
    R: 'static,
    T: 'static
{
    type Input = I;
    type Nonterm = Statepoint<N, R>;
    type Term = T;
    type Exit = Box<[Statepoint<R, T>]>;

    #[inline]
    fn each_step(_input: &I, states: Box<[Statepoint<Statepoint<N, R>, T>]>) -> 
        Statepoint<Box<[Statepoint<Self::Nonterm, T>]>, Self::Exit> 
    {
        if states.iter().any(|val| match val {
            Statepoint::Nonterminal(Statepoint::Nonterminal(_)) => true,
            _ => false 
        }) {
            Statepoint::Nonterminal(states)
        } else {
            let vec = states.into_vec().into_iter().map(|val| 
                match val {
                    Statepoint::Nonterminal(v) => match v {
                        Statepoint::Terminal(k) => Statepoint::Nonterminal(k),
                        _ => unreachable!("No currently pending nodes")
                    },
                    Statepoint::Terminal(k) => Statepoint::Terminal(k)
                }
            ).collect::<Vec<_>>();
            Statepoint::Terminal(vec.into_boxed_slice())
        }
    }
}

/// Runs nodes until one terminates, resolving to a tuple of the terminating
/// index and its terminal state when it does. 
pub struct ParallelRacer<I, N, T>  {
    _who_cares: PhantomData<(I, N, T)>
}

impl<I, N, T> ParallelDecider for ParallelRacer<I, N, T> where 
    I: 'static,
    N: 'static,
    T: 'static + Clone
{
    type Input = I;
    type Nonterm = N;
    type Term = T;
    type Exit = (usize, T);

    #[inline]
    fn each_step(_input: &I, states: Box<[Statepoint<N, T>]>) -> 
        Statepoint<Box<[Statepoint<N, T>]>, (usize, T)> 
    {
        let mut retval = Option::None;
        for value in states.iter().enumerate() {
            if let Statepoint::Terminal(val) = value.1 {
                retval = Option::Some((value.0, val.clone()));
                break;
            }
        };
        match retval {
            Option::None => Statepoint::Nonterminal(states),
            Option::Some(v) => Statepoint::Terminal(v)
        }

    }
}

#[cfg(test)]
mod tests {
    use base_nodes::MachineWrapper;
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use stackbt_automata_impl::automaton::Automaton;
    use stackbt_automata_impl::internal_state_machine::{InternalTransition,
        InternalStateMachine};
    use stackbt_automata_impl::ref_state_machine::{ReferenceTransition,
        RefStateMachine};
    use serial_node::{Enumerable, EnumNode};
    use map_wrappers::{OutputNodeMap, OutputMappedNode};
    use control_wrappers::{NodeGuard, GuardedNode};
    use node_runner::NodeRunner;
    use std::marker::PhantomData;

    #[derive(Copy, Clone, Default)]
    struct IndefiniteIncrement;

    impl InternalTransition for IndefiniteIncrement {
        type Input = i64;
        type Internal = i64;
        type Action = Statepoint<i64, i64>;

        fn step(input: &i64, state: &mut i64) -> Statepoint<i64, i64> {
            if *input >= 0 {
                *state += 1;
                Statepoint::Nonterminal(*state)
            } else {
                Statepoint::Terminal(*state)
            }
        }
    }


    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    enum IndexEnum {
        First,
        Second
    }
    
    impl Enumerable for IndexEnum {
        fn zero() -> Self {
            IndexEnum::First
        }

        fn successor(self) -> Option<Self> {
            match self {
                IndexEnum::First => Option::Some(IndexEnum::Second),
                IndexEnum::Second => Option::None
            }
        }
    }

    enum MultiMachine {
        First(MachineWrapper<InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>),
        Second(MachineWrapper<InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>)
    }

    impl BehaviorTreeNode for MultiMachine {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;

        fn step(self, input: &i64) -> NodeResult<i64, i64, Self> {
            match self {
                MultiMachine::First(n) => {
                    match n.step(input) {
                        NodeResult::Nonterminal(r, m) => NodeResult::Nonterminal(
                            r,
                            MultiMachine::First(m)
                        ),
                        NodeResult::Terminal(t) => NodeResult::Terminal(t)
                    }
                },
                MultiMachine::Second(n) => {
                    match n.step(input) {
                        NodeResult::Nonterminal(r, m) => NodeResult::Nonterminal(
                            r,
                            MultiMachine::Second(m)
                        ),
                        NodeResult::Terminal(t) => NodeResult::Terminal(t)
                    }
                }
            }
        }
    }
    
    impl EnumNode for MultiMachine {

        type Discriminant = IndexEnum;

        fn new(thing: IndexEnum) -> MultiMachine {
            match thing {
                IndexEnum::First => MultiMachine::First(
                    MachineWrapper::default()
                ),
                IndexEnum::Second => MultiMachine::Second(
                    MachineWrapper::default()
                )
            }
        }

        fn discriminant(&self) -> IndexEnum {
            match self {
                MultiMachine::First(_) => IndexEnum::First,
                MultiMachine::Second(_) => IndexEnum::Second
            }
        }
    }

    #[test]
    fn serial_runner_test() {
        use serial_node::{SerialBranchNode, NontermReturn};
        use node_compositions::SerialRunner;
        let test_node = SerialBranchNode::<MultiMachine, SerialRunner<_, _, _>, ()>
            ::default();
        let test_node_1 = match test_node.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::First);
                        assert_eq!(v, 1);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_2 = match test_node_1.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::First);
                        assert_eq!(v, 2);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_3 = match test_node_2.step(&-1) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Terminal(e, v) => {
                        assert_eq!(e, IndexEnum::First);
                        assert_eq!(v, 2);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_4 = match test_node_3.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::Second);
                        assert_eq!(v, 1);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        match test_node_4.step(&-3) {
            NodeResult::Terminal(_) => (),
            _ => unreachable!("Expected terminal transition")
        };
    }

    struct ValSep;

    impl OutputNodeMap for ValSep {
        type NontermIn = i64;
        type NontermOut = i64;
        type TermIn = i64;
        type TermOut = Option<i64>;

        fn nonterminal_transform(inval: i64) -> i64 {
            inval
        }

        fn terminal_transform(inval: i64) -> Option<i64> {
            if inval >= 2 {
                Option::Some(inval)
            } else {
                Option::None
            }
        }
    }

    enum WrappedMachine {
        First(OutputMappedNode<MachineWrapper<InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>, ValSep>),
        Second(OutputMappedNode<MachineWrapper<InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>, ValSep>)
    }

    impl BehaviorTreeNode for WrappedMachine {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = Option<i64>;

        fn step(self, input: &i64) -> NodeResult<i64, Option<i64>, Self> {
            match self {
                WrappedMachine::First(n) => {
                    match n.step(input) {
                        NodeResult::Nonterminal(r, m) => NodeResult::Nonterminal(
                            r,
                            WrappedMachine::First(m)
                        ),
                        NodeResult::Terminal(t) => NodeResult::Terminal(t)
                    }
                },
                WrappedMachine::Second(n) => {
                    match n.step(input) {
                        NodeResult::Nonterminal(r, m) => NodeResult::Nonterminal(
                            r,
                            WrappedMachine::Second(m)
                        ),
                        NodeResult::Terminal(t) => NodeResult::Terminal(t)
                    }
                }
            }
        }
    }
    
    impl EnumNode for WrappedMachine {
        type Discriminant = IndexEnum;

        fn new(thing: IndexEnum) -> WrappedMachine {
            match thing {
                IndexEnum::First => WrappedMachine::First(
                    OutputMappedNode::default()
                ),
                IndexEnum::Second => WrappedMachine::Second(
                    OutputMappedNode::default()
                )
            }
        }

        fn discriminant(&self) -> IndexEnum {
            match self {
                WrappedMachine::First(_) => IndexEnum::First,
                WrappedMachine::Second(_) => IndexEnum::Second
            }
        }
    }

    #[test]
    fn serial_selector_test() {
        use serial_node::{SerialBranchNode, NontermReturn};
        use node_compositions::SerialSelector;
        let test_node = SerialBranchNode::<WrappedMachine, SerialSelector< _, _, _>, 
            Option<_>>::default();
        let test_node_1 = match test_node.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::First);
                        assert_eq!(v, 1);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_2 = match test_node_1.step(&-1) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Terminal(e, v) => {
                        assert_eq!(e, IndexEnum::First);
                        match v {
                            Option::None => (),
                            _ => unreachable!("Expected subordinate failure")
                        }
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_3 = match test_node_2.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::Second);
                        assert_eq!(v, 1);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_4 = match test_node_3.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::Second);
                        assert_eq!(v, 2);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_5 = match test_node_4.step(&3) {
            NodeResult::Nonterminal(ret, n) => {
                match ret {
                    NontermReturn::Nonterminal(e, v) => {
                        assert_eq!(e, IndexEnum::Second);
                        assert_eq!(v, 3);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        match test_node_5.step(&-3) {
            NodeResult::Terminal(t) => {
                match t {
                    Option::Some(k) => assert_eq!(k, (IndexEnum::Second, 3)),
                    _ => unreachable!("Expected return with success")
                }
            },
            _ => unreachable!("Expected terminal transition")
        };

    }

    #[derive(Copy, Clone)]
    enum TwoCycler {
        First, 
        Second
    }

    impl Default for TwoCycler {
        fn default() -> TwoCycler {
            TwoCycler::First
        }
    }
    
    impl ReferenceTransition for TwoCycler {
        type Input = ();
        type Action = Statepoint<(), ()>;
        fn step(self, input: &()) -> (Self::Action, Self) {
            match self {
                TwoCycler::First => (Statepoint::Nonterminal(()), TwoCycler::Second),
                TwoCycler::Second => (Statepoint::Terminal(()), TwoCycler::First)
            }
        }
    }

    #[derive(Copy, Clone)]
    enum ThreeCycler {
        First,
        Second, 
        Third
    }

    impl Default for ThreeCycler {
        fn default() -> ThreeCycler {
            ThreeCycler::First
        }
    }

    impl ReferenceTransition for ThreeCycler {
        type Input = ();
        type Action = Statepoint<(), ()>;
        fn step(self, input: &()) -> (Self::Action, Self) {
            match self {
                ThreeCycler::First => (Statepoint::Nonterminal(()), ThreeCycler::Second),
                ThreeCycler::Second => (Statepoint::Nonterminal(()), ThreeCycler::Third),
                ThreeCycler::Third => (Statepoint::Terminal(()), ThreeCycler::First)
            }
        }
    }

    #[derive(Copy, Clone)]
    struct ParMachine {
        first: RefStateMachine<'static, TwoCycler>,
        second: RefStateMachine<'static, ThreeCycler>
    }

    impl Default for ParMachine {
        fn default() -> ParMachine {
            ParMachine {
                first: RefStateMachine::new(TwoCycler::default()),
                second: RefStateMachine::new(ThreeCycler::default())
            }
        }
    }

    #[derive(Copy, Clone, Default)]
    struct ParMachineController;

    impl InternalTransition for ParMachineController {
        type Input = ();
        type Internal = ParMachine;
        type Action = Box<[Statepoint<Statepoint<(), ()>, ()>]>;

        fn step(input: &(), mach: &mut ParMachine) -> Box<[Statepoint<
            Statepoint<(), ()>, ()>]> 
        {
            let thing = vec![
                Statepoint::Nonterminal(mach.first.transition(input)), 
                Statepoint::Nonterminal(mach.second.transition(input))
            ];
            thing.into_boxed_slice()
        }
    }

    #[test]
    fn parallel_runner_test() {
        use parallel_node::ParallelBranchNode;
        use node_compositions::ParallelRunner;
        let test_node = ParallelBranchNode::<InternalStateMachine<
            ParMachineController>, ParallelRunner<_, _, _, _>>::default();
        let test_node_1 = match test_node.step(&()) {
            NodeResult::Nonterminal(v, n) => match v.as_ref() {
                [
                    Statepoint::Nonterminal(Statepoint::Nonterminal(())),
                    Statepoint::Nonterminal(Statepoint::Nonterminal(()))
                ] => n,
                _ => unreachable!("Expected only (nonterminal, nonterminal)")
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_2 = match test_node_1.step(&()) {
            NodeResult::Nonterminal(v, n) => match v.as_ref() {
                [
                    Statepoint::Nonterminal(Statepoint::Terminal(())),
                    Statepoint::Nonterminal(Statepoint::Nonterminal(()))
                ] => n,
                _ => unreachable!("Expected only (terminal, nonterminal)")
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_3 = match test_node_2.step(&()) {
            NodeResult::Nonterminal(v, n) => match v.as_ref() {
                [
                    Statepoint::Nonterminal(Statepoint::Nonterminal(())),
                    Statepoint::Nonterminal(Statepoint::Terminal(()))
                ] => n,
                _ => unreachable!("Expected only (nonterminal, terminal)")
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_4 = match test_node_3.step(&()) {
            NodeResult::Nonterminal(v, n) => match v.as_ref() {
                [
                    Statepoint::Nonterminal(Statepoint::Terminal(())),
                    Statepoint::Nonterminal(Statepoint::Nonterminal(()))
                ] => n,
                _ => unreachable!("Expected only (terminal, nonterminal)")
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_5 = match test_node_4.step(&()) {
            NodeResult::Nonterminal(v, n) => match v.as_ref() {
                [
                    Statepoint::Nonterminal(Statepoint::Nonterminal(())),
                    Statepoint::Nonterminal(Statepoint::Nonterminal(()))
                ] => n,
                _ => unreachable!("Expected only (nonterminal, nonterminal)")
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        match test_node_5.step(&()) {
            NodeResult::Terminal(_) => (),
            _ => unreachable!("Expected terminal transition")
        };
    }

    #[derive(Default, Copy, Clone)]
    struct WrapParMachineController;

    impl InternalTransition for WrapParMachineController {
        type Input = ();
        type Internal = ParMachine;
        type Action = Box<[Statepoint<(), ()>]>;

        fn step(input: &(), mach: &mut ParMachine) -> Box<[Statepoint<(), ()>]> {
            let thing = vec![
                mach.first.transition(input), 
                mach.second.transition(input)
            ];
            thing.into_boxed_slice()
        }
    }

    #[test]
    fn parallel_racer_test() {
        use parallel_node::ParallelBranchNode;
        use node_compositions::ParallelRacer;
        let test_node = ParallelBranchNode::<InternalStateMachine<
            WrapParMachineController>, ParallelRacer<_, _, _>>::default();
        let test_node_1 = match test_node.step(&()) {
            NodeResult::Nonterminal(_, n) => n,
            _ => unreachable!("Expected nonterminal transition")
        };
        match test_node_1.step(&()) {
            NodeResult::Terminal(_) => (),
            _ => unreachable!("Expected terminal transition")
        };
    }
}