use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;

/// A collection of nodes, all of which have the same input, nonterminal, 
/// and terminal types. The collection is associated with a particular 
/// enumeration, which is intended to enumerate the behavior tree nodes 
/// in the implementation. 
pub trait NodeCollection: Default {
    type Input;
    type Nonterminal;
    type Terminal;

    /// Run one particular node in the collection, automatically restarting 
    /// the node if it terminates. 
    fn run_all(&mut self, &Self::Input) -> Box<[Statepoint<Self::Nonterminal, 
        Self::Terminal>]>;
}

pub enum ParallelDecision<R, X> {
    Stay(R),
    Exit(X)
}

pub trait ParallelDecider {
    type Input: 'static;
    type Nonterm: 'static;
    type Term: 'static;
    type Exit;
    fn each_step(&Self::Input, Box<[Statepoint<Self::Nonterm, Self::Term>]>) -> 
        ParallelDecision<Box<[Statepoint<Self::Nonterm, Self::Term>]>, Self::Exit>;
}

pub struct ParallelBranchNode<C, D> where
    C: NodeCollection + 'static,
    D: ParallelDecider<Input=C::Input, Nonterm=C::Nonterminal, 
        Term=C::Terminal>
{
    collection: C,
    _exists_tuple: PhantomData<D>
}

impl<C, D> ParallelBranchNode<C, D> where
    C: NodeCollection + 'static,
    D: ParallelDecider<Input=C::Input, Nonterm=C::Nonterminal, 
        Term=C::Terminal>
{
    fn new() -> ParallelBranchNode<C, D> {
        ParallelBranchNode {
            collection: C::default(),
            _exists_tuple: PhantomData
        }
    }

    fn from_existing(existing: C) -> ParallelBranchNode<C, D> {
        ParallelBranchNode {
            collection: existing,
            _exists_tuple: PhantomData
        }
    }
}

impl<C, D> Default for ParallelBranchNode<C, D> where
    C: NodeCollection + 'static,
    D: ParallelDecider<Input=C::Input, Nonterm=C::Nonterminal, 
        Term=C::Terminal>
{
    fn default() -> ParallelBranchNode<C, D> {
        ParallelBranchNode::new()
    }
}

impl<C, D> BehaviorTreeNode for ParallelBranchNode<C, D> where 
    C: NodeCollection + 'static,
    D: ParallelDecider<Input=C::Input, Nonterm=C::Nonterminal, 
        Term=C::Terminal>
{
    type Input = C::Input;
    type Nonterminal = Box<[Statepoint<C::Nonterminal, C::Terminal>]>;
    type Terminal = D::Exit;

    fn step(self, input: &C::Input) -> NodeResult<Self::Nonterminal, D::Exit, Self> {
        let mut coll = self.collection;
        let mut results = coll.run_all(input);
        let decision = D::each_step(input, results);
        match decision {
            ParallelDecision::Stay(r) => NodeResult::Nonterminal(
                r,
                Self::from_existing(coll)
            ),
            ParallelDecision::Exit(t) => NodeResult::Terminal(t)
        }
    }
}

#[cfg(test)]
mod tests {
    
    use base_nodes::LeafNode;
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use node_runner::NodeRunner;
    use parallel_node::{NodeCollection, ParallelDecider, ParallelDecision};
    use stackbt_automata_impl::automaton::Automaton;
    use stackbt_automata_impl::internal_state_machine::{InternalTransition,
        InternalStateMachine};

    #[derive(Copy, Clone, Default)]
    struct IndefiniteIncrement;

    impl InternalTransition for IndefiniteIncrement {
        type Input = i64;
        type Internal = i64;
        type Action = Statepoint<i64, i64>;

        fn step(input: &i64, state: &mut i64) -> Statepoint<i64, i64> {
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
        first: NodeRunner<LeafNode<'static, InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>>,
        second: NodeRunner<LeafNode<'static, InternalStateMachine<'static, 
            IndefiniteIncrement>, i64, i64>>,
    }

    impl NodeCollection for MultiMachine {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;

        fn run_all(&mut self, input: &i64) -> Box<[Statepoint<i64, i64>]> {
            let vec = vec![
                self.first.transition(input),
                {
                    let val = -*input;
                    self.second.transition(&val)
                }
            ];
            vec.into_boxed_slice()
        }
    }

    struct MagicNumDecider;

    impl ParallelDecider for MagicNumDecider {
        type Input = i64;
        type Nonterm = i64;
        type Term = i64;
        type Exit = ();

        fn each_step(input: &i64, slicebox: Box<[Statepoint<i64, i64>]>) -> 
            ParallelDecision<Box<[Statepoint<i64, i64>]>, ()>
        {
            if *input == 0 {
                ParallelDecision::Exit(())
            } else {
                ParallelDecision::Stay(slicebox)
            }
        }
    }

    #[test]
    fn parallel_node_test() {
        use parallel_node::ParallelBranchNode;
        let par_node = ParallelBranchNode::<MultiMachine, MagicNumDecider>::new();
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
