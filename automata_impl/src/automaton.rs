use std::ops::FnMut;
use std::iter::Iterator;

/// The automaton trait is used to represent agents which, at a regular rate, 
/// take input, process it, and return an action. Most of them also change 
/// their internal state each transition. 
pub trait Automaton<'k> {
    type Input: 'k;
    type Action;

    fn transition(&mut self, input: &Self::Input) -> Self::Action;
    fn as_fnmut<'t>(&'t mut self) -> Box<FnMut(&Self::Input) -> Self::Action + 't> where 
        'k: 't;
    fn into_fnmut(self) -> Box<FnMut(&Self::Input) -> Self::Action + 'k>;

    fn into_scan_iter<K>(self, iter: K) -> Box<Iterator<Item=Self::Action> + 'k> where 
        K: Iterator<Item = Self::Input> + 'k,
        Self: Sized + 'k
    {
        Box::new(iter.scan(self, move |state: &mut Self, input: Self::Input| {
            Option::Some(state.transition(&input))
        }))
    }
        
}

/// Marker trait for Finite State Automata, which are a restricted class of 
/// automata that are quite well behaved. In particular, they occupy fixed 
/// memory, and thus do not need extra allocation to operate. 
pub trait FiniteStateAutomaton<'k>: Automaton<'k> {}