use std::{
    ffi::OsString,
    fs::{self, create_dir_all},
    io,
    marker::PhantomData,
    path::PathBuf,
};

use cpt::{data::ELEMENT_TABLE, element::LookupElement};

use crate::{
    atom::visitor::VisitCollection,
    builder_typestate::{No, ToAssign, Yes},
    lattice::LatticeModel,
    model_type::{cell::CellModel, msi::MsiModel, BandStructureExport, DefaultExport},
};

use super::{
    castep_param::{BandStructureParam, CastepParam, GeomOptParam, Task},
    ms_aux_files::MsAuxWriter,
};

#[derive(Debug)]
/// Struct to present seed files export.
/// The `'a` lifetime is the lifetime for the reference to the cell.`
pub struct SeedWriter<'a, T>
where
    T: Task,
{
    cell: &'a LatticeModel<CellModel>,
    param: CastepParam<T>,
    seed_name: &'a str,
    export_loc: PathBuf,
    potential_loc: PathBuf,
}

/// General methods for `SeedWriter<T>`
impl<'a, T> SeedWriter<'a, T>
where
    T: Task,
{
    /// Call the builder
    pub fn build(cell: &'a LatticeModel<CellModel>) -> SeedWriterBuilder<'a, T, No> {
        SeedWriterBuilder::<T, No>::new(cell)
    }
    /// method to handle export directory creation.
    pub fn create_export_dir(&self) -> Result<PathBuf, io::Error> {
        let dir_name = format!("{}_{}", self.seed_name, "opt");
        let dir_loc: OsString = self.export_loc.clone().into();
        let export_loc = PathBuf::from(dir_loc).join(dir_name);
        create_dir_all(&export_loc)?;
        Ok(export_loc)
    }
    /// Private method to handle file path starting with seed name and ending
    /// with custom extension suffix.
    fn path_builder(&self, extension: &str) -> Result<PathBuf, io::Error> {
        let export_loc = self.create_export_dir()?;
        let filename = format!("{}{}", self.seed_name, extension);
        Ok(export_loc.join(filename))
    }
    /// Copy the potential files for the elements in the cell to the seed folder.
    /// It is suggest to do this only in release version. Because the potential files
    /// take up much disk space.
    /// You can control this behaviour with `[cfg(not(debug_assertions))]`
    pub fn copy_potentials(&self) -> Result<(), io::Error> {
        let element_list = self.cell.element_set();
        element_list
            .iter()
            .try_for_each(|elm| -> Result<(), io::Error> {
                let pot_file = ELEMENT_TABLE.get_by_symbol(elm).unwrap().potential();
                let pot_src_path = self.potential_loc.join(pot_file);
                let dest_dir = self.create_export_dir()?;
                let pot_dest_path = dest_dir.join(pot_file);
                if !pot_dest_path.exists() {
                    fs::copy(pot_src_path, pot_dest_path)?;
                    Ok(())
                } else {
                    Ok(())
                }
            })
    }
    fn write_lsf_script(&self) -> Result<(), io::Error> {
        let target_dir = self.create_export_dir()?;
        let cell_name = self.seed_name;
        let cmd = format!("/home-yw/Soft/msi/MS70/MaterialsStudio7.0/etc/CASTEP/bin/RunCASTEP.sh -np $NP {cell_name}");
        let prefix = r#"APP_NAME=intelY_mid
NP=12
NP_PER_NODE=12
OMP_NUM_THREADS=1
RUN="RAW"

"#;
        let content = format!("{prefix}{cmd}");
        let lsf_filepath = target_dir.join("MS70_YW_CASTEP.lsf");
        fs::write(lsf_filepath, content)
    }

    fn write_hpc_sh_script(&self) -> Result<(), io::Error> {
        let target_dir = self.create_export_dir()?;
        let cell_name = self.seed_name;
        let template = r#"#PBS -N HPL_short_run
#PBS -q simple_q
#PBS -l walltime=168:00:00
#PBS -l nodes=1:ppn=24
#PBS -V

cd $PBS_O_WORKDIR

NCPU=`wc -l < $PBS_NODEFILE`
NNODES=`uniq $PBS_NODEFILE | wc -l`

echo ------------------------------------------------------
echo ' This job is allocated on '${NCPU}' cpu(s)'
echo 'Job is running on node(s): '
cat $PBS_NODEFILE
echo ------------------------------------------------------
echo PBS: qsub is running on $PBS_O_HOST
echo PBS: originating queue is $PBS_O_QUEUE
echo PBS: executing queue is $PBS_QUEUE
echo PBS: working directory is $PBS_O_WORKDIR
echo PBS: execution mode is $PBS_ENVIRONMENT
echo PBS: job identifier is $PBS_JOBID
echo PBS: job name is $PBS_JOBNAME
echo PBS: node file is $PBS_NODEFILE
echo PBS: number of nodes is $NNODES
echo PBS: current home directory is $PBS_O_HOME
echo PBS: PATH = $PBS_O_PATH
echo ------------------------------------------------------

##For openmpi-intel
##export LD_LIBRARY_PATH=/share/apps/openmpi-1.8.8-intel/lib:$LD_LIBRARY_PATH
##export PATH=/share/apps/openmpi-1.8.8-intel/bin:$PATH

cat $PBS_NODEFILE >./hostfile
"#;
        let run_cmd = format!(
            "mpirun --mca btl ^tcp --hostfile hostfile /home/bhuang/castep.mpi {}",
            cell_name
        );
        let post_cmd = "rm ./hostfile";
        let script = format!("{template}{run_cmd}\n{post_cmd}");
        let script_path = target_dir.join("hpc.pbs.sh");
        fs::write(script_path, script)
    }

    pub fn seed_name(&self) -> &str {
        self.seed_name
    }
}

/// Conversion from `SeedWriter<GeomOptParam>` to `SeedWriter<BandStructureParam>`
impl<'a> From<SeedWriter<'a, GeomOptParam>> for SeedWriter<'a, BandStructureParam> {
    fn from(geom_writer: SeedWriter<'a, GeomOptParam>) -> Self {
        let SeedWriter {
            cell,
            param,
            seed_name,
            export_loc,
            potential_loc,
        } = geom_writer;
        Self {
            cell,
            param: param.into(),
            seed_name,
            export_loc,
            potential_loc,
        }
    }
}

