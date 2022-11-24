use crate::{model_type::ModelInfo, Transformation};
use std::cmp::Ordering;

use na::Point3;
/// Struct that defines an atom.
#[derive(Debug, Clone)]
pub struct Atom<T: ModelInfo> {
    /// The symbol of the element.
    element_symbol: String,
    /// The atomic number of the element in periodic table.
    element_id: u32,
    /// The cartesian coordinate of the atom.
    xyz: Point3<f64>,
    /// The fractional coordinate of the atom in a lattice.
    /// Optional since it relies on lattice vectors.
    fractional_xyz: Option<Point3<f64>>,
    /// The id of the atom in the parsed model.
    atom_id: u32,
    /// Format type
    format_type: T,
}

impl<T> Atom<T>
where
    T: ModelInfo,
{
    /// Creates a new [`Atom`].
    pub fn new(
        element_symbol: String,
        element_id: u32,
        xyz: Point3<f64>,
        atom_id: u32,
        model_type: T,
    ) -> Self {
        Self {
            element_symbol,
            element_id,
            xyz,
            fractional_xyz: None,
            atom_id,
            format_type: model_type,
        }
    }

    /// Returns a reference to the element symbol of this [`Atom<Format>`].
    pub fn element_symbol(&self) -> &str {
        self.element_symbol.as_ref()
    }
    /// Sets the element symbol of this [`Atom<Format>`].
    pub fn set_element_symbol(&mut self, element_symbol: String) {
        self.element_symbol = element_symbol;
    }

    /// Returns the element id of this [`Atom<Format>`].
    pub fn element_id(&self) -> u32 {
        self.element_id
    }
    /// Sets the element id of this [`Atom<Format>`].
    pub fn set_element_id(&mut self, element_id: u32) {
        self.element_id = element_id;
    }

    /// Returns a reference to the xyz of this [`Atom<Format>`].
    pub fn xyz(&self) -> &Point3<f64> {
        &self.xyz
    }

    /// Sets the xyz of this [`Atom<Format>`].
    pub fn set_xyz(&mut self, xyz: Point3<f64>) {
        self.xyz = xyz;
    }

    /// Returns the atom id of this [`Atom<Format>`].
    pub fn atom_id(&self) -> u32 {
        self.atom_id
    }
    /// Sets the atom id of this [`Atom<Format>`].
    pub fn set_atom_id(&mut self, atom_id: u32) {
        self.atom_id = atom_id;
    }

    /// Returns a reference to the format of this [`Atom<Format>`].
    pub fn model_type(&self) -> &T {
        &self.format_type
    }
    /// Sets the format of this [`Atom<Format>`].
    pub fn set_model_type(&mut self, model_type: T) {
        self.format_type = model_type;
    }

    pub fn fractional_xyz(&self) -> Option<&Point3<f64>> {
        self.fractional_xyz.as_ref()
    }

    pub fn set_fractional_xyz(&mut self, fractional_xyz: Option<Point3<f64>>) {
        self.fractional_xyz = fractional_xyz;
    }
}

// impl Export for Vec<Atom> {
//     fn format_output(&self) -> String {
//         let atom_strings: Vec<String> = self.iter().map(|atom| atom.format_output()).collect();
//         atom_strings.concat()
//     }
// }

impl<T> Transformation for Atom<T>
where
    T: ModelInfo,
{
    fn rotate(&mut self, rotate_quatd: &na::UnitQuaternion<f64>) {
        self.set_xyz(rotate_quatd.transform_point(self.xyz()))
    }

    fn translate(&mut self, translate_matrix: &na::Translation<f64, 3>) {
        self.set_xyz(translate_matrix.transform_point(self.xyz()))
    }
}

impl<T> Ord for Atom<T>
where
    T: ModelInfo,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.atom_id.cmp(&other.atom_id)
    }
}

impl<T> PartialOrd for Atom<T>
where
    T: ModelInfo,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for Atom<T>
where
    T: ModelInfo,
{
    fn eq(&self, other: &Self) -> bool {
        self.atom_id == other.atom_id
    }
}

impl<T> Eq for Atom<T> where T: ModelInfo {}
