use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

/// Trait for an enumeration of nodes, all of which have the same input, 
/// nonterminals, and terminals. Each variant corresponds to a different 
/// value of Enumeration. 
pub trait NodeEnumeration<N> where 
    N: BehaviorTreeNode + ?Sized
{
    /// The type used to enumerate the variants of implementations of this 
    /// trait. std::mem::Discriminant works for comparing variants of an enum, 
    /// but not for enumerating or matching against them, hence this 
    /// associated type. 
    type Enumeration: Default + Copy + IntoIterator<Item=Self::Enumeration>;

    fn new(Self::Enumeration) -> Self;
    fn from_existing(N) -> Self;
    fn discriminant(&self) -> Self::Enumeration;
    fn inner_val(self) -> N;
}

pub enum NontermDecision<T, X> {
    Step,
    Trans(T),
    Exit(X)
}

pub enum TermDecision<T, X> {
    Trans(T),
    Exit(X)
}

pub enum NontermReturn<E, N, T> {
    Nonterminal(E, N),
    Terminal(E, T)
}

pub trait SerialDecider<E, N, T, X> {
    fn on_nonterminal(E, &N) -> NontermDecision<E, X>;
    fn on_terminal(E, &T) -> TermDecision<E, X>;
}

pub struct SerialBranchNode<E, N, D, X> where
    N: BehaviorTreeNode + ?Sized,
    E: NodeEnumeration<N>,
    D: SerialDecider<E::Enumeration, N::Nonterminal, N::Terminal, X>
{
    node: E,
    _exists_tuple: PhantomData<(N, D, X)>
}

impl<E, N, D, X> SerialBranchNode<E, N, D, X> where 
    N: BehaviorTreeNode + ?Sized,
    E: NodeEnumeration<N>,
    D: SerialDecider<E::Enumeration, N::Nonterminal, N::Terminal, X>
{
    fn new(variant: E::Enumeration) -> SerialBranchNode<E, N, D, X> {
        SerialBranchNode {
            node: E::new(variant),
            _exists_tuple: PhantomData
        }
    }

    fn from_existing_node(existing: N) -> SerialBranchNode<E, N, D, X> {
        SerialBranchNode {
            node: E::from_existing(existing),
            _exists_tuple: PhantomData
        }
    }
}

impl<E, N, D, X> Default for SerialBranchNode<E, N, D, X> where 
    N: BehaviorTreeNode + ?Sized,
    E: NodeEnumeration<N>,
    D: SerialDecider<E::Enumeration, N::Nonterminal, N::Terminal, X>
{
    fn default() -> SerialBranchNode<E, N, D, X> {
        SerialBranchNode::new(E::Enumeration::default())
    }
}

impl<E, N, D, X> BehaviorTreeNode for SerialBranchNode<E, N, D, X> where
    N: BehaviorTreeNode + ?Sized,
    E: NodeEnumeration<N>,
    D: SerialDecider<E::Enumeration, N::Nonterminal, N::Terminal, X>
{
    type Input = N::Input;
    type Nonterminal = NontermReturn<E::Enumeration, N::Nonterminal, N::Terminal>;
    type Terminal = X;

    fn step(self, input: &N::Input) -> NodeResult<Self::Nonterminal, X, Self> {
        let discriminant = self.node.discriminant();
        match self.node.inner_val().step(input) {
            NodeResult::Nonterminal(i, n) => {
                match D::on_nonterminal(discriminant, &i) {
                    NontermDecision::Step => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, i),
                        Self::from_existing_node(n)
                    ),
                    NontermDecision::Trans(e) => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, i),
                        Self::new(e)
                    ),
                    NontermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            NodeResult::Terminal(i) => {
                match D::on_terminal(discriminant, &i) {
                    TermDecision::Trans(e) => NodeResult::Nonterminal(
                        NontermReturn::Terminal(discriminant, i),
                        Self::new(e)
                    ),
                    TermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            }
        }
    }
}