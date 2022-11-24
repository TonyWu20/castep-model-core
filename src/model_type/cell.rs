use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
};

use crate::{
    atom::Atom,
    lattice::{LatticeModel, LatticeVectors},
    param_writer::ms_aux_files::{KptAux, TrjAux},
    Transformation,
};

use cpt::{data::ELEMENT_TABLE, element::LookupElement};
use na::{UnitQuaternion, Vector, Vector3};

use super::{msi::MsiModel, ModelInfo};

#[derive(Clone, Debug, PartialEq)]
/// Struct to represent `cell`format.
pub struct CellModel {
    /// List of k-points. Each k-point has xyz and a weight factor.
    kpoints_list: Vec<[f64; 4]>,
    kpoints_grid: [u8; 3],
    kpoints_mp_spacing: Option<f64>,
    kpoints_mp_offset: [f64; 3],
    /// Option in `IONIC_CONSTRAINTS`
    fix_all_cell: bool,
    /// Option in `IONIC_CONSTRAINTS`
    fix_com: bool,
    external_efield: [f64; 3],
    /// The order is `Rxx`, `Rxy`, `Rxz`, `Ryy`, `Ryz`, `Rzz`
    external_pressure: [f64; 6],
}

impl ModelInfo for CellModel {}

/// Default `CellFormat` values
impl Default for CellModel {
    fn default() -> Self {
        Self {
            kpoints_list: vec![[0.0, 0.0, 0.0, 1.0]],
            kpoints_grid: [1, 1, 1],
            kpoints_mp_spacing: None,
            kpoints_mp_offset: [0.0, 0.0, 0.0],
            fix_all_cell: true,
            fix_com: false,
            external_efield: [0.0, 0.0, 0.0],
            external_pressure: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }
}

/// Methods for `CellFormat`
impl CellModel {
    fn write_block(block: (String, String)) -> String {
        let (block_name, content) = block;
        format!(
            "%BlOCK {}\n{}%ENDBLOCK {}\n\n",
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
        let rot_axis = a_vec.cross(&x_axis).normalize();
        let rot_quatd: UnitQuaternion<f64> = UnitQuaternion::new(rot_axis * a_to_x_angle);
        msi_model.as_mut().rotate(&rot_quatd);
        let new_lat_vec = LatticeVectors::new(
            msi_model
                .as_ref()
                .lattice_vectors()
                .unwrap()
                .vectors()
                .to_owned(),
            CellModel::default(),
        );
        let fractional_coord_matrix = msi_model
            .as_ref()
            .lattice_vectors()
            .unwrap()
            .fractional_coord_matrix();
        let mut cell_atoms: Vec<Atom<CellModel>> = msi_model
            .as_ref()
            .atoms()
            .iter()
            .map(|atom| -> Atom<CellModel> {
                let fractional_coord = fractional_coord_matrix * atom.xyz();
                let mut new_atom = Atom::new(
                    atom.element_symbol().to_string(),
                    atom.element_id(),
                    *atom.xyz(),
                    atom.atom_id(),
                    CellModel::default(),
                );
                new_atom.set_fractional_xyz(Some(fractional_coord));
                new_atom
            })
            .collect();
        cell_atoms.sort_by_key(|a| a.element_id());
        Self::new(Some(new_lat_vec), cell_atoms, CellModel::default())
    }
}

/// Methods only for `LatticeModel<CellFormat>`
impl LatticeModel<CellModel> {
    /// Formatted *fractional coordinates*
    fn positions_str(&self) -> String {
        let coords_strings: Vec<String> = self
            .atoms()
            .iter()
            .map(|atom| format!("{}", atom))
            .collect();
        let coords = coords_strings.concat();
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
            .model_type()
            .kpoints_list
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
            .model_type()
            .kpoints_list
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
            self.model_type().fix_all_cell,
            self.model_type().fix_com,
            self.ionic_constraints()
        );
        let [ex, ey, ez] = self.model_type().external_efield;
        let external_efield = CellModel::write_block((
            "EXTERNAL_EFIELD".to_string(),
            format!("{:16.10}{:16.10}{:16.10}\n", ex, ey, ez),
        ));
        let [rxx, rxy, rxz, ryy, ryz, rzz] = self.model_type().external_pressure;
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
        let element_list = self.list_element();
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
        let element_list = self.list_element();
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
        let element_list = self.list_element();
        let lcao_strings: Vec<String> = element_list
            .iter()
            .map(|elm| {
                let lcao_state = ELEMENT_TABLE.get_by_symbol(elm).unwrap().lcao();
                format!("{:>8}{:9}\n", elm, lcao_state)
            })
            .collect();
        CellModel::write_block(("SPECIES_LCAO_STATES".to_string(), lcao_strings.concat()))
    }
    /// Export to ".cell" for geometry optimization task.
    pub fn cell_export(&self) -> String {
        let lattice_vector_string = format!("{}", self.lattice_vectors().unwrap());
        let cell_text = vec![
            lattice_vector_string,
            self.positions_str(),
            self.kpoints_list_str(),
            self.misc_options(),
            self.species_mass(),
            self.species_pot_str(),
            self.species_lcao_str(),
        ];
        cell_text.concat()
    }
    /// Export to "_DOS.cell" for band structure calculation task.
    pub fn bs_cell_export(&self) -> String {
        let lattice_vector_string = format!("{}", self.lattice_vectors().unwrap());
        let cell_text = vec![
            lattice_vector_string,
            self.positions_str(),
            self.bs_kpoints_list_str(),
            self.kpoints_list_str(),
            self.misc_options(),
            self.species_mass(),
            self.species_pot_str(),
            self.species_lcao_str(),
        ];
        cell_text.concat()
    }
    pub fn get_final_cutoff_energy(&self, potentials_loc: &str) -> Result<f64, io::Error> {
        let mut energy: f64 = 0.0;
        self.list_element()
            .iter()
            .try_for_each(|elm| -> Result<(), io::Error> {
                let potential_file = ELEMENT_TABLE.get_by_symbol(elm).unwrap().potential();
                let potential_path = format!("{potentials_loc}/{potential_file}");
                if let Ok(file) = File::open(&potential_path) {
                    let file = BufReader::new(file);
                    let fine_energy: u32 = file
                        .lines()
                        .find(|line| line.as_ref().unwrap().contains("FINE"))
                        .map(|line| {
                            let num_str = line.as_ref().unwrap().split_whitespace().next().unwrap();
                            num_str.parse::<u32>().unwrap()
                        })
                        .unwrap();
                    let round_bigger_tenth = |num: u32| -> f64 {
                        match num % 10 {
                            0 => num as f64,
                            _ => ((num / 10 + 1) * 10) as f64,
                        }
                    };
                    let ultra_fine_energy = round_bigger_tenth((fine_energy as f64 * 1.1) as u32);
                    energy = if energy > ultra_fine_energy {
                        energy
                    } else {
                        ultra_fine_energy
                    };
                    Ok(())
                } else {
                    panic!(
                        "Error while reading potential file for element: {}, {}",
                        elm, potential_path
                    )
                }
            })?;
        Ok(energy)
    }
    pub fn spin_total(&self) -> u8 {
        self.atoms()
            .iter()
            .map(|atom| -> u8 {
                ELEMENT_TABLE
                    .get_by_symbol(atom.element_symbol())
                    .unwrap()
                    .spin
            })
            .reduce(|total, next| total + next)
            .unwrap()
    }
    /// Build `KptAux` struct
    pub fn build_kptaux(&self) -> KptAux {
        KptAux::new(
            self.model_type().kpoints_list.clone(),
            self.model_type().kpoints_grid,
            self.model_type().kpoints_mp_spacing,
            self.model_type().kpoints_mp_offset,
        )
    }
    /// Build `TrjAux` struct
    pub fn build_trjaux(&self) -> TrjAux {
        let atom_ids: Vec<u32> = self.atoms().iter().map(|atom| atom.atom_id()).collect();
        TrjAux::new(atom_ids)
    }
}

impl Display for Atom<CellModel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let atom_element = self.element_symbol();
        let spin = ELEMENT_TABLE.get_by_symbol(atom_element).unwrap().spin();
        if spin > 0 {
            writeln!(
                f,
                "{:>3}{:20.16}{:20.16}{:20.16} SPIN={:14.10}",
                atom_element,
                self.fractional_xyz().unwrap().x,
                self.fractional_xyz().unwrap().y,
                self.fractional_xyz().unwrap().z,
                spin as f64
            )
        } else {
            writeln!(
                f,
                "{:>3}{:20.16}{:20.16}{:20.16}",
                atom_element,
                self.fractional_xyz().unwrap().x,
                self.fractional_xyz().unwrap().y,
                self.fractional_xyz().unwrap().z,
            )
        }
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
