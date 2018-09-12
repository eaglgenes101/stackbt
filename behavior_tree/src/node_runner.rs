use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use stackbt_automata_impl::automaton::Automaton;
use std::mem::swap;

pub struct NodeRunner<I, N> where 
    N: BehaviorTreeNode<Input=I> + 'static,
    I: 'static
{
    node: Option<N>
}

impl<I, N> NodeRunner<I, N> where 
    N: BehaviorTreeNode<Input=I> + 'static,
    I: 'static
{
    pub fn new(node: N) -> NodeRunner<I, N> {
        NodeRunner {
            node: Option::Some(node)
        }
    }
}

impl<I, N> Default for NodeRunner<I, N> where 
    N: BehaviorTreeNode<Input=I> + Default + 'static,
    I: 'static
{
    fn default() -> NodeRunner<I, N> {
        NodeRunner::new(N::default())
    }
}

fn node_runner_transition<I, N>(node: &mut Option<N>, input: &I) 
    -> Statepoint<N::Nonterminal, N::Terminal> where 
    N: BehaviorTreeNode<Input=I> + Clone + 'static
{
    let mut result = Option::None;
    swap(node, &mut result);
    match result {
        Option::Some(n) => {
            match n.step(input) {
                NodeResult::Nonterminal(s, a) => {
                    *node = Option::Some(a);
                    Statepoint::Nonterminal(s)
                },
                NodeResult::Terminal(t) => {
                    *node = Option::Some(N::default());
                    Statepoint::Terminal(t)
                }
            }
        },
        _ => panic!("Node runner was poisoned!")
    }
}

impl<I, N> Automaton<'static> for NodeRunner<I, N> where 
    N: BehaviorTreeNode< Input=I> + Clone + 'static,
    I: 'static
{
    type Input = I;
    type Action = Statepoint<N::Nonterminal, N::Terminal>;
    fn transition(&mut self, input: &I) -> Statepoint<N::Nonterminal, N::Terminal> {
        node_runner_transition(&mut self.node, input)
    }

    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&Self::Input) -> Self::Action + 't> where 
        'static: 't 
    {
        let node_part = &mut self.node;
        Box::new(move |input: &I| {
            node_runner_transition(node_part, input)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&Self::Input) -> Self::Action> {
        let mut node_part = self.node;
        Box::new(move |input: &I| {
            node_runner_transition(&mut node_part, input)
        })
    }
}