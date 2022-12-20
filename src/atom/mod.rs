use crate::{error::InvalidIndex, model_type::ModelInfo, CellModel, MsiModel, Transformation};
use std::{cmp::Ordering, ops::Add};

use na::Point3;

mod atom_builder;
pub mod visitor;

pub use atom_builder::AtomCollectionBuilder;
#[derive(Debug, Clone)]
/// Struct that defines an atom.
pub struct Atom<T: ModelInfo> {
    /// The symbol of the element.
    element_symbol: String,
    /// The atomic number of the element in periodic table.
    atomic_number: u8,
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

pub struct AtomView<'a, T: ModelInfo> {
    element_symbol: &'a str,
    atomic_number: &'a u8,
    xyz: &'a Point3<f64>,
    fractional_xyz: Option<&'a Point3<f64>>,
    atom_id: &'a u32,
    format_type: T,
}

impl<'a, T: ModelInfo> AtomView<'a, T> {
    pub fn xyz(&self) -> &Point3<f64> {
        self.xyz
    }

    pub fn element_symbol(&self) -> &str {
        self.element_symbol
    }

    pub fn atomic_number(&self) -> &u8 {
        self.atomic_number
    }

    pub fn fractional_xyz(&self) -> Option<&Point3<f64>> {
        self.fractional_xyz
    }

    pub fn atom_id(&self) -> &u32 {
        self.atom_id
    }
}

