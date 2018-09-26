use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

/// Trait for a type which is exhaustively enumerable. 
pub trait Enumerable: Copy + Sized {
    /// Get the first value of the enumerable. 
    fn zero() -> Self;
    /// Get the subsequent values of the enumerable. 
    fn successor(self) -> Option<Self>;
}

/// Trait for an enumeration of nodes, all of which have the same input, 
/// nonterminals, and terminals. Each variant corresponds to a different 
/// value of Enumeration. 
pub trait EnumNode: BehaviorTreeNode {
    /// The type used to enumerate the variants of implementations of this 
    /// trait. std::mem::Discriminant works for comparing variants of an enum, 
    /// but not for enumerating or matching against them, hence this 
    /// associated type. 
    type Discriminant: Enumerable;

    /// Initialize a new node with discriminant with the given discriminant. 
    fn new(Self::Discriminant) -> Self;

    /// Determine the discriminant of an enumerated node. 
    fn discriminant(&self) -> Self::Discriminant;
}

/// Enumeration of the possible decisions when the child node reaches a 
/// nonterminal state. 
pub enum NontermDecision<E, T, X> {
    /// Step the current subnode. 
    Step(T),
    /// Transition from the current subnode to a new one. 
    Trans(E, T),
    /// Exit the current supernode entirely. 
    Exit(X)
}

/// Enumeration of the possible decisions when the child node reaches a 
/// terminal state. 
pub enum TermDecision<E, T, X> {
    /// Transition from the current subnode to a new one. 
    Trans(E, T),
    /// Exit the current supernode entirely. 
    Exit(X)
}

/// Return type of the SerialBranchNode. 
pub enum NontermReturn<E, N, T> {
    /// Nonterminal of a subnode. 
    Nonterminal(E, N),
    /// Terminal of a subnode. 
    Terminal(E, T)
}

/// Trait for the transition behavior of a SerialBranchNode. 
pub trait SerialDecider {
    /// Type enumerating the discriminants of the node enumeration. 
    type Enum;
    /// Type of the inputs of the subnodes. 
    type Input;
    /// Type of the nonterminals of the subnodes. 
    type Nonterm;
    /// Type of the terminals of the subnodes. 
    type Term;
    /// Supernode terminal type. 
    type Exit;
    /// Given a reference to the input and the current nonterminal state, 
    /// decide what to do from the nonterminal statepoint. 
    fn on_nonterminal(&Self::Input, Self::Enum, Self::Nonterm) -> NontermDecision<
        Self::Enum, Self::Nonterm, Self::Exit>;
    /// Given a reference to the input and the current terminal state, decide 
    /// what to do from the terminal statepoint. 
    fn on_terminal(&Self::Input, Self::Enum, Self::Term) -> TermDecision<
        Self::Enum, Self::Term, Self::Exit>;
}

pub struct SerialBranchNode<E, D, X> where
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    node: E,
    _exists_tuple: PhantomData<(D, X)>
}

impl<E, D, X> SerialBranchNode<E, D, X> where 
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    /// Create a new serial branch node for the given discriminant. 
    pub fn new(variant: E::Discriminant) -> SerialBranchNode<E, D, X> {
        SerialBranchNode {
            node: E::new(variant),
            _exists_tuple: PhantomData
        }
    }

    /// Wrap an existing enumerated node in a serial branch node. 
    pub fn from_existing(existing: E) -> SerialBranchNode<E, D, X> {
        SerialBranchNode {
            node: existing,
            _exists_tuple: PhantomData
        }
    }
}

impl<E, D, X> Default for SerialBranchNode<E, D, X> where 
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    fn default() -> SerialBranchNode<E, D, X> {
        SerialBranchNode::new(E::Discriminant::zero())
    }
}

