use std::ops::Fn;
use automaton::Automaton;
use std::marker::PhantomData;

pub struct InternalAutomaton<'f, I, S, A, C: Fn(&I, &mut S) -> A + 'f> {
    current_state: S,
    calling: &'f C,
    _i_life_check: PhantomData<I>,
    _a_life_check: PhantomData<A>,
}

impl<'f, I, S, A, C: Fn(&I, &mut S) -> A> InternalAutomaton<'f, I, S, A, C> {
    pub fn new(init_state: S, to_call: &'f C) -> InternalAutomaton<I, S, A, C> {
        InternalAutomaton {
            current_state: init_state,
            calling: to_call,
            _i_life_check: PhantomData,
            _a_life_check: PhantomData,
        }
    }

    fn step(&mut self, input: &I) -> A {
        (self.calling)(&input, &mut self.current_state)
    }
}

impl<'f, I, S, A, C: Fn(&I, &mut S) -> A> Automaton<'f, I, A> for 
    InternalAutomaton<'f, I, S, A, C> 
{
    fn transition(&mut self, input: &I) -> A {
        self.step(&input)
    }
}