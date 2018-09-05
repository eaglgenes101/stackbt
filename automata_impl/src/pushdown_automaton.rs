use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

pub enum PushdownTransition<A> {
    Push(A),
    Change(A),
    Pop
}

pub enum TerminalPushdownTransition<A> {
    Push(A),
    Change(A)
}

/// Implementation of a pushdown automaton which builds upon existing state 
/// machines. Somewhat more powerful than state machines, but in return, 
/// requires some allocable space and some extra tolerance for amortized 
/// runtime costs. 
pub struct PushdownAutomaton <I, A, N, T> where 
    N: FiniteStateAutomaton<I, PushdownTransition<A>>,
    T: FiniteStateAutomaton<I, TerminalPushdownTransition<A>>
{
    bottom: T,
    stack: Vec<N>,
    _i_exists: PhantomData<I>,
    _a_exists: PhantomData<A>
}

