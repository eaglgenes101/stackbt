use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use stackbt_automata_impl::automaton::Automaton;

pub trait WaitCondition {
    type Input;
    type Nonterminal;
    type Terminal;
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
    pub fn new() -> PredicateWait<F> {
        PredicateWait {
            _who_cares: PhantomData
        }
    }

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

    fn step(self, input: &F::Input) -> NodeResult<F::Nonterminal, F::Terminal, Self> {
        match F::do_end(input) {
            Statepoint::Terminal(t) => NodeResult::Terminal(t),
            Statepoint::Nonterminal(n) => NodeResult::Nonterminal(n, self)
        }
    }
}

pub trait NodeFn {
    type Input;
    type Output;
    fn call(&Self::Input) -> Self::Output;
}

/// Node which serves as a wrapper around a function, immediately terminating 
/// with its return value. 
pub struct Evaluation<F> where 
    F: NodeFn
{
    _who_cares: PhantomData<F>
}

impl<F> Evaluation<F> where 
    F: NodeFn
{
    pub fn new() -> Evaluation<F> {
        Evaluation {
            _who_cares: PhantomData
        }
    }

    pub fn with(_type_helper: F) -> Evaluation<F> {
        Evaluation {
            _who_cares: PhantomData
        }
    }
}

impl<F> Default for Evaluation<F> where 
    F: NodeFn
{
    fn default() -> Evaluation<F> {
        Evaluation::new()
    }
}

impl<F> BehaviorTreeNode for Evaluation<F> where 
    F: NodeFn
{
    type Input = F::Input;
    type Nonterminal = ();
    type Terminal = F::Output;

    fn step(self, input: &F::Input) -> NodeResult<(), F::Output, Self> {
        NodeResult::Terminal(F::call(input))
    }
}

pub struct LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + 'k
{
    machine: M,
    _m_bound: PhantomData<&'k M>,
    _exists_tuple: PhantomData<(N, T)>,
}

impl<'k, M, N, T> LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + 'k
{
    pub fn new(machine: M) -> LeafNode<'k, M, N, T> {
        LeafNode { 
            machine: machine,
            _m_bound: PhantomData,
            _exists_tuple: PhantomData
        }
    }
}

impl<'k, M, N, T> Default for LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + Default + 'k
{
    fn default() -> LeafNode<'k, M, N, T> {
        LeafNode::new(M::default())
    }
}

impl<'k, M, N, T> BehaviorTreeNode for LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + 'k
{
    type Input = M::Input;
    type Nonterminal = N;
    type Terminal = T;

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
    use base_nodes::{WaitCondition, NodeFn};

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

    impl NodeFn for EvaluationValue {
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
        use base_nodes::LeafNode;
        let machine = InternalStateMachine::with(ThingLeaf, 0);
        let thing = LeafNode::new(machine);
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