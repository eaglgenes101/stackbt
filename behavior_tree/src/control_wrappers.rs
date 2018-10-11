use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GuardFailure<N>(pub N); 

/// Guard wrapper for a node, which, if the guard condition fails, causes an 
/// abnormal exit of the node. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GuardedNode<N, G> where
    N: BehaviorTreeNode,
    G: Fn(&N::Input, &N::Nonterminal) -> bool
{
    node: N,
    guard: G
}

impl<N, G> GuardedNode<N, G> where 
    N: BehaviorTreeNode,
    G: Fn(&N::Input, &N::Nonterminal) -> bool
{
    /// Create a new guarded node. 
    pub fn new(guard: G, node: N) -> GuardedNode<N, G> {
        GuardedNode {
            node: node,
            guard: guard
        }
    }
}

impl<N, G> BehaviorTreeNode for GuardedNode<N, G> where
    N: BehaviorTreeNode,
    G: Fn(&N::Input, &N::Nonterminal) -> bool
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
                if (self.guard)(input, &n) {
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
pub enum StepDecision<N> {
    /// Don't step the machine. 
    Pause, 
    /// Step the machine as normal. 
    Play, 
    /// Dispose the current machine, and initialize a new one in its place. 
    Reset(N), 
    /// Reset the machine, and then subsequently step it. 
    ResetPlay(N)
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
    N: BehaviorTreeNode,
    S: Fn(&N::Input) -> StepDecision<N>
{
    node: N,
    stepper: S
}

impl<N, S> StepControlledNode<N, S> where 
    N: BehaviorTreeNode,
    S: Fn(&N::Input) -> StepDecision<N>
{
    /// Create a new step controlled node. 
    pub fn new(stepper: S, node: N) -> StepControlledNode<N, S> {
        StepControlledNode {
            node: node,
            stepper: stepper
        }
    }
}

impl<N, S> BehaviorTreeNode for StepControlledNode<N, S> where 
    N: BehaviorTreeNode,
    S: Fn(&N::Input) -> StepDecision<N>
{
    type Input = N::Input;
    type Nonterminal = StepCtrlNonterm<N::Nonterminal>;
    type Terminal = N::Terminal;
    
    #[inline]
    fn step(self, input: &N::Input) -> NodeResult<Self::Nonterminal, 
        N::Terminal, Self> 
    {
        match (self.stepper)(input) {
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
            StepDecision::Reset(new_node) => {
                NodeResult::Nonterminal(StepCtrlNonterm::Paused, Self::new(
                    self.stepper,
                    new_node
                ))
            },
            StepDecision::ResetPlay(mut new_machine) => {
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

/// A post-run resetting wrapper for a node, which may reset a node after 
/// it runs. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PostResetNode<N, P> where 
    N: BehaviorTreeNode,
    P: Fn(&N::Input, Statepoint<&N::Nonterminal, &N::Terminal>) -> Option<N>
{
    node: N,
    resetter: P
}

impl<N, P> PostResetNode<N, P> where 
    N: BehaviorTreeNode,
    P: Fn(&N::Input, Statepoint<&N::Nonterminal, &N::Terminal>) -> Option<N>
{
    /// Create a new step controlled node. 
    pub fn new(resetter: P, node: N) -> PostResetNode<N, P> {
        PostResetNode {
            node: node,
            resetter: resetter
        }
    }
}

impl <N, P> BehaviorTreeNode for PostResetNode<N, P> where 
    N: BehaviorTreeNode,
    P: Fn(&N::Input, Statepoint<&N::Nonterminal, &N::Terminal>) -> Option<N>
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
                match (self.resetter)(input, Statepoint::Nonterminal(&v)) {
                    Option::Some(k) => NodeResult::Nonterminal(
                        PostResetNonterm::ManualReset(v),
                        Self::new(self.resetter, k)
                    ),
                    Option::None => NodeResult::Nonterminal(
                        PostResetNonterm::NoReset(v),
                        Self::new(self.resetter, n)
                    )
                }
            },
            NodeResult::Terminal(t) => {
                match (self.resetter)(input, Statepoint::Terminal(&t)) {
                    Option::Some(n) => NodeResult::Nonterminal(
                        PostResetNonterm::EndReset(t),
                        Self::new(self.resetter, n)
                    ),
                    Option::None => NodeResult::Terminal(t)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use stackbt_automata_impl::ref_state_machine::ReferenceTransition;
    use base_nodes::{PredicateWait};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use control_wrappers::{StepDecision};

    #[test]
    fn guarded_node_test() {
        use control_wrappers::{GuardedNode, GuardFailure};
        let base_node = PredicateWait::new(|input: &i64| {
            if *input > 0 {
                Statepoint::Nonterminal(*input)
            } else {
                Statepoint::Terminal(*input)
            }
        });
        let wrapped_node = GuardedNode::new(|input: &i64, _o: &i64| {
            *input > 5
        }, base_node);
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
        let wrapped_node = StepControlledNode::new(|input: &i64|{
            match *input {
                -1 =>  StepDecision::Pause,
                -2 => StepDecision::Reset(
                    MachineWrapper::new(RefStateMachine::new(Ratchet::Zero))
                ),
                7 => StepDecision::ResetPlay(
                    MachineWrapper::new(RefStateMachine::new(Ratchet::Zero))
                ),
                _ => StepDecision::Play
            }
        }, base_node);
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

    #[test]
    fn post_reset_test() {
        use control_wrappers::{PostResetNode, PostResetNonterm};
        use base_nodes::MachineWrapper;
        use stackbt_automata_impl::ref_state_machine::RefStateMachine;
        let machine = RefStateMachine::new(Ratchet::Zero);
        let base_node = MachineWrapper::new(machine);
        let wrapped_node = PostResetNode::new(|input: &i64, _o: Statepoint<&i64, &()>|{
            if *input == -5 || *input == 5 {
                Option::Some(MachineWrapper::new(RefStateMachine::new(Ratchet::Zero)))
            } else {
                Option::None
            }
        }, base_node);
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