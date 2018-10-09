use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use stackbt_automata_impl::automaton::Automaton;

/// Parallel decider, which given the input and a slice of statepoints, 
/// decides whether to forward the statepoint box or to consume the 
/// statepoint box and exit. 
pub trait ParallelDecider {
    /// Type of the input to distribute among the parallel nodes. 
    type Input: 'static;
    /// Type of the nonterminals returned by each of the parallel nodes. 
    type Nonterm: 'static;
    ///  Type of the terminals returned by each of the parallel nodes. 
    type Term: 'static;
    /// Type of the terminal returned by the parallel node itself. 
    type Exit;
    /// Given the input and the boxed statepoint slice, return a statepoint 
    /// of either that boxed statepoint slice or a terminal value. 
    fn each_step(&self, &Self::Input, Box<[Statepoint<Self::Nonterm, Self::Term>]>) -> 
        Statepoint<Box<[Statepoint<Self::Nonterm, Self::Term>]>, Self::Exit>;
}

/// A parallel branch node, which is composed of a ParallelDecider on top of 
/// an automaton which returns boxed slices of statepoints. 
/// 
/// The idea is that the automaton this node is built on is a slice of 
/// node runners which, each step, are all executed with the same input, 
/// returning a boxed slice consisting of the statepoints reached by the 
/// nodes. To this end, StackBT's automata_impl library automatically 
/// implements the appropriate automaton trait on slices of automata which
/// return the same inputs and actions. 
/// 
/// However, the automaton used does not need to be slices of node runners, 
/// and this library does take advantage of this for testing by constructing 
/// test parallel nodes upon internal state machines returning statepoint 
/// slices. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ParallelBranchNode<C, D> where
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>>,
    D: ParallelDecider
{
    collection: C,
    decider: D
}

impl<C, D> ParallelBranchNode<C, D> where
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>>,
    D: ParallelDecider
{
    /// Create a new parallel branch node. 
    pub fn new(decider: D, machine: C) -> ParallelBranchNode<C, D> {
        ParallelBranchNode {
            collection: machine,
            decider: decider
        }
    }
}

impl<C, D> Default for ParallelBranchNode<C, D> where
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>> + Default,
    D: ParallelDecider + Default
{
    fn default() -> ParallelBranchNode<C, D> {
        ParallelBranchNode::new(D::default(), C::default())
    }
}

impl<C, D> BehaviorTreeNode for ParallelBranchNode<C, D> where 
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>>,
    D: ParallelDecider
{
    type Input = C::Input;
    type Nonterminal = C::Action;
    type Terminal = D::Exit;

    #[inline]
    fn step(self, input: &C::Input) -> NodeResult<Self::Nonterminal, D::Exit, Self> {
        let mut coll = self.collection;
        let results = coll.transition(input);
        let decision = self.decider.each_step(input, results);
        match decision {
            Statepoint::Nonterminal(ret) => NodeResult::Nonterminal(
                ret,
                Self::new(self.decider, coll)
            ),
            Statepoint::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

#[cfg(test)]
mod tests {
    use base_nodes::MachineWrapper;
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use node_runner::NodeRunner;
    use parallel_node::ParallelDecider;
    use stackbt_automata_impl::automaton::Automaton;
    use stackbt_automata_impl::internal_state_machine::{InternalTransition,
        InternalStateMachine};

    #[derive(Copy, Clone, Default)]
    struct IndefiniteIncrement;

    impl InternalTransition for IndefiniteIncrement {
        type Input = i64;
        type Internal = i64;
        type Action = Statepoint<i64, i64>;

        fn step(&self, input: &i64, state: &mut i64) -> Statepoint<i64, i64> {
            if *input > 0 {
                *state += 1;
                Statepoint::Nonterminal(*state)
            } else {
                *state = 0;
                Statepoint::Terminal(*state)
            }
        }
    }

    #[derive(Default)]
    struct MultiMachine {
        first: NodeRunner<MachineWrapper<InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>>,
        second: NodeRunner<MachineWrapper<InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>>,
    }

    #[derive(Copy, Clone, Default)]
    struct MultiMachineManipulator;

    impl InternalTransition for MultiMachineManipulator {
        type Input = i64;
        type Internal = MultiMachine;
        type Action = Box<[Statepoint<i64, i64>]>;

        fn step(&self, input: &i64, mach: &mut MultiMachine) -> Self::Action {
            let vec = vec![
                mach.first.transition(input),
                {
                    let val = -*input;
                    mach.second.transition(&val)
                }
            ];
            vec.into_boxed_slice()
        }
    }

    #[derive(Default)]
    struct MagicNumDecider;

    impl ParallelDecider for MagicNumDecider {
        type Input = i64;
        type Nonterm = i64;
        type Term = i64;
        type Exit = ();

        fn each_step(&self, input: &i64, slicebox: Box<[Statepoint<i64, i64>]>) -> 
            Statepoint<Box<[Statepoint<i64, i64>]>, ()>
        {
            if *input == 0 {
                Statepoint::Terminal(())
            } else {
                Statepoint::Nonterminal(slicebox)
            }
        }
    }

    #[test]
    fn parallel_node_test() {
        use parallel_node::ParallelBranchNode;
        let par_node = ParallelBranchNode::<InternalStateMachine<
            MultiMachineManipulator>, MagicNumDecider>::default();
        let par_node_1 = match par_node.step(&4) {
            NodeResult::Nonterminal(v, n) => {
                match v[0] {
                    Statepoint::Nonterminal(i) => assert_eq!(i, 1),
                    _ => unreachable!( "Expected subordinate nonterminal transition")
                };
                match v[1] {
                    Statepoint::Terminal(i) => assert_eq!(i, 0),
                    _ => unreachable!("Expected subordinate terminal transition"),
                };
                n
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition")
        };
        let par_node_2 = match par_node_1.step(&3) {
            NodeResult::Nonterminal(v, n) => {
                match v[0] {
                    Statepoint::Nonterminal(i) => assert_eq!(i, 2),
                    Statepoint::Terminal(_) => unreachable!(
                        "Expected subordinate nonterminal transition"
                    )
                };
                match v[1] {
                    Statepoint::Nonterminal(_) => unreachable!(
                        "Expected subordinate terminal transition"
                    ),
                    Statepoint::Terminal(i) => assert_eq!(i, 0),
                };
                n
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition")
        };
        let par_node_3 = match par_node_2.step(&-3) {
            NodeResult::Nonterminal(v, n) => {
                match v[0] {
                    Statepoint::Nonterminal(_) => unreachable!(
                        "Expected subordinate terminal transition"
                    ),
                    Statepoint::Terminal(i) => assert_eq!(i, 0),
                };
                match v[1] {
                    Statepoint::Nonterminal(i) => assert_eq!(i, 1),
                    Statepoint::Terminal(_) => unreachable!(
                        "Expected subordinate nonterminal transition"
                    )
                };
                n
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition")
        };
        let par_node_4 = match par_node_3.step(&-3) {
            NodeResult::Nonterminal(v, n) => {
                match v[0] {
                    Statepoint::Nonterminal(_) => unreachable!(
                        "Expected subordinate terminal transition"
                    ),
                    Statepoint::Terminal(i) => assert_eq!(i, 0),
                };
                match v[1] {
                    Statepoint::Nonterminal(i) => assert_eq!(i, 2),
                    Statepoint::Terminal(_) => unreachable!(
                        "Expected subordinate nonterminal transition"
                    )
                };
                n
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition")
        };
    }
}
