use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;
use std::mem::swap;

/// Nonterminal pushdown transition. Push pushes a new state machine onto the 
/// pushdown stack, Stay keeps the state machine as it is, and Pop removes the 
/// uppermost state machine from the stack. State machines may change state in 
/// addition to manipulating the stack. 
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PushdownTransition<A, N> {
    Push(A, N),
    Stay(A),
    Pop(A)
}

/// Terminal pushdown transition. Push pushes a new state machine onto the 
/// pushdown stack, whilt Stay keeps the state machine as it is. State 
/// machines may change state in addition to manipulating the stack.
#[derive(Copy, Clone, Debug, Eq, PartialEq)] 
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
    T: FiniteStateAutomaton<'k> + 'k,
{
    bottom: Option<T>,
    stack: Vec<N>,
    _i_exists: PhantomData<&'k I>,
    _a_exists: PhantomData<A>
}

impl<'k, I, A, N, T> PushdownAutomaton<'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k,
{
    pub fn new(terminal: T) -> PushdownAutomaton<'k, I, A, N, T> {
        PushdownAutomaton {
            bottom: Option::Some(terminal),
            stack: Vec::new(),
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }

    pub fn from_iterable<K, S>(terminal: T, prepush: S)
    -> PushdownAutomaton<'k, I, A, N, T> where
        K: Iterator<Item = N>,
        S: IntoIterator<Item = N, IntoIter = K> 
    {
        PushdownAutomaton::from_iter(terminal, prepush.into_iter())
    }

    pub fn from_iter<K>(terminal: T, prepush: K) 
    -> PushdownAutomaton<'k, I, A, N, T> where 
        K: Iterator<Item = N>
    {
        let to_use_vec = prepush.collect();
        PushdownAutomaton {
            bottom: Option::Some(terminal),
            stack: to_use_vec,
            _i_exists: PhantomData,
            _a_exists: PhantomData,
        }
    }
}

#[inline]
fn stack_elm_transition<'k, I, A, N, T> 
    (stack: &mut Vec<N>, bottom: &mut Option<T>, input: &I) -> A where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k,
{
    match stack.pop() {
        Option::Some(mut val) => {
            match val.transition(input) {
                PushdownTransition::Push(act, new) => {
                    stack.push(val);
                    stack.push(new.into());
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
            let mut tmp_bottom = Option::None;
            swap(&mut tmp_bottom, bottom);
            let mut tmp_some = tmp_bottom.unwrap();
            match tmp_some.transition(input) {
                TerminalTransition::Push(act, new) => {
                    stack.push(new.into());
                    *bottom = Option::Some(tmp_some);
                    act
                },
                TerminalTransition::Stay(act) => {
                    *bottom = Option::Some(tmp_some);
                    act
                }
            }
        }
    }
}

impl<'k, I, A, N, T> Automaton<'k> for PushdownAutomaton<'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k,
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

#[cfg(test)]
mod test {
    use automaton::Automaton;
    use internal_state_machine::{InternalTransition, InternalStateMachine};
    use pushdown_automaton::{
            PushdownAutomaton, PushdownTransition, TerminalTransition};

    #[derive(Copy, Clone)]
    struct TerminalFunction;
    #[derive(Copy, Clone)]
    struct NonterminalFunction;

    impl InternalTransition for TerminalFunction {
        type Internal = i64;
        type Input = i64;
        type Action = TerminalTransition<i64, 
            InternalStateMachine<'static, NonterminalFunction>>;
        fn step (new: &i64, internal: &mut i64) -> Self::Action {
            if *new == 0 {
                TerminalTransition::Push(*internal, InternalStateMachine::with(
                    NonterminalFunction, 
                    0
                ))
            } else {
                let orig_internal = *internal;
                *internal = *new;
                TerminalTransition::Stay(orig_internal)
            }
        }
    }

    impl InternalTransition for NonterminalFunction {
        type Internal = i64;
        type Input = i64;
        type Action = PushdownTransition<i64, 
            InternalStateMachine<'static, NonterminalFunction>>;
        fn step (new: &i64, internal: &mut i64) -> Self::Action {
            if *new == 0 {
                PushdownTransition::Push(*internal, InternalStateMachine::with(
                    NonterminalFunction, 
                    0
                ))
            } else if *new < 0 {
                PushdownTransition::Pop(*internal)
            } else {
                let orig_internal = *internal;
                *internal = *new;
                PushdownTransition::Stay(orig_internal)
            }
        }
    }

    #[test]
    fn check_def () {
        //from_iterable constructor used to assist type inference
        let mut test_pushdown = PushdownAutomaton::from_iterable(
            InternalStateMachine::with(TerminalFunction, 0),
            Vec::<InternalStateMachine<NonterminalFunction>>::new()
        );
        // 0|
        assert_eq!(test_pushdown.transition(&3), 0);
        // 3|
        assert_eq!(test_pushdown.transition(&4), 3);
        // 4|
        assert_eq!(test_pushdown.transition(&0), 4);
        // 4| 0,
        assert_eq!(test_pushdown.transition(&5), 0);
        // 4| 5,
        assert_eq!(test_pushdown.transition(&0), 5);
        // 4| 5, 0,
        assert_eq!(test_pushdown.transition(&9), 0);
        // 4| 5, 9,
        assert_eq!(test_pushdown.transition(&-2), 9);
        // 4| 5,
        assert_eq!(test_pushdown.transition(&-1), 5);
        // 4|
        assert_eq!(test_pushdown.transition(&2), 4);
    }

}