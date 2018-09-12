use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

pub enum NontermDecision<T> {
    Step,
    TransA,
    TransB,
    Exit(T)
}

pub enum TermDecision<T> {
    TransA,
    TransB,
    Exit(T)
}

pub enum NontermReturn<A, B> where 
    A: BehaviorTreeNode,
    B: BehaviorTreeNode
{
    NontermA(A::Nonterminal),
    NontermB(B::Nonterminal),
    TermA(A::Terminal),
    TermB(B::Terminal)
}

pub trait SerialBranchDecider<A, B, E> where 
    A: BehaviorTreeNode,
    B: BehaviorTreeNode,
{
    fn on_a_nonterminal(&A::Nonterminal) -> NontermDecision<E>;
    fn on_b_nonterminal(&B::Nonterminal) -> NontermDecision<E>;
    fn on_a_terminal(&A::Terminal) -> TermDecision<E>;
    fn on_b_terminal(&B::Terminal) -> TermDecision<E>;
}

pub enum HeterogeneousSerialNode<I, A, B, E, D> where 
    A: BehaviorTreeNode<Input=I> + Default,
    B: BehaviorTreeNode<Input=I> + Default,
    D: SerialBranchDecider<A, B, E>
{
    A(A, PhantomData<(E, D)>),
    B(B, PhantomData<(E, D)>)
}

impl<I, A, B, E, D> HeterogeneousSerialNode<I, A, B, E, D> where 
    A: BehaviorTreeNode<Input=I> + Default,
    B: BehaviorTreeNode<Input=I> + Default,
    D: SerialBranchDecider<A, B, E>
{
    fn new_a(machine: A) -> HeterogeneousSerialNode<I, A, B, E, D> {
        HeterogeneousSerialNode::A(
            machine, 
            PhantomData
        )
    }

    fn new_b(machine: B) -> HeterogeneousSerialNode<I, A, B, E, D> {
        HeterogeneousSerialNode::B(
            machine,
            PhantomData
        )
    }
}

impl<I, A, B, E, D> Default for HeterogeneousSerialNode<I, A, B, E, D> where 
    A: BehaviorTreeNode<Input=I> + Default,
    B: BehaviorTreeNode<Input=I> + Default,
    D: SerialBranchDecider<A, B, E>
{
    fn default() -> HeterogeneousSerialNode<I, A, B, E, D> {
        HeterogeneousSerialNode::new_a(A::default())
    }
}

impl <I, A, B, E, D> BehaviorTreeNode for 
    HeterogeneousSerialNode<I, A, B, E, D> where
    A: BehaviorTreeNode<Input=I> + Default,
    B: BehaviorTreeNode<Input=I> + Default,
    D: SerialBranchDecider<A, B, E>
{
    type Input = I;
    type Nonterminal = NontermReturn<A, B>;
    type Terminal = E;

    //Because of the nature of the macros that output calls to this function, 
    //call graphs involving this function end up rather elongated. The inline 
    //annotation nudges the compiler to try flattening the call graph, so it 
    //can try to roll it back up into something better optimized. 
    #[inline]
    fn step(self, input: &Self::Input) -> NodeResult<NontermReturn<A, B>, E, Self> {
        match self {
            HeterogeneousSerialNode::A(m, _e) => match m.step(input) {
                NodeResult::Nonterminal(r, a) => match D::on_a_nonterminal(&r) {
                    NontermDecision::Step => NodeResult::Nonterminal(
                        NontermReturn::NontermA(r),
                        HeterogeneousSerialNode::new_a(a)
                    ),
                    NontermDecision::TransA => NodeResult::Nonterminal(
                        NontermReturn::NontermA(r),
                        HeterogeneousSerialNode::new_a(A::default())
                    ),
                    NontermDecision::TransB => NodeResult::Nonterminal(
                        NontermReturn::NontermA(r),
                        HeterogeneousSerialNode::new_b(B::default())
                    ),
                    NontermDecision::Exit(x) => NodeResult::Terminal(x)
                },
                NodeResult::Terminal(r) => match D::on_a_terminal(&r) {
                    TermDecision::TransA => NodeResult::Nonterminal(
                        NontermReturn::TermA(r),
                        HeterogeneousSerialNode::new_a(A::default())
                    ),
                    TermDecision::TransB => NodeResult::Nonterminal(
                        NontermReturn::TermA(r),
                        HeterogeneousSerialNode::new_b(B::default())
                    ),
                    TermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            HeterogeneousSerialNode::B(m, _e) => match m.step(input) {
                NodeResult::Nonterminal(r, b) => match D::on_b_nonterminal(&r) {
                    NontermDecision::Step => NodeResult::Nonterminal(
                        NontermReturn::NontermB(r),
                        HeterogeneousSerialNode::new_b(b)
                    ),
                    NontermDecision::TransA => NodeResult::Nonterminal(
                        NontermReturn::NontermB(r),
                        HeterogeneousSerialNode::new_a(A::default())
                    ),
                    NontermDecision::TransB => NodeResult::Nonterminal(
                        NontermReturn::NontermB(r),
                        HeterogeneousSerialNode::new_b(B::default())
                    ),
                    NontermDecision::Exit(x) => NodeResult::Terminal(x)
                },
                NodeResult::Terminal(r) => match D::on_b_terminal(&r) {
                    TermDecision::TransA => NodeResult::Nonterminal(
                        NontermReturn::TermB(r),
                        HeterogeneousSerialNode::new_a(A::default())
                    ),
                    TermDecision::TransB => NodeResult::Nonterminal(
                        NontermReturn::TermB(r),
                        HeterogeneousSerialNode::new_b(B::default())
                    ),
                    TermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
        }
    }
}




