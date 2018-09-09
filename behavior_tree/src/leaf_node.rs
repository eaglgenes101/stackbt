use stackbt_automata_impl::automaton::Automaton;
use node_traits::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

pub struct LeafNode<'k, M, N> where 
    M: Automaton<'k> + Default + 'k, 
    M::Action: Clone, 
    N: Into<fn(&M::Action) -> bool> + Default + Copy
{
    machine: M,
    check: N,
    _m_bound: PhantomData<&'k M>
}

impl<'k, M, N> LeafNode<'k, M, N> where 
    M: Automaton<'k> + Default + 'k, 
    M::Action: Clone, 
    N: Into<fn(&M::Action) -> bool> + Default + Copy
{
    pub fn new() -> LeafNode<'k, M, N> {
        LeafNode { 
            machine: M::default(),
            check: N::default(),
            _m_bound: PhantomData
        }
    }
}

impl<'k, M, N> Default for LeafNode<'k, M, N> where 
    M: Automaton<'k> + Default + 'k, 
    M::Action: Clone, 
    N: Into<fn(&M::Action) -> bool> + Default + Copy
{
    fn default() -> LeafNode<'k, M, N> {
        LeafNode::new()
    }
}

impl<'k, M, N> BehaviorTreeNode for LeafNode<'k, M, N> where 
    M: Automaton<'k> + Default + 'k, 
    M::Action: Clone, 
    N: Into<fn(&M::Action) -> bool> + Default + Copy
{
    type Input = M::Input;
    type Output = M::Action;
    type Nonterminal = M::Action;
    type Terminal = M::Action;

    fn step(mut self, input: &Self::Input) -> NodeResult<Self> {
        let thing = self.machine.transition(input);
        if self.check.into()(&thing) {
            NodeResult::Nonterminal(thing, self)
        } else {
            NodeResult::Terminal(thing)
        }
    }
}