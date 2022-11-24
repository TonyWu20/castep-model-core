use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use crate::builder_typestate::{No, ToAssign, Yes};

#[derive(Debug)]
enum FiniteBasisCorr {
    No,
    Manual,
    Auto,
}

impl Display for FiniteBasisCorr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::No => write!(f, "0"),
            Self::Manual => write!(f, "1"),
            Self::Auto => write!(f, "2"),
        }
    }
}

/// Trait to limit the type passed to `CastepParam<T>`
pub trait Task: Default + Display {}

#[derive(Debug)]
/// Struct to represent a Castep parameter file.
pub struct CastepParam<T: Task> {
    xc_functional: String,
    spin_polarized: bool,
    spin: u8,
    opt_strategy: String,
    page_wvfns: u32,
    cut_off_energy: f64,
    grid_scale: f64,
    fine_grid_scale: f64,
    finite_basis_corr: FiniteBasisCorr,
    elec_energy_tol: f64,
    max_scf_cycles: u32,
    fix_occupancy: bool,
    metals_method: String,
    mixing_scheme: String,
    mix_charge_amp: f64,
    mix_spin_amp: f64,
    mix_charge_gmax: f64,
    mix_spin_gmax: f64,
    mix_history_length: u32,
    perc_extra_bands: u32,
    smearing_width: f64,
    spin_fix: u32,
    num_dump_cycles: u32,
    calculate_elf: bool,
    calculate_stress: bool,
    popn_calculate: bool,
    calculate_hirshfeld: bool,
    calculate_densdiff: bool,
    popn_bond_cutoff: f64,
    pdos_calculate_weights: bool,
    extra_setting: T,
}

impl<T: Task> CastepParam<T> {
    pub fn build() -> CastepParamBuilder<T, No, No> {
        CastepParamBuilder::<T, No, No>::new()
    }
}

impl From<CastepParam<GeomOptParam>> for CastepParam<BandStructureParam> {
    fn from(geom_param: CastepParam<GeomOptParam>) -> Self {
        CastepParam {
            spin: geom_param.spin,
            cut_off_energy: geom_param.cut_off_energy,
            ..Default::default()
        }
    }
}

/// Parameters in `Geometry Optimization` only.
pub struct GeomOptParam {
    geom_energy_tol: f64,
    geom_force_tol: f64,
    geom_stress_tol: f64,
    geom_disp_tol: f64,
    geom_max_iter: u32,
    geom_method: String,
    fixed_npw: bool,
}

impl Task for GeomOptParam {}

impl Default for GeomOptParam {
    fn default() -> Self {
        Self {
            geom_energy_tol: 5e-5,
            geom_force_tol: 0.1,
            geom_stress_tol: 0.2,
            geom_disp_tol: 0.005,
            geom_max_iter: 6000,
            geom_method: "BFGS".into(),
            fixed_npw: false,
        }
    }
}

impl Display for GeomOptParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = format!(
            r#"geom_energy_tol :   {:22.15e}
geom_force_tol :        {:18.15}
geom_stress_tol :        {:18.15}
geom_disp_tol :        {:18.15}
geom_max_iter :     {}
geom_method : {}
fixed_npw : {}"#,
            self.geom_energy_tol,
            self.geom_force_tol,
            self.geom_stress_tol,
            self.geom_disp_tol,
            self.geom_max_iter,
            self.geom_method,
            self.fixed_npw
        );
        write!(f, "{}", content)
    }
}

/// Parameters in `Band Structure` task only.
pub struct BandStructureParam {
    bs_nextra_bands: u32,
    bs_xc_functional: String,
    bs_eigenvalue_tol: f64,
    bs_write_eigenvalues: bool,
}

impl Task for BandStructureParam {}

impl Default for BandStructureParam {
    fn default() -> Self {
        Self {
            bs_nextra_bands: 72,
            bs_xc_functional: "PBE".into(),
            bs_eigenvalue_tol: 1e-5,
            bs_write_eigenvalues: true,
        }
    }
}

impl Display for BandStructureParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = format!(
            r#"bs_nextra_bands :       {}
bs_xc_functional : {}
bs_eigenvalue_tol :   {:22.15e}
bs_write_eigenvalues : {}"#,
            self.bs_nextra_bands,
            self.bs_xc_functional,
            self.bs_eigenvalue_tol,
            self.bs_write_eigenvalues
        );
        write!(f, "{}", content)
    }
}

