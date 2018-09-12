use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

pub enum NontermDecision<T> {
    Step,
    ResetA,
    ResetB,
    ResetBoth,
    Exit(T)
}

pub enum TermADecision<T> {
    StepB,
    ResetB,
    Exit(T)
}

pub enum TermBDecision<T> {
    StepA,
    ResetA,
    Exit(T)
}

pub enum TermBothDecision<T> {
    Reset,
    Exit(T)
}

pub enum NontermReturn<A, B> where 
    A: BehaviorTreeNode,
    B: BehaviorTreeNode
{
    NontermBoth(A::Nonterminal, B::Nonterminal),
    TermANotB(A::Terminal, B::Nonterminal),
    TermBNotA(A::Nonterminal, B::Terminal),
    TermBoth(A::Terminal, B::Terminal)
}

pub trait ParallelBranchDecider<A, B, E> where 
    A: BehaviorTreeNode,
    B: BehaviorTreeNode
{
    fn on_nonterm(&A::Nonterminal, &B::Nonterminal) -> NontermDecision<E>;
    fn on_aterm(&A::Terminal, &B::Nonterminal) -> TermADecision<E>;
    fn on_bterm(&A::Nonterminal, &B::Terminal) -> TermBDecision<E>;
    fn on_bothterm(&A::Terminal, &B::Terminal) -> TermBothDecision<E>;
}

pub struct HeterogeneousParallelNode<A, B, E, D> where 
    A: BehaviorTreeNode,
    B: BehaviorTreeNode,
    D: ParallelBranchDecider<A, B, E>
{
    machine_a: A,
    machine_b: B,
    _exists_tuple: PhantomData<(E, D)>
}

impl <A, B, E, D> HeterogeneousParallelNode<A, B, E, D> where 
    A: BehaviorTreeNode,
    B: BehaviorTreeNode,
    D: ParallelBranchDecider<A, B, E>
{
    fn new(machine_a: A, machine_b: B) -> HeterogeneousParallelNode<A, B, E, D> {
        HeterogeneousParallelNode {
            machine_a: machine_a,
            machine_b: machine_b,
            _exists_tuple: PhantomData
        }
    }
}

impl <A, B, E, D> Default for HeterogeneousParallelNode<A, B, E, D> where 
    A: BehaviorTreeNode + Default,
    B: BehaviorTreeNode + Default,
    D: ParallelBranchDecider<A, B, E>
{
    fn default() -> HeterogeneousParallelNode<A, B, E, D> {
        HeterogeneousParallelNode {
            machine_a: A::default(),
            machine_b: B::default(),
            _exists_tuple: PhantomData
        }
    }
}

impl <A, B, E, D> BehaviorTreeNode for 
    HeterogeneousParallelNode<A, B, E, D> where 
    A: BehaviorTreeNode + Default,
    B: BehaviorTreeNode + Default,
    D: ParallelBranchDecider<A, B, E>
{
    type Input = (A::Input, B::Input);
    type Nonterminal = NontermReturn<A, B>;
    type Terminal = E;

    //Because of the nature of the macros that output calls to this function, 
    //call graphs involving this function end up rather elongated. The inline 
    //annotation nudges the compiler to try flattening the call graph, so it 
    //can try to roll it back up into something better optimized. 
    #[inline]
    fn step(self, input: &Self::Input) -> NodeResult<NontermReturn<A, B>, E, Self> {
        match (self.machine_a.step(&input.0), self.machine_b.step(&input.1)) {
            (NodeResult::Nonterminal(s, a), NodeResult::Nonterminal(t, b)) => {
                match D::on_nonterm(&s, &t) {
                    NontermDecision::Step => NodeResult::Nonterminal(
                        NontermReturn::NontermBoth(s, t),
                        HeterogeneousParallelNode::new(a, b)
                    ),
                    NontermDecision::ResetA => NodeResult::Nonterminal(
                        NontermReturn::NontermBoth(s, t),
                        HeterogeneousParallelNode::new(A::default(), b)
                    ),
                    NontermDecision::ResetB => NodeResult::Nonterminal(
                        NontermReturn::NontermBoth(s, t),
                        HeterogeneousParallelNode::new(a, B::default())
                    ),
                    NontermDecision::ResetBoth => NodeResult::Nonterminal(
                        NontermReturn::NontermBoth(s, t),
                        HeterogeneousParallelNode::default()
                    ),
                    NontermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            (NodeResult::Terminal(s), NodeResult::Nonterminal(t, b)) => {
                match D::on_aterm(&s, &t) {
                    TermADecision::StepB => NodeResult::Nonterminal(
                        NontermReturn::TermANotB(s, t),
                        HeterogeneousParallelNode::new(A::default(), b)
                    ),
                    TermADecision::ResetB => NodeResult::Nonterminal(
                        NontermReturn::TermANotB(s, t),
                        HeterogeneousParallelNode::default()
                    ),
                    TermADecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            (NodeResult::Nonterminal(s, a), NodeResult::Terminal(t)) => {
                match D::on_bterm(&s, &t) {
                    TermBDecision::StepA => NodeResult::Nonterminal(
                        NontermReturn::TermBNotA(s, t),
                        HeterogeneousParallelNode::new(a, B::default())
                    ),
                    TermBDecision::ResetA => NodeResult::Nonterminal(
                        NontermReturn::TermBNotA(s, t),
                        HeterogeneousParallelNode::default()
                    ),
                    TermBDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            (NodeResult::Terminal(s), NodeResult::Terminal(t)) => {
                match D::on_bothterm(&s, &t) {
                    TermBothDecision::Reset => NodeResult::Nonterminal(
                        NontermReturn::TermBoth(s, t),
                        HeterogeneousParallelNode::default()
                    ),
                    TermBothDecision::Exit(x) => NodeResult::Terminal(x)
                }
            }
        }
    }
}