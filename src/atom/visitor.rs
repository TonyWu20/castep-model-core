use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader},
};

use castep_periodic_table::{data::ELEMENT_TABLE, element::LookupElement};
use nalgebra::{Point3, Vector3};

use crate::{error::InvalidIndex, LatticeModel, ModelInfo};

use super::{AtomCollection, AtomView};

pub fn get_xyz_by_id<T: ModelInfo>(
    atom_collection: &AtomCollection<T>,
    atom_id: u32,
) -> Option<&Point3<f64>> {
    atom_collection.xyz_coords().get((atom_id - 1) as usize)
}

pub fn get_multiple_xyz_by_id<'a, 'b, T: ModelInfo>(
    atom_collection: &'a AtomCollection<T>,
    atom_ids: &'b [u32],
) -> Vec<Option<&'a Point3<f64>>> {
    atom_ids
        .iter()
        .map(|&id| atom_collection.xyz_coords().get((id - 1) as usize))
        .collect()
}

pub trait VisitCollection<T: ModelInfo> {
    fn get_xyz_by_id(&self, atom_id: u32) -> Option<&Point3<f64>>;
    fn get_multiple_xyz_by_id<'a, 'b>(
        &'a self,
        atom_ids: &'b [u32],
    ) -> Vec<Option<&'a Point3<f64>>>;
    fn view_atom_at_index(&self, index: usize) -> Result<AtomView<T>, InvalidIndex>;
    fn view_atom_by_id(&self, atom_id: u32) -> Result<AtomView<T>, InvalidIndex>;
    fn get_vector_ab(&self, a_id: u32, b_id: u32) -> Result<Vector3<f64>, InvalidIndex>;
    fn element_set(&self) -> Vec<String>;
    fn spin_total(&self) -> u8;
    fn get_final_cutoff_energy(&self, potentials_loc: &str) -> Result<f64, io::Error>;
}

impl<T> VisitCollection<T> for AtomCollection<T>
where
    T: ModelInfo,
{
    fn get_xyz_by_id(&self, atom_id: u32) -> Option<&Point3<f64>> {
        self.xyz_coords().get((atom_id - 1) as usize)
    }

    fn get_multiple_xyz_by_id<'a, 'b>(
        &'a self,
        atom_ids: &'b [u32],
    ) -> Vec<Option<&'a Point3<f64>>> {
        atom_ids
            .iter()
            .map(|&id| self.xyz_coords().get((id - 1) as usize))
            .collect()
    }
    fn view_atom_at_index(&self, index: usize) -> Result<AtomView<T>, InvalidIndex> {
        let element_symbol = self
            .element_symbols()
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
    fn view_atom_by_id(&self, atom_id: u32) -> Result<AtomView<T>, InvalidIndex> {
        let index = (atom_id - 1) as usize;
        self.view_atom_at_index(index)
    }

    fn get_vector_ab(&self, a_id: u32, b_id: u32) -> Result<Vector3<f64>, InvalidIndex> {
        if a_id != b_id {
            let atom_a_xyz = self.get_xyz_by_id(a_id).unwrap();
            let atom_b_xyz = self.get_xyz_by_id(b_id).unwrap();
            Ok(atom_b_xyz - atom_a_xyz)
        } else {
            Err(InvalidIndex)
        }
    }

    fn element_set(&self) -> Vec<String> {
        let mut elm_list: Vec<(String, u8)> = vec![];
        elm_list.extend(
            self.element_symbols()
                .iter()
                .zip(self.atomic_nums().iter())
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

    fn spin_total(&self) -> u8 {
        self.element_symbols()
            .iter()
            .map(|symbol| -> u8 { ELEMENT_TABLE.get_by_symbol(symbol).unwrap().spin })
            .reduce(|total, next| total + next)
            .unwrap()
    }

    fn get_final_cutoff_energy(&self, potentials_loc: &str) -> Result<f64, io::Error> {
        let mut energy: f64 = 0.0;
        self.element_set()
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
}

impl<T> VisitCollection<T> for LatticeModel<T>
where
    T: ModelInfo,
{
    fn get_xyz_by_id(&self, atom_id: u32) -> Option<&Point3<f64>> {
        self.atoms().get_xyz_by_id(atom_id)
    }

    fn get_multiple_xyz_by_id<'a, 'b>(
        &'a self,
        atom_ids: &'b [u32],
    ) -> Vec<Option<&'a Point3<f64>>> {
        self.atoms().get_multiple_xyz_by_id(atom_ids)
    }
    fn view_atom_at_index(&self, index: usize) -> Result<AtomView<T>, InvalidIndex> {
        self.atoms().view_atom_at_index(index)
    }

    fn view_atom_by_id(&self, atom_id: u32) -> Result<AtomView<T>, InvalidIndex> {
        self.atoms().view_atom_by_id(atom_id)
    }

    fn get_vector_ab(&self, a_id: u32, b_id: u32) -> Result<Vector3<f64>, InvalidIndex> {
        self.atoms().get_vector_ab(a_id, b_id)
    }

    fn element_set(&self) -> Vec<String> {
        self.atoms().element_set()
    }

    fn spin_total(&self) -> u8 {
        self.atoms().spin_total()
    }

    fn get_final_cutoff_energy(&self, potentials_loc: &str) -> Result<f64, io::Error> {
        self.atoms().get_final_cutoff_energy(potentials_loc)
    }
}
