use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
use std::marker::PhantomData;
use std::iter::Iterator;

/// A collection of nodes, all of which have the same input, nonterminal, 
/// and terminal types. The collection is associated with a particular 
/// enumeration, each of whose elements is associated with a fixed 
/// construction procedure. 
pub trait NodeCollection<N>: Default where
    N: BehaviorTreeNode + ?Sized
{
    type Enumeration: 'static + Default + Copy + IntoIterator<Item=Self::Enumeration>;

    /// The iterator type returned by into_iter. This associated type is 
    /// needed so that associated type sizes can be statically determined 
    /// by the compiler to avoid runtime allocation and indirection. 
    type Iter: ExactSizeIterator<Item=N>;

    /// Construct a fresh new collection of newly initialized nodes. 
    fn new() -> Self;

    /// Construct and return a node with the same specific type as would be 
    /// constructed for that ordinal node. 
    fn get_for(Self::Enumeration) -> N;

    /// Consume the node iterator and construct a new collection of nodes 
    /// given these existing nodes. 
    fn from_iter<I>(I) -> Self where I: Iterator<Item=N>;

    /// Render the node collection as an iterator, in a well-specified 
    /// iteration order. 
    fn into_iter(self) -> Self::Iter;
}

pub enum ParallelDecision<E, N, X> {
    Stay(Box<Fn(E, &N) -> bool>),
    Exit(X)
}

pub trait ParallelDecider<E, N, T, X> where 
    N: 'static,
    T: 'static
{
    fn each_step<'k, K>(K) -> ParallelDecision<E, N, X> where
        K: Iterator<Item=&'k Statepoint<N, T>> + 'k;
}

pub struct ParallelBranchNode<C, N, D, X> where
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<C::Enumeration, N::Nonterminal, N::Terminal, X>
{
    collection: C,
    _exists_tuple: PhantomData<(N, D, X)>
}

impl<C, N, D, X> ParallelBranchNode<C, N, D, X> where
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<C::Enumeration, N::Nonterminal, N::Terminal, X>
{
    fn new() -> ParallelBranchNode<C, N, D, X> {
        ParallelBranchNode {
            collection: C::default(),
            _exists_tuple: PhantomData
        }
    }

    fn from_iter<I>(iter: I) -> ParallelBranchNode<C, N, D, X> where 
        I: Iterator<Item=N>
    {
        ParallelBranchNode {
            collection: C::from_iter(iter),
            _exists_tuple: PhantomData
        }
    }
}

impl<C, N, D, X> Default for ParallelBranchNode<C, N, D, X> where
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<C::Enumeration, N::Nonterminal, N::Terminal, X>
{
    fn default() -> ParallelBranchNode<C, N, D, X> {
        ParallelBranchNode::new()
    }
}

impl<C, N, D, X> BehaviorTreeNode for ParallelBranchNode<C, N, D, X> where 
    N: BehaviorTreeNode + ?Sized + 'static,
    C: NodeCollection<N>,
    D: ParallelDecider<C::Enumeration, N::Nonterminal, N::Terminal, X>
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
            ParallelDecision::Stay(i) => {
                let new_state;
                {
                    let new_state_iter = nodes.into_iter()
                        .zip(states.iter().zip(C::Enumeration::default().into_iter()))
                        .map(|(node, (state, count))| match state {
                            Statepoint::Nonterminal(n) => if i(count, &n) {
                                C::get_for(count)
                            } else {
                                node.unwrap()
                            },
                            Statepoint::Terminal(_) => C::get_for(count)
                        });
                    new_state = ParallelBranchNode::from_iter(new_state_iter);
                }
                NodeResult::Nonterminal(states, new_state)
            },
            ParallelDecision::Exit(t) => NodeResult::Terminal(t)
        }
        
    }
}
