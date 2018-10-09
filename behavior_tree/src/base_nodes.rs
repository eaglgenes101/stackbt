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
    fn do_end(&self, &Self::Input) -> Statepoint<Self::Nonterminal, Self::Terminal>;
}

impl<I, N, T> WaitCondition for Fn(&I) -> Statepoint<N, T> {
    type Input = I;
    type Nonterminal = N;
    type Terminal = T;
    fn do_end(&self, input: &I) -> Statepoint<N, T> {
        self(input)
    }
}

/// Node whose function is to stall within itself until a function of its 
/// input return a terminal state, then terminates at that state. 
/// 
/// # Example
/// ```
/// use stackbt_behavior_tree::behavior_tree_node::{Statepoint, 
///     BehaviorTreeNode, NodeResult};
/// use stackbt_behavior_tree::base_nodes::{WaitCondition, PredicateWait};
/// 
/// struct Echoer;
/// 
/// impl WaitCondition for Echoer {
///     type Input = Statepoint<(), ()>;
///     type Nonterminal = ();
///     type Terminal = ();
///     fn do_end(input: &Statepoint<(), ()>) -> Statepoint<(), ()> {
///         input.clone()
///     }
/// }
/// 
/// let echo_node_0 = PredicateWait::with(Echoer);
/// let echo_node_1 = match echo_node_0.step(&Statepoint::Nonterminal(())) {
///     NodeResult::Nonterminal(_, n) => n,
///     _ => unreachable!("Node doesn't return terminal")
/// };
/// match echo_node_1.step(&Statepoint::Terminal(())) {
///     NodeResult::Terminal(t) => (), //Expected case
///     _ => unreachable!("Node doesn't return nonterminal")
/// };
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PredicateWait<F> where 
    F: WaitCondition
{
    condition: F
}

impl<F> PredicateWait<F> where 
    F: WaitCondition
{
    /// Create a new predicate wait node. 
    pub fn new(cond: F) -> PredicateWait<F> {
        PredicateWait {
            condition: cond
        }
    }
}

impl<F> Default for PredicateWait<F> where 
    F: WaitCondition + Default
{
    fn default() -> PredicateWait<F> {
        PredicateWait::new(F::default())
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
        match self.condition.do_end(input) {
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
    fn call(&self, &Self::Input) -> Self::Output;
}

impl<I, O> CallWrapper for Fn(&I) -> O {
    type Input = I;
    type Output = O;
    fn call(&self, input: &I) -> O {
        self(input)
    }
}

/// Node which calls a function wrapper with its input, immediately 
/// terminating with its return value. 
/// # Example
/// ```
/// use stackbt_behavior_tree::behavior_tree_node::{Statepoint, 
///     BehaviorTreeNode, NodeResult};
/// use stackbt_behavior_tree::base_nodes::{CallWrapper, Evaluation};
/// 
/// struct IsThree;
/// 
/// impl CallWrapper for IsThree {
///     type Input = ();
///     type Output = i64;
///     fn call(_input: &()) -> i64 {
///         3
///     }
/// }
/// 
/// let three_node = Evaluation::with(IsThree);
/// match three_node.step(&()) {
///     NodeResult::Terminal(t) => assert_eq!(t, 3), //Expected case
///     _ => unreachable!("Node doesn't return nonterminal")
/// };
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Evaluation<F> where 
    F: CallWrapper
{
    to_call: F
}

impl<F> Evaluation<F> where 
    F: CallWrapper
{
    /// Create a new evaluation node. 
    pub fn new(to_call: F) -> Evaluation<F> {
        Evaluation {
            to_call
        }
    }
}

impl<F> Default for Evaluation<F> where 
    F: CallWrapper + Default
{
    fn default() -> Evaluation<F> {
        Evaluation::new(F::default())
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
        NodeResult::Terminal(self.to_call.call(input))
    }
}

/// Node wrapper for an automaton. 
#[derive(Copy, Clone, PartialEq, Debug)]
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
        fn do_end(&self, i: &i64) -> Statepoint<(), ()> {
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
        let thing = PredicateWait::new(ThingPred);
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
        fn call(&self, val: &i64) -> i64 {
            *val
        }
    }

    #[test]
    fn evaluation_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use base_nodes::Evaluation;
        let thing = Evaluation::new(EvaluationValue);
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

        fn step(&self, increment: &i64, accumulator: &mut i64) -> Statepoint<i64, i64> {
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
        let machine = InternalStateMachine::new(ThingLeaf, 0);
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