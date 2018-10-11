use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

/// Wrapper for a node which converts between the provided input type and 
/// the input type expected by the node. 
#[derive(PartialEq, Debug)]
pub struct InputMappedNode<N, M, I> where 
    N: BehaviorTreeNode,
    M: Fn(&I) -> N::Input
{
    node: N,
    mapper: M,
    _junk: PhantomData<I>
}

impl<N, M, I> Clone for InputMappedNode<N, M, I> where 
    N: BehaviorTreeNode + Clone,
    M: Fn(&I) -> N::Input + Clone
{
    fn clone(&self) -> Self {
        InputMappedNode {
            node: self.node.clone(),
            mapper: self.mapper.clone(),
            _junk: PhantomData
        }
    }
}

impl<N, M, I> Copy for InputMappedNode<N, M, I> where 
    N: BehaviorTreeNode + Copy,
    M: Fn(&I) -> N::Input + Copy
{}

impl<N, M, I> InputMappedNode<N, M, I> where
    N: BehaviorTreeNode,
    M: Fn(&I) -> N::Input
{
    /// Create a new input mapped node. 
    pub fn new(mapper: M, node: N) -> InputMappedNode<N, M, I> {
        InputMappedNode {
            node,
            mapper,
            _junk: PhantomData
        }
    }
}

