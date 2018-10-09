use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

/// Mapping between different input types. 
pub trait InputNodeMap {
    /// The input type for the input mapper, which is taken as the input type 
    /// of the wrapper. 
    type In;
    /// The output type for the input mapper, which is then fed into the 
    /// enclosed behavior tree node. 
    type Out;
    /// Map between the input supplied and the output to feed into the 
    /// enclosed behavior tree node. 
    fn input_transform(&self, &Self::In) -> Self::Out;
}

/// Wrapper for a node which converts between the provided input type and 
/// the input type expected by the node. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct InputMappedNode<N, M> where 
    N: BehaviorTreeNode,
    M: InputNodeMap<Out=N::Input> 
{
    node: N,
    mapper: M
}

impl<N, M> InputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<Out=N::Input>
{
    /// Create a new input mapped node. 
    pub fn new(mapper: M, node: N) -> InputMappedNode<N, M> {
        InputMappedNode {
            node,
            mapper
        }
    }
}

impl<N, M> Default for InputMappedNode<N, M> where
    N: BehaviorTreeNode + Default,
    M: InputNodeMap<Out=N::Input> + Default
{
    fn default() -> InputMappedNode<N, M> {
        InputMappedNode::new(M::default(), N::default())
    }
}

impl<N, M> BehaviorTreeNode for InputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<Out=N::Input>
{
    type Input = M::In;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    #[inline]
    fn step(self, input: &M::In) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        match self.node.step(&self.mapper.input_transform(input)) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                n,
                InputMappedNode::new(self.mapper, m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

/// Mapping between different output types. 
pub trait OutputNodeMap {
    /// The nonterminal input type, received from the enclosed behavior tree 
    /// node. 
    type NontermIn;
    /// The nonterminal output type, which is the type returned by the 
    /// wrapper. 
    type NontermOut;
    /// The terminal input type, received from the enclosed behavior tree 
    /// node. 
    type TermIn;
    /// The terminal output type, which is the type returned by the wrapper. 
    type TermOut;
    /// Map between the nonterminal input returned by the automaton and the 
    /// nonterminal output to return. 
    fn nonterminal_transform(&self, Self::NontermIn) -> Self::NontermOut;
    /// Map between the terminal input returned by the automaton and the 
    /// terminal output to return. 
    fn terminal_transform(&self, Self::TermIn) -> Self::TermOut;
}

/// Convenient wrapper for two closures which together cover the nonterminal 
/// and terminal cases of the output map. 
pub struct OutputMapStruct<M, N, S, T, P, Q> where 
    P: Fn(M) -> N,
    Q: Fn(S) -> T
{
    nonterm_map: P,
    term_map: Q,
    _exists_tuple: PhantomData<(M, N, S, T)>
}

impl<M, N, S, T, P, Q> OutputMapStruct<M, N, S, T, P, Q> where 
    P: Fn(M) -> N,
    Q: Fn(S) -> T
{
    pub fn new(nonterm_map: P, term_map: Q) -> OutputMapStruct<M, N, S, T, P, Q> {
        OutputMapStruct {
            nonterm_map,
            term_map,
            _exists_tuple: PhantomData
        }
    }
}

impl<M, N, S, T, P, Q> OutputNodeMap for OutputMapStruct<M, N, S, T, P, Q> where 
    P: Fn(M) -> N,
    Q: Fn(S) -> T
{
    type NontermIn = M;
    type NontermOut = N;
    type TermIn = S;
    type TermOut = T;

    fn nonterminal_transform(&self, input: M) -> N {
        (self.nonterm_map)(input)
    }

    fn terminal_transform(&self, input: S) -> T {
        (self.term_map)(input)
    }
}

/// Wrapper for a node which converts between the statepoints emitted by the 
/// node and the ones exposed by the wrapper. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct OutputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    node: N,
    mapper: M
}

impl<N, M> OutputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    /// Create an new output mapped node. 
    pub fn new(mapper: M, node: N) -> OutputMappedNode<N, M> {
        OutputMappedNode {
            node,
            mapper
        }
    }
}

impl<N, M> Default for OutputMappedNode<N, M> where
    N: BehaviorTreeNode + Default,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal> + Default
{
    fn default() -> OutputMappedNode<N, M> {
        OutputMappedNode::new(M::default(), N::default())
    }
}

impl<N, M> BehaviorTreeNode for OutputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    type Input = N::Input;
    type Nonterminal = M::NontermOut;
    type Terminal = M::TermOut;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<M::NontermOut, M::TermOut, Self> {
        match self.node.step(input) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                self.mapper.nonterminal_transform(n),
                OutputMappedNode::new(self.mapper, m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(
                self.mapper.terminal_transform(t)
            )
        }
    }
}

