use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// Nonterminal pushdown transition for the pushdown automaton. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PushdownTransition<A, N> {
    /// Push a new frame onto the pushdown stack. 
    Push(A, N),
    /// Keep the frames on the stack as is. 
    Stay(A),
    /// Remove the topmost frame from the stack. 
    Pop(A)
}

/// Terminal pushdown transition for the pushdown automaton. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TerminalTransition<A, N> {
    /// Push a new frame onto the pushdown stack. 
    Push(A, N),
    /// Keep the frames on the stack as is. 
    Stay(A)
}

/// Implementation of a pushdown automaton which builds upon existing state 
/// machines. Somewhat more powerful than state machines, but in return, 
/// requires some allocable space and some extra tolerance for amortized 
/// runtime costs. 
#[derive(Clone, PartialEq, Debug)]
pub struct PushdownAutomaton <'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k,
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
    /// Create a new pushdown automaton. 
    pub fn new(terminal: T) -> PushdownAutomaton<'k, I, A, N, T> {
        PushdownAutomaton {
            bottom: Option::Some(terminal),
            stack: Vec::new(),
            _i_exists: PhantomData,
            _a_exists: PhantomData
        }
    }

    /// Create a new pushdown automaton from an existing iterable collection 
    /// of finite state machines. 
    pub fn from_iterable<K, S>(terminal: T, prepush: S)
    -> PushdownAutomaton<'k, I, A, N, T> where
        K: Iterator<Item = N>,
        S: IntoIterator<Item = N, IntoIter = K> 
    {
        PushdownAutomaton::from_iter(terminal, prepush.into_iter())
    }

    /// Create a new pushdown automaton from an iterator supplying finite 
    /// state machines. 
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

impl<'k, I, A, N, T> Automaton<'k> for PushdownAutomaton<'k, I, A, N, T> where 
    I: 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, N>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, N>> + 'k,
{
    type Input = I;
    type Action = A;
    #[inline]
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
                let mut tmp_some = self.bottom
                    .take()
                    .expect("Pushdown automaton was poisoned");
                match tmp_some.transition(input) {
                    TerminalTransition::Push(act, new) => {
                        self.stack.push(new);
                        self.bottom = Option::Some(tmp_some);
                        act
                    },
                    TerminalTransition::Stay(act) => {
                        self.bottom = Option::Some(tmp_some);
                        act
                    }
                }
            }
        }
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
        fn step (&self, new: &i64, internal: &mut i64) -> Self::Action {
            if *new == 0 {
                TerminalTransition::Push(*internal, InternalStateMachine::new(
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
        fn step (&self, new: &i64, internal: &mut i64) -> Self::Action {
            if *new == 0 {
                PushdownTransition::Push(*internal, InternalStateMachine::new(
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
            InternalStateMachine::new(TerminalFunction, 0),
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