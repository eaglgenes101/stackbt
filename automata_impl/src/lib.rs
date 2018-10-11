#![cfg_attr(feature = "unsized_locals", feature(unsized_locals))]

/// The Automaton trait and the FiniteStateAutomaton trait. 
pub mod automaton;
/// The RefStateMachine finite state machine implementation. 
pub mod ref_state_machine;
/// The InternalStateMachine finite state machine implementation. 
pub mod internal_state_machine;
/// The DualStateMachine finite state machine implementation. 
pub mod dual_state_machine;
/// Stateless automaton. 
pub mod stateless_mapper;
/// A pushdown automaton implementation based on finite state machines. 
pub mod pushdown_automaton;
/// Combinators for automata. 
pub mod automata_combinators;