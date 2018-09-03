use std::ops::Fn;
use automaton::Automaton;
use std::marker::PhantomData;

/// State machine implemented through 

/// State machine implemented through a closure reference wrapper struct. 
/// Each step, the currently referenced closure is called, returning an 
/// action and a reference to the closure to call for the next step. 
pub struct RefStateMachine<'f, I, A, C: Fn(&I) -> (A, &'f C) + 'f>
{
    current_state: &'f C,
    _i_life_check: PhantomData<I>,
    _a_life_check: PhantomData<A>,
}

impl <'f, I, A, C: Fn(&I) -> (A, &'f C)> RefStateMachine<'f, I, A, C>
{
    pub fn new(init_state: &'f C) -> RefStateMachine<I, A, C> {
        RefStateMachine {
            current_state: init_state,
            _i_life_check: PhantomData,
            _a_life_check: PhantomData
        }
    }

    fn step(&'f mut self, input: &I) -> A {
        let (action, next_state) = (self.current_state)(input);
        self.current_state = next_state;
        action
    }
}

impl <'f, I, A, C: Fn(&I) -> (A, &'f C)> Automaton<'f, I, A> for RefStateMachine<'f, I, A, C> 
{
    fn transition(&'f mut self, input: &I) -> A {
        self.step(input)
    }
}