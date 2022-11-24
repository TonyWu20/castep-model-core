// use cpt::{data::ELEMENT_TABLE, element::LookupElement};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

pub mod castep_param;
pub mod ms_aux_files;
pub mod seed_writer;

pub trait MyFilePath: AsRef<Path> + Into<OsString> + Clone {}
impl MyFilePath for PathBuf {}

//     fn copy_smcastep_extension(&self, target_root_dir: &str) -> Result<(), Box<dyn Error>>;
//     fn write_lsf_script(&self, target_root_dir: &str) -> Result<(), Box<dyn Error>> {
//         let target_dir = self.export_destination(target_root_dir)?;
//         let cell_name = self.get_lattice_name();
//         let cmd = format!("/home-yw/Soft/msi/MS70/MaterialsStudio7.0/etc/CASTEP/bin/RunCASTEP.sh -np $NP {cell_name}");
//         let prefix = r#"APP_NAME=intelY_mid
// NP=12
// NP_PER_NODE=12
// OMP_NUM_THREADS=1
// RUN="RAW"
//
// "#;
//         let content = format!("{prefix}{cmd}");
//         let lsf_filepath = target_dir.join("MS70_YW_CASTEP.lsf");
//         fs::write(lsf_filepath, content)?;
//         Ok(())
//     }
// }
