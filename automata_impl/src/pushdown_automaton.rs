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
pub struct PushdownAutomaton <'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k> + 'k,
    T: FiniteStateAutomaton<'k> + 'k
{
    bottom: T,
    stack: Vec<N>,
    _i_exists: PhantomData<&'k I>,
    _a_exists: PhantomData<A>
}

impl<'k, I, A, N, T> PushdownAutomaton<'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k> + 'k,
    T: FiniteStateAutomaton<'k> + 'k
{
    pub fn new<K, S>(terminal: T, prepush: K) -> PushdownAutomaton<'k, I, A, N, T> where 
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

#[inline]
fn stack_elm_transition<'k, I, A, N, T> (stack: &mut Vec<N>, bottom: &mut T, input: &I)
     -> A where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k
{
    match stack.pop() {
        Option::Some(mut val) => {
            match val.transition(input) {
                PushdownTransition::Push(act, new) => {
                    stack.push(val);
                    stack.push(new);
                    act
                },
                PushdownTransition::Stay(act) => {
                    stack.push(val);
                    act
                },
                PushdownTransition::Pop(act) => act
            }
        },
        Option::None => {
            match bottom.transition(input) {
                TerminalTransition::Push(act, new) => {
                    stack.push(new);
                    act
                },
                TerminalTransition::Stay(act) => act
            }
        }
    }
}

impl<'k, I, A, N, T> Automaton<'k> for PushdownAutomaton<'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k
{
    type Input = I;
    type Action = A;
    #[inline]
    fn transition(&mut self, input: &I) -> A {
        stack_elm_transition(&mut self.stack, &mut self.bottom, &input)
    }

    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&I) -> A + 't> where 'k: 't {
        let mut stack_part = &mut self.stack;
        let mut bottom_part = &mut self.bottom;
        Box::new(move |input: &I| -> A {
            stack_elm_transition(stack_part, bottom_part, &input)
        })
    }

    fn into_fnmut(self) -> Box<FnMut(&I) -> A + 'k> {
        let mut bottom_part = self.bottom;
        let mut stack_part = self.stack;
        Box::new(move |input: &I| -> A {
            stack_elm_transition(&mut stack_part, &mut bottom_part, &input)
        })
    }
}



