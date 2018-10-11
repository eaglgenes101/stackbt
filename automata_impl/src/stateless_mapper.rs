use automaton::{Automaton, FiniteStateAutomaton};
use std::marker::PhantomData;

/// "Automaton" whose purpose is to serve as a stateless mapping
/// between its input and output. Useful for plumbing state machines with 
/// incompatible types together. 
#[derive(PartialEq, Debug)]
pub struct StatelessMapper<'k, I, A, C> where 
    C: Fn(&I) -> A + 'k,
    I: 'k
{
    closure: C,
    _closure_bounds: PhantomData<&'k C>,
    _junk: PhantomData<(I, A)>
}

impl<'k, I, A, C> Clone for StatelessMapper<'k, I, A, C> where 
    C: Fn(&I) -> A + 'k + Clone,
    I: 'k
{
    fn clone(&self) -> Self {
        StatelessMapper {
            closure: self.closure.clone(),
            _closure_bounds: PhantomData,
            _junk: PhantomData
        }
    }
}

impl<'k, I, A, C> Copy for StatelessMapper<'k, I, A, C> where 
    C: Fn(&I) -> A + 'k + Copy,
    I: 'k
{}

impl<'k, I, A, C> StatelessMapper<'k, I, A, C> where 
    C: Fn(&I) -> A + 'k,
    I: 'k
{
    pub fn new(closure: C) -> Self {
        StatelessMapper {
            closure: closure,
            _closure_bounds: PhantomData,
            _junk: PhantomData
        }
    }
}

impl<'k, I, A, C> Automaton<'k> for StatelessMapper<'k, I, A, C> where 
    C: Fn(&I) -> A + 'k,
    I: 'k
{
    type Input = I;
    type Action = A;

    fn transition(&mut self, input: &I) -> A {
        (self.closure)(input)
    }
}

impl<'k, I, A, C> FiniteStateAutomaton<'k> for StatelessMapper<'k, I, A, C> where 
    C: Fn(&I) -> A + 'k + Copy,
    I: 'k
{}