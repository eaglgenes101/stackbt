use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;

pub struct GuardFailure<N>(N); 

pub trait NodeGuard {
    type Input;
    type Nonterminal;
    fn test(&Self::Input, &Self::Nonterminal) -> bool;
}

/// Guarded node, which executes the node it guards only as long as a guard 
/// condition holds. 
pub struct GuardedNode<N, G> where
    N: BehaviorTreeNode,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal>
{
    node: N,
    _exists_tuple: PhantomData<G>
}

impl<N, G> GuardedNode<N, G> where 
    N: BehaviorTreeNode,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal>
{
    pub fn new(node: N) -> GuardedNode<N, G> {
        GuardedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_helper: G, node: N) -> GuardedNode<N, G> {
        GuardedNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, G> Default for GuardedNode<N, G> where 
    N: BehaviorTreeNode + Default,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal>
{
    fn default() -> GuardedNode<N, G> {
        GuardedNode::new(N::default())
    }
}

impl<N, G> BehaviorTreeNode for GuardedNode<N, G> where
    N: BehaviorTreeNode,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal>
{
    type Input = N::Input;
    type Nonterminal = N::Nonterminal;
    type Terminal = Result<N::Terminal, GuardFailure<N::Nonterminal>>;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<N::Nonterminal, 
        Self::Terminal, Self> 
    {
        match self.node.step(input) {
            NodeResult::Nonterminal(n, m) => {
                if G::test(input, &n) {
                    NodeResult::Nonterminal(n, GuardedNode::new(m))
                } else {
                    NodeResult::Terminal(Result::Err(GuardFailure(n)))
                }
            },
            NodeResult::Terminal(t) => NodeResult::Terminal(
                Result::Ok(t)
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

pub trait StepControl {
    type Input;
    fn controlled_step(&Self::Input) -> StepDecision;
}

pub enum StepCtrlNonterm<I> {
    Stepped(I),
    Paused
}

pub struct StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input>
{
    node: N,
    _exists_tuple: PhantomData<S>
}

impl<N, S> StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input>
{
    pub fn new(node: N) -> StepControlledNode<N, S> {
        StepControlledNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_assist: S, node: N) -> StepControlledNode<N, S> {
        StepControlledNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}

impl<N, S> Default for StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input>
{
    fn default() -> StepControlledNode<N, S> {
        StepControlledNode::new(N::default())
    }
}

impl<N, S> BehaviorTreeNode for StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input>
{
    type Input = N::Input;
    type Nonterminal = StepCtrlNonterm<N::Nonterminal>;
    type Terminal = N::Terminal;
    
    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<Self::Nonterminal, 
        N::Terminal, Self> 
    {
        match S::controlled_step(input) {
            StepDecision::Pause => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, self)
            },
            StepDecision::Play => {
                match self.node.step(input) {
                    NodeResult::Nonterminal(n, m) => {
                        NodeResult::Nonterminal(
                            StepCtrlNonterm::Stepped(n), 
                            Self::new(m)
                        )
                    },
                    NodeResult::Terminal(t) => NodeResult::Terminal(t)
                }
            },
            StepDecision::Reset => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, Self::default())
            },
            StepDecision::ResetPlay => {
                let mut new_machine = N::default();
                match new_machine.step(input) {
                    NodeResult::Nonterminal(n, m) => {
                        NodeResult::Nonterminal(
                            StepCtrlNonterm::Stepped(n), 
                            Self::new(m)
                        )
                    },
                    NodeResult::Terminal(t) => NodeResult::Terminal(t)
                }
            }
        }
    }
}

pub enum ResetDecision<N> {
    NoReset(N),
    Reset(N)
}

pub enum PostResetNonterm<N, T> {
    NoReset(N),
    ManualReset(N),
    EndReset(T)
}

pub trait PostResetControl {
    type Input;
    type Nonterminal;
    type Terminal;
    fn do_reset(&Self::Input, Statepoint<&Self::Nonterminal, 
        &Self::Terminal>) -> bool;
}

pub struct PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal>
{
    node: N,
    _exists_tuple: PhantomData<P>
}

impl<N, P> PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal>
{
    pub fn new(node: N) -> PostResetNode<N, P> {
        PostResetNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }

    pub fn with(_type_assist: P, node: N) -> PostResetNode<N, P> {
        PostResetNode {
            node: node,
            _exists_tuple: PhantomData
        }
    }
}


impl<N, P> Default for PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal>
{
    fn default() -> PostResetNode<N, P> {
        PostResetNode::new(N::default())
    }
}

impl <N, P> BehaviorTreeNode for PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal>
{
    type Input = N::Input;
    type Nonterminal = PostResetNonterm<N::Nonterminal, N::Terminal>;
    type Terminal = N::Terminal;

    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<Self::Nonterminal, 
        N::Terminal, Self> 
    {
        match self.node.step(input) {
            NodeResult::Nonterminal(v, n) => {
                if P::do_reset(input, Statepoint::Nonterminal(&v)) {
                    NodeResult::Nonterminal(
                        PostResetNonterm::ManualReset(v), 
                        Self::default()
                    )
                } else {
                    NodeResult::Nonterminal(
                        PostResetNonterm::NoReset(v), 
                        Self::new(n)
                    )
                }
            },
            NodeResult::Terminal(t) => {
                if P::do_reset(input, Statepoint::Terminal(&t)) {
                    NodeResult::Nonterminal(
                        PostResetNonterm::EndReset(t),
                        Self::default()
                    )
                } else {
                    NodeResult::Terminal(t)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use stackbt_automata_impl::ref_state_machine::ReferenceTransition;
    use base_nodes::{WaitCondition, PredicateWait};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use control_wrappers::{NodeGuard, StepControl, StepDecision, PostResetControl};

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

    struct MagGuard;

    impl NodeGuard for MagGuard {
        type Input = i64;
        type Nonterminal = i64;
        fn test(input: &i64, _whocares: &i64) -> bool {
            *input > 5
        }
    }

    #[test]
    fn guarded_node_test() {
        use control_wrappers::{GuardedNode, GuardFailure};
        let base_node = PredicateWait::with(Echoer);
        let wrapped_node = GuardedNode::with(MagGuard, base_node);
        let wrapped_node_1 = match wrapped_node.step(&7) {
            NodeResult::Nonterminal(v, m) => {
                assert_eq!(v, 7);
                m
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal state")
        };
        match wrapped_node_1.step(&4) {
            NodeResult::Nonterminal(_, _) => unreachable!("Expected terminal state"),
            NodeResult::Terminal(x) => {
                match x {
                    Result::Err(GuardFailure(x)) => {
                        assert_eq!(x, 4)
                    },
                    Result::Ok(_) => unreachable!("Expected guard failure")
                }
            }
        };
    }

    struct MagicPlayer;

    impl StepControl for MagicPlayer {
        type Input = i64;
        fn controlled_step(input: &i64) -> StepDecision {
            match *input {
                -1 =>  StepDecision::Pause,
                -2 => StepDecision::Reset,
                7 => StepDecision::ResetPlay,
                _ => StepDecision::Play
            }
        }
    }

    #[derive(Copy, Clone)]
    enum Ratchet {
        Zero,
        One,
        Two,
        Three
    }

    impl Default for Ratchet {
        fn default() -> Ratchet {
            Ratchet::Zero
        }
    }

    impl ReferenceTransition for Ratchet {
        type Input = i64;
        type Action = Statepoint<i64, ()>;

        fn step(self, input: &i64) -> (Statepoint<i64, ()>, Self) {
            match self {
                Ratchet::Zero => match input {
                    3 => (Statepoint::Nonterminal(3), Ratchet::Three),
                    2 => (Statepoint::Nonterminal(2), Ratchet::Two),
                    1 => (Statepoint::Nonterminal(1), Ratchet::One),
                    _ => (Statepoint::Nonterminal(0), Ratchet::Zero)
                },
                Ratchet::One => match input {
                    3 => (Statepoint::Nonterminal(3), Ratchet::Three),
                    2 => (Statepoint::Nonterminal(2), Ratchet::Two),
                    _ => (Statepoint::Nonterminal(1), Ratchet::One)
                },
                Ratchet::Two => match input {
                    3 => (Statepoint::Nonterminal(3), Ratchet::Three),
                    _ => (Statepoint::Nonterminal(2), Ratchet::Two),
                },
                Ratchet::Three => (Statepoint::Terminal(()), Ratchet::Three)
            }
        }
    }

    #[test]
    fn step_control_test() {
        use control_wrappers::{StepControlledNode, StepCtrlNonterm};
        use base_nodes::MachineWrapper;
        use stackbt_automata_impl::ref_state_machine::RefStateMachine;
        let machine = RefStateMachine::new(Ratchet::Zero);
        let base_node = MachineWrapper::new(machine);
        let wrapped_node = StepControlledNode::with(MagicPlayer, base_node);
        let wrapped_node_1 = match wrapped_node.step(&-1) {
            NodeResult::Nonterminal(v, m) => {
                match v {
                    StepCtrlNonterm::Paused => (),
                    _ => unreachable!("Node was paused")
                };
                m
            },
            _ => unreachable!("Expected nonterminal transition"),
        };
        let wrapped_node_2 = match wrapped_node_1.step(&2) {
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition"),
            NodeResult::Nonterminal(v, m) => {
                match v {
                    StepCtrlNonterm::Stepped(x) => assert_eq!(x, 2),
                    _ => unreachable!("Node was played"),
                };
                m
            }
        };
        let wrapped_node_3 = match wrapped_node_2.step(&-2) {
            NodeResult::Nonterminal(v, m) => {
                match v {
                    StepCtrlNonterm::Paused => (),
                    _ => unreachable!("Node was reset")
                };
                m
            },
            _ => unreachable!("Expected nonterminal transition"),
        };
        let wrapped_node_4 = match wrapped_node_3.step(&2) {
            NodeResult::Nonterminal(v, m) => {
                match v {
                    StepCtrlNonterm::Paused => unreachable!("Node was played"),
                    StepCtrlNonterm::Stepped(x) => assert_eq!(x, 2)
                };
                m
            },
            _ => unreachable!("Expected nonterminal transition"),
        };
        match wrapped_node_4.step(&7) {
            NodeResult::Nonterminal(v, _) => {
                match v {
                    StepCtrlNonterm::Paused => unreachable!("Node was played"),
                    StepCtrlNonterm::Stepped(x) => assert_eq!(x, 0)
                };
            },
            _ => unreachable!("Expected nonterminal transition"),
        };
    }

    struct Resetter;

    impl PostResetControl for Resetter {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = ();

        fn do_reset(input: &i64, _output: Statepoint<&i64, &()>) -> bool {
            *input == -5 || *input == 5
        }
    }

    #[test]
    fn post_reset_test() {
        use control_wrappers::{PostResetNode, PostResetNonterm};
        use base_nodes::MachineWrapper;
        use stackbt_automata_impl::ref_state_machine::RefStateMachine;
        let machine = RefStateMachine::new(Ratchet::Zero);
        let base_node = MachineWrapper::new(machine);
        let wrapped_node = PostResetNode::with(Resetter, base_node);
        let wrapped_node_1 = match wrapped_node.step(&1) {
            NodeResult::Nonterminal(v, n) => {
                match v {
                    PostResetNonterm::NoReset(val) => assert_eq!(val, 1),
                    _ => unreachable!("Node was not reset")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let wrapped_node_2 = match wrapped_node_1.step(&5) {
            NodeResult::Nonterminal(v, n) => {
                match v {
                    PostResetNonterm::ManualReset(val) => assert_eq!(val, 1),
                    _ => unreachable!("Node was not reset")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let wrapped_node_3 = match wrapped_node_2.step(&0) {
            NodeResult::Nonterminal(v, n) => {
                match v {
                    PostResetNonterm::NoReset(val) => assert_eq!(val, 0),
                    _ => unreachable!("Node was manually reset")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let wrapped_node_4 = match wrapped_node_3.step(&3) {
            NodeResult::Nonterminal(v, n) => {
                match v {
                    PostResetNonterm::NoReset(val) => assert_eq!(val, 3),
                    _ => unreachable!("Node was end reset")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let wrapped_node_5 = match wrapped_node_4.step(&5) {
            NodeResult::Nonterminal(v, n) => {
                match v {
                    PostResetNonterm::EndReset(val) => assert_eq!(val, ()),
                    _ => unreachable!("Node was end reset")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let wrapped_node_6 = match wrapped_node_5.step(&3) {
            NodeResult::Nonterminal(v, n) => {
                match v {
                    PostResetNonterm::NoReset(val) => assert_eq!(val, 3),
                    _ => unreachable!("Node was end reset")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        match wrapped_node_6.step(&3) {
            NodeResult::Terminal(()) => (),
            _ => unreachable!("Expected terminal transition")
        };
    }
}