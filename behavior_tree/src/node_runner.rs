use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use stackbt_automata_impl::automaton::{Automaton, FiniteStateAutomaton};

/// Automaton implementation which wraps a behavior tree node and forwards 
/// input to it and transitions back from it, automatically restarting the
/// node if it terminates. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct NodeRunner<N, C> where 
    N: BehaviorTreeNode + 'static,
    C: Fn() -> N
{
    constructor: C,
    node: Option<N>
}

impl<N, C> NodeRunner<N, C> where 
    N: BehaviorTreeNode + 'static,
    C: Fn() -> N
{
    /// Create a new node runner from a behavior tree node. 
    pub fn new(constructor: C) -> NodeRunner<N, C> {
        let new_node = constructor();
        NodeRunner {
            constructor: constructor, 
            node: Option::Some(new_node)
        }
    }
}

impl<N, C> Automaton<'static> for NodeRunner<N, C> where 
    N: BehaviorTreeNode + 'static,
    C: Fn() -> N
{
    type Input = N::Input;
    type Action = Statepoint<N::Nonterminal, N::Terminal>;
    #[inline]
    fn transition(&mut self, input: &N::Input) -> Statepoint<N::Nonterminal, N::Terminal> {
        match self.node
            .take()
            .expect("Node runner was poisoned")
            .step(input) 
        {
            NodeResult::Nonterminal(s, a) => {
                self.node = Option::Some(a);
                Statepoint::Nonterminal(s)
            },
            NodeResult::Terminal(t) => {
                self.node = Option::Some((self.constructor)());
                Statepoint::Terminal(t)
            }
        }
    }
}

impl<N, C> FiniteStateAutomaton<'static> for NodeRunner<N, C> where 
    N: BehaviorTreeNode + 'static + Copy,
    C: Fn() -> N + Copy
{}

#[cfg(test)]
mod tests {
    use behavior_tree_node::Statepoint;

    #[test]
    fn runner_test() {
        use stackbt_automata_impl::automaton::Automaton;
        use base_nodes::PredicateWait;
        use node_runner::NodeRunner;
        let constructor = | | PredicateWait::new(|i: &i64| {
            if *i == 0 {
                Statepoint::Terminal(())
            } else {
                Statepoint::Nonterminal(())
            }
        });
        let mut machine = NodeRunner::new(constructor);
        match machine.transition(&1) {
            Statepoint::Nonterminal(_) => (),
            _ => unreachable!("Expected nonterminal state")
        };
        match machine.transition(&0) {
            Statepoint::Terminal(_) => (),
            _ => unreachable!("Expected terminal state"),
        };
        match machine.transition(&1) {
            Statepoint::Nonterminal(_) => (),
            _ => unreachable!("Expected nonterminal state")
        };
    }
}