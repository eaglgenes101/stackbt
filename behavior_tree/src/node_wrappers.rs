use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

pub trait InputNodeMap<H, I> {
    fn input_transform(&H) -> &I;
}

pub struct InputMappedNode<N, I, M> where 
    N: BehaviorTreeNode,
    M: InputNodeMap<I, N::Input> 
{
    node: N,
    _exists_tuple: PhantomData<(I, M)>
}

impl<N, I, M> InputMappedNode<N, I, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<I, N::Input>
{
    pub fn new(node: N) -> InputMappedNode<N, I, M> {
        InputMappedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, I, M> Default for InputMappedNode<N, I, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<I, N::Input>
{
    fn default() -> InputMappedNode<N, I, M> {
        InputMappedNode::new(N::default())
    }
}

impl<N, I, M> BehaviorTreeNode for InputMappedNode<N, I, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<I, N::Input>
{
    type Input = I;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    fn step(self, input: &I) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        match self.node.step(M::input_transform(input)) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                n,
                InputMappedNode::new(m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

pub trait OutputNodeMap<O, P, T, U> {
    fn nonterminal_transform(O) -> P;
    fn terminal_transform(T) -> U;
}

pub struct OutputMappedNode<N, O, U, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<N::Nonterminal, O, N::Terminal, U>
{
    node: N,
    _exists_tuple: PhantomData<(O, U, M)>
}

impl<N, O, U, M> OutputMappedNode<N, O, U, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<N::Nonterminal, O, N::Terminal, U>
{
    pub fn new(node: N) -> OutputMappedNode<N, O, U, M> {
        OutputMappedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, O, U, M> Default for OutputMappedNode<N, O, U, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<N::Nonterminal, O, N::Terminal, U>
{
    fn default() -> OutputMappedNode<N, O, U, M> {
        OutputMappedNode::new(N::default())
    }
}

impl<N, O, U, M> BehaviorTreeNode for OutputMappedNode<N, O, U, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<N::Nonterminal, O, N::Terminal, U>
{
    type Input = N::Input;
    type Nonterminal = O;
    type Terminal = U;

    fn step(self, input: &N::Input) -> NodeResult<O, U, Self> {
        match self.node.step(input) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                M::nonterminal_transform(n),
                OutputMappedNode::new(m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(
                M::terminal_transform(t)
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
    pub fn new(machine: M) -> GuardedNode<M, G> {
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
    pub fn new(machine: M) -> StepControlledNode<M, S> {
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

pub trait LazyConstructor<N: BehaviorTreeNode> {
    fn create(&N::Input) -> N;
}

pub struct LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<N>
{
    node: Option<N>,
    _exists_tuple: PhantomData<M>
}

impl<N, M> LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<N>
{
    pub fn new() -> LazyConstructedNode<N, M> {
        LazyConstructedNode{
            node: Option::None,
            _exists_tuple: PhantomData
        }
    }

    pub fn from_existing(node: N) -> LazyConstructedNode<N, M> {
        LazyConstructedNode {
            node: Option::Some(node),
            _exists_tuple: PhantomData
        }
    }
}

impl<N, M> Default for LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<N>
{
    fn default() -> LazyConstructedNode<N, M> {
        LazyConstructedNode::new()
    }
}

impl<N, M> BehaviorTreeNode for LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<N>
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    fn step(self, input: &N::Input) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        let node = if let Option::Some(n) = self.node {
            n
        } else {
            M::create(input)
        };
        match node.step(input) {
            NodeResult::Nonterminal(n, i) => NodeResult::Nonterminal(
                n, LazyConstructedNode::from_existing(i)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}


pub trait CustomConstructor<N: BehaviorTreeNode> {
    fn create() -> N;
}

pub struct CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<N>
{
    node: N,
    _exists_tuple: PhantomData<M>
}

impl<N, M> CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<N>
{
    pub fn new(node: N) -> CustomConstructedNode<N, M> {
        CustomConstructedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, M> Default for CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<N>
{
    fn default() -> CustomConstructedNode<N, M> {
        CustomConstructedNode::new(M::create())
    }
}

impl<N, M> BehaviorTreeNode for CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<N>
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    fn step(self, input: &N::Input) -> NodeResult<N::Nonterminal, N::Terminal, Self> {

        match self.node.step(input) {
            NodeResult::Nonterminal(n, i) => NodeResult::Nonterminal(
                n, CustomConstructedNode::new(i)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}