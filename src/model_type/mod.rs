use std::fmt::Debug;

pub mod cell;
pub mod msi;

pub trait ModelInfo: Clone + Debug {}