/// Methods for `SeedWriter<GeomOptParam>`
impl<'a> SeedWriter<'a, GeomOptParam> {
    pub fn write_seed_files(&self) -> Result<(), io::Error> {
        let ms_param = MsAuxWriter::build(self.seed_name, &self.export_loc)
            .with_kptaux(self.cell.build_kptaux())
            .with_trjaux(self.cell.build_trjaux())
            .with_potentials_loc(&self.potential_loc)
            .build();
        ms_param.write_kptaux()?;
        ms_param.write_bs_kptaux()?;
        ms_param.write_trjaux()?;
        let param_path = self.path_builder(".param")?;
        fs::write(param_path, format!("{}", self.param))?;
        let cell_path = self.path_builder(".cell")?;
        fs::write(cell_path, DefaultExport::export(&self.cell))?;
        let msi_path = self.path_builder(".msi")?;
        let msi_model: LatticeModel<MsiModel> = self.cell.into();
        fs::write(msi_path, msi_model.export())?;
        self.write_lsf_script()?;
        self.write_hpc_sh_script()?;
        Ok(())
    }
}

/// Methods for `SeedWriter<BandStructureParam>`
impl<'a> SeedWriter<'a, BandStructureParam> {
    pub fn write_seed_files(&self) -> Result<(), io::Error> {
        let ms_param = MsAuxWriter::build(self.seed_name, &self.export_loc)
            .with_kptaux(self.cell.build_kptaux())
            .with_trjaux(self.cell.build_trjaux())
            .with_potentials_loc(&self.potential_loc)
            .build();
        ms_param.write_bs_kptaux()?;
        let param_path = self.path_builder("_DOS.param")?;
        fs::write(param_path, format!("{}", self.param))?;
        let cell_path = self.path_builder("_DOS.cell")?;
        fs::write(cell_path, BandStructureExport::export(&self.cell))?;
        Ok(())
    }
}