/// Lazy constructor for a node, depending on the first input. 
pub trait LazyConstructor {
    /// Type of the behavior tree node to create. 
    type Creates: BehaviorTreeNode;
    /// Create a new behavior tree node. 
    fn create(&self, &<Self::Creates as BehaviorTreeNode>::Input) -> Self::Creates;
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum LazyConstructedInner<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N>
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
    M: LazyConstructor<Creates=N>
{
    inside: Option<LazyConstructedInner<N, M>>
}

impl<N, M> LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N>
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

impl<N, M> Default for LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N> + Default
{
    fn default() -> LazyConstructedNode<N, M> {
        LazyConstructedNode::new(M::default())
    }
}

impl<N, M> BehaviorTreeNode for LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N>
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        let mut mut_self = self;
        let node = match mut_self.inside.take().unwrap() {
            LazyConstructedInner::Node(n) => n,
            LazyConstructedInner::Pending(c) => c.create(input)
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

/// Eager constructor for a node. 
pub trait CustomConstructor {
    /// Type of the behavior tree node to create. 
    type Creates: BehaviorTreeNode;
    /// Create a new behavior tree node. 
    fn create(&self) -> Self::Creates;
}

/// Wrapper for a node which designates a default constructor for that node, 
/// constructing it from the constructor. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<Creates=N>
{
    node: N,
    _exists_tuple: PhantomData<M>
}

impl<N, M> CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<Creates=N>
{
    /// Create a new custom constructed behavior tree node. 
    pub fn new(constructor: &M) -> CustomConstructedNode<N, M> {
        CustomConstructedNode {
            node: constructor.create(),
            _exists_tuple: PhantomData
        }
    }

    /// Create a new custom constructed behavior tree node, using a dummy 
    /// object to supply type of the constructor. 
    pub fn from_existing(node: N) -> CustomConstructedNode<N, M> {
        CustomConstructedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, M> Default for CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<Creates=N> + Default
{
    fn default() -> CustomConstructedNode<N, M> {
        CustomConstructedNode::new(&M::default())
    }
}

impl<N, M> BehaviorTreeNode for CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<Creates=N>
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        match self.node.step(input) {
            NodeResult::Nonterminal(n, i) => NodeResult::Nonterminal(
                n, CustomConstructedNode::from_existing(i)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

#[cfg(test)]
mod tests {
    use map_wrappers::{InputNodeMap, OutputNodeMap, LazyConstructor, CustomConstructor};
    use stackbt_automata_impl::internal_state_machine::{InternalTransition, 
        InternalStateMachine};
    use base_nodes::{WaitCondition, MachineWrapper, PredicateWait};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};

    struct Echoer;

    impl WaitCondition for Echoer {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;
        fn do_end(&self, input: &i64) -> Statepoint<i64, i64> {
            if *input > 0 {
                Statepoint::Nonterminal(*input)
            } else {
                Statepoint::Terminal(*input)
            }
        }
    }

    struct InMap;

    impl InputNodeMap for InMap {
        type In = i64;
        type Out = i64;
        fn input_transform(&self, input: &i64) -> i64 {
            -input
        }
    }

    #[test]
    fn input_map_test() {
        use map_wrappers::InputMappedNode;
        let base_node = PredicateWait::new(Echoer);
        let wrapped_node = InputMappedNode::new(InMap, base_node);
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

    struct OutMap;

    impl OutputNodeMap for OutMap {
        type NontermIn = i64;
        type NontermOut = i64;
        type TermIn = i64;
        type TermOut = i64;
        fn nonterminal_transform(&self, val: i64) -> i64 {
            val + 1
        }

        fn terminal_transform(&self, val: i64) -> i64 {
            val - 1
        }
    }

    #[test]
    fn output_map_test() {
        use map_wrappers::OutputMappedNode;
        let base_node = PredicateWait::new(Echoer);
        let wrapped_node = OutputMappedNode::new(OutMap, base_node);
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

    struct LazyWrapper;

    impl LazyConstructor for LazyWrapper 
    {
        type Creates = MachineWrapper<InternalStateMachine<'static,
            IndefinitePlayback>, i64, i64>;

        fn create(&self, input: &i64) -> Self::Creates {
            MachineWrapper::new(InternalStateMachine::new(IndefinitePlayback, *input))
        }
    }

    #[test]
    fn lazy_constructor_test() {
        use map_wrappers::LazyConstructedNode;
        let new_node = LazyConstructedNode::new(LazyWrapper);
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
        let new_node_2 = LazyConstructedNode::new(LazyWrapper);
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

    struct FixedWrapper; 

    impl CustomConstructor for FixedWrapper 
    {
        type Creates = MachineWrapper<InternalStateMachine<'static,
            IndefinitePlayback>, i64, i64>;
        fn create(&self) -> Self::Creates {
            MachineWrapper::new(InternalStateMachine::new(IndefinitePlayback, 12))
        }
    }
    
    #[test]
    fn custom_constructor_test() {
        use map_wrappers::CustomConstructedNode;
        let new_node = CustomConstructedNode::new(&FixedWrapper);
        match new_node.step(&2) {
            NodeResult::Nonterminal(x, _) => assert_eq!(x, 12),
            _ => unreachable!("Expected nonterminal state")
        };
    }
}