use std::fmt::Debug;

use crate::{CellModel, MsiModel};

pub mod cell;
pub mod msi;

pub trait ModelInfo: Debug + Clone + Default {}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings<T: ModelInfo> {
    /// List of k-points. Each k-point has xyz and a weight factor.
    kpoints_list: Vec<[f64; 4]>,
    /// An array to specify the grid of k-point used in this model
    kpoints_grid: [u8; 3],
    /// Spacing of k-point.
    kpoints_mp_spacing: Option<f64>,
    /// Offset of the k-points from the origin.
    kpoints_mp_offset: [f64; 3],
    /// Option in `IONIC_CONSTRAINTS` in cell format
    fix_all_cell: bool,
    /// Option in `IONIC_CONSTRAINTS` in cell format
    fix_com: bool,
    /// Option in `cell` format
    external_efield: [f64; 3],
    /// The order is `Rxx`, `Rxy`, `Rxz`, `Ryy`, `Ryz`, `Rzz`
    external_pressure: [f64; 6],
    /// A parameter in `msi` format
    cry_display: (u32, u32),
    /// A parameter in `msi` format
    periodic_type: u8,
    /// A parameter in `msi` format
    space_group: String,
    /// A parameter in `msi` format
    cry_tolerance: f64,
    format_marker: T,
}

impl Settings<MsiModel> {
    pub fn new_msi_settings(periodic_type: u8, space_group: &str, cry_tolerance: f64) -> Self {
        Self {
            periodic_type,
            space_group: space_group.into(),
            cry_tolerance,
            ..Default::default()
        }
    }
}

impl<T: ModelInfo> Default for Settings<T> {
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
            periodic_type: 100_u8,
            space_group: "1 1".to_string(),
            cry_tolerance: 0.05,
            cry_display: (192, 256),
            format_marker: T::default(),
        }
    }
}

/// Methods exposed to `CellModel` only
impl Settings<CellModel> {
    pub fn kpoints_list(&self) -> &[[f64; 4]] {
        self.kpoints_list.as_ref()
    }

    pub fn kpoints_grid(&self) -> [u8; 3] {
        self.kpoints_grid
    }

    pub fn kpoints_mp_spacing(&self) -> Option<f64> {
        self.kpoints_mp_spacing
    }

    pub fn kpoints_mp_offset(&self) -> [f64; 3] {
        self.kpoints_mp_offset
    }

    pub fn fix_all_cell(&self) -> bool {
        self.fix_all_cell
    }

    pub fn fix_com(&self) -> bool {
        self.fix_com
    }

    pub fn external_efield(&self) -> [f64; 3] {
        self.external_efield
    }

    pub fn external_pressure(&self) -> [f64; 6] {
        self.external_pressure
    }
}

/// Methods exposed to `MsiModel` only
impl Settings<MsiModel> {
    pub fn periodic_type(&self) -> u8 {
        self.periodic_type
    }

    pub fn space_group(&self) -> &str {
        self.space_group.as_ref()
    }

    pub fn cry_tolerance(&self) -> f64 {
        self.cry_tolerance
    }
}
