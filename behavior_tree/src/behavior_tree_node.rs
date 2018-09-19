/// In this library, behavior trees are implemented in a very generalized 
/// manner, making them very versatile. At each step, a behavior tree node 
/// steps some, and stop at either an exit point, which abstracts over that 
/// node's terminal states, or a decision point, which abstracts over that 
/// node's nonterminal states. At a terminal state, only a transition to an 
/// entirely new behavior tree node is possible. However, at a decision point,
/// the parent behavior tree node can either decide to step the behavior tree
/// node as normal, or cause a transition to an entirely new behavior tree 
/// node, which abandons the original child node. The parent can also 
/// themselves transition to an exit point, which necessarily causes their 
/// children to be halted and dropped. 

/// A generic enum which are provided to help implementations of certain 
/// behavior tree nodes choose whether a particular state is nonterminal or 
/// terminal, and to work with nonterminal or terminal states their children 
/// have themselves chosen. 
pub enum Statepoint<N, T> {
    Nonterminal(N),
    Terminal(T),
}

/// The return value of behavior tree nodes. To statically prevent further 
/// running after a node reaches a terminal state, the whole node is taken by 
/// move during a step. At a nonterminal, the nonterminal decision point value 
/// is returned along with the modified behavior tree node, while at a terminal, 
/// only the terminal decision point value is returned, with the node instance 
/// dropped and never to return. 
pub enum NodeResult<N, T, M> {
    Nonterminal(N, M),
    Terminal(T)
}

/// The behavior tree node trait itself. 
pub trait BehaviorTreeNode {
    type Input;
    type Nonterminal;
    type Terminal;
    fn step(self, input: &Self::Input) -> 
        NodeResult<Self::Nonterminal, Self::Terminal, Self> where 
        Self: Sized;
}


