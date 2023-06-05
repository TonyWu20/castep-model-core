use std::fmt::Display;

use crate::{
    atom::{visitor::VisitCollection, AtomCollection},
    lattice::{LatticeModel, LatticeVectors},
    param_writer::ms_aux_files::{KptAux, TrjAux},
    Transformation,
};

use cpt::{data::ELEMENT_TABLE, element::LookupElement};
use na::{UnitQuaternion, Vector, Vector3};
use nalgebra::Point3;

use super::{msi::MsiModel, BandStructureExport, DefaultExport, ModelInfo, Settings};

#[derive(Debug, Clone, Default)]
/// A unit struct to mark `cell`format.
pub struct CellModel;

impl ModelInfo for CellModel {}

/// Methods for `CellFormat`
impl CellModel {
    fn write_block(block: (String, String)) -> String {
        let (block_name, content) = block;
        format!(
            "%BLOCK {}\n{}%ENDBLOCK {}\n\n",
            block_name, content, block_name
        )
    }
}

/// Transition from `LatticeModel<MsiFormat>` to `LatticeModel<CellFormat>`
impl<T> From<T> for LatticeModel<CellModel>
where
    T: AsRef<LatticeModel<MsiModel>> + AsMut<LatticeModel<MsiModel>>,
{
    fn from(mut msi_model: T) -> Self {
        let x_axis: Vector3<f64> = Vector::x();
        let a_vec = msi_model
            .as_ref()
            .lattice_vectors()
            .unwrap()
            .vectors()
            .column(0);
        let a_to_x_angle = a_vec.angle(&x_axis);
        if a_to_x_angle != 0.0 {
            let rot_axis = a_vec.cross(&x_axis).normalize();
            let rot_quatd: UnitQuaternion<f64> = UnitQuaternion::new(rot_axis * a_to_x_angle);
            msi_model.as_mut().rotate(&rot_quatd);
        }
        let new_lat_vec = LatticeVectors::new(
            msi_model
                .as_ref()
                .lattice_vectors()
                .unwrap()
                .vectors()
                .to_owned(),
        );
        let fractional_coord_matrix = msi_model
            .as_ref()
            .lattice_vectors()
            .unwrap()
            .fractional_coord_matrix();
        let mut cell_atoms: AtomCollection<CellModel> = msi_model.as_ref().atoms().clone().into();
        let frac_coords: Vec<Point3<f64>> = cell_atoms
            .xyz_coords()
            .iter()
            .map(|xyz| fractional_coord_matrix * xyz)
            .collect();
        cell_atoms
            .fractional_xyz_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, f_xyz)| {
                *f_xyz = Some(*frac_coords.get(i).unwrap());
            });
        Self::new(Some(new_lat_vec), cell_atoms, Settings::default())
    }
}

