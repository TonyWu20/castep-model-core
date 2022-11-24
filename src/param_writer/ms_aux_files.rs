use std::{
    error::Error,
    ffi::OsString,
    fs::{self, create_dir_all},
    io,
    path::{Path, PathBuf},
};

use glob::glob;
use rayon::prelude::*;

use super::MyFilePath;
#[derive(Debug)]
/// Writer of `Materials Studio` required auxilliary files when running `Castep` tasks.
pub struct MsAuxWriter<P: MyFilePath> {
    filestem: String,
    export_loc: P,
    potentials_loc: P,
    kptaux: KptAux,
    trjaux: TrjAux,
    task: String,
}

impl<P: MyFilePath> MsAuxWriter<P> {
    /// Call the builder
    pub fn build<'a>(filestem: &str, export_loc: &'a P) -> ParamWriterBuilder<'a, P> {
        ParamWriterBuilder::new(filestem, export_loc)
    }
    /// Private method for handling path creation.
    fn path_builder(&self, extension: &str) -> Result<PathBuf, io::Error> {
        let dir_name = format!("{}_{}", self.filestem, self.task);
        let dir_loc: OsString = self.export_loc.clone().into();
        let export_loc = PathBuf::from(dir_loc).join(&dir_name);
        create_dir_all(&export_loc)?;
        let filename = format!("{}{}", self.filestem, extension);
        Ok(export_loc.join(filename))
    }
    /// Write `.kptaux` file for `Geometry Optimization` tasks.
    pub fn write_kptaux(&self) -> Result<(), io::Error> {
        let kptaux_path = self.path_builder(".kptaux")?;
        fs::write(kptaux_path, self.kptaux.export())
    }
    /// Write `.kptaux` file for `Band Structure` tasks.
    pub fn write_bs_kptaux(&self) -> Result<(), io::Error> {
        let kptaux_path = self.path_builder("_DOS.kptaux")?;
        fs::write(kptaux_path, self.kptaux.export())
    }
    /// Write `.trjaux` file for trajection of atoms in the cell.
    pub fn write_trjaux(&self) -> Result<(), io::Error> {
        let trjaux_path = self.path_builder(".trjaux")?;
        fs::write(trjaux_path, self.trjaux.export())
    }
}

#[derive(Debug)]
/// Builder for `ParamWriter<P>`
pub struct ParamWriterBuilder<'a, P: MyFilePath> {
    filestem: String,
    export_loc: &'a P,
    potentials_loc: Option<&'a P>,
    kptaux: Option<KptAux>,
    trjaux: Option<TrjAux>,
    task: Option<String>,
}
impl<'a, P: MyFilePath> ParamWriterBuilder<'a, P> {
    pub fn new(filestem: &str, export_loc: &'a P) -> Self {
        Self {
            filestem: filestem.to_string(),
            export_loc,
            potentials_loc: None,
            kptaux: None,
            trjaux: None,
            task: Some("opt".to_string()),
        }
    }
    /// Set potentials_loc
    pub fn with_potentials_loc(self, potentials_loc: &'a P) -> Self {
        Self {
            filestem: self.filestem,
            export_loc: self.export_loc,
            potentials_loc: Some(potentials_loc),
            kptaux: self.kptaux,
            trjaux: self.trjaux,
            task: self.task,
        }
    }
    /// Provide the `KptAux` struct
    pub fn with_kptaux(self, kptaux: KptAux) -> Self {
        Self {
            filestem: self.filestem,
            export_loc: self.export_loc,
            potentials_loc: self.potentials_loc,
            kptaux: Some(kptaux),
            trjaux: self.trjaux,
            task: self.task,
        }
    }
    /// Provide the `TrjAux` struct
    pub fn with_trjaux(self, trjaux: TrjAux) -> Self {
        Self {
            filestem: self.filestem,
            export_loc: self.export_loc,
            potentials_loc: self.potentials_loc,
            kptaux: self.kptaux,
            trjaux: Some(trjaux),
            task: self.task,
        }
    }
    /// Set task
    pub fn set_task(self, task: &str) -> Self {
        Self {
            filestem: self.filestem,
            export_loc: self.export_loc,
            potentials_loc: self.potentials_loc,
            kptaux: self.kptaux,
            trjaux: self.trjaux,
            task: Some(task.into()),
        }
    }
    /// Build `MsAuxWriter` from given fields.
    pub fn build(self) -> MsAuxWriter<P> {
        MsAuxWriter {
            filestem: self.filestem,
            export_loc: self.export_loc.to_owned(),
            potentials_loc: self.potentials_loc.unwrap().to_owned(),
            kptaux: self.kptaux.unwrap(),
            trjaux: self.trjaux.unwrap(),
            task: self.task.unwrap(),
        }
    }
}
/// File `.kptaux`
#[derive(Debug)]
pub struct KptAux {
    kpoints: Vec<[f64; 4]>,
    /// This specifies the dimensions of the Monkhorst-Pack grid requested in the directions of the reciprocal space lattice vectors.
    mp_grid: [u8; 3],
    /// The single entry is the maximum distance between k-points on the Monkhorst-Pack grid.
    /// The dimensions of the grid will be chosen such that the maximum separation of k-points is less than this.
    /// Default unit is Å^-1
    mp_spacing: Option<f64>,
    /// This specifies the offset of the Monkhorst-Pack grid with respect to the origin of the Brillouin zone.
    /// The three entries are the offset in fractional coordinates relative to the reciprocal lattice vectors.
    mp_offset: [f64; 3],
}