impl<E, D, X> BehaviorTreeNode for SerialBranchNode<E, D, X> where
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    type Input = E::Input;
    type Nonterminal = NontermReturn<E::Discriminant, E::Nonterminal, E::Terminal>;
    type Terminal = X;

    #[inline]
    fn step(self, input: &E::Input) -> NodeResult<Self::Nonterminal, X, Self> {
        let discriminant = self.node.discriminant();
        match self.node.step(input) {
            NodeResult::Nonterminal(i, n) => {
                match D::on_nonterminal(input, discriminant, i) {
                    NontermDecision::Step(j) => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, j),
                        Self::from_existing(n)
                    ),
                    NontermDecision::Trans(e, j) => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, j),
                        Self::new(e)
                    ),
                    NontermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            NodeResult::Terminal(i) => {
                match D::on_terminal(input, discriminant, i) {
                    TermDecision::Trans(e, j) => NodeResult::Nonterminal(
                        NontermReturn::Terminal(discriminant, j),
                        Self::new(e)
                    ),
                    TermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use base_nodes::{PredicateWait, WaitCondition};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use serial_node::{Enumerable, EnumNode, SerialDecider, NontermDecision, TermDecision};

    #[derive(Copy, Clone)]
    enum PosNegEnum {
        Positive,
        Negative
    }

    impl Enumerable for PosNegEnum {
        fn zero() -> Self {
            PosNegEnum::Positive
        }

        fn successor(self) -> Option<Self> {
            match self {
                PosNegEnum::Positive => Option::Some(PosNegEnum::Negative),
                PosNegEnum::Negative => Option::None
            }
        }
    }

    #[derive(Copy, Clone, Default)]
    struct PositiveRepeater;

    impl WaitCondition for PositiveRepeater {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;
        fn do_end(input: &i64) -> Statepoint<i64, i64> {
            if *input >= 0 {
                Statepoint::Nonterminal(*input)
            } else {
                Statepoint::Terminal(*input)
            }
        }
    }

    #[derive(Copy, Clone, Default)]
    struct NegativeRepeater;

    impl WaitCondition for NegativeRepeater {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;
        fn do_end(input: &i64) -> Statepoint<i64, i64> {
            if *input >= 0 {
                Statepoint::Nonterminal(-*input)
            } else {
                Statepoint::Terminal(-*input)
            }
        }
    }

    enum MultiMachine {
        Positive(PredicateWait<PositiveRepeater>),
        Negative(PredicateWait<NegativeRepeater>)
    }

    impl BehaviorTreeNode for MultiMachine {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;

        fn step(self, input: &i64) -> NodeResult<i64, i64, Self> {
            match self {
                MultiMachine::Positive(n) => {
                    match n.step(input) {
                        NodeResult::Nonterminal(r, m) => NodeResult::Nonterminal(
                            r,
                            MultiMachine::Positive(m)
                        ),
                        NodeResult::Terminal(t) => NodeResult::Terminal(t)
                    }
                },
                MultiMachine::Negative(n) => {
                    match n.step(input) {
                        NodeResult::Nonterminal(r, m) => NodeResult::Nonterminal(
                            r,
                            MultiMachine::Negative(m)
                        ),
                        NodeResult::Terminal(t) => NodeResult::Terminal(t)
                    }
                }
            }
        }
    }
    
    impl EnumNode for MultiMachine {

        type Discriminant = PosNegEnum;

        fn new(thing: PosNegEnum) -> MultiMachine {
            match thing {
                PosNegEnum::Positive => MultiMachine::Positive(
                    PredicateWait::default()
                ),
                PosNegEnum::Negative => MultiMachine::Negative(
                    PredicateWait::default()
                )
            }
        }

        fn discriminant(&self) -> PosNegEnum {
            match self {
                MultiMachine::Positive(_) => PosNegEnum::Positive,
                MultiMachine::Negative(_) => PosNegEnum::Negative
            }
        }
    }

    struct Switcharound;

    impl SerialDecider for Switcharound {
        type Enum = PosNegEnum;
        type Input = i64;
        type Nonterm = i64;
        type Term = i64;
        type Exit = ();
        
        fn on_nonterminal(_i: &i64, _s: PosNegEnum, o: i64) -> NontermDecision<
            PosNegEnum, i64, ()> 
        {
            NontermDecision::Step(o)
        }

        fn on_terminal(_i: &i64, state: PosNegEnum, o: i64) -> TermDecision<
            PosNegEnum, i64, ()> 
        {
            match state {
                PosNegEnum::Positive => TermDecision::Trans(PosNegEnum::Negative, o),
                PosNegEnum::Negative => TermDecision::Trans(PosNegEnum::Positive, o)
            }
        }
    }

    #[test]
    fn serial_switcharound_test() {
        use serial_node::{SerialBranchNode, NontermReturn};
        let test_node = SerialBranchNode::<MultiMachine, Switcharound, _>
            ::new(PosNegEnum::Positive);
        let test_node_1 = match test_node.step(&5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Nonterminal(s, v) => {
                        match s {
                            PosNegEnum::Positive => (),
                            _ => unreachable!("Expected positive")
                        }
                        assert_eq!(v, 5);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_2 = match test_node_1.step(&-5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Terminal(s, v) => {
                        match s {
                            PosNegEnum::Positive => (),
                            _ => unreachable!("Expected positive")
                        }
                        assert_eq!(v, -5)
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition"),
                };
                n
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition")
        };
        let test_node_3 = match test_node_2.step(&5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Nonterminal(s, v) => {
                        match s {
                            PosNegEnum::Negative => (),
                            _ => unreachable!("Expected negative"),
                        }
                        assert_eq!(v, -5)
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_4 = match test_node_3.step(&-5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Terminal(s, v) => {
                        match s {
                            PosNegEnum::Negative => (),
                            _ => unreachable!("Expected negative"),
                        }
                        assert_eq!(v, 5)
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition"),
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        match test_node_4.step(&5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Nonterminal(s, v) => {
                        match s {
                            PosNegEnum::Positive => (),
                            _ => unreachable!("Expected positive")
                        }
                        assert_eq!(v, 5);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
    }

}