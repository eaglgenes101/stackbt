/// The automaton trait is used to represent agents which, at a regular rate, 
/// take input, process it, and return an action. Most of them also change 
/// their internal state each transition. 
pub trait Automaton<I, A> {
    #[must_use]
    fn transition(&mut self, input: &I) -> A;
}

/// Marker trait for Finite State Automata, which are a restricted class of 
/// automata that are quite well behaved. In particular, they occupy fixed 
/// memory, and thus do not need extra allocation to operate. 
pub trait FiniteStateAutomaton<I, A>: Automaton<I, A> {}