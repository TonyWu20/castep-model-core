use std::{cmp::Ordering, fmt::Display, marker::PhantomData};

use nalgebra::Point3;

use crate::{builder_typestate::No, ModelInfo};

use super::AtomCollection;

pub trait BuildState {}
pub struct Ready {}
impl BuildState for Ready {}
impl BuildState for No {}

pub struct AtomCollectionBuilder<T, S>
where
    T: ModelInfo,
    S: BuildState,
{
    element_symbols: Option<Vec<String>>,
    atomic_nums: Option<Vec<u8>>,
    xyz_coords: Option<Vec<Point3<f64>>>,
    fractional_xyz: Option<Vec<Option<Point3<f64>>>>,
    atom_ids: Option<Vec<u32>>,
    size: usize,
    format_type: T,
    state: PhantomData<S>,
}

#[derive(Debug)]
pub enum AtomCollectionBuildingError {
    InconsistentSize { curr: usize, expected: usize },
    MissingField { missed: String },
}

impl Display for AtomCollectionBuildingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomCollectionBuildingError::InconsistentSize { curr, expected } => write!(
                f,
                "InconsistentSize: current: {} vs expected: {}",
                curr, expected
            ),
            AtomCollectionBuildingError::MissingField { missed: miss } => {
                write!(f, "MissingField: {} value is `None`.", miss)
            }
        }
    }
}

impl<T: ModelInfo, S: BuildState> AtomCollectionBuilder<T, S> {
    pub fn new(size: usize) -> AtomCollectionBuilder<T, No> {
        AtomCollectionBuilder {
            element_symbols: None,
            atomic_nums: None,
            xyz_coords: None,
            fractional_xyz: None,
            atom_ids: None,
            size,
            format_type: T::default(),
            state: PhantomData,
        }
    }
    /// Supply the `element_symbols` for an `AtomCollection`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `element_ids` has a different vector size
    /// with the builder's given size.
    pub fn with_element_symbols(
        mut self,
        element_symbols: &[String],
    ) -> Result<Self, AtomCollectionBuildingError> {
        match element_symbols.len().cmp(&self.size) {
            Ordering::Equal => {
                self.element_symbols = Some(element_symbols.to_vec());
                Ok(self)
            }
            _ => Err(AtomCollectionBuildingError::InconsistentSize {
                curr: element_symbols.len(),
                expected: self.size,
            }),
        }
    }
    /// Supply the `atomic_nums` for an `AtomCollection`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `element_ids` has a different vector size
    /// with the builder's given size.
    pub fn with_atomic_nums(
        mut self,
        atomic_nums: &[u8],
    ) -> Result<Self, AtomCollectionBuildingError> {
        match atomic_nums.len().cmp(&self.size) {
            Ordering::Equal => {
                self.atomic_nums = Some(atomic_nums.to_vec());
                Ok(self)
            }
            _ => Err(AtomCollectionBuildingError::InconsistentSize {
                curr: atomic_nums.len(),
                expected: self.size,
            }),
        }
    }
    /// Supply the `xyz_coords` for an `AtomCollection`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `xyz_coords` has a different vector size
    /// with the builder's given size.
    pub fn with_xyz_coords(
        mut self,
        xyz_coords: &[Point3<f64>],
    ) -> Result<Self, AtomCollectionBuildingError> {
        match xyz_coords.len().cmp(&self.size) {
            Ordering::Equal => {
                self.xyz_coords = Some(xyz_coords.to_vec());
                Ok(self)
            }
            _ => Err(AtomCollectionBuildingError::InconsistentSize {
                curr: xyz_coords.len(),
                expected: self.size,
            }),
        }
    }
    /// Supply the `fractional_xyz` for an `AtomCollection`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `fractional_xyz` has a different vector size
    /// with the builder's given size.
    pub fn with_fractional_xyz(
        mut self,
        fractional_xyz: &[Option<Point3<f64>>],
    ) -> Result<Self, AtomCollectionBuildingError> {
        match fractional_xyz.len().cmp(&self.size) {
            Ordering::Equal => {
                self.fractional_xyz = Some(fractional_xyz.to_vec());
                Ok(self)
            }
            _ => Err(AtomCollectionBuildingError::InconsistentSize {
                curr: fractional_xyz.len(),
                expected: self.size,
            }),
        }
    }
    /// Supply the `atom_ids` for an `AtomCollection`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `atom_ids` has a different vector size
    /// with the builder's given size.
    pub fn with_atom_ids(mut self, atom_ids: &[u32]) -> Result<Self, AtomCollectionBuildingError> {
        match atom_ids.len().cmp(&self.size) {
            Ordering::Equal => {
                self.atom_ids = Some(atom_ids.to_vec());
                Ok(self)
            }
            _ => Err(AtomCollectionBuildingError::InconsistentSize {
                curr: atom_ids.len(),
                expected: self.size,
            }),
        }
    }
    pub fn finish(self) -> Result<AtomCollectionBuilder<T, Ready>, AtomCollectionBuildingError> {
        if self.atomic_nums.is_none() {
            return Err(AtomCollectionBuildingError::MissingField {
                missed: "atomic_nums".into(),
            });
        }
        if self.element_symbols.is_none() {
            return Err(AtomCollectionBuildingError::MissingField {
                missed: "element_symbols".into(),
            });
        }
        if self.xyz_coords.is_none() {
            return Err(AtomCollectionBuildingError::MissingField {
                missed: "xyz_coords".into(),
            });
        }
        if self.fractional_xyz.is_none() {
            return Err(AtomCollectionBuildingError::MissingField {
                missed: "fractional_xyz".into(),
            });
        }
        if self.atom_ids.is_none() {
            return Err(AtomCollectionBuildingError::MissingField {
                missed: "atom_ids".into(),
            });
        }
        let Self {
            element_symbols,
            atomic_nums,
            xyz_coords,
            fractional_xyz,
            atom_ids,
            size,
            format_type,
            state: _,
        } = self;
        Ok(AtomCollectionBuilder {
            element_symbols,
            atomic_nums,
            xyz_coords,
            fractional_xyz,
            atom_ids,
            size,
            format_type,
            state: PhantomData,
        })
    }
}

impl<T: ModelInfo> AtomCollectionBuilder<T, Ready> {
    pub fn build(self) -> AtomCollection<T> {
        AtomCollection {
            element_symbols: self.element_symbols.unwrap(),
            atomic_nums: self.atomic_nums.unwrap(),
            xyz_coords: self.xyz_coords.unwrap(),
            fractional_xyz: self.fractional_xyz.unwrap(),
            atom_ids: self.atom_ids.unwrap(),
            size: self.size,
            format_type: T::default(),
        }
    }
}
