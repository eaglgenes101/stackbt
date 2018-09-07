/// In this framework, state machines expose decision points which abstract 
/// over their nonterminal states as well as exit points which abstract over 
/// their terminal states. At a terminal state, only a transition to an 
/// entirely new state is possible. However, at a decision point, the parent 
/// state machine can either decide to step the state machine as normal, or 
/// cause a transition to an entirely new state machine, which abandons the 
/// original child. The parent can also decide to cause a transition to an 
/// exit point, which necessarily causes the child state machine to halt and 
/// be dropped. 
/// 
/// Parent state machines can also run their children concurrently. In this 
/// case, at each point in the cartesian product of their statepoints, the 
/// parent can decide whether to continue, and decide to continue or 
/// transition their children, or to return to its own parent, which causes 
/// all children to be abandoned. 

/// An generic enum which each composable state machine exposes for its 
/// statepoints
pub enum Statepoint<S, T> where S: AutomatonNode<T>
{
    Nonterminal(S::Nonterminal),
    Terminal(S::Terminal),
}

pub enum NodeResult<S, T> where S: AutomatonNode<T> 
{
    Nonterminal(S::Nonterminal, S),
    Terminal(S::Terminal)
}

pub trait AutomatonNode<T>: Default {
    type Input;
    type Nonterminal: Into<T>;
    type Terminal: Into<T> + Clone;
    fn step(self, input: &Self::Input) -> NodeResult<Self, T>;
}
