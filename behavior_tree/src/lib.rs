//! Behavior trees are a common means of composing behavior in game AI. They 
//! are related to state machines, and in many cases can be reduced to one, 
//! but are conceptually easier to reason about, especially when the 
//! complexity gets larger, and a state machine's state flows start to 
//! resemble goto spaghetti. 
//!
//! In this library, behavior trees are implemented in a very generalized 
//! manner, making them very versatile. At each step, a behavior tree node 
//! steps some, and stop at either an exit point, which abstracts over that 
//! node's terminal states, or a decision point, which abstracts over that 
//! node's nonterminal states. At a terminal state, only a transition to an 
//! entirely new behavior tree node is possible. However, at a decision point,
//! the parent behavior tree node can either decide to step the behavior tree
//! node as normal, or cause a transition to an entirely new behavior tree 
//! node, which abandons the original child node. The parent can also 
//! themselves transition to an exit point, which necessarily causes their 
//! children to be halted and dropped. 
//! 
//! Behavior trees here are also implemented in a zero-cost manner. The 
//! behavior tree is not an actual structure in memory, but a logical 
//! structure, which when translated to code, reduces to a type whose state 
//! transitions are very similar to the state machine one would write by hand,
//! but without the tedium or the copypaste errors. Only the memory needed to 
//! hold the state of the active nodes is used, and the conceptual tree-walk 
//! is translated to something more like a state machine transition in code, 
//! especially if optimizations are turned on. 

#![cfg_attr(feature = "try_trait", feature(try_trait))]

extern crate stackbt_automata_impl;

/// The base leaf nodes on which behavior trees are built. 
pub mod base_nodes;
/// The behavior tree node trait and associated enums. 
pub mod behavior_tree_node;
/// An automaton wrapper for behavior tree nodes. 
pub mod node_runner;
/// A serial running node controller. 
pub mod serial_node;
/// A parallel running node controller. 
pub mod parallel_node;
/// An assortment of mapping wrappers for behavior tree nodes. 
pub mod map_wrappers;
/// An assortment of controlling wrappers for behavior tree nodes. 
pub mod control_wrappers;
/// An assortment of serial and parallel node controllers. 
pub mod node_compositions;