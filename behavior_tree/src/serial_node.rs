use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use std::marker::PhantomData;

/// Trait for an enumeration of nodes, all of which have the same input, 
/// nonterminals, and terminals. Each variant corresponds to a different 
/// value of Enumeration. 
pub trait EnumNode: BehaviorTreeNode {

    /// The type used to enumerate the variants of implementations of this 
    /// trait. std::mem::Discriminant works for comparing variants of an enum, 
    /// but not for enumerating or matching against them, hence this 
    /// associated type. 
    type Discriminant: Default + Copy + IntoIterator<Item=Self::Discriminant>;

    fn new(Self::Discriminant) -> Self;
    fn discriminant(&self) -> Self::Discriminant;
}

/// Enumeration of the possible decisions when the child node reaches a 
/// nonterminal state. 
pub enum NontermDecision<T, X> {
    Step,
    Trans(T),
    Exit(X)
}

/// Enumeration of the possible decisions when the child node reaches a 
/// terminal state. 
pub enum TermDecision<T, X> {
    Trans(T),
    Exit(X)
}

/// Return type of the SerialBranchNode. 
pub enum NontermReturn<E, N, T> {
    Nonterminal(E, N),
    Terminal(E, T)
}

/// Trait for the transition behavior of a SerialBranchNode. 
pub trait SerialDecider {
    type Enum;
    type Nonterm;
    type Term;
    type Exit;
    fn on_nonterminal(Self::Enum, &Self::Nonterm) -> NontermDecision<Self::Enum, Self::Exit>;
    fn on_terminal(Self::Enum, &Self::Term) -> TermDecision<Self::Enum, Self::Exit>;
}

pub struct SerialBranchNode<E, D, X> where
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    node: E,
    _exists_tuple: PhantomData<(D, X)>
}

impl<E, D, X> SerialBranchNode<E, D, X> where 
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    fn new(variant: E::Discriminant) -> SerialBranchNode<E, D, X> {
        SerialBranchNode {
            node: E::new(variant),
            _exists_tuple: PhantomData
        }
    }

    fn from_existing(existing: E) -> SerialBranchNode<E, D, X> {
        SerialBranchNode {
            node: existing,
            _exists_tuple: PhantomData
        }
    }
}

impl<E, D, X> Default for SerialBranchNode<E, D, X> where 
    E: EnumNode,
    E::Discriminant: Default, 
    D: SerialDecider<Enum=E::Discriminant, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    fn default() -> SerialBranchNode<E, D, X> {
        SerialBranchNode::new(E::Discriminant::default())
    }
}

impl<E, D, X> BehaviorTreeNode for SerialBranchNode<E, D, X> where
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Nonterm=E::Nonterminal, 
        Term=E::Terminal, Exit=X>
{
    type Input = E::Input;
    type Nonterminal = NontermReturn<E::Discriminant, E::Nonterminal, E::Terminal>;
    type Terminal = X;

    fn step(self, input: &E::Input) -> NodeResult<Self::Nonterminal, X, Self> {
        let discriminant = self.node.discriminant();
        match self.node.step(input) {
            NodeResult::Nonterminal(i, n) => {
                match D::on_nonterminal(discriminant, &i) {
                    NontermDecision::Step => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, i),
                        Self::from_existing(n)
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

#[cfg(test)]
mod tests {
    use base_nodes::{PredicateWait, WaitCondition};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use serial_node::{EnumNode, SerialDecider, NontermDecision, TermDecision};

    #[derive(Copy, Clone)]
    enum PosNegEnum {
        Positive,
        Negative
    }

    impl Default for PosNegEnum {
        fn default() -> PosNegEnum {
            PosNegEnum::Positive
        }
    }

    struct PosNegEnumIterator {
        f: Option<PosNegEnum>
    }

    impl Iterator for PosNegEnumIterator {
        type Item = PosNegEnum;
        fn next(&mut self) -> Option<PosNegEnum> {
            match self.f {
                Option::Some(PosNegEnum::Positive) => {
                    self.f = Option::Some(PosNegEnum::Negative);
                    Option::Some(PosNegEnum::Positive)
                },
                Option::Some(PosNegEnum::Negative) => {
                    self.f = Option::None;
                    Option::Some(PosNegEnum::Negative)
                },
                Option::None => Option::None
            }
        }
    }

    impl IntoIterator for PosNegEnum {
        type Item = PosNegEnum;
        type IntoIter = PosNegEnumIterator;

        fn into_iter(self) -> PosNegEnumIterator {
            PosNegEnumIterator {
                f: Option::Some(self)
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

        fn step(self, input: &i64) -> NodeResult<i64, i64, Self> where 
            Self: Sized 
        {
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
        type Nonterm = i64;
        type Term = i64;
        type Exit = ();
        fn on_nonterminal(_s: PosNegEnum, _o: &i64) -> NontermDecision<PosNegEnum, ()> {
            NontermDecision::Step
        }

        fn on_terminal(state: PosNegEnum, _o: &i64) -> TermDecision<PosNegEnum, ()> {
            match state {
                PosNegEnum::Positive => TermDecision::Trans(PosNegEnum::Negative),
                PosNegEnum::Negative => TermDecision::Trans(PosNegEnum::Positive)
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