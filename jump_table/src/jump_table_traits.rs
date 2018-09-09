/// Marker trait for jump tables. Automatically implemented by the jump table 
/// macro, but you can also instantiate this yourself. 
/// 
/// Because of limitations of the type system, T should really be a function 
/// pointer type, and code that does otherwise can and probably will break 
/// without warning as soon as Rust features permitting such trait bounds 
/// emerge and stabilize. 
pub trait JumpTable<T>: Into<T>{}

