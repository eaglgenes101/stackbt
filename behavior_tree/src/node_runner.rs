use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use stackbt_automata_impl::automaton::Automaton;

/// Automaton implementation which wraps a behavior tree node and forwards 
/// input to it and transitions back from it, automatically restarting the
/// node if it terminates. 
pub struct NodeRunner<N> where 
    N: BehaviorTreeNode + 'static
{
    node: Option<N>
}

impl<N> NodeRunner<N> where 
    N: BehaviorTreeNode + 'static
{
    pub fn new(node: N) -> NodeRunner<N> {
        NodeRunner {
            node: Option::Some(node)
        }
    }
}

impl<N> Default for NodeRunner<N> where 
    N: BehaviorTreeNode + Default + 'static
{
    fn default() -> NodeRunner<N> {
        NodeRunner::new(N::default())
    }
}

impl<N> Automaton<'static> for NodeRunner<N> where 
    N: BehaviorTreeNode + Default + 'static
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
                self.node = Option::Some(N::default());
                Statepoint::Terminal(t)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use base_nodes::WaitCondition;
    use behavior_tree_node::Statepoint;

    #[derive(Default)]
    struct ThingPred;

    impl WaitCondition for ThingPred {
        type Input = i64;
        type Nonterminal = ();
        type Terminal = ();
        fn do_end(i: &i64) -> Statepoint<(), ()> {
            if *i == 0 {
                Statepoint::Terminal(())
            } else {
                Statepoint::Nonterminal(())
            }
        }
    }

    #[test]
    fn runner_test() {
        use stackbt_automata_impl::automaton::Automaton;
        use base_nodes::PredicateWait;
        use node_runner::NodeRunner;
        let node = PredicateWait::with(ThingPred);
        let mut machine = NodeRunner::new(node);
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