use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use stackbt_automata_impl::automaton::Automaton;

/// Node whose function is to stall within itself until a function of its 
/// input return a terminal state, then terminates at that state. 
/// 
/// # Example
/// ```
/// use stackbt_behavior_tree::behavior_tree_node::{Statepoint, 
///     BehaviorTreeNode, NodeResult};
/// use stackbt_behavior_tree::base_nodes::{WaitCondition, PredicateWait};
/// 
/// struct Echoer;
/// 
/// impl WaitCondition for Echoer {
///     type Input = Statepoint<(), ()>;
///     type Nonterminal = ();
///     type Terminal = ();
///     fn do_end(input: &Statepoint<(), ()>) -> Statepoint<(), ()> {
///         input.clone()
///     }
/// }
/// 
/// let echo_node_0 = PredicateWait::with(Echoer);
/// let echo_node_1 = match echo_node_0.step(&Statepoint::Nonterminal(())) {
///     NodeResult::Nonterminal(_, n) => n,
///     _ => unreachable!("Node doesn't return terminal")
/// };
/// match echo_node_1.step(&Statepoint::Terminal(())) {
///     NodeResult::Terminal(t) => (), //Expected case
///     _ => unreachable!("Node doesn't return nonterminal")
/// };
/// ```

#[derive(PartialEq, Debug)]
pub struct PredicateWait<I, N, T, C> where 
    C: Fn(&I) -> Statepoint<N, T>
{
    closure: C,
    _junk: PhantomData<(I, N, T)>
}

impl<I, N, T, C> Clone for PredicateWait<I, N, T, C> where 
    C: Fn(&I) -> Statepoint<N, T> + Clone 
{
    fn clone(&self) -> Self {
        PredicateWait {
            closure: self.closure.clone(),
            _junk: PhantomData
        }
    }
}

impl<I, N, T, C> Copy for PredicateWait<I, N, T, C> where 
    C: Fn(&I) -> Statepoint<N, T> + Copy
{}

impl<I, N, T, C> PredicateWait<I, N, T, C> where 
    C: Fn(&I) -> Statepoint<N, T>
{
    /// Create a new predicate wait node. 
    pub fn new(closure: C) -> Self {
        PredicateWait {
            closure: closure,
            _junk: PhantomData
        }
    }
}

impl<I, N, T, C> BehaviorTreeNode for PredicateWait<I, N, T, C> where 
    C: Fn(&I) -> Statepoint<N, T>
{
    type Input = I;
    type Nonterminal = N;
    type Terminal = T;

    #[inline]
    fn step(self, input: &I) -> NodeResult<N, T, Self> {
        match (self.closure)(input) {
            Statepoint::Terminal(t) => NodeResult::Terminal(t),
            Statepoint::Nonterminal(n) => NodeResult::Nonterminal(n, self)
        }
    }
}

/// Node which calls a function wrapper with its input, immediately 
/// terminating with its return value. 
/// # Example
/// ```
/// use stackbt_behavior_tree::behavior_tree_node::{Statepoint, 
///     BehaviorTreeNode, NodeResult};
/// use stackbt_behavior_tree::base_nodes::{CallWrapper, Evaluation};
/// 
/// struct IsThree;
/// 
/// impl CallWrapper for IsThree {
///     type Input = ();
///     type Output = i64;
///     fn call(_input: &()) -> i64 {
///         3
///     }
/// }
/// 
/// let three_node = Evaluation::with(IsThree);
/// match three_node.step(&()) {
///     NodeResult::Terminal(t) => assert_eq!(t, 3), //Expected case
///     _ => unreachable!("Node doesn't return nonterminal")
/// };
/// ```
#[derive(PartialEq, Debug)]
pub struct Evaluation<I, O, C> where 
    C: Fn(&I) -> O
{
    closure: C,
    _junk: PhantomData<(I, O)>
}

impl<I, O, C> Clone for Evaluation<I, O, C> where
    C: Fn(&I) -> O + Clone 
{
    fn clone(&self) -> Self {
        Evaluation {
            closure: self.closure.clone(),
            _junk: PhantomData
        }
    }
}

impl<I, O, C> Copy for Evaluation<I, O, C> where
    C: Fn(&I) -> O + Copy
{}

impl<I, O, C> Evaluation<I, O, C> where
    C: Fn(&I) -> O
{
    /// Create a new evaluation node. 
    pub fn new(closure: C) -> Self {
        Evaluation {
            closure: closure,
            _junk: PhantomData
        }
    }
}

impl<I, O, C> BehaviorTreeNode for Evaluation<I, O, C> where 
    C: Fn(&I) -> O
{
    type Input = I;
    type Nonterminal = ();
    type Terminal = O;

    #[inline]
    fn step(self, input: &I) -> NodeResult<(), O, Self> {
        NodeResult::Terminal((self.closure)(input))
    }
}

#[derive(PartialEq, Debug)]
pub struct CallLoop<I, O, C> where 
    C: Fn(&I) -> O
{
    closure: C,
    _junk: PhantomData<(I, O)>
}

impl<I, O, C> Clone for CallLoop<I, O, C> where
    C: Fn(&I) -> O + Clone 
{
    fn clone(&self) -> Self {
        CallLoop {
            closure: self.closure.clone(),
            _junk: PhantomData
        }
    }
}

impl<I, O, C> Copy for CallLoop<I, O, C> where
    C: Fn(&I) -> O + Copy
{}

impl<I, O, C> CallLoop<I, O, C> where
    C: Fn(&I) -> O
{
    /// Create a new evaluation node. 
    pub fn new(closure: C) -> Self {
        CallLoop {
            closure: closure,
            _junk: PhantomData
        }
    }
}

