use nalgebra::Point3;

use crate::ModelInfo;

use super::AtomCollection;

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