impl<'a, T: ModelInfo> From<AtomView<'a, T>> for Atom<T> {
    fn from(src: AtomView<'a, T>) -> Self {
        Self {
            element_symbol: src.element_symbol().into(),
            atomic_number: *src.atomic_number(),
            xyz: src.xyz().to_owned(),
            fractional_xyz: src.fractional_xyz().copied(),
            atom_id: *src.atom_id(),
            format_type: T::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Struct of `Atom` as data-driven design.
pub struct AtomCollection<T: ModelInfo> {
    element_symbols: Vec<String>,
    atomic_nums: Vec<u8>,
    xyz_coords: Vec<Point3<f64>>,
    fractional_xyz: Vec<Option<Point3<f64>>>,
    atom_ids: Vec<u32>,
    size: usize,
    format_type: T,
}

impl<T: ModelInfo> AtomCollection<T> {
    /// Update the `element_symbol` at the given index.
    /// # Errors
    /// This function will return an error if the index is out of bounds.
    pub fn update_symbol_at(&mut self, index: usize, new_symbol: &str) -> Result<(), InvalidIndex> {
        *self.element_symbols.get_mut(index).ok_or(InvalidIndex)? = new_symbol.into();
        Ok(())
    }
    /// Update the `element_id` at the given index.
    /// # Errors
    /// This function will return an error if the index is out of bounds.
    pub fn update_elm_id_at(&mut self, index: usize, new_elm_id: u8) -> Result<(), InvalidIndex> {
        *self.atomic_nums.get_mut(index).ok_or(InvalidIndex)? = new_elm_id;
        Ok(())
    }
    /// Update the `xyz` at the given index.
    /// # Errors
    /// This function will return an error if the index is out of bounds.
    pub fn update_xyz_at(
        &mut self,
        index: usize,
        new_xyz: Point3<f64>,
    ) -> Result<(), InvalidIndex> {
        *self.xyz_coords.get_mut(index).ok_or(InvalidIndex)? = new_xyz;
        Ok(())
    }
    /// Update the `fractional_xyz` at the given index.
    /// # Errors
    /// This function will return an error if the index is out of bounds.
    pub fn update_frac_xyz_at(
        &mut self,
        index: usize,
        new_frac: Option<Point3<f64>>,
    ) -> Result<(), InvalidIndex> {
        *self.fractional_xyz.get_mut(index).ok_or(InvalidIndex)? = new_frac;
        Ok(())
    }
    /// Update the `atom_id` at the given index.
    /// # Errors
    /// This function will return an error if the index is out of bounds.
    pub fn update_atom_id_at(
        &mut self,
        index: usize,
        new_atom_id: u32,
    ) -> Result<(), InvalidIndex> {
        *self.atom_ids.get_mut(index).ok_or(InvalidIndex)? = new_atom_id;
        Ok(())
    }
    /// Update the whole atom at the given index.
    /// # Errors
    /// This function will return an error if the index is out of bounds.
    pub fn update_atom_at(&mut self, index: usize, new_atom: Atom<T>) -> Result<(), InvalidIndex> {
        let Atom {
            element_symbol,
            atomic_number: element_id,
            xyz,
            fractional_xyz,
            atom_id,
            format_type: _,
        } = new_atom;
        self.update_symbol_at(index, &element_symbol)?;
        self.update_elm_id_at(index, element_id)?;
        self.update_xyz_at(index, xyz)?;
        self.update_frac_xyz_at(index, fractional_xyz)?;
        self.update_atom_id_at(index, atom_id)?;
        Ok(())
    }
    pub fn view_atom_at(&self, index: usize) -> Result<AtomView<T>, InvalidIndex> {
        let element_symbol = self
            .element_symbols
            .get(index)
            .ok_or(InvalidIndex)?
            .as_str();
        let element_id = self.atomic_nums.get(index).ok_or(InvalidIndex)?;
        let xyz = self.xyz_coords.get(index).ok_or(InvalidIndex)?;
        let fractional_xyz = self.fractional_xyz.get(index).ok_or(InvalidIndex)?.as_ref();
        let atom_id = self.atom_ids.get(index).ok_or(InvalidIndex)?;
        Ok(AtomView {
            element_symbol,
            atomic_number: element_id,
            xyz,
            fractional_xyz,
            atom_id,
            format_type: T::default(),
        })
    }

    pub fn element_symbols(&self) -> &[String] {
        self.element_symbols.as_ref()
    }

    pub fn atomic_nums(&self) -> &[u8] {
        self.atomic_nums.as_ref()
    }

    pub fn xyz_coords(&self) -> &[Point3<f64>] {
        self.xyz_coords.as_ref()
    }

    pub fn xyz_coords_mut(&mut self) -> &mut [Point3<f64>] {
        self.xyz_coords.as_mut()
    }

    pub fn fractional_xyz(&self) -> &[Option<Point3<f64>>] {
        self.fractional_xyz.as_ref()
    }

    pub fn fractional_xyz_mut(&mut self) -> &mut [Option<Point3<f64>>] {
        self.fractional_xyz.as_mut()
    }

    pub fn atom_ids(&self) -> &[u32] {
        self.atom_ids.as_ref()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T: ModelInfo> From<Vec<Atom<T>>> for AtomCollection<T> {
    fn from(src: Vec<Atom<T>>) -> Self {
        let atom_num = src.len();
        let mut output = AtomCollection {
            element_symbols: Vec::with_capacity(atom_num),
            atomic_nums: Vec::with_capacity(atom_num),
            xyz_coords: Vec::with_capacity(atom_num),
            fractional_xyz: Vec::with_capacity(atom_num),
            atom_ids: Vec::with_capacity(atom_num),
            size: atom_num,
            format_type: T::default(),
        };
        for atom in src.into_iter() {
            output.element_symbols.push(atom.element_symbol);
            output.atomic_nums.push(atom.atomic_number);
            output.xyz_coords.push(atom.xyz);
            output.fractional_xyz.push(atom.fractional_xyz);
            output.atom_ids.push(atom.atom_id);
        }
        output
    }
}

impl<'a, T: ModelInfo> From<&'a AtomCollection<T>> for Vec<AtomView<'a, T>> {
    fn from(src: &'a AtomCollection<T>) -> Self {
        (0..src.size)
            .into_iter()
            .map(|i| src.view_atom_at(i).unwrap())
            .collect()
    }
}

impl<T: ModelInfo> From<AtomCollection<T>> for Vec<Atom<T>> {
    fn from(src: AtomCollection<T>) -> Self {
        (0..src.size)
            .into_iter()
            .map(|i| -> Atom<T> {
                let view = src.view_atom_at(i).unwrap();
                view.into()
            })
            .collect()
    }
}

impl<T: ModelInfo> Add for AtomCollection<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let size_self = self.size;
        let size_rhs = rhs.size;
        let new_size = size_self + size_rhs;
        AtomCollection {
            element_symbols: vec![self.element_symbols, rhs.element_symbols].concat(),
            atomic_nums: vec![self.atomic_nums, rhs.atomic_nums].concat(),
            xyz_coords: vec![self.xyz_coords, rhs.xyz_coords].concat(),
            fractional_xyz: vec![self.fractional_xyz, rhs.fractional_xyz].concat(),
            atom_ids: vec![self.atom_ids, rhs.atom_ids].concat(),
            size: new_size,
            format_type: T::default(),
        }
    }
}

impl<T> Atom<T>
where
    T: ModelInfo,
{
    /// Creates a new [`Atom`].
    pub fn new(element_symbol: String, atomic_number: u8, xyz: Point3<f64>, atom_id: u32) -> Self {
        Self {
            element_symbol,
            atomic_number,
            xyz,
            fractional_xyz: None,
            atom_id,
            format_type: T::default(),
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
    pub fn element_id(&self) -> u8 {
        self.atomic_number
    }
    /// Sets the element id of this [`Atom<Format>`].
    pub fn set_element_id(&mut self, atomic_number: u8) {
        self.atomic_number = atomic_number;
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

impl<T> Transformation for AtomCollection<T>
where
    T: ModelInfo,
{
    fn rotate(&mut self, rotate_quatd: &na::UnitQuaternion<f64>) {
        self.xyz_coords
            .iter_mut()
            .for_each(|point| *point = rotate_quatd.transform_point(point))
    }

    fn translate(&mut self, translate_matrix: &na::Translation<f64, 3>) {
        self.xyz_coords
            .iter_mut()
            .for_each(|point| *point = translate_matrix.transform_point(point))
    }
}

impl From<AtomCollection<MsiModel>> for AtomCollection<CellModel> {
    fn from(src: AtomCollection<MsiModel>) -> Self {
        let AtomCollection {
            element_symbols,
            atomic_nums: element_ids,
            xyz_coords,
            fractional_xyz,
            atom_ids,
            size,
            format_type: _,
        } = src;
        Self {
            element_symbols,
            atomic_nums: element_ids,
            xyz_coords,
            fractional_xyz,
            atom_ids,
            size,
            format_type: CellModel::default(),
        }
    }
}

impl<T: ModelInfo> AsRef<AtomCollection<T>> for &AtomCollection<T> {
    fn as_ref(&self) -> &AtomCollection<T> {
        self
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
