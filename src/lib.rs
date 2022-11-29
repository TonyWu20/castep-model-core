#![allow(dead_code)]
pub mod atom;
pub mod builder_typestate;
pub mod error;
pub mod lattice;
pub mod model_type;
pub mod param_writer;
pub mod parser;
#[cfg(test)]
mod test;

extern crate castep_periodic_table as cpt;
extern crate nalgebra as na;

use na::UnitQuaternion;

pub use atom::Atom;
pub use lattice::LatticeModel;
pub use model_type::cell::CellModel;
pub use model_type::msi::MsiModel;
pub use model_type::ModelInfo;

/// Transformation for atoms and lattices.
pub trait Transformation {
    fn rotate(&mut self, rotate_quatd: &UnitQuaternion<f64>);
    fn translate(&mut self, translate_matrix: &na::Translation<f64, 3>);
}
