#![cfg_attr(feature = "try_trait", feature(try_trait))]
#![cfg_attr(feature = "unsized_locals", feature(unsized_locals))]

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