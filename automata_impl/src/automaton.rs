use std::ops::FnMut;
use std::iter::Iterator;

/// The automaton trait is used to represent agents which, at a regular rate, 
/// take input, process it, and return an action. Most of them also change 
/// their internal state each transition. 
pub trait Automaton<'k, I, A> {
    #[must_use]
    fn transition(&mut self, input: &I) -> A;
    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&I) -> A + 't> where 'k: 't;
    fn into_fnmut(self) -> Box<FnMut(&I) -> A + 'k>;
}

pub fn automaton_scan<'k, I, A, K, W>(iter: K, auto: W) -> impl Iterator<Item=A> where 
    K: Iterator<Item = I>,
    W: Automaton<'k, I, A> + 'k,
{
    iter.scan(auto, move |state: &mut W, input: I| {
        Option::Some(state.transition(&input))
    })
}

/// Marker trait for Finite State Automata, which are a restricted class of 
/// automata that are quite well behaved. In particular, they occupy fixed 
/// memory, and thus do not need extra allocation to operate. 
pub trait FiniteStateAutomaton<'k, I, A>: Automaton<'k, I, A> {}