/// Methods only for `LatticeModel<CellFormat>`
impl LatticeModel<CellModel> {
    /// Formatted *fractional coordinates*
    fn positions_str(&self) -> String {
        let coords = format!("{}", self.atoms());
        CellModel::write_block(("POSITIONS_FRAC".to_string(), coords))
    }
    /**
    This data block contains a list of k-points at which the Brillouin zone will be sampled during a self consistent calculation to find the electronic ground state, along with the associated weights
    # Format:
    ```
    %BLOCK KPOINTS_LIST
        R1i     R1j     R1k     R1w
        R2i     R2j     R2k     R2w
        .
        .
        .
    %ENDBLOCK KPOINTS_LIST
    ```
    The first three entries on a line are the fractional positions of the k-point relative to the reciprocal space lattice vectors.
    The final entry on a line is the weight of the k-point relative to the others specified. The sum of the weights must be equal to 1.
    */
    fn kpoints_list_str(&self) -> String {
        let kpoints_list: Vec<String> = self
            .settings()
            .kpoints_list()
            .iter()
            .map(|kpoint| {
                let [x, y, z, weight] = kpoint;
                format!("{:20.16}{:20.16}{:20.16}{:20.16}\n", x, y, z, weight)
            })
            .collect();
        CellModel::write_block(("KPOINTS_LIST".to_string(), kpoints_list.concat()))
    }
    /// For output in `.cell` for `BandStructure` calculation.
    fn bs_kpoints_list_str(&self) -> String {
        let kpoints_list: Vec<String> = self
            .settings()
            .kpoints_list()
            .iter()
            .map(|kpoint| {
                let [x, y, z, weight] = kpoint;
                format!("{:20.16}{:20.16}{:20.16}{:20.16}\n", x, y, z, weight)
            })
            .collect();
        CellModel::write_block(("BS_KPOINTS_LIST".to_string(), kpoints_list.concat()))
    }
    /// No constraints. Future: adapt to settings
    fn ionic_constraints(&self) -> String {
        CellModel::write_block(("IONIC_CONSTRAINTS".to_string(), "".to_string()))
    }
    /// Miscellaneous parameters
    fn misc_options(&self) -> String {
        let fix = format!(
            "FIX_ALL_CELL : {}\n\nFIX_COM : {}\n{}",
            self.settings().fix_all_cell(),
            self.settings().fix_com(),
            self.ionic_constraints()
        );
        let [ex, ey, ez] = self.settings().external_efield();
        let external_efield = CellModel::write_block((
            "EXTERNAL_EFIELD".to_string(),
            format!("{:16.10}{:16.10}{:16.10}\n", ex, ey, ez),
        ));
        let [rxx, rxy, rxz, ryy, ryz, rzz] = self.settings().external_pressure();
        let external_pressure = CellModel::write_block((
            "EXTERNAL_PRESSURE".to_string(),
            format!(
                r#"{:16.10}{:16.10}{:16.10}
                {:16.10}{:16.10}
                                {:16.10}
"#,
                rxx, rxy, rxz, ryy, ryz, rzz
            ),
        ));
        let mut misc = String::new();
        misc.push_str(&fix);
        misc.push_str(&external_efield);
        misc.push_str(&external_pressure);
        misc
    }
    /**
    Species and mass table
    # Example:
    ```
    %BLOCK SPECIES_MASS
           O     15.9989995956
          Al     26.9820003510
          Ti     47.9000015259
          Cs    132.9049987793
    %ENDBLOCK SPECIES_MASS
    ```
    */
    fn species_mass(&self) -> String {
        let element_list = self.element_set();
        let mass_strings: Vec<String> = element_list
            .iter()
            .map(|elm| -> String {
                let mass: f64 = ELEMENT_TABLE.get_by_symbol(elm).unwrap().mass();
                format!("{:>8}{:17.10}\n", elm, mass)
            })
            .collect();
        CellModel::write_block(("SPECIES_MASS".to_string(), mass_strings.concat()))
    }
    /**
    Species and potential table
    # Example:
    ```
    %BLOCK SPECIES_POT
       O  O_00.usp
      Al  Al_00.usp
      Ti  Ti_00.uspcc
      Cs  Cs_00.usp
    %ENDBLOCK SPECIES_POT
    ```
    */
    fn species_pot_str(&self) -> String {
        let element_list = self.element_set();
        let pot_strings: Vec<String> = element_list
            .iter()
            .map(|elm| {
                let pot_file = ELEMENT_TABLE.get_by_symbol(elm).unwrap().potential();
                format!("{:>8}  {}\n", elm, pot_file)
            })
            .collect();
        CellModel::write_block(("SPECIES_POT".to_string(), pot_strings.concat()))
    }
    /**
    This data block defines the size of the LCAO basis set used for population analysis.
    # Example:
    ```
    %BLOCK SPECIES_LCAO_STATES
       O         2
      Al         2
      Ti         3
      Cs         4
    %ENDBLOCK SPECIES_LCAO_STATES
    ```
    */
    fn species_lcao_str(&self) -> String {
        let element_list = self.element_set();
        let lcao_strings: Vec<String> = element_list
            .iter()
            .map(|elm| {
                let lcao_state = ELEMENT_TABLE.get_by_symbol(elm).unwrap().lcao();
                format!("{:>8}{:9}\n", elm, lcao_state)
            })
            .collect();
        CellModel::write_block(("SPECIES_LCAO_STATES".to_string(), lcao_strings.concat()))
    }
    /// Build `KptAux` struct
    pub fn build_kptaux(&self) -> KptAux {
        KptAux::new(
            self.settings().kpoints_list().to_vec(),
            self.settings().kpoints_grid(),
            self.settings().kpoints_mp_spacing(),
            self.settings().kpoints_mp_offset(),
        )
    }
    /// Build `TrjAux` struct
    pub fn build_trjaux(&self) -> TrjAux {
        TrjAux::new(self.atoms().atom_ids().to_vec())
    }
}

impl Display for AtomCollection<CellModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all_positions_str: Vec<String> = self
            .element_symbols()
            .iter()
            .zip(self.fractional_xyz().iter())
            .map(|(symbol, frac_xyz)| -> String {
                let spin = ELEMENT_TABLE.get_by_symbol(symbol).unwrap().spin();
                let spin_str = if spin > 0 {
                    format!(" SPIN={:14.10}", spin)
                } else {
                    "".into()
                };
                let frac_xyz = frac_xyz.unwrap();
                format!(
                    "{:>3}{:20.16}{:20.16}{:20.16}{spin_str}",
                    symbol, frac_xyz.x, frac_xyz.y, frac_xyz.z
                )
            })
            .collect();
        let joined_positions_str = all_positions_str.join("\n");
        write!(f, "{}\n", joined_positions_str)
    }
}

impl Display for LatticeVectors<CellModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_vector: Vec<String> = self
            .vectors()
            .column_iter()
            .map(|col| format!("{:24.18}{:24.18}{:24.18}\n", col.x, col.y, col.z))
            .collect();
        let formatted_vector = formatted_vector.concat();
        let output = CellModel::write_block(("LATTICE_CART".to_string(), formatted_vector));
        write!(f, "{}", &output)
    }
}

impl<T> DefaultExport<CellModel> for T
where
    T: AsRef<LatticeModel<CellModel>>,
{
    fn export(&self) -> String {
        let lattice_vector_string = format!("{}", self.as_ref().lattice_vectors().unwrap());
        let cell_text = vec![
            lattice_vector_string,
            self.as_ref().positions_str(),
            self.as_ref().kpoints_list_str(),
            self.as_ref().misc_options(),
            self.as_ref().species_mass(),
            self.as_ref().species_pot_str(),
            self.as_ref().species_lcao_str(),
        ];
        cell_text.concat()
    }
}

impl<T> BandStructureExport<CellModel> for T
where
    T: AsRef<LatticeModel<CellModel>>,
{
    fn export(&self) -> String {
        let lattice_vector_string = format!("{}", self.as_ref().lattice_vectors().unwrap());
        let cell_text = vec![
            lattice_vector_string,
            self.as_ref().positions_str(),
            self.as_ref().bs_kpoints_list_str(),
            self.as_ref().kpoints_list_str(),
            self.as_ref().misc_options(),
            self.as_ref().species_mass(),
            self.as_ref().species_pot_str(),
            self.as_ref().species_lcao_str(),
        ];
        cell_text.concat()
    }
}
