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
pub struct PushdownAutomaton <'k, I, A, G, N, T> where 
    I: 'k,
    G: Into<N> + 'k,
    N: FiniteStateAutomaton<'k> + 'k,
    T: FiniteStateAutomaton<'k> + 'k,
{
    bottom: Option<T>,
    stack: Vec<N>,
    _i_exists: PhantomData<&'k I>,
    _exists_tuple: PhantomData<(A, G)>,
}

impl<'k, I, A, G, N, T> PushdownAutomaton<'k, I, A, G, N, T> where 
    I: 'k,
    G: Into<N> + 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, G>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, G>> + 'k,
{
    pub fn new(terminal: T) -> PushdownAutomaton<'k, I, A, G, N, T> where 
    {
        PushdownAutomaton {
            bottom: Option::Some(terminal),
            stack: Vec::new(),
            _i_exists: PhantomData,
            _exists_tuple: PhantomData,
        }
    }

    pub fn from_iterable<K, S>(terminal: T, prepush: S)
    -> PushdownAutomaton<'k, I, A, G, N, T> where
        K: Iterator<Item = N>,
        S: IntoIterator<Item = N, IntoIter = K> 
    {
        PushdownAutomaton::from_iter(terminal, prepush.into_iter())
    }

    pub fn from_iter<K>(terminal: T, prepush: K) 
    -> PushdownAutomaton<'k, I, A, G, N, T> where 
        K: Iterator<Item = N>
    {
        let to_use_vec = prepush.collect();
        PushdownAutomaton {
            bottom: Option::Some(terminal),
            stack: to_use_vec,
            _i_exists: PhantomData,
            _exists_tuple: PhantomData,
        }
    }
}

#[inline]
fn stack_elm_transition<'k, I, A, G, N, T> 
    (stack: &mut Vec<N>, bottom: &mut Option<T>, input: &I) -> A where 
    I: 'k,
    G: Into<N> + 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, G>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, G>> + 'k,
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

impl<'k, I, A, G, N, T> Automaton<'k> for PushdownAutomaton<'k, I, A, G, N, T> where 
    I: 'k,
    G: Into<N> + 'k,
    N: FiniteStateAutomaton<'k, Input=I, Action=PushdownTransition<A, G>> + 'k,
    T: FiniteStateAutomaton<'k, Input=I, Action=TerminalTransition<A, G>> + 'k,
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
    use internal_state_machine::InternalStateMachine;
    use pushdown_automaton::{
            PushdownAutomaton, PushdownTransition, TerminalTransition};

    #[derive(Copy, Clone)]
    struct NonterminalProxy;

    impl From<NonterminalProxy> for InternalStateMachine<'static, i64, i64, 
        PushdownTransition<i64, NonterminalProxy>, NonterminalFunction> 
    {
        fn from(_this: NonterminalProxy) -> InternalStateMachine<'static, i64, i64, 
            PushdownTransition<i64, NonterminalProxy>, NonterminalFunction> 
        {
            InternalStateMachine::new(NonterminalFunction, 0)
        }
    }

    #[derive(Copy, Clone)]
    struct TerminalFunction;
    #[derive(Copy, Clone)]
    struct NonterminalFunction;

    fn terminal_transition (new: &i64, internal: &mut i64) 
        -> TerminalTransition<i64, NonterminalProxy> 
    {
        if *new == 0 {
            TerminalTransition::Push(*internal, NonterminalProxy)
        } else {
            let orig_internal = *internal;
            *internal = *new;
            TerminalTransition::Stay(orig_internal)
        }
    }

    fn nonterminal_transition (new: &i64, internal: &mut i64)
        -> PushdownTransition<i64, NonterminalProxy>
    {
        if *new == 0 {
            PushdownTransition::Push(*internal, NonterminalProxy)
        } else if *new < 0 {
            PushdownTransition::Pop(*internal)
        } else {
            let orig_internal = *internal;
            *internal = *new;
            PushdownTransition::Stay(orig_internal)
        }
    }

    impl From<TerminalFunction> for fn(&i64, &mut i64) 
        -> TerminalTransition<i64, NonterminalProxy>
    {
        fn from(this: TerminalFunction) -> fn(&i64, &mut i64) 
            -> TerminalTransition<i64, NonterminalProxy>
        {
            terminal_transition
        }
    }

    impl From<NonterminalFunction> for fn(&i64, &mut i64)
        -> PushdownTransition<i64, NonterminalProxy>
    {
        fn from(this: NonterminalFunction) -> fn(&i64, &mut i64)
            -> PushdownTransition<i64, NonterminalProxy>
        {
            nonterminal_transition
        }
    }

    #[test]
    fn check_def () {
        //from_iterable constructor used to assist type inference
        let mut test_pushdown = PushdownAutomaton::from_iterable(
            InternalStateMachine::new(TerminalFunction, 0),
            Vec::<InternalStateMachine<i64, i64, PushdownTransition<i64, NonterminalProxy>, 
                NonterminalFunction>>::new()
        );
        // 0|
        assert!(test_pushdown.transition(&3) == 0);
        // 3|
        assert!(test_pushdown.transition(&4) == 3);
        // 4|
        assert!(test_pushdown.transition(&0) == 4);
        // 4| 0,
        assert!(test_pushdown.transition(&5) == 0);
        // 4| 5,
        assert!(test_pushdown.transition(&0) == 5);
        // 4| 5, 0,
        assert!(test_pushdown.transition(&9) == 0);
        // 4| 5, 9,
        assert!(test_pushdown.transition(&-2) == 9);
        // 4| 5,
        assert!(test_pushdown.transition(&-1) == 5);
        // 4|
        assert!(test_pushdown.transition(&2) == 4);
    }

}