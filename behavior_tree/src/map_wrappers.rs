use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

pub trait InputNodeMap {
    type In;
    type Out;
    fn input_transform(&Self::In) -> Self::Out;
}

pub struct InputMappedNode<N, M> where 
    N: BehaviorTreeNode,
    M: InputNodeMap<Out=N::Input> 
{
    node: N,
    _exists_tuple: PhantomData<M>
}

impl<N, M> InputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<Out=N::Input>
{
    pub fn new(node: N) -> InputMappedNode<N, M> {
        InputMappedNode {
            node,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_helper: M, node: N) -> InputMappedNode<N, M> {
        InputMappedNode {
            node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, M> Default for InputMappedNode<N, M> where
    N: BehaviorTreeNode + Default,
    M: InputNodeMap<Out=N::Input>
{
    fn default() -> InputMappedNode<N, M> {
        InputMappedNode::new(N::default())
    }
}

impl<N, M> BehaviorTreeNode for InputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: InputNodeMap<Out=N::Input>
{
    type Input = M::In;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

    fn step(self, input: &M::In) -> NodeResult<N::Nonterminal, N::Terminal, Self> {
        match self.node.step(&M::input_transform(input)) {
            NodeResult::Nonterminal(n, m) => NodeResult::Nonterminal(
                n,
                InputMappedNode::new(m)
            ),
            NodeResult::Terminal(t) => NodeResult::Terminal(t)
        }
    }
}

pub trait OutputNodeMap {
    type NontermIn;
    type NontermOut;
    type TermIn;
    type TermOut;
    fn nonterminal_transform(Self::NontermIn) -> Self::NontermOut;
    fn terminal_transform(Self::TermIn) -> Self::TermOut;
}

pub struct OutputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    node: N,
    _exists_tuple: PhantomData<M>
}

impl<N, M> OutputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    pub fn new(node: N) -> OutputMappedNode<N, M> {
        OutputMappedNode {
            node,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_helper: M, node: N) -> OutputMappedNode<N, M> {
        OutputMappedNode {
            node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, M> Default for OutputMappedNode<N, M> where
    N: BehaviorTreeNode + Default,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    fn default() -> OutputMappedNode<N, M> {
        OutputMappedNode::new(N::default())
    }
}

impl<N, M> BehaviorTreeNode for OutputMappedNode<N, M> where
    N: BehaviorTreeNode,
    M: OutputNodeMap<NontermIn = N::Nonterminal, TermIn = N::Terminal>
{
    type Input = N::Input;
    type Nonterminal = M::NontermOut;
    type Terminal = M::TermOut;

    fn step(self, input: &N::Input) -> NodeResult<M::NontermOut, M::TermOut, Self> {
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

pub trait LazyConstructor {
    type Creates: BehaviorTreeNode;
    fn create(&<Self::Creates as BehaviorTreeNode>::Input) -> Self::Creates;
}

pub struct LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N>
{
    node: Option<N>,
    _exists_tuple: PhantomData<M>
}

impl<N, M> LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N>
{
    pub fn new() -> LazyConstructedNode<N, M> {
        LazyConstructedNode{
            node: Option::None,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_assist: M) -> LazyConstructedNode<N, M> {
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
    M: LazyConstructor<Creates=N>
{
    fn default() -> LazyConstructedNode<N, M> {
        LazyConstructedNode::new()
    }
}

impl<N, M> BehaviorTreeNode for LazyConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: LazyConstructor<Creates=N>
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


pub trait CustomConstructor {
    type Creates: BehaviorTreeNode;
    fn create() -> Self::Creates;
}

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
    pub fn new() -> CustomConstructedNode<N, M> {
        CustomConstructedNode {
            node: M::create(),
            _exists_tuple: PhantomData
        }
    }

    pub fn from_existing(node: N) -> CustomConstructedNode<N, M> {
        CustomConstructedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_assist: M) -> CustomConstructedNode<N, M> {
        CustomConstructedNode {
            node: M::create(),
            _exists_tuple: PhantomData
        }
    }
}

impl<N, M> Default for CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<Creates=N>
{
    fn default() -> CustomConstructedNode<N, M> {
        CustomConstructedNode::new()
    }
}

impl<N, M> BehaviorTreeNode for CustomConstructedNode<N, M> where
    N: BehaviorTreeNode,
    M: CustomConstructor<Creates=N>
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = N::Terminal;

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
    use base_nodes::{WaitCondition, LeafNode, PredicateWait};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};

    struct Echoer;

    impl WaitCondition for Echoer {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;
        fn do_end(input: &i64) -> Statepoint<i64, i64> {
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
        fn input_transform(input: &i64) -> i64 {
            -input
        }
    }

    #[test]
    fn input_map_test() {
        use map_wrappers::InputMappedNode;
        let base_node = PredicateWait::with(Echoer);
        let wrapped_node = InputMappedNode::with(InMap, base_node);
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
        fn nonterminal_transform(val: i64) -> i64 {
            val + 1
        }

        fn terminal_transform(val: i64) -> i64 {
            val - 1
        }
    }

    #[test]
    fn output_map_test() {
        use map_wrappers::OutputMappedNode;
        let base_node = PredicateWait::with(Echoer);
        let wrapped_node = OutputMappedNode::with(OutMap, base_node);
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

        fn step(input: &i64, state: &mut i64) -> Statepoint<i64, i64> {
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
        type Creates = LeafNode<InternalStateMachine<'static,
            IndefinitePlayback>, i64, i64>;

        fn create(input: &i64) -> Self::Creates {
            LeafNode::new(InternalStateMachine::with(IndefinitePlayback, *input))
        }
    }

    #[test]
    fn lazy_constructor_test() {
        use map_wrappers::LazyConstructedNode;
        let new_node = LazyConstructedNode::with(LazyWrapper);
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
        let new_node_2 = LazyConstructedNode::with(LazyWrapper);
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
        type Creates = LeafNode<InternalStateMachine<'static,
            IndefinitePlayback>, i64, i64>;
        fn create() -> Self::Creates {
            LeafNode::new(InternalStateMachine::with(IndefinitePlayback, 12))
        }
    }
    #[test]
    fn custom_constructor_test() {
        use map_wrappers::CustomConstructedNode;
        let new_node = CustomConstructedNode::with(FixedWrapper);
        match new_node.step(&2) {
            NodeResult::Nonterminal(x, _) => assert_eq!(x, 12),
            _ => unreachable!("Expected nonterminal state")
        };
    }
}