impl<N, M, I> BehaviorTreeNode for InputMappedNode<N, M, I> where
    N: BehaviorTreeNode,
    M: Fn(&I) -> N::Input
{
    type Input = I;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    #[inline]
    fn step(self, input: &I) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        match self.node.step(&(self.mapper)(input)) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                n,
                InputMappedNode::new(self.mapper, m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

/// Wrapper for a node which converts between the statepoints emitted by the 
/// node and the ones exposed by the wrapper. 
#[derive(PartialEq, Debug)]
pub struct OutputMappedNode<N, M, O, S, T> where
    N: BehaviorTreeNode,
    M: Fn(N::Nonterminal) -> S,
    O: Fn(N::Terminal) -> T
{
    node: N,
    nonterminal_mapper: M,
    terminal_mapper: O,
    _junk: PhantomData<(S, T)>
}

impl<N, M, O, S, T> Clone for OutputMappedNode<N, M, O, S, T> where
    N: BehaviorTreeNode + Clone,
    M: Fn(N::Nonterminal) -> S + Clone,
    O: Fn(N::Terminal) -> T + Clone
{
    /// Create an new output mapped node. 
    fn clone(&self) -> Self {
        OutputMappedNode {
            node: self.node.clone(),
            nonterminal_mapper: self.nonterminal_mapper.clone(),
            terminal_mapper: self.terminal_mapper.clone(),
            _junk: PhantomData
        }
    }
}

impl<N, M, O, S, T> Copy for OutputMappedNode<N, M, O, S, T> where
    N: BehaviorTreeNode + Copy,
    M: Fn(N::Nonterminal) -> S + Copy,
    O: Fn(N::Terminal) -> T + Copy
{}

impl<N, M, O, S, T> OutputMappedNode<N, M, O, S, T> where
    N: BehaviorTreeNode,
    M: Fn(N::Nonterminal) -> S,
    O: Fn(N::Terminal) -> T
{
    /// Create an new output mapped node. 
    pub fn new(nonterm: M, term: O, node: N) -> Self {
        OutputMappedNode {
            node,
            nonterminal_mapper: nonterm,
            terminal_mapper: term,
            _junk: PhantomData
        }
    }
}

impl<N, M, O, S, T> BehaviorTreeNode for OutputMappedNode<N, M, O, S, T> where
    N: BehaviorTreeNode,
    M: Fn(N::Nonterminal) -> S,
    O: Fn(N::Terminal) -> T
{
    type Input = N::Input;
    type Nonterminal = S;
    type Terminal = T;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<S, T, Self> {
        match self.node.step(input) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                (self.nonterminal_mapper)(n),
                OutputMappedNode::new(
                    self.nonterminal_mapper,
                    self.terminal_mapper, 
                    m
                )
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(
                (self.terminal_mapper)(t)
            )
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum LazyConstructedInner<N, M> where
    N: BehaviorTreeNode,
    M: Fn(&N::Input) -> N
{
    Node(N),
    Pending(M)
}

/// Wrapper for for a node, which defers initialization until the first input 
/// is supplied, after which the node is constructed using this input as a 
/// parameter. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: Fn(&N::Input) -> N
{
    inside: Option<LazyConstructedInner<N, M>>
}

impl<N, M> LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: Fn(&N::Input) -> N
{
    /// Create a new lazily constructed behavior tree node. 
    pub fn new(maker: M) -> LazyConstructedNode<N, M> {
        LazyConstructedNode{
            inside: Option::Some(LazyConstructedInner::Pending(maker))
        }
    }

    /// Wrap an existing behavior tree node in the lazily constructed node
    /// wrapper.
    pub fn from_existing(node: N) -> LazyConstructedNode<N, M> {
        LazyConstructedNode {
            inside: Option::Some(LazyConstructedInner::Node(node))
        }
    }
}

impl<N, M> BehaviorTreeNode for LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: Fn(&N::Input) -> N
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        let mut mut_self = self;
        let node = match mut_self.inside.take().unwrap() {
            LazyConstructedInner::Node(n) => n,
            LazyConstructedInner::Pending(c) => c(input)
        };
        match node.step(input) {
            NodeResult::Nonterminal(v, n) => NodeResult::Nonterminal(
                v,
                LazyConstructedNode::from_existing(n)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

#[cfg(test)]
mod tests {
    use stackbt_automata_impl::internal_state_machine::{InternalTransition, 
        InternalStateMachine};
    use base_nodes::{MachineWrapper, PredicateWait};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};

    #[test]
    fn input_map_test() {
        use map_wrappers::InputMappedNode;
        let base_node = PredicateWait::new(|input: &i64| {
            if *input > 0 {
                Statepoint::Nonterminal(*input)
            } else {
                Statepoint::Terminal(*input)
            }
        });
        let wrapped_node = InputMappedNode::new(|input: &i64| -input, base_node);
        let wrapped_node_1 = match wrapped_node.step(&-5) {
            NodeResult::Nonterminal(v, m) => {
                assert_eq!(v, 5);
                m
            },
            _ => unreachable!("Expected nonterminal state")
        };
        match wrapped_node_1.step(&4) {
            NodeResult::Terminal(x) => assert_eq!(x, -4),
            _ => unreachable!("Expected terminal state"),
        };
    }

    #[test]
    fn output_map_test() {
        use map_wrappers::OutputMappedNode;
        let base_node = PredicateWait::new(|input: &i64| {
            if *input > 0 {
                Statepoint::Nonterminal(*input)
            } else {
                Statepoint::Terminal(*input)
            }
        });
        let wrapped_node = OutputMappedNode::new(
            |val: i64| val+1, 
            |val: i64| val-1,
            base_node
        );
        let wrapped_node_1 = match wrapped_node.step(&5) {
            NodeResult::Nonterminal(v, m) => {
                assert_eq!(v, 6);
                m
            },
            _ => unreachable!("Expected nonterminal state")
        };
        match wrapped_node_1.step(&-4) {
            NodeResult::Terminal(x) => assert_eq!(x, -5),
            _ => unreachable!("Expected terminal state"),
        };
    }

    #[derive(Copy, Clone, Default)]
    struct IndefinitePlayback;

    impl InternalTransition for IndefinitePlayback {
        type Input = i64;
        type Internal = i64;
        type Action = Statepoint<i64, i64>;

        fn step(&self, input: &i64, state: &mut i64) -> Statepoint<i64, i64> {
            if *input > 0 {
                Statepoint::Nonterminal(*state)
            } else {
                Statepoint::Terminal(*state)
            }
        }
    }

    #[test]
    fn lazy_constructor_test() {
        use map_wrappers::LazyConstructedNode;
        let new_node = LazyConstructedNode::new(|input: &i64| {
            MachineWrapper::new(InternalStateMachine::new(IndefinitePlayback, *input))
        });
        let new_node_1 = match new_node.step(&2) {
            NodeResult::Nonterminal(x, y) => {
                assert_eq!(x, 2);
                y
            },
            _ => unreachable!("Expected nonterminal state")
        };
        match new_node_1.step(&4) {
            NodeResult::Nonterminal(x, _) => assert_eq!(x, 2),
            _ => unreachable!("Expected nonterminal state")
        };
        let new_node_2 = LazyConstructedNode::new(|input: &i64| {
            MachineWrapper::new(InternalStateMachine::new(IndefinitePlayback, *input))
        });
        let new_node_3 = match new_node_2.step(&5) {
            NodeResult::Nonterminal(x, y) => {
                assert_eq!(x, 5);
                y
            },
            _ => unreachable!("Expected nonterminal state")
        };
        match new_node_3.step(&10) {
            NodeResult::Nonterminal(x, _) => assert_eq!(x, 5),
            _ => unreachable!("Expected nonterminal state")
        };
    }
}