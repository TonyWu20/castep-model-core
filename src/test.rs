use crate::{CellModel, LatticeModel, MsiModel};
use std::fs::{self, read_to_string};
#[test]
fn test_cell_msi() {
    let msi_file = read_to_string("./SAC_GDY_V.msi").unwrap();
    let msi_model: LatticeModel<MsiModel> = LatticeModel::try_from(msi_file.as_str()).unwrap();
    fs::write("Test_msi.msi", msi_model.msi_export()).unwrap();
    let cell_model: LatticeModel<CellModel> = msi_model.into();
    fs::write("Test_cell.cell", cell_model.cell_export()).unwrap();
    let msi_back: LatticeModel<MsiModel> = cell_model.into();
    fs::write("Test_msi_back.msi", msi_back.msi_export()).unwrap();
}
