use std::str::FromStr;

use nom::Err;

use crate::lattice::LatticeModel;
use crate::model_type::msi::MsiModel;

use self::state_machine::MsiParser;

extern crate nom;

mod state_machine;

impl FromStr for LatticeModel<MsiModel> {
    type Err = Err<&'static str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MsiParser::new(s).starts().analyze().build_lattice_model())
    }
}

#[cfg(test)]
#[test]
fn test_parser() {
    use std::fs::read_to_string;

    let file_content = read_to_string("SAC_GDY_V.msi").unwrap();
    let model: LatticeModel<MsiModel> = LatticeModel::from_str(&file_content).unwrap();
    println!("{:?}", model);
}