impl<T> Default for CastepParam<T>
where
    T: Task,
{
    fn default() -> Self {
        Self {
            xc_functional: "PBE".into(),
            spin_polarized: true,
            spin: 0,
            opt_strategy: "Speed".into(),
            page_wvfns: 0,
            cut_off_energy: 0.0,
            grid_scale: 1.5,
            fine_grid_scale: 1.5,
            finite_basis_corr: FiniteBasisCorr::No,
            elec_energy_tol: 1e-5,
            max_scf_cycles: 6000,
            fix_occupancy: false,
            metals_method: "dm".into(),
            mixing_scheme: "Pulay".into(),
            mix_charge_amp: 0.5,
            mix_spin_amp: 2.0,
            mix_charge_gmax: 1.5,
            mix_spin_gmax: 1.5,
            mix_history_length: 20,
            perc_extra_bands: 72,
            smearing_width: 0.1,
            spin_fix: 6,
            num_dump_cycles: 0,
            calculate_elf: false,
            calculate_stress: false,
            popn_calculate: false,
            calculate_hirshfeld: true,
            calculate_densdiff: false,
            popn_bond_cutoff: 3.0,
            pdos_calculate_weights: true,
            extra_setting: T::default(),
        }
    }
}

impl<T> Display for CastepParam<T>
where
    T: Task,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = format!(
            r#"task : GeometryOptimization
comment : CASTEP calculation from Materials Studio
xc_functional : {}
spin_polarized : {}
spin :        {}
opt_strategy : {}
page_wvfns :        {}
cut_off_energy :      {:18.15}
grid_scale :        {:18.15}
fine_grid_scale :        {:18.15}
finite_basis_corr :        {}
elec_energy_tol :   {:18.15}
max_scf_cycles :     {}
fix_occupancy : {}
metals_method : {}
mixing_scheme : {}
mix_charge_amp :        {:18.15}
mix_spin_amp :        {:18.15}
mix_charge_gmax :        {:18.15}
mix_spin_gmax :        {:18.15}
mix_history_length :       {}
perc_extra_bands : {}
smearing_width :        {:18.15}
spin_fix :        {}
num_dump_cycles : {}
{}
calculate_ELF : {}
calculate_stress : {}
popn_calculate : {}
calculate_hirshfeld : {}
calculate_densdiff : {}
popn_bond_cutoff :        {:18.15}
pdos_calculate_weights : {}
"#,
            self.xc_functional,
            self.spin_polarized,
            self.spin,
            self.opt_strategy,
            self.page_wvfns,
            self.cut_off_energy,
            self.grid_scale,
            self.fine_grid_scale,
            self.finite_basis_corr,
            self.elec_energy_tol,
            self.max_scf_cycles,
            self.fix_occupancy,
            self.metals_method,
            self.mixing_scheme,
            self.mix_charge_amp,
            self.mix_spin_amp,
            self.mix_charge_gmax,
            self.mix_spin_gmax,
            self.mix_history_length,
            self.perc_extra_bands,
            self.smearing_width,
            self.spin_fix,
            self.num_dump_cycles,
            self.extra_setting,
            self.calculate_elf,
            self.calculate_stress,
            self.popn_calculate,
            self.calculate_hirshfeld,
            self.calculate_densdiff,
            self.popn_bond_cutoff,
            self.pdos_calculate_weights
        );
        write!(f, "{}", content)
    }
}

/// Builder for `CastepParam<T>`
#[derive(Default, Debug)]
pub struct CastepParamBuilder<T, SpinSet, CutOffSet>
where
    T: Task,
    SpinSet: ToAssign,
    CutOffSet: ToAssign,
{
    task: T,
    spin_total: u8,
    cut_off_energy: f64,
    spin_set: PhantomData<SpinSet>,
    cut_off_set: PhantomData<CutOffSet>,
}

/// Methods when parameters are not all ready.
impl<T, S, C> CastepParamBuilder<T, S, C>
where
    T: Task,
    S: ToAssign,
    C: ToAssign,
{
    pub fn new() -> CastepParamBuilder<T, No, No> {
        CastepParamBuilder {
            task: T::default(),
            spin_total: 0_u8,
            cut_off_energy: 0.0,
            spin_set: PhantomData,
            cut_off_set: PhantomData,
        }
    }
    pub fn with_spin_total(self, spin_total: u8) -> CastepParamBuilder<T, Yes, C> {
        CastepParamBuilder {
            task: self.task,
            spin_total,
            cut_off_energy: self.cut_off_energy,
            spin_set: PhantomData,
            cut_off_set: PhantomData,
        }
    }
    pub fn with_cut_off_energy(self, cut_off_energy: f64) -> CastepParamBuilder<T, S, Yes> {
        CastepParamBuilder {
            task: self.task,
            spin_total: self.spin_total,
            cut_off_energy,
            spin_set: PhantomData,
            cut_off_set: PhantomData,
        }
    }
}

/// When parameters are all settled, build `CastepParam<T>`
impl<T> CastepParamBuilder<T, Yes, Yes>
where
    T: Task,
{
    pub fn build(&self) -> CastepParam<T> {
        CastepParam {
            spin: self.spin_total,
            cut_off_energy: self.cut_off_energy,
            ..Default::default()
        }
    }
}
