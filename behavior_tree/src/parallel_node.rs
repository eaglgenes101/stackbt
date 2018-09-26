use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use stackbt_automata_impl::automaton::Automaton;

pub trait ParallelDecider {
    type Input: 'static;
    type Nonterm: 'static;
    type Term: 'static;
    type Exit;
    fn each_step(&Self::Input, Box<[Statepoint<Self::Nonterm, Self::Term>]>) -> 
        Statepoint<Box<[Statepoint<Self::Nonterm, Self::Term>]>, Self::Exit>;
}

pub struct ParallelBranchNode<C, D> where
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>>,
    D: ParallelDecider
{
    collection: C,
    _exists_tuple: PhantomData<D>
}

impl<C, D> ParallelBranchNode<C, D> where
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>>,
    D: ParallelDecider
{
    pub fn new(machine: C) -> ParallelBranchNode<C, D> {
        ParallelBranchNode {
            collection: machine,
            _exists_tuple: PhantomData
        }
    }

    pub fn from_existing(existing: C) -> ParallelBranchNode<C, D> {
        ParallelBranchNode {
            collection: existing,
            _exists_tuple: PhantomData
        }
    }
}

impl<C, D> Default for ParallelBranchNode<C, D> where
    C: Automaton<'static, Input=D::Input, Action=Box<[Statepoint<D::Nonterm, 
        D::Term>]>> + Default,
    D: ParallelDecider
{
    fn default() -> ParallelBranchNode<C, D> {
        ParallelBranchNode::new(C::default())
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
        let decision = D::each_step(input, results);
        match decision {
            Statepoint::Nonterminal(ret) => NodeResult::Nonterminal(
                ret,
                Self::from_existing(coll)
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

        fn step(input: &i64, mach: &mut MultiMachine) -> Self::Action {
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

    struct MagicNumDecider;

    impl ParallelDecider for MagicNumDecider {
        type Input = i64;
        type Nonterm = i64;
        type Term = i64;
        type Exit = ();

        fn each_step(input: &i64, slicebox: Box<[Statepoint<i64, i64>]>) -> 
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
