use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

pub enum PushdownTransition<A, N> {
    Push(A, N),
    Stay(A),
    Pop(A)
}

pub enum TerminalTransition<A, N> {
    Push(A, N),
    Stay(A)
}

/// Implementation of a pushdown automaton which builds upon existing state 
/// machines. Somewhat more powerful than state machines, but in return, 
/// requires some allocable space and some extra tolerance for amortized 
/// runtime costs. 
pub struct PushdownAutomaton <I, A, N, T> where 
    N: FiniteStateAutomaton<I, PushdownTransition<A, N>>,
    T: FiniteStateAutomaton<I, TerminalTransition<A, N>>
{
    bottom: T,
    stack: Vec<N>,
    _i_exists: PhantomData<I>,
    _a_exists: PhantomData<A>
}

impl<I, A, N, T> PushdownAutomaton<I, A, N, T> where 
    N: FiniteStateAutomaton<I, PushdownTransition<A, N>>,
    T: FiniteStateAutomaton<I, TerminalTransition<A, N>>
{
    pub fn new<K, S>(terminal: T, prepush: K) -> PushdownAutomaton<I, A, N, T> where 
        K: Iterator<Item = N>,
        S: IntoIterator<Item = N, IntoIter = K>
    {
        let to_use_vec = prepush.collect();
        PushdownAutomaton {
            bottom: terminal,
            stack: to_use_vec,
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }
}

impl<I, A, N, T> Automaton<I, A> for PushdownAutomaton<I, A, N, T> where 
    N: FiniteStateAutomaton<I, PushdownTransition<A, N>>,
    T: FiniteStateAutomaton<I, TerminalTransition<A, N>>
{
    fn transition(&mut self, input: &I) -> A {
        match self.stack.pop() {
            Option::Some(mut val) => {
                match val.transition(input) {
                    PushdownTransition::Push(act, new) => {
                        self.stack.push(val);
                        self.stack.push(new);
                        act
                    },
                    PushdownTransition::Stay(act) => {
                        self.stack.push(val);
                        act
                    },
                    PushdownTransition::Pop(act) => act
                }
            },
            Option::None => {
                match self.bottom.transition(input) {
                    TerminalTransition::Push(act, new) => {
                        self.stack.push(new);
                        act
                    },
                    TerminalTransition::Stay(act) => act
                }
            }
        }
    }
}

