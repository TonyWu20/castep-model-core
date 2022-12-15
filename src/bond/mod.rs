use crate::ModelInfo;

#[derive(Debug)]
pub enum BondType {
    Single,
    Double,
    Triple,
}

#[derive(Debug)]
pub struct Bond((u32, u32));

#[derive(Debug)]
pub struct Bonds<T: ModelInfo> {
    bonds: Vec<Bond>,
    bond_types: Vec<BondType>,
    format_type: T,
}
