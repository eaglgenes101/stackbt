use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use stackbt_automata_impl::automaton::Automaton;

/// Wait condition for a predicate wait node. 
pub trait WaitCondition {
    /// Type of the input to take. 
    type Input;
    /// Type of the nonterminal statepoints to return. 
    type Nonterminal;
    /// Type of the terminal statepoints to return. 
    type Terminal;
    /// Given the input, determine whether to return a nonterminal state, or 
    /// a terminal state. 
    fn do_end(&Self::Input) -> Statepoint<Self::Nonterminal, Self::Terminal>;
}

/// Node whose function is to stall within itself until a function of its 
/// input return a terminal state, then terminates at that state. 
pub struct PredicateWait<F> where 
    F: WaitCondition
{
    _who_cares: PhantomData<F>
}

impl<F> PredicateWait<F> where 
    F: WaitCondition
{
    /// Create a new predicate wait node. 
    pub fn new() -> PredicateWait<F> {
        PredicateWait {
            _who_cares: PhantomData
        }
    }

    /// Create a new predicate wait node, using a dummy object to supply 
    /// the type of the waiting predicate. 
    pub fn with(_type_helper: F) -> PredicateWait<F> {
        PredicateWait {
            _who_cares: PhantomData
        }
    }
}

impl<F> Default for PredicateWait<F> where 
    F: WaitCondition
{
    fn default() -> PredicateWait<F> {
        PredicateWait::new()
    }
}

impl<F> BehaviorTreeNode for PredicateWait<F> where 
    F: WaitCondition
{
    type Input = F::Input;
    type Nonterminal = F::Nonterminal;
    type Terminal = F::Terminal;

    #[inline]
    fn step(self, input: &F::Input) -> NodeResult<F::Nonterminal, F::Terminal, Self> {
        match F::do_end(input) {
            Statepoint::Terminal(t) => NodeResult::Terminal(t),
            Statepoint::Nonterminal(n) => NodeResult::Nonterminal(n, self)
        }
    }
}

/// Wrapper for a function call, intended for Evaluation. 
pub trait CallWrapper {
    /// Type of the input to take. 
    type Input;
    /// Type of the output to return. 
    type Output;
    /// Given the input, determine the value to immediately terminate with. 
    fn call(&Self::Input) -> Self::Output;
}

/// Node which calls a function wrapper with its input, immediately 
/// terminating with its return value. 
pub struct Evaluation<F> where 
    F: CallWrapper
{
    _who_cares: PhantomData<F>
}

impl<F> Evaluation<F> where 
    F: CallWrapper
{
    /// Create a new evaluation node. 
    pub fn new() -> Evaluation<F> {
        Evaluation {
            _who_cares: PhantomData
        }
    }

    /// Create a new evaluation node, using a dummy object to supply the type
    /// of the function wrapper. 
    pub fn with(_type_helper: F) -> Evaluation<F> {
        Evaluation {
            _who_cares: PhantomData
        }
    }
}

impl<F> Default for Evaluation<F> where 
    F: CallWrapper
{
    fn default() -> Evaluation<F> {
        Evaluation::new()
    }
}

impl<F> BehaviorTreeNode for Evaluation<F> where 
    F: CallWrapper
{
    type Input = F::Input;
    type Nonterminal = ();
    type Terminal = F::Output;

    #[inline]
    fn step(self, input: &F::Input) -> NodeResult<(), F::Output, Self> {
        NodeResult::Terminal(F::call(input))
    }
}

/// Node wrapper for an automaton. 
pub struct MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static
{
    machine: M,
    _m_bound: PhantomData<&'static M>,
    _exists_tuple: PhantomData<(N, T)>,
}

impl<M, N, T> MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static
{
    /// Create a new machine wrapping node. 
    pub fn new(machine: M) -> MachineWrapper<M, N, T> {
        MachineWrapper { 
            machine,
            _m_bound: PhantomData,
            _exists_tuple: PhantomData
        }
    }
}

impl<M, N, T> Default for MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + Default + 'static
{
    fn default() -> MachineWrapper<M, N, T> {
        MachineWrapper::new(M::default())
    }
}

impl<M, N, T> BehaviorTreeNode for MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static
{
    type Input = M::Input;
    type Nonterminal = N;
    type Terminal = T;

    #[inline]
    fn step(self, input: &M::Input) -> NodeResult<N, T, Self> {
        let mut mach = self;
        match mach.machine.transition(input) {
            Statepoint::Nonterminal(thing) => {
                NodeResult::Nonterminal(thing, mach)
            },
            Statepoint::Terminal(thing) => {
                NodeResult::Terminal(thing)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use behavior_tree_node::Statepoint;
    use stackbt_automata_impl::internal_state_machine::InternalTransition;
    use base_nodes::{WaitCondition, CallWrapper};

    struct ThingPred;

    impl WaitCondition for ThingPred {
        type Input = i64;
        type Nonterminal = ();
        type Terminal = ();
        fn do_end(i: &i64) -> Statepoint<(), ()> {
            if *i == 0 {
                Statepoint::Terminal(())
            } else {
                Statepoint::Nonterminal(())
            }
        }
    }

    #[test]
    fn pred_wait_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use base_nodes::PredicateWait;
        let thing = PredicateWait::with(ThingPred);
        let thing_1 = match thing.step(&4) {
            NodeResult::Nonterminal(_, x) => x,
            _ => unreachable!("Expected nonterminal state")
        };
        match thing_1.step(&0) {
            NodeResult::Terminal(_) => (),
            _ => unreachable!("Expected terminal state"),
        }
    }

    struct EvaluationValue;

    impl CallWrapper for EvaluationValue {
        type Input = i64;
        type Output = i64;
        fn call(val: &i64) -> i64 {
            *val
        }
    }

    #[test]
    fn evaluation_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use base_nodes::Evaluation;
        let thing = Evaluation::with(EvaluationValue);
        match thing.step(&5) {
            NodeResult::Terminal(t) => assert!(t == 5),
            _ => unreachable!("Expected terminal"),
        };
    }

    #[derive(Copy, Clone)]
    struct ThingLeaf;

    impl InternalTransition for ThingLeaf {
        type Internal = i64;
        type Input = i64;
        type Action = Statepoint<i64, i64>;

        fn step(increment: &i64, accumulator: &mut i64) -> Statepoint<i64, i64> {
            if *increment == 0 {
                Statepoint::Terminal(*accumulator)
            } else {
                let orig_acc = *accumulator;
                *accumulator += increment;
                Statepoint::Nonterminal(orig_acc)
            }
        }
    }

    impl Default for ThingLeaf {
        fn default() -> ThingLeaf {
            ThingLeaf
        }
    }

    #[test]
    fn leaf_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use stackbt_automata_impl::internal_state_machine::InternalStateMachine;
        use base_nodes::MachineWrapper;
        let machine = InternalStateMachine::with(ThingLeaf, 0);
        let thing = MachineWrapper::new(machine);
        let thing_1 = match thing.step(&4) {
            NodeResult::Nonterminal(a, b) => {
                assert_eq!(a, 0);
                b
            },
            _ => unreachable!("Expected nonterminal state")
        };
        let thing_2 = match thing_1.step(&3) {
            NodeResult::Nonterminal(a, b) => {
                assert_eq!(a, 4);
                b
            },
            _ => unreachable!("Expected nonterminal state")
        };
        match thing_2.step(&0) {
            NodeResult::Terminal(t) => assert_eq!(t, 7),
            _ => unreachable!("Expected terminal state"),
        };
    }
}