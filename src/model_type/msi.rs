use std::fmt::Display;

use na::{UnitQuaternion, Vector, Vector3};

use crate::{
    atom::{Atom, AtomCollection, AtomCollectionBuilder, AtomView},
    bond::Bonds,
    builder_typestate::No,
    lattice::{LatticeModel, LatticeVectors},
    Transformation,
};

use super::{cell::CellModel, ModelInfo};

#[derive(Debug, Clone, Default)]
/// A unit struct to mark `msi` format
pub struct MsiModel;

impl ModelInfo for MsiModel {}

#[derive(Debug, Default)]
/// Struct representing the structure of a `msi` file.
pub struct MsiFile {
    cry_display: (u32, u32),
    periodic_type: u8,
    space_group: String,
    lattice_vectors: LatticeVectors<MsiModel>,
    cry_tolerance: f64,
    atoms: AtomCollection<MsiModel>,
    bonds: Option<Bonds<MsiModel>>,
}

/// Display trait for `Atom<MsiModel>`
impl Display for Atom<MsiModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"  ({item_id} Atom
    (A C ACL "{elm_id} {elm}")
    (A C Label "{elm}")
    (A D XYZ ({x:.12} {y:.12} {z:.12}))
    (A I Id {atom_id})
  )
"#,
            item_id = self.atom_id() + 1,
            elm_id = self.element_id(),
            elm = self.element_symbol(),
            x = self.xyz().x,
            y = self.xyz().y,
            z = self.xyz().z,
            atom_id = self.atom_id(),
        )
    }
}

/// Display trait for `LatticeVectors<MsiModel>`
impl Display for LatticeVectors<MsiModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vector_a = self.vectors().column(0);
        let vector_b = self.vectors().column(1);
        let vector_c = self.vectors().column(2);
        let vector_a_line = format!(
            "  (A D A3 ({:.12} {:.12} {:.12}))\n",
            vector_a.x, vector_a.y, vector_a.z
        );
        let vector_b_line = format!(
            "  (A D B3 ({:.12} {:.12} {:.12}))\n",
            vector_b.x, vector_b.y, vector_b.z
        );
        let vector_c_line = format!(
            "  (A D C3 ({:.12} {:.12} {:.12}))",
            vector_c.x, vector_c.y, vector_c.z
        );
        writeln!(f, "{vector_a_line}{vector_b_line}{vector_c_line}")
    }
}

impl<T> From<T> for AtomCollection<MsiModel>
where
    T: AsRef<AtomCollection<CellModel>>,
{
    fn from(src: T) -> Self {
        let builder = AtomCollectionBuilder::<MsiModel, No>::new(src.as_ref().size());
        builder
            .with_element_symbols(src.as_ref().element_symbols())
            .unwrap()
            .with_atomic_nums(src.as_ref().atomic_nums())
            .unwrap()
            .with_xyz_coords(src.as_ref().xyz_coords())
            .unwrap()
            .with_fractional_xyz(src.as_ref().fractional_xyz())
            .unwrap()
            .with_atom_ids(src.as_ref().atom_ids())
            .unwrap()
            .finish()
            .unwrap()
            .build()
    }
}

impl<T> From<T> for LatticeModel<MsiModel>
where
    T: AsRef<LatticeModel<CellModel>>,
{
    fn from(cell_model: T) -> Self {
        let new_lat_vec = LatticeVectors::new(
            cell_model
                .as_ref()
                .lattice_vectors()
                .unwrap()
                .vectors()
                .to_owned(),
        );
        // Convert the SoA to AoS for easier sorting.
        let msi_atoms: AtomCollection<MsiModel> = cell_model.as_ref().atoms().into();
        let mut msi_atom_array: Vec<Atom<MsiModel>> = msi_atoms.into();
        msi_atom_array.sort_by_key(|a| a.atom_id());
        // Convert AoS back to SoA.
        let msi_atom_collection: AtomCollection<MsiModel> = msi_atom_array.into();
        let mut msi_model = Self::new(Some(new_lat_vec), msi_atom_collection);
        let y_axis: Vector3<f64> = Vector::y();
        let b_vec = cell_model
            .as_ref()
            .lattice_vectors()
            .unwrap()
            .vectors()
            .column(1);
        let b_to_y_angle = b_vec.angle(&y_axis);
        if b_to_y_angle != 0.0 {
            let rot_axis = b_vec.cross(&y_axis).normalize();
            let rot_quatd: UnitQuaternion<f64> = UnitQuaternion::new(rot_axis * b_to_y_angle);
            msi_model.rotate(&rot_quatd);
        }
        msi_model
    }
}

impl LatticeModel<MsiModel> {
    pub fn msi_export(&self) -> String {
        if let Some(lattice_vectors) = self.lattice_vectors() {
            let headers_vectors: Vec<String> = vec![
                "# MSI CERIUS2 DataModel File Version 4 0\n".to_string(),
                "(1 Model\n".to_string(),
                "  (A I CRY/DISPLAY (192 256))\n".to_string(),
                format!("  (A I PeriodicType {})\n", self.settings().periodic_type()),
                format!("  (A C SpaceGroup \"{}\")\n", self.settings().space_group()),
                format!("{}", lattice_vectors),
                format!(
                    "  (A D CRY/TOLERANCE {})\n",
                    self.settings().cry_tolerance()
                ),
            ];
            format!("{}{})", headers_vectors.concat(), self.atoms())
        } else {
            let headers = "# MSI CERIUS2 DataModel File Version 4 0\n(1 Model\n";
            format!("{}{})", headers, self.atoms())
        }
    }
}

impl<'a> Display for AtomView<'a, MsiModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"  ({item_id} Atom
    (A C ACL "{elm_id} {elm}")
    (A C Label "{elm}")
    (A D XYZ ({x:.12} {y:.12} {z:.12}))
    (A I Id {atom_id})
  )
"#,
            item_id = self.atom_id() + 1,
            elm_id = self.element_id(),
            elm = self.element_symbol(),
            x = self.xyz().x,
            y = self.xyz().y,
            z = self.xyz().z,
            atom_id = self.atom_id(),
        )
    }
}

impl Display for AtomCollection<MsiModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msi_atom_strings: Vec<String> = (0..self.size())
            .into_iter()
            .map(|i| {
                let atom_view = self.view_atom_at(i).unwrap();
                format!("{}", atom_view)
            })
            .collect();
        write!(f, "{}", msi_atom_strings.concat())
    }
}
