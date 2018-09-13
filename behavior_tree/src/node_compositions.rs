use behavior_tree_node::Statepoint;
use serial_node::{SerialDecider, NontermDecision, TermDecision};
use parallel_node::{ParallelDecider, ParallelDecision};
use std::marker::PhantomData;

/// Runs all nodes in sequence, one at a time, regardless of how they resolve 
/// in the end. 
pub struct SerialRunner<E, N, T> where E: IntoIterator<Item=E> {
    _who_cares: PhantomData<(E, N, T)>
}

impl<E, N, T> SerialDecider<E, N, T, ()> for SerialRunner<E, N, T> where 
    E: IntoIterator<Item=E>
{
    fn on_nonterminal(_ordinal: E, _statept: &N) -> NontermDecision<E, ()> {
        NontermDecision::Step
    }

    fn on_terminal(ordinal: E, _statept: &T) -> TermDecision<E, ()> {
        match ordinal.into_iter().next() {
            Option::Some(e) => {
                TermDecision::Trans(e)
            },
            Option::None => TermDecision::Exit(())
        }
    }
}

/// Runs nodes in sequence until one resolves successfully, or all of them 
/// fail. 
pub struct SerialSelector<E, N> where E: IntoIterator<Item=E> {
    _who_cares: PhantomData<(E, N)>
}

impl<E, N> SerialDecider<E, N, bool, Option<E>> for SerialSelector<E, N> where 
    E: IntoIterator<Item=E>
{
    fn on_nonterminal(_ordinal: E, _statept: &N) -> NontermDecision<E, Option<E>> {
        NontermDecision::Step
    }

    fn on_terminal(ordinal: E, statept: &bool) -> TermDecision<E, Option<E>> {
        if *statept {
            TermDecision::Exit(Option::Some(ordinal))
        } else {
            match ordinal.into_iter().next() {
                Option::Some(e) => {
                    TermDecision::Trans(e)
                },
                Option::None => TermDecision::Exit(Option::None)
            }
        }
    }
}

/// Runs nodes in sequence while they resolve successfully. Stops as soon as 
/// a node fails. 
pub struct SerialSequence<E, N> where E: IntoIterator<Item=E> {
    _who_cares: PhantomData<(E, N)>
}

impl<E, N> SerialDecider<E, N, bool, Option<E>> for SerialSequence<E, N> where 
    E: IntoIterator<Item=E>
{
    fn on_nonterminal(_ordinal: E, _statept: &N) -> NontermDecision<E, Option<E>> {
        NontermDecision::Step
    }

    fn on_terminal(ordinal: E, statept: &bool) -> TermDecision<E, Option<E>> {
        if !*statept {
            TermDecision::Exit(Option::Some(ordinal))
        } else {
            match ordinal.into_iter().next() {
                Option::Some(e) => {
                    TermDecision::Trans(e)
                },
                Option::None => TermDecision::Exit(Option::None)
            }
        }
    }
}

/// Runs all nodes concurrently until they all return a trap state (indicated 
/// by either a terminal or a nonterminal returning a statepoint terminal). 
pub struct ParallelRunner<E, N, R, T> where E: IntoIterator<Item=E> {
    _who_cares: PhantomData<(E, N, R, T)>
}

impl<E, N, R, T> ParallelDecider<E, Statepoint<N, R>, T, ()> for 
    ParallelRunner<E, N, R, T> where 
    E: IntoIterator<Item=E>,
    N: 'static,
    R: 'static,
    T: 'static
{
    fn each_step<'k, K>(iterthing: K) -> ParallelDecision<E, Statepoint<N, R>, ()> where
        K: Iterator<Item=&'k Statepoint<Statepoint<N, R>, T>> + 'k
    {
        let mut it = iterthing;
        if it.any(|elm| match elm {
            Statepoint::Nonterminal(Statepoint::Nonterminal(_)) => true,
            _ => false
        }) {
            ParallelDecision::Stay(Box::new(|_, _| false))
        } else {
            ParallelDecision::Exit(())
        }
    }
}

/// Runs all nodes concurrently, stopping all nodes as soon as one terminates. 
pub struct ParallelRacer<E, N> where E: IntoIterator<Item=E> {
    _who_cares: PhantomData<(E, N)>
}

impl<E, N, T> ParallelDecider<E, N, T, ()> for ParallelRacer<E, N> where 
    E: IntoIterator<Item=E>,
    N: 'static,
    T: 'static
{
    fn each_step<'k, K>(iterthing: K) -> ParallelDecision<E, N, ()> where
        K: Iterator<Item=&'k Statepoint<N, T>> + 'k
    {
        let mut it = iterthing;
        if it.any(|elm| match elm {
            Statepoint::Terminal(_) => true,
            _ => false
        }) {
            ParallelDecision::Exit(())
        } else {
            ParallelDecision::Stay(Box::new(|_, _| false))
        }
    }
}