#[derive(Debug)]
/// Builder for `SeedWriter`.
pub struct SeedWriterBuilder<'a, T, WithPotentialLoc>
where
    T: Task,
    WithPotentialLoc: ToAssign,
{
    cell: &'a LatticeModel<CellModel>,
    param: Option<CastepParam<T>>,
    seed_name: &'a str,
    export_loc: PathBuf,
    potential_loc: PathBuf,
    potential_set_state: PhantomData<WithPotentialLoc>,
}

/// Methods for building and setting the `SeedWriterBuilder`
impl<'a, T, P> SeedWriterBuilder<'a, T, P>
where
    T: Task,
    P: ToAssign,
{
    /// Create a new builder. The `cell` is the mandatory field and thus it is required.
    pub fn new(cell: &'a LatticeModel<CellModel>) -> SeedWriterBuilder<T, No> {
        SeedWriterBuilder {
            cell,
            param: None,
            seed_name: "",
            export_loc: PathBuf::new(),
            potential_loc: PathBuf::new(),
            potential_set_state: PhantomData,
        }
    }
    /// Set potential loc and transit to the state ready to build a `SeedWriter<T>`
    pub fn with_potential_loc(self, potential_loc: &'a str) -> SeedWriterBuilder<T, Yes> {
        let new_potential_loc = self.potential_loc.join(potential_loc);
        let Self {
            cell,
            param,
            seed_name,
            export_loc,
            potential_loc: _,
            potential_set_state: _,
        } = self;
        SeedWriterBuilder {
            cell,
            param,
            seed_name,
            export_loc,
            potential_loc: new_potential_loc,
            potential_set_state: PhantomData,
        }
    }
    /// Set the `export_loc`
    pub fn with_export_loc(self, export_loc: &'a str) -> SeedWriterBuilder<T, P> {
        let new_export_loc = self.export_loc.join(export_loc);
        let Self {
            cell,
            param,
            seed_name,
            export_loc: _,
            potential_loc,
            potential_set_state,
        } = self;
        SeedWriterBuilder {
            cell,
            param,
            seed_name,
            export_loc: new_export_loc,
            potential_loc,
            potential_set_state,
        }
    }
    /// Set new `seed_name`
    pub fn with_seed_name(self, new_seed_name: &'a str) -> SeedWriterBuilder<T, P> {
        let Self {
            cell,
            param,
            seed_name: _,
            export_loc,
            potential_loc,
            potential_set_state,
        } = self;
        SeedWriterBuilder {
            cell,
            param,
            seed_name: new_seed_name,
            export_loc,
            potential_loc,
            potential_set_state,
        }
    }
}

/// The state of `SeedWriterBuilder<'a, T, P>` ready to build the `SeedWriter<'a,T>`
impl<'a, T> SeedWriterBuilder<'a, T, Yes>
where
    T: Task + 'static,
{
    pub fn build(self) -> SeedWriter<'a, T> {
        let param = CastepParam::<T>::build()
            .with_spin_total(self.cell.spin_total())
            .with_cut_off_energy(
                self.cell
                    .get_final_cutoff_energy(self.potential_loc.to_str().unwrap())
                    .unwrap(),
            )
            .build();
        let Self {
            cell,
            param: _,
            seed_name,
            export_loc,
            potential_loc,
            potential_set_state: _,
        } = self;
        SeedWriter {
            cell,
            param,
            seed_name,
            export_loc,
            potential_loc,
        }
    }
}
