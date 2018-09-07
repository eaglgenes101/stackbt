use std::marker::Sized;

pub trait JumpTableEntry {
    type FnType;
}

pub trait JumpTable {
    type FnType;
}

