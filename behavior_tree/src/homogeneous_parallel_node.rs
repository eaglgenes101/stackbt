use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use std::iter::Iterator;

/// A collection of nodes, all of which have the same input, nonterminal, 
/// and terminal types. Implementors of this trait are expected to choose 
/// some fixed tuple of node constructions such that for each valid index n of 
/// the collection, get_for(n) yields a node that is constructed in the same 
/// manner, and every iterator derived from into_iter() gives a similarly 
/// constructed node at index n of iteration. 
/// 
/// In addition, if an iterator was derived from a collection, feeding it back 
/// into from_iter() will result in the same collection of nodes as before, 
/// and replacing individual nodes with corresponding nodes constructed by 
/// get_for() will result in these replacements occuring in place of their 
/// originals in iteration, and all unreplaced nodes being yielded by the 
/// iterator just as if the nodes were never replaced. 
pub trait NodeCollection<N>: Default where
    N: BehaviorTreeNode + ?Sized
{
    /// The iterator type returned by into_iter. This associated type is 
    /// needed so that associated type sizes can be statically determined 
    /// by the compiler to avoid runtime allocation and indirection. 
    type Iter: Iterator<Item=N>;

    /// Construct a fresh new collection of newly initialized nodes. 
    fn new() -> Self;

    /// Construct and return a node with the same specific type as would be 
    /// constructed for that ordinal node. 
    fn get_for(usize) -> Option<N>;

    /// Consume the node iterator and construct a new collection of nodes 
    /// given these existing nodes. 
    fn from_iter<I>(I) -> Self where I: Iterator<Item=N>;

    /// Render the node collection as an iterator, in a well-specified 
    /// iteration order. 
    fn into_iter(self) -> Self::Iter;
}

pub enum Decision<N, I, X> where 
    I: Fn(&N) -> bool
{
    Stay(I, PhantomData<N>),
    Exit(X)
}

pub trait ParallelDecider<N, X> where
    N: BehaviorTreeNode + ?Sized + 'static
{
    type GenClosure: Fn(&N::Nonterminal) -> bool;

    fn each_step<'k, I>(I) -> Decision<N::Nonterminal, Self::GenClosure, X> where
        I: Iterator<Item=&'k Statepoint<N::Nonterminal, N::Terminal>> + 'k;
}

pub struct HomogeneousParallelNode<C, N, D, X> where
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<N, X>
{
    collection: C,
    _exists_tuple: PhantomData<(N, D, X)>
}

impl<C, N, D, X> HomogeneousParallelNode<C, N, D, X> where
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<N, X>
{
    fn new() -> HomogeneousParallelNode<C, N, D, X> {
        HomogeneousParallelNode {
            collection: C::default(),
            _exists_tuple: PhantomData
        }
    }

    fn from_iter<I>(iter: I) -> HomogeneousParallelNode<C, N, D, X> where 
        I: Iterator<Item=N>
    {
        HomogeneousParallelNode {
            collection: C::from_iter(iter),
            _exists_tuple: PhantomData
        }
    }
}

impl<C, N, D, X> Default for HomogeneousParallelNode<C, N, D, X> where
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<N, X> 
{
    fn default() -> HomogeneousParallelNode<C, N, D, X> {
        HomogeneousParallelNode::new()
    }
}

impl<C, N, D, X> BehaviorTreeNode for HomogeneousParallelNode<C, N, D, X> where 
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<N, X> 
{
    type Input = N::Input;
    type Nonterminal = Vec<Statepoint<N::Nonterminal, N::Terminal>>;
    type Terminal = X;

    fn step(self, input: &N::Input) -> NodeResult<Self::Nonterminal, X, Self> {
        let node_iter: C::Iter = self.collection.into_iter();
        let (states, nodes): (Vec<_>, Vec<_>) = node_iter
            .map(|node| match node.step(input) {
                NodeResult::Nonterminal(n, m) => {
                    (Statepoint::Nonterminal(n), Option::Some(m))
                },
                NodeResult::Terminal(t) => {
                    (Statepoint::Terminal(t), Option::None)
                }
            })
            .unzip();
        let decision = D::each_step(states.iter());
        match decision {
            Decision::Stay(i, _e) => {
                let new_state;
                {
                    let new_state_iter = nodes.into_iter()
                        .zip(states.iter().enumerate())
                        .map(|(node, (count, state))| match state {
                            Statepoint::Nonterminal(n) => if i(&n) {
                                C::get_for(count).unwrap()
                            } else {
                                node.unwrap()
                            },
                            Statepoint::Terminal(_) => C::get_for(count).unwrap()
                        });
                    new_state = HomogeneousParallelNode::from_iter(new_state_iter);
                }
                NodeResult::Nonterminal(
                    states,
                    new_state
                )
            },
            Decision::Exit(t) => NodeResult::Terminal(t)
        }
        
    }
}
