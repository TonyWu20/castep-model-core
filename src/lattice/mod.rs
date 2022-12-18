use std::{collections::HashSet, ops::Add};

use na::{Matrix3, Vector3};

use crate::{
    atom::{AtomCollection, AtomView},
    error::InvalidIndex,
    model_type::{ModelInfo, Settings},
    Transformation,
};

#[derive(Debug, Clone)]
pub struct LatticeModel<T: ModelInfo> {
    lattice_vectors: Option<LatticeVectors<T>>,
    atoms: AtomCollection<T>,
    settings: Settings<T>,
}

impl<T> LatticeModel<T>
where
    T: ModelInfo,
{
    pub fn new(
        lattice_vectors: Option<LatticeVectors<T>>,
        atoms: AtomCollection<T>,
        settings: Settings<T>,
    ) -> Self {
        Self {
            lattice_vectors,
            atoms,
            settings,
        }
    }

    /// Returns the lattice vectors of this [`LatticeModel<T>`].
    pub fn lattice_vectors(&self) -> Option<&LatticeVectors<T>> {
        self.lattice_vectors.as_ref()
    }
    pub fn atoms(&self) -> &AtomCollection<T> {
        &self.atoms
    }

    pub fn atoms_mut(&mut self) -> &mut AtomCollection<T> {
        &mut self.atoms
    }
    pub fn view_atom_by_id(&self, atom_id: u32) -> Result<AtomView<T>, InvalidIndex> {
        self.atoms().view_atom_at((atom_id - 1) as usize)
    }
    // pub fn get_mut_atom_by_id(&mut self, atom_id: u32) -> Result<&mut Atom<T>, InvalidIndex> {
    //     self.atoms_mut()
    //         .get_mut(atom_id as usize - 1)
    //         .ok_or(InvalidIndex)
    // }
    pub fn get_vector_ab(&self, a_id: u32, b_id: u32) -> Result<Vector3<f64>, InvalidIndex> {
        if a_id != b_id {
            let atom_a_xyz = self.view_atom_by_id(a_id)?.xyz().to_owned();
            let atom_b_xyz = self.view_atom_by_id(b_id)?.xyz().to_owned();
            Ok(atom_b_xyz - atom_a_xyz)
        } else {
            Err(InvalidIndex)
        }
    }
    pub fn list_element(&self) -> Vec<String> {
        let mut elm_list: Vec<(String, u8)> = vec![];
        elm_list.extend(
            self.atoms()
                .element_symbols()
                .iter()
                .zip(self.atoms.atomic_nums().iter())
                .map(|(sym, id)| (sym.to_string(), *id))
                .collect::<Vec<(String, u8)>>()
                .drain(..)
                .collect::<HashSet<(String, u8)>>()
                .into_iter(),
        );
        elm_list.sort_unstable_by(|a, b| {
            let (_, id_a) = a;
            let (_, id_b) = b;
            id_a.cmp(id_b)
        });
        elm_list
            .iter()
            .map(|(name, _)| name.to_string())
            .collect::<Vec<String>>()
    }

    pub fn lattice_vectors_mut(&mut self) -> &mut Option<LatticeVectors<T>> {
        &mut self.lattice_vectors
    }

    pub fn settings(&self) -> &Settings<T> {
        &self.settings
    }
}

impl<T: ModelInfo> AsRef<LatticeModel<T>> for LatticeModel<T> {
    fn as_ref(&self) -> &LatticeModel<T> {
        self
    }
}

impl<T: ModelInfo> AsMut<LatticeModel<T>> for LatticeModel<T> {
    fn as_mut(&mut self) -> &mut LatticeModel<T> {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LatticeVectors<T: ModelInfo> {
    vectors: Matrix3<f64>,
    model_type: T,
}

impl<T> Transformation for LatticeVectors<T>
where
    T: ModelInfo,
{
    fn rotate(&mut self, rotate_quatd: &na::UnitQuaternion<f64>) {
        let new_vectors = rotate_quatd.to_rotation_matrix() * self.vectors();
        self.set_vectors(new_vectors)
    }

    fn translate(&mut self, _translate_matrix: &na::Translation<f64, 3>) {}
}

impl<T> LatticeVectors<T>
where
    T: ModelInfo,
{
    pub fn new(vectors: Matrix3<f64>) -> Self {
        Self {
            vectors,
            model_type: T::default(),
        }
    }

    pub fn fractional_coord_matrix(&self) -> Matrix3<f64> {
        let lattice_vectors = self.vectors();
        let vec_a = lattice_vectors.column(0);
        let vec_b = lattice_vectors.column(1);
        let vec_c = lattice_vectors.column(2);
        let len_a: f64 = vec_a.norm();
        let len_b: f64 = vec_b.norm();
        let len_c: f64 = vec_c.norm();
        let (alpha, beta, gamma) = (
            vec_b.angle(&vec_c),
            vec_a.angle(&vec_c),
            vec_a.angle(&vec_b),
        );
        let vol = vec_a.dot(&vec_b.cross(&vec_c));
        let to_cart = Matrix3::new(
            len_a,
            len_b * gamma.cos(),
            len_c * beta.cos(),
            0.0,
            len_b * gamma.sin(),
            len_c * (alpha.cos() - beta.cos() * gamma.cos()) / gamma.sin(),
            0.0,
            0.0,
            vol / (len_a * len_b * gamma.sin()),
        );
        to_cart.try_inverse().unwrap()
    }

    pub fn vectors(&self) -> &Matrix3<f64> {
        &self.vectors
    }

    pub fn set_vectors(&mut self, vectors: Matrix3<f64>) {
        self.vectors = vectors;
    }
}

impl<T> Transformation for LatticeModel<T>
where
    T: ModelInfo,
{
    fn rotate(&mut self, rotate_quatd: &na::UnitQuaternion<f64>) {
        self.atoms_mut().rotate(rotate_quatd);
        if let Some(lattice_vectors) = self.lattice_vectors_mut() {
            lattice_vectors.rotate(rotate_quatd);
        }
    }

    fn translate(&mut self, translate_matrix: &na::Translation<f64, 3>) {
        self.atoms_mut().translate(translate_matrix);
    }
}

/// Implementation of `Add` for merging `LatticeModel<T>`
/// Both `self` and `rhs` will be consumed.
impl<T> Add for LatticeModel<T>
where
    T: ModelInfo,
{
    type Output = LatticeModel<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let new_atoms = self.atoms + rhs.atoms;
        let Self {
            lattice_vectors,
            atoms: _,
            settings,
        } = self;
        Self {
            lattice_vectors,
            atoms: new_atoms,
            settings,
        }
    }
}
