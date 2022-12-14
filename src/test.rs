use crate::{model_type::DefaultExport, CellModel, LatticeModel, MsiModel};
use std::{
    fs::{self, read_to_string},
    str::FromStr,
};
#[test]
fn test_cell_msi() {
    let msi_file = read_to_string("./SAC_GDY_V.msi").unwrap();
    let msi_model: LatticeModel<MsiModel> = LatticeModel::from_str(msi_file.as_str()).unwrap();
    fs::write("Test_msi.msi", msi_model.export()).unwrap();
    let cell_model: LatticeModel<CellModel> = msi_model.into();
    fs::write("Test_cell.cell", cell_model.export()).unwrap();
    let msi_back: LatticeModel<MsiModel> = cell_model.into();
    fs::write("Test_msi_back.msi", msi_back.export()).unwrap();
}
