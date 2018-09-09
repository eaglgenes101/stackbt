use node_traits::{BehaviorTreeNode, NodeResult, Statepoint};
use stackbt_automata_impl::automaton::Automaton;
use std::mem::swap;
use std::marker::PhantomData;

pub enum NodeRunnerState<'k, I, N> where 
    N: BehaviorTreeNode<Input=I>,
    I: 'k
{
    RunningNode(N, PhantomData<&'k I>),
    Terminal(N::Terminal),
    Poisoned
}

pub struct NodeRunner<'k, I, N> where 
    N: BehaviorTreeNode<Input=I> + 'k,
    I: 'k
{
    node: NodeRunnerState<'k, I, N>
}

impl<'k, I, N> NodeRunner<'k, I, N> where 
    N: BehaviorTreeNode<Input=I> + 'k,
    I: 'k
{
    pub fn new() -> NodeRunner<'k, I, N> {
        NodeRunner {
            node: NodeRunnerState::RunningNode(N::default(), PhantomData)
        }
    }
}

impl<'k, I, N> Default for NodeRunner<'k, I, N> where 
    N: BehaviorTreeNode<Input=I> + 'k,
    I: 'k
{
    fn default() -> NodeRunner<'k, I, N> {
        NodeRunner::new()
    }
}

fn node_runner_transition<'k, I, N>(node: &mut NodeRunnerState<I, N>, input: &I) 
    -> Statepoint<N> where 
    N: BehaviorTreeNode<Input=I> + 'k,
    I: 'k
{
    let mut result = NodeRunnerState::Poisoned;
    swap(node, &mut result);
    match result {
        NodeRunnerState::RunningNode(n, _) => {
            match n.step(input) {
                NodeResult::Nonterminal(s, a) => {
                    *node = NodeRunnerState::RunningNode(a, PhantomData);
                    Statepoint::Nonterminal(s)
                },
                NodeResult::Terminal(t) => {
                    *node = NodeRunnerState::Terminal(t.clone());
                    Statepoint::Terminal(t)
                }
            }
        },
        NodeRunnerState::Terminal(t) => {
            *node = NodeRunnerState::Terminal(t.clone());
            Statepoint::Terminal(t)
        },
        _ => panic!("Node runner was poisoned!")
    }
}

impl<'k, I, N> Automaton<'k> for NodeRunner<'k, I, N> where 
    N: BehaviorTreeNode< Input=I> + 'k,
    I: 'k
{
    type Input = I;
    type Action = Statepoint<N>;
    fn transition(&mut self, input: &I) -> Statepoint<N> {
        node_runner_transition(&mut self.node, input)
    }

    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&Self::Input) -> Self::Action + 't> where 
        'k: 't 
    {
        let node_part = &mut self.node;
        Box::new(move |input: &I| {
            node_runner_transition(node_part, input)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&Self::Input) -> Self::Action + 'k> {
        let mut node_part = self.node;
        Box::new(move |input: &I| {
            node_runner_transition(&mut node_part, input)
        })
    }

}