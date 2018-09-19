use behavior_tree_node::Statepoint;
use serial_node::{SerialDecider, NontermDecision, TermDecision};
use parallel_node::{ParallelDecider, ParallelDecision};
use std::marker::PhantomData;

/// Runs all nodes in sequence, one at a time, regardless of how they resolve 
/// in the end. 
pub struct SerialRunner<E, N, T> where E: IntoIterator<Item=E> {
    _who_cares: PhantomData<(E, N, T)>
}

impl<E, N, T> SerialDecider for SerialRunner<E, N, T> where 
    E: IntoIterator<Item=E>
{
    type Enum = E;
    type Nonterm = N;
    type Term = T;
    type Exit = ();
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

impl<E, N> SerialDecider for SerialSelector<E, N> where 
    E: IntoIterator<Item=E>
{
    type Enum = E;
    type Nonterm = N;
    type Term = bool;
    type Exit = Option<E>;

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

impl<E, N> SerialDecider for SerialSequence<E, N> where 
    E: IntoIterator<Item=E>
{
    type Enum = E;
    type Nonterm = N;
    type Term = bool;
    type Exit = Option<E>;
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

pub struct ParallelRunner<I, N, R, T> {
    _who_cares: PhantomData<(I, N, R, T)>
}

impl<I, N, R, T> ParallelDecider for ParallelRunner<I, N, R, T> where 
    I: 'static,
    N: 'static,
    R: 'static + Clone,
    T: 'static + Clone
{
    type Input = I;
    type Nonterm = Statepoint<N, R>;
    type Term = T;
    type Exit = Box<[Statepoint<R, T>]>;

    fn each_step(_input: &I, states: Box<[Statepoint<Statepoint<N, R>, T>]>) -> 
        ParallelDecision<Box<[Statepoint<Statepoint<N, R>, T>]>, Box<[Statepoint<R, T>]>> 
    {
        if states.iter().any(|val| match val {
            Statepoint::Nonterminal(Statepoint::Nonterminal(_)) => true,
            _ => false 
        }) {
            ParallelDecision::Stay(states)
        } else {
            ParallelDecision::Exit(
                states.iter().map(|val| match val {
                    Statepoint::Nonterminal(v) => match v {
                        Statepoint::Terminal(k) => Statepoint::Nonterminal(k.clone()),
                        _ => unreachable!("No currently pending nodes")
                    },
                    Statepoint::Terminal(k) => Statepoint::Terminal(k.clone())
                }).collect::<Vec<_>>().into_boxed_slice()
            )
        }
    }
}

pub struct ParallelRacer<I, N, T>  {
    _who_cares: PhantomData<(I, N, T)>
}

impl<I, N, T> ParallelDecider for ParallelRacer<I, N, T> where 
    I: 'static,
    N: 'static,
    T: 'static + Clone
{
    type Input = I;
    type Nonterm = N;
    type Term = T;
    type Exit = (usize, T);

    fn each_step(_input: &I, states: Box<[Statepoint<N, T>]>) -> 
        ParallelDecision<Box<[Statepoint<N, T>]>, (usize, T)> 
    {
        let mut retval = Option::None;
        for value in states.iter().enumerate() {
            if let Statepoint::Terminal(val) = value.1 {
                retval = Option::Some((value.0, val.clone()));
                break;
            }
        };
        match retval {
            Option::None => ParallelDecision::Stay(states),
            Option::Some(v) => ParallelDecision::Exit(v)
        }

    }
}