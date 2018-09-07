use node_traits::{AutomatonNode, NodeResult, Statepoint};
use stackbt_automata_impl::automaton::Automaton;
use std::mem::swap;

pub enum NodeRunnerState<I, T, N> where 
    N: AutomatonNode<T, Input=I>,
    T: 'static, 
    I: 'static
{
    RunningNode(N),
    Terminal(N::Terminal),
    Poisoned
}

pub struct NodeRunner<I, T, N> where 
    N: AutomatonNode<T, Input=I> + 'static,
    T: 'static,
    I: 'static
{
    node: NodeRunnerState<I, T, N>
}

impl<I, T, N> NodeRunner<I, T, N> where 
    N: AutomatonNode<T, Input=I> + 'static,
    T: 'static,
    I: 'static
{
    pub fn new() -> NodeRunner<I, T, N> {
        NodeRunner {
            node: NodeRunnerState::RunningNode(N::default())
        }
    }
}

impl<I, T, N> Default for NodeRunner<I, T, N> where 
    N: AutomatonNode<T, Input=I> + 'static,
    T: 'static,
    I: 'static
{
    fn default() -> NodeRunner<I, T, N> {
        NodeRunner::new()
    }
}

fn node_runner_transition<I, T, N>(node: &mut NodeRunnerState<I, T, N>, input: &I) 
    -> Statepoint<N, T> where 
    N: AutomatonNode<T, Input=I> + 'static,
    T: 'static,
    I: 'static
{
    let mut result = NodeRunnerState::Poisoned;
    swap(node, &mut result);
    match result {
        NodeRunnerState::RunningNode(n) => {
            match n.step(input) {
                NodeResult::Nonterminal(s, a) => {
                    *node = NodeRunnerState::RunningNode(a);
                    Statepoint::Nonterminal(s)
                },
                NodeResult::Terminal(t) => {
                    *node = NodeRunnerState::Terminal(t.clone());
                    Statepoint::Terminal(t)
                }
            }
        },
        NodeRunnerState::Terminal(t) => Statepoint::Terminal(t),
        _ => panic!("Node runner was poisoned!")
    }
}

impl<I, T, N> Automaton<'static> for NodeRunner<I, T, N> where 
    N: AutomatonNode<T, Input=I> + 'static,
    T: 'static,
    I: 'static
{
    type Input = I;
    type Action = Statepoint<N, T>;
    fn transition(&mut self, input: &I) -> Statepoint<N, T> {
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

    fn into_fnmut(self) -> Box<FnMut(&Self::Input) -> Self::Action + 'static> {
        let mut node_part = self.node;
        Box::new(move |input: &I| {
            node_runner_transition(&mut node_part, input)
        })
    }

}