impl KptAux {
    pub fn new(
        kpoints: Vec<[f64; 4]>,
        mp_grid: [u8; 3],
        mp_spacing: Option<f64>,
        mp_offset: [f64; 3],
    ) -> Self {
        Self {
            kpoints,
            mp_grid,
            mp_spacing,
            mp_offset,
        }
    }

    pub fn export(&self) -> String {
        let [grid_x, grid_y, grid_z] = self.mp_grid;
        let mp_grid_text = format!("MP_GRID : {:>8}{:>8}{:>8}", grid_x, grid_y, grid_z);
        let [off_x, off_y, off_z] = self.mp_offset;
        let mp_offset_text = format!(
            "MP_OFFSET : {:22.18e}{:22.18e}{:22.18e}",
            off_x, off_y, off_z
        );
        format!(
            "{}\n{}\n{}",
            mp_grid_text,
            mp_offset_text,
            self.kpoint_images()
        )
    }
    /// Initial Rough version
    /// TODO: generate from `kpoints` and `mp_grid`
    fn kpoint_images(&self) -> String {
        r#"BLOCK KPOINT_IMAGES
   1   1
ENDBLOCK KPOINT_IMAGES"#
            .into()
    }
}
/// File '.trjaux'
#[derive(Debug)]
pub struct TrjAux {
    /// Id ordered as in `cell`. Directly imported from `LatticeModel<CellModel>`
    atom_ids: Vec<u32>,
}

impl TrjAux {
    pub fn new(atom_ids: Vec<u32>) -> Self {
        Self { atom_ids }
    }

    pub fn export(&self) -> String {
        {
            let mut trjaux_contents = String::new();
            let trjaux_header = r#"# Atom IDs to appear in any .trj file to be generated.
# Correspond to atom IDs which will be used in exported .msi file
# required for animation/analysis of trajectory within Cerius2.
"#;
            trjaux_contents.push_str(trjaux_header);
            self.atom_ids.iter().for_each(|atom_id| {
                trjaux_contents.push_str(&format!("{}\n", atom_id));
            });
            let trjaux_ending = r#"#Origin  0.000000000000000e+000  0.000000000000000e+000  0.000000000000000e+000"#;
            trjaux_contents.push_str(trjaux_ending);
            trjaux_contents
        }
    }
}

/// Scan the generated `msi` files, create a perl script to be run in `Materials Studio`
/// to save as `xsd` format.
pub fn to_xsd_scripts(target_root_dir: &str) -> Result<(), Box<dyn Error>> {
    let msi_pattern = format!("{target_root_dir}/**/*.msi");
    let item_collection = glob(&msi_pattern)
        .expect("Failed to read glob pattern")
        .into_iter()
        .par_bridge()
        .into_par_iter()
        .map(|entry| -> Option<String> {
            match entry {
                Ok(path) => {
                    let stem = path.file_stem().unwrap();
                    let parent = path.parent().unwrap();
                    Some(format!(
                        r#""{}/{}""#,
                        parent.to_str().unwrap(),
                        stem.to_str().unwrap()
                    ))
                }
                Err(e) => {
                    println!("glob entry match error: {:?}", e);
                    None
                }
            }
        })
        .collect::<Vec<Option<String>>>()
        .iter()
        .map(|entry| -> String { entry.as_ref().unwrap().to_string() })
        .collect::<Vec<String>>();
    let all_files_text = item_collection.join(", ");
    let headlines = r#"#!perl
use strict;
use Getopt::Long;
use MaterialsScript qw(:all);
"#;
    let array_text = format!("my @params = (\n{});\n", all_files_text);
    let actions = r#"foreach my $item (@params) {
    my $doc = $Documents{"${item}.msi"};
    $doc->CalculateBonds;
    $doc->Export("${item}.xsd");
    $doc->Save;
    $doc->Close;
}"#;
    let contents = format!("{headlines}{array_text}{actions}");
    fs::write(Path::new("msi_to_xsd.pl"), contents)?;
    Ok(())
}