impl<I, O, C> BehaviorTreeNode for CallLoop<I, O, C> where 
    C: Fn(&I) -> O
{
    type Input = I;
    type Nonterminal = O;
    type Terminal = ();

    #[inline]
    fn step(self, input: &I) -> NodeResult<O, (), Self> {
        NodeResult::Nonterminal((self.closure)(input), self)
    }
}

/// Node wrapper for an automaton. 
#[derive(PartialEq, Debug)]
pub struct MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static
{
    machine: M,
    _m_bound: PhantomData<&'static M>,
    _exists_tuple: PhantomData<(N, T)>,
}

impl<M, N, T> Clone for MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static + Clone
{
    fn clone(&self) -> Self {
        MachineWrapper { 
            machine: self.machine.clone(),
            _m_bound: PhantomData,
            _exists_tuple: PhantomData
        }
    }
}

impl<M, N, T> Copy for MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static + Copy
{}

impl<M, N, T> MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static
{
    /// Create a new machine wrapping node. 
    pub fn new(machine: M) -> MachineWrapper<M, N, T> {
        MachineWrapper { 
            machine,
            _m_bound: PhantomData,
            _exists_tuple: PhantomData
        }
    }
}

impl<M, N, T> BehaviorTreeNode for MachineWrapper<M, N, T> where 
    M: Automaton<'static, Action=Statepoint<N, T>> + 'static
{
    type Input = M::Input;
    type Nonterminal = N;
    type Terminal = T;

    #[inline]
    fn step(self, input: &M::Input) -> NodeResult<N, T, Self> {
        let mut mach = self;
        match mach.machine.transition(input) {
            Statepoint::Nonterminal(thing) => {
                NodeResult::Nonterminal(thing, mach)
            },
            Statepoint::Terminal(thing) => {
                NodeResult::Terminal(thing)
            }
        }
    }
}

/// Node wrapper for an automaton. 
#[derive(PartialEq, Debug)]
pub struct MachineLoop<M> where 
    M: Automaton<'static> + 'static
{
    machine: M,
    _m_bound: PhantomData<&'static M>,
}

impl<M> Clone for MachineLoop<M> where 
    M: Automaton<'static> + 'static + Clone
{
    fn clone(&self) -> Self {
        MachineLoop { 
            machine: self.machine.clone(),
            _m_bound: PhantomData,
        }
    }
}

impl<M> Copy for MachineLoop<M> where 
    M: Automaton<'static> + 'static + Copy
{}

impl<M> MachineLoop<M> where 
    M: Automaton<'static> + 'static
{
    /// Create a new machine wrapping node. 
    pub fn new(machine: M) -> MachineLoop<M> {
        MachineLoop { 
            machine,
            _m_bound: PhantomData
        }
    }
}

impl<M> BehaviorTreeNode for MachineLoop<M> where 
    M: Automaton<'static> + 'static
{
    type Input = M::Input;
    type Nonterminal = M::Action;
    type Terminal = ();

    #[inline]
    fn step(self, input: &M::Input) -> NodeResult<M::Action, (), Self> {
        let mut mut_self = self;
        NodeResult::Nonterminal(mut_self.machine.transition(input), mut_self)
    }
}

#[cfg(test)]
mod tests {
    use behavior_tree_node::Statepoint;
    use stackbt_automata_impl::internal_state_machine::InternalTransition;

    #[test]
    fn pred_wait_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use base_nodes::PredicateWait;
        let thing = PredicateWait::new(|i: &i64| {
            if *i == 0 {
                Statepoint::Terminal(())
            } else {
                Statepoint::Nonterminal(())
            }
        });
        let thing_1 = match thing.step(&4) {
            NodeResult::Nonterminal(_, x) => x,
            _ => unreachable!("Expected nonterminal state")
        };
        match thing_1.step(&0) {
            NodeResult::Terminal(_) => (),
            _ => unreachable!("Expected terminal state"),
        }
    }

    #[test]
    fn evaluation_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use base_nodes::Evaluation;
        let thing = Evaluation::new(|val: &i64| *val);
        match thing.step(&5) {
            NodeResult::Terminal(t) => assert!(t == 5),
            _ => unreachable!("Expected terminal"),
        };
    }

    #[derive(Copy, Clone)]
    struct ThingLeaf;

    impl InternalTransition for ThingLeaf {
        type Internal = i64;
        type Input = i64;
        type Action = Statepoint<i64, i64>;

        fn step(&self, increment: &i64, accumulator: &mut i64) -> Statepoint<i64, i64> {
            if *increment == 0 {
                Statepoint::Terminal(*accumulator)
            } else {
                let orig_acc = *accumulator;
                *accumulator += increment;
                Statepoint::Nonterminal(orig_acc)
            }
        }
    }

    impl Default for ThingLeaf {
        fn default() -> ThingLeaf {
            ThingLeaf
        }
    }

    #[test]
    fn leaf_test() {
        use behavior_tree_node::{BehaviorTreeNode, NodeResult};
        use stackbt_automata_impl::internal_state_machine::InternalStateMachine;
        use base_nodes::MachineWrapper;
        let machine = InternalStateMachine::new(ThingLeaf, 0);
        let thing = MachineWrapper::new(machine);
        let thing_1 = match thing.step(&4) {
            NodeResult::Nonterminal(a, b) => {
                assert_eq!(a, 0);
                b
            },
            _ => unreachable!("Expected nonterminal state")
        };
        let thing_2 = match thing_1.step(&3) {
            NodeResult::Nonterminal(a, b) => {
                assert_eq!(a, 4);
                b
            },
            _ => unreachable!("Expected nonterminal state")
        };
        match thing_2.step(&0) {
            NodeResult::Terminal(t) => assert_eq!(t, 7),
            _ => unreachable!("Expected terminal state"),
        };
    }
}