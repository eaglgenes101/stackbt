use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GuardFailure<N>(pub N); 

/// A node guard predicate. 
pub trait NodeGuard {
    /// Type of the input to take. 
    type Input;
    /// Type of the nonterminal to take. 
    type Nonterminal;
    /// Given references to the input taken and the nonterminal returned, 
    /// determine whether to keep running the node (true) or end its 
    /// execution prematurely (false). 
    fn test(&self, &Self::Input, &Self::Nonterminal) -> bool;
}

/// Guard wrapper for a node, which, if the guard condition fails, causes an 
/// abnormal exit of the node. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GuardedNode<N, G> where
    N: BehaviorTreeNode,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal>
{
    node: N,
    guard: G
}

impl<N, G> GuardedNode<N, G> where 
    N: BehaviorTreeNode,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal>
{
    /// Create a new guarded node. 
    pub fn new(guard: G, node: N) -> GuardedNode<N, G> {
        GuardedNode {
            node: node,
            guard: guard
        }
    }
}

impl<N, G> Default for GuardedNode<N, G> where 
    N: BehaviorTreeNode + Default,
    G: NodeGuard<Input=N::Input, Nonterminal=N::Nonterminal> + Default
{
    fn default() -> GuardedNode<N, G> {
        GuardedNode::new(G::default(), N::default())
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
                if self.guard.test(input, &n) {
                    NodeResult::Nonterminal(n, GuardedNode::new(self.guard, m))
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

/// Enumeration of the possible decisions of a StepControl controller.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum StepDecision {
    /// Don't step the machine. 
    Pause, 
    /// Step the machine as normal. 
    Play, 
    /// Dispose the current machine, and initialize a new one in its place. 
    Reset, 
    /// Reset the machine, and then subsequently step it. 
    ResetPlay
}

/// Step controller for a node. 
pub trait StepControl {
    /// Type of the input to take. 
    type Input;
    /// Given a reference to the input, determine whether to pause, play, 
    /// and/or reset the enclosed node before it starts. 
    fn controlled_step(&self, &Self::Input) -> StepDecision;
}

impl<I> StepControl for Fn(&I) -> StepDecision {
    type Input = I;
    fn controlled_step(&self, input: &I) -> StepDecision {
        self(input)
    }
}

/// Nonterminal enum for a step-controlled node. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum StepCtrlNonterm<I> {
    /// The node was stepped as normal, perhaps after resetting it. 
    Stepped(I),
    /// The node was paused, and maybe reset. 
    Paused
}

/// A step-controlling wrapper for a node, which may pause, step, and/or 
/// reset a node depending on inputs, before the node goes forward. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input>
{
    node: N,
    stepper: S
}

impl<N, S> StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input>
{
    /// Create a new step controlled node. 
    pub fn new(stepper: S, node: N) -> StepControlledNode<N, S> {
        StepControlledNode {
            node: node,
            stepper: stepper
        }
    }
}

impl<N, S> Default for StepControlledNode<N, S> where 
    N: BehaviorTreeNode + Default,
    S: StepControl<Input=N::Input> + Default
{
    fn default() -> StepControlledNode<N, S> {
        StepControlledNode::new(S::default(), N::default())
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
        match self.stepper.controlled_step(input) {
            StepDecision::Pause => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, self)
            },
            StepDecision::Play => {
                match self.node.step(input) {
                    NodeResult::Nonterminal(n, m) => {
                        NodeResult::Nonterminal(
                            StepCtrlNonterm::Stepped(n), 
                            Self::new(self.stepper, m)
                        )
                    },
                    NodeResult::Terminal(t) => NodeResult::Terminal(t)
                }
            },
            StepDecision::Reset => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, Self::new(
                    self.stepper,
                    N::default()
                ))
            },
            StepDecision::ResetPlay => {
                let mut new_machine = N::default();
                match new_machine.step(input) {
                    NodeResult::Nonterminal(n, m) => {
                        NodeResult::Nonterminal(
                            StepCtrlNonterm::Stepped(n), 
                            Self::new(self.stepper, m)
                        )
                    },
                    NodeResult::Terminal(t) => NodeResult::Terminal(t)
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PostResetNonterm<N, T> {
    /// The node was not reset. 
    NoReset(N),
    /// The node was reset from a nonterminal state. 
    ManualReset(N),
    /// The node was reset from a terminal state. 
    EndReset(T)
}

/// A post-resetting wrapper for a node, which after a node plays, may 
/// or may not reset that node. 
pub trait PostResetControl {
    /// Type of the input to take. 
    type Input;
    /// Type of the nonterminal to take. 
    type Nonterminal;
    /// Type of the terminal to take. 
    type Terminal;
    /// Given a reference to the input and a statepoint corresponding to the 
    /// enclosed node's state, return whether to reset it (true) or not 
    /// (false) after it runs. 
    fn do_reset(&self, &Self::Input, Statepoint<&Self::Nonterminal, 
        &Self::Terminal>) -> bool;
}

impl<I, N, T> PostResetControl for Fn(&I, Statepoint<&N, &T>) -> bool {
    type Input = I;
    type Nonterminal = N;
    type Terminal = T;
    fn do_reset(&self, input: &I, state: Statepoint<&N, &T>) -> bool {
        self(input, state)
    }
}

/// A post-run resetting wrapper for a node, which may reset a node after 
/// it runs. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal>
{
    node: N,
    resetter: P
}

impl<N, P> PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal>
{
    /// Create a new step controlled node. 
    pub fn new(resetter: P, node: N) -> PostResetNode<N, P> {
        PostResetNode {
            node: node,
            resetter: resetter
        }
    }
}


impl<N, P> Default for PostResetNode<N, P> where 
    N: BehaviorTreeNode + Default,
    P: PostResetControl<Input=N::Input, Nonterminal=N::Nonterminal, 
        Terminal=N::Terminal> + Default
{
    fn default() -> PostResetNode<N, P> {
        PostResetNode::new(P::default(), N::default())
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
                if self.resetter.do_reset(input, Statepoint::Nonterminal(&v)) {
                    NodeResult::Nonterminal(
                        PostResetNonterm::ManualReset(v), 
                        Self::new(self.resetter, N::default())
                    )
                } else {
                    NodeResult::Nonterminal(
                        PostResetNonterm::NoReset(v), 
                        Self::new(self.resetter, n)
                    )
                }
            },
            NodeResult::Terminal(t) => {
                if self.resetter.do_reset(input, Statepoint::Terminal(&t)) {
                    NodeResult::Nonterminal(
                        PostResetNonterm::EndReset(t),
                        Self::new(self.resetter, N::default())
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
        fn do_end(&self, input: &i64) -> Statepoint<i64, i64> {
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
        fn test(&self, input: &i64, _whocares: &i64) -> bool {
            *input > 5
        }
    }

    #[test]
    fn guarded_node_test() {
        use control_wrappers::{GuardedNode, GuardFailure};
        let base_node = PredicateWait::new(Echoer);
        let wrapped_node = GuardedNode::new(MagGuard, base_node);
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
        fn controlled_step(&self, input: &i64) -> StepDecision {
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
        let wrapped_node = StepControlledNode::new(MagicPlayer, base_node);
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

        fn do_reset(&self, input: &i64, _output: Statepoint<&i64, &()>) -> bool {
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
        let wrapped_node = PostResetNode::new(Resetter, base_node);
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