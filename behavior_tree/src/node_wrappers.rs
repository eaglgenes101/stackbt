use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

/// Perform transform of input and output of an existing composable state 
/// machine. 
pub trait NodeMap<H, I, M, N, S, T> {
    fn input_transform(&H) -> I;
    fn nonterminal_transform(N) -> M;
    fn terminal_transform(T) -> S;
}

pub struct MappedNode<I, N, T, M, W> where 
    M: BehaviorTreeNode,
    W: NodeMap<I, M::Input, N, M::Nonterminal, T, M::Terminal> 
{
    machine: M,
    _exists_tuple: PhantomData<(I, N, T, W)>
}

impl<I, N, T, M, W> MappedNode<I, N, T, M, W> where 
    M: BehaviorTreeNode,
    W: NodeMap<I, M::Input, N, M::Nonterminal, T, M::Terminal> 
{
    fn new(node: M) -> MappedNode<I, N, T, M, W> {
        MappedNode {
            machine: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<I, N, T, M, W> Default for MappedNode<I, N, T, M, W> where 
    M: BehaviorTreeNode + Default,
    W: NodeMap<I, M::Input, N, M::Nonterminal, T, M::Terminal> 
{
    fn default() -> MappedNode<I, N, T, M, W> {
        MappedNode::new(M::default())
    }
}

impl<I, N, T, M, W> BehaviorTreeNode for MappedNode<I, N, T, M, W> where 
    M: BehaviorTreeNode + Default,
    W: NodeMap<I, M::Input, N, M::Nonterminal, T, M::Terminal> 
{
    type Input = I;
    type Nonterminal = N;
    type Terminal = T;
    
    fn step(self, input: &I) -> NodeResult<N, T, Self> {
        match self.machine.step(&W::input_transform(input)) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                W::nonterminal_transform(n),
                MappedNode::new(m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(
                W::terminal_transform(t)
            )
        }
    }
}

pub enum GuardedTerminal<N, T> {
    GuardFailure(N),
    NormalTermination(T)
}

pub trait NodeGuard<I> {
    fn test(&I) -> bool;
}

/// Guarded node, which executes the node it guards only as long as a guard 
/// condition holds. 
pub struct GuardedNode<M, G> where
    M: BehaviorTreeNode,
    G: NodeGuard<M::Input>
{
    machine: M,
    _exists_tuple: PhantomData<G>
}

impl<M, G> GuardedNode<M, G> where 
    M: BehaviorTreeNode,
    G: NodeGuard<M::Input>
{
    fn new(machine: M) -> GuardedNode<M, G> {
        GuardedNode {
            machine: machine,
            _exists_tuple: PhantomData
        }
    }
}

impl<M, G> Default for GuardedNode<M, G> where 
    M: BehaviorTreeNode,
    G: NodeGuard<M::Input>
{
    fn default() -> GuardedNode<M, G> {
        GuardedNode::new(M::default())
    }
}

impl<M, G> BehaviorTreeNode for GuardedNode<M, G> where
    M: BehaviorTreeNode,
    G: NodeGuard<M::Input>
{
    type Input = M::Input;
    type Nonterminal = M::Nonterminal;
    type Terminal = GuardedTerminal<M::Nonterminal, M::Terminal>;

    fn step(self, input: &M::Input) -> NodeResult<M::Nonterminal, Self::Terminal, Self> {
        match self.machine.step(input) {
            NodeResult::Nonterminal(n, m) => {
                if G::test(input) {
                    NodeResult::Nonterminal(n, GuardedNode::new(m))
                } else {
                    NodeResult::Terminal(GuardedTerminal::GuardFailure(n))
                }
            },
            NodeResult::Terminal(t) => NodeResult::Terminal(
                GuardedTerminal::NormalTermination(t)
            )
        }
    }
}

pub enum StepDecision {
    Pause, 
    Play, 
    Reset, 
    ResetPlay
}

pub trait StepControl<I> {
    fn controlled_step(&I) -> StepDecision;
}

pub enum StepCtrlNonterm<I> {
    Stepped(I),
    Paused
}

pub struct StepControlledNode<M, S> where 
    M: BehaviorTreeNode,
    S: StepControl<M::Input>
{
    machine: M,
    _exists_tuple: PhantomData<S>
}

impl<M, S> StepControlledNode<M, S> where 
    M: BehaviorTreeNode,
    S: StepControl<M::Input>
{
    fn new(machine: M) -> StepControlledNode<M, S> {
        StepControlledNode {
            machine: machine,
            _exists_tuple: PhantomData
        }
    }
}

impl<M, S> Default for StepControlledNode<M, S> where 
    M: BehaviorTreeNode + Default,
    S: StepControl<M::Input>
{
    fn default() -> StepControlledNode<M, S> {
        StepControlledNode::new(M::default())
    }
}

impl<M, S> BehaviorTreeNode for StepControlledNode<M, S> where 
    M: BehaviorTreeNode + Default,
    S: StepControl<M::Input> 
{
    type Input = M::Input;
    type Nonterminal = StepCtrlNonterm<M::Nonterminal>;
    type Terminal = M::Terminal;
    
    fn step(self, input: &M::Input) -> NodeResult<Self::Nonterminal, M::Terminal, Self> {
        match S::controlled_step(input) {
            StepDecision::Pause => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, self)
            },
            StepDecision::Play => {
                match self.machine.step(input) {
                    NodeResult::Nonterminal(n, m) => {
                        NodeResult::Nonterminal(StepCtrlNonterm::Stepped(n), Self::new(m))
                    },
                    NodeResult::Terminal(t) => NodeResult::Terminal(t)
                }
            },
            StepDecision::Reset => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, Self::default())
            },
            StepDecision::ResetPlay => {
                let mut new_machine = M::default();
                match new_machine.step(input) {
                    NodeResult::Nonterminal(n, m) => {
                        NodeResult::Nonterminal(StepCtrlNonterm::Stepped(n), Self::new(m))
                    },
                    NodeResult::Terminal(t) => NodeResult::Terminal(t)
                }
            }
        }
    }
}