use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use stackbt_automata_impl::automaton::Automaton;

/// Node whose function is to stall within itself until a function of its 
/// input return a terminal state, then terminates at that state. 
struct PredicateWait<I, N, T, F> where 
    F: Fn(&I) -> Statepoint<N, T> + Default
{
    inside_fn: F,
    _exists_tuple: PhantomData<(I, N, T)>
}

impl<I, N, T, F> PredicateWait<I, N, T, F> where 
    F: Fn(&I) -> Statepoint<N, T> + Default
{
    fn new(inside_fn: F) -> PredicateWait<I, N, T, F> {
        PredicateWait {
            inside_fn: inside_fn,
            _exists_tuple: PhantomData
        }
    }
}

impl<I, N, T, F> Default for PredicateWait<I, N, T, F> where 
    F: Fn(&I) -> Statepoint<N, T> + Default
{
    fn default() -> PredicateWait<I, N, T, F> {
        PredicateWait::new(F::default())
    }
}

impl<I, N, T, F> BehaviorTreeNode for PredicateWait<I, N, T, F> where 
    F: Fn(&I) -> Statepoint<N, T> + Default
{
    type Input = I;
    type Nonterminal = N;
    type Terminal = T;

    fn step(self, input: &I) -> NodeResult<N, T, PredicateWait<I, N, T, F>> {
        match (self.inside_fn)(input) {
            Statepoint::Terminal(t) => NodeResult::Terminal(t),
            Statepoint::Nonterminal(n) => NodeResult::Nonterminal(n, self)
        }
    }
}

/// Node which serves as a wrapper around a function, immediately terminating 
/// with its return value. 
struct Evaluation<I, R, F> where 
    F: Fn(&I) -> R + Default
{
    inside_fn: F,
    _exists_tuple: PhantomData<(I, R)>
}

impl<I, R, F> Evaluation<I, R, F> where 
    F: Fn(&I) -> R + Default
{
    fn new(inside_fn: F) -> Evaluation<I, R, F> {
        Evaluation {
            inside_fn: inside_fn,
            _exists_tuple: PhantomData
        }
    }
}

impl<I, R, F> Default for Evaluation<I, R, F> where 
    F: Fn(&I) -> R + Default
{
    fn default() -> Evaluation<I, R, F> {
        Evaluation::new(F::default())
    }
}

impl<I, R, F> BehaviorTreeNode for Evaluation<I, R, F> where 
    F: Fn(&I) -> R + Default
{
    type Input = I;
    type Nonterminal = ();
    type Terminal = R;

    fn step(self, input: &I) -> NodeResult<(), R, Self> {
        NodeResult::Terminal((self.inside_fn)(input))
    }
}

pub struct LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + 'k, 
    T: Clone
{
    machine: M,
    _m_bound: PhantomData<&'k M>,
    _exists_tuple: PhantomData<(N, T)>,
}

impl<'k, M, N, T> LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + 'k, 
    T: Clone
{
    pub fn new(machine: M) -> LeafNode<'k, M, N, T> {
        LeafNode { 
            machine: machine,
            _m_bound: PhantomData,
            _exists_tuple: PhantomData
        }
    }
}

impl<'k, M, N, T> Default for LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + Default + 'k, 
    T: Clone
{
    fn default() -> LeafNode<'k, M, N, T> {
        LeafNode::new(M::default())
    }
}

impl<'k, M, N, T> BehaviorTreeNode for LeafNode<'k, M, N, T> where 
    M: Automaton<'k, Action=Statepoint<N, T>> + Default + 'k, 
    T: Clone
{
    type Input = M::Input;
    type Nonterminal = N;
    type Terminal = T;

    fn step(mut self, input: &Self::Input) -> NodeResult<N, T, Self> {
        match self.machine.transition(input) {
            Statepoint::Nonterminal(thing) => {
                NodeResult::Nonterminal(thing, self)
            },
            Statepoint::Terminal(thing) => {
                NodeResult::Terminal(thing)
            }
        }
    }
}