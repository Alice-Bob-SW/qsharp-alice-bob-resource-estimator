//!Cat-qubit power, cabling, and noise utilities.
//!This module provides:
//!- Cryostat cabling models (attenuation, thermal conductance, passive heat loads)
//!- Hardware parameter container with derived coefficients
//!All temperatures are in Kelvin, frequencies in angular units (rad/s),
//!and powers/energies follow the conventions in the associated report/notes. 

use std::collections::HashMap;
use std::f64::consts::PI;

use crate::cables::{CABLES_DB};
/// Frequency constants (radians per second)
pub const TWOPI: f64 = 2.0 * PI;
/// Base frequency unit (1 Hz in angular units).
pub const HZ: f64  = TWOPI;
/// 1 kHz in angular units.
pub const KHZ: f64 = TWOPI * 1.0e3;
/// 1 MHz in angular units.
pub const MHZ: f64 = TWOPI * 1.0e6;
/// 1 GHz in angular units.
pub const GHZ: f64 = TWOPI * 1.0e9;

// Time constants
/// Microsecond in seconds.
pub const US: f64 = 1.0e-6;
/// Nanosecond in seconds.
pub const NS: f64 = 1.0e-9;

// Unit constants
/// Picohenry in henry.
pub const PH: f64 = 1.0e-12; // picoHenry

// Physical constants
/// Reduced Planck constant ℏ [J·s].
pub const HBAR: f64 = 1.054_571_817e-34;
/// Boltzmann constant k_B [J/K].
pub const KB:   f64 = 1.380_649e-23;

/// Cryostat stage temperatures [K], ordered cold → warm.
/// Order: MXC, 100 mK, Still, 4 K, 50 K.
pub const STAGE_TEMPS: [f64;5] = [0.012, 0.1, 0.97, 3.34, 35.2];
/// External / room temperature [K].
pub const T_EXT: f64 = 300.0;

/// Utility function converting dB to linear
pub fn db_to_val(db: f64) -> f64 {
    10f64.powf(db / 10.0)
}
// Adjust imports to your project structure:
// use crate::cables;
// use crate::{STAGE_TEMPS, T_EXT};
// use crate::{STAGE_TEMPS, T_EXT};  // if you have these as globals

/// Logical type of line in the cryostat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineType {
    /// Drive line (qubit drive / readout, etc).
    Drive,
    /// Pump line (parametric drive).
    Pump,
    /// Flux / fast-flux line.
    Ffl,
    /// DC line (slow bias, wiring, etc).
    Dc,
}

/// One physical line from MXC up to 50K/room temperature.
#[derive(Debug, Clone)]
pub struct Line {
    /// Logical type of line (drive/pump/ffl/dc).
    pub line_type: LineType,

    /// Cable technology per stage span; length matches number of spans.
    /// Stage ordering (cold → warm): MXC → 100mK → Still → 4K → 50K
    pub technology: Vec<String>,

    /// Per-span lengths [m], ordered cold → warm.
    pub lengths: Vec<f64>,

    /// Optional fixed pads per stage [dB].
    pub fixed_pads_db: Option<Vec<f64>>,

    /// Keys into the cable DB for different technologies.
    pub cable_db_keys: HashMap<String, String>,
}

impl Default for Line {
    fn default() -> Self {
        // Python default: np.flipud([219.29e-3, 304.04e-3, 248.64e-3, 161.58e-3, 165.88e-3])
        // → reversed order
        let lengths = vec![
            165.88e-3,
            161.58e-3,
            248.64e-3,
            304.04e-3,
            219.29e-3,
        ];

        let mut cable_db_keys = HashMap::new();
        cable_db_keys.insert("coax".to_string(), "SC-86/50-SCN-CN".to_string());
        cable_db_keys.insert("dc_PhBr".to_string(), "PhBr_36AWG".to_string());
        cable_db_keys.insert("dc_NbTi".to_string(), "NbTi_36AWG".to_string());

        let line_type = LineType::Drive;
        // Mirrors __post_init__ default for non-DC lines
        let technology = vec![
            "dense".to_string(),
            "dense".to_string(),
            "dense".to_string(),
            "coax".to_string(),
            "coax".to_string(),
        ];

        Self {
            line_type,
            technology,
            lengths,
            fixed_pads_db: None,
            cable_db_keys,
        }
    }
}

impl Line {
    /// Constructor that mirrors the Python __post_init__ behavior:
    /// if `technology` is `None`, pick based on line type.
    pub fn new(
        line_type: LineType,
        technology: Option<Vec<String>>,
        lengths: Option<Vec<f64>>,
        fixed_pads_db: Option<Vec<f64>>,
        cable_db_keys: Option<HashMap<String, String>>,
    ) -> Self {
        let mut line = Line {
            line_type,
            technology: technology.unwrap_or_default(),
            lengths: lengths.unwrap_or_else(|| Line::default().lengths),
            fixed_pads_db,
            cable_db_keys: cable_db_keys.unwrap_or_else(|| Line::default().cable_db_keys),
        };

        // Emulate __post_init__
        if line.technology.is_empty() {
            line.technology = match line.line_type {
                LineType::Dc => vec![
                    "dc_NbTi".into(),
                    "dc_NbTi".into(),
                    "dc_NbTi".into(),
                    "dc_PhBr".into(),
                    "dc_PhBr".into(),
                ],
                _ => vec![
                    "dense".into(),
                    "dense".into(),
                    "dense".into(),
                    "coax".into(),
                    "coax".into(),
                ],
            };
        }

        line
    }

    /// Pure cable attenuation per meter (dB/m) for a cable technology.
    fn pure_cable_attenuation_per_m(tech: &str) -> f64 {
        match tech {
            "dense" => {
                // 1.5 dB/m
                1.5
            }
            "coax" => {
                // 3.2 dB/m at 4 K
                3.2
            }
            "dc_PhBr" | "dc_NbTi" => {
                // DC lines assumed to have negligible RF attenuation here.
                0.0
            }
            _ => panic!("Unsupported cable technology: {tech:?}"),
        }
    }

    /// Attenuation per stage [dB].
    pub fn attenuation(&self) -> Vec<f64> {
        // Base pads: override if provided, else defaults by type.
        let base: Vec<f64> = if let Some(ref pads) = self.fixed_pads_db {
            pads.clone()
        } else {
            match self.line_type {
                LineType::Drive => vec![26.0, 20.0, 10.0, 10.0, 0.0],
                LineType::Pump  => vec![0.0, 15.0, 13.0, 15.0, 0.0],
                LineType::Ffl   => vec![0.0, 0.0, 0.0, 10.0, 6.0],
                LineType::Dc    => vec![0.0; 5],
            }
        };

        let cable_db: Vec<f64> = self
            .technology
            .iter()
            .zip(self.lengths.iter())
            .map(|(t, l)| Self::pure_cable_attenuation_per_m(t) * l)
            .collect();

        base.into_iter()
            .zip(cable_db.into_iter())
            .map(|(b, c)| b + c)
            .collect()
    }

    /// Cumulative attenuation up the stack [dB].
    pub fn cumulative_attenuation(&self) -> Vec<f64> {
        let att = self.attenuation();
        let mut result = Vec::with_capacity(att.len());
        let mut sum = 0.0;
        for a in att {
            sum += a;
            result.push(sum);
        }
        result
    }

    /// Return G_eff(T) [W / (m·K)] for a given cable technology.
    ///
    /// This is now fully wired to `CABLES_DB`, mirroring the Python code:
    ///   coax: outer + dielectric + inner
    ///   dense: analytic power-law
    ///   dc_*: inner only
    fn get_conductance_func(
        &self,
        tech: &str,
    ) -> Box<dyn Fn(f64) -> f64 + 'static> {
        match tech {
            "coax" => {
                let key = self
                    .cable_db_keys
                    .get("coax")
                    .cloned()
                    .unwrap_or_else(|| "SC-86/50-SCN-CN".to_string());

                let cable = CABLES_DB
                    .get(&key)
                    .unwrap_or_else(|| panic!("Unknown cable key: {key}"));

                // `cable` is a &'static Cable (from the static DB), so we can
                // safely capture it by reference in a 'static closure.
                Box::new(move |t: f64| {
                    cable.outer.material.k(t).unwrap() * cable.outer_area()
                        + cable.dielectric.material.k(t).unwrap() * cable.dielectric_area()
                        + cable.inner.material.k(t).unwrap() * cable.inner_area()
                })
            }
            "dense" => {
                // A * 4.6 * T^0.56 with A = 1.3e-9
                let a = 1.3e-9_f64;
                Box::new(move |t: f64| a * 4.6 * t.powf(0.56))
            }
            "dc_PhBr" => {
                let key = self
                    .cable_db_keys
                    .get("dc_PhBr")
                    .cloned()
                    .unwrap_or_else(|| "PhBr_36AWG".to_string());

                let cable = CABLES_DB
                    .get(&key)
                    .unwrap_or_else(|| panic!("Unknown cable key: {key}"));

                Box::new(move |t: f64| {
                    cable.inner.material.k(t).unwrap() * cable.inner_area()
                })
            }
            "dc_NbTi" => {
                let key = self
                    .cable_db_keys
                    .get("dc_NbTi")
                    .cloned()
                    .unwrap_or_else(|| "NbTi_36AWG".to_string());

                let cable = CABLES_DB
                    .get(&key)
                    .unwrap_or_else(|| panic!("Unknown cable key: {key}"));

                Box::new(move |t: f64| {
                    cable.inner.material.k(t).unwrap() * cable.inner_area()
                })
            }
            _ => panic!("Unsupported cable technology: {tech:?}"),
        }
    }

    /// Effective thermal conductance per unit length G_eff(T) [W/(m·K)],
    /// using the span that sandwiches temperature T.
    ///
    pub fn conductance(&self, t: f64, stage_temps: &[f64]) -> f64 {
        let min_temp = stage_temps
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);

        if t < min_temp {
            panic!("Query temperature must be ≥ MXC temperature.");
        }

        // Equivalent of np.searchsorted(stage_temps, T, side="right") - 1,
        // clamped into [0, len-1].
        let mut idx = 0usize;
        while idx + 1 < stage_temps.len() && t >= stage_temps[idx + 1] {
            idx += 1;
        }

        let tech = &self.technology[idx];
        let g = self.get_conductance_func(tech);
        g(t)
    }

    /// Simple numeric integration (Simpson’s rule) for a closure f over [a, b].
    fn integrate<F>(f: F, a: f64, b: f64, n: usize) -> f64
    where
        F: Fn(f64) -> f64,
    {
        // n must be even for Simpson's rule
        let n = if n % 2 == 0 { n } else { n + 1 };
        let h = (b - a) / n as f64;

        let mut sum = f(a) + f(b);
        for i in 1..n {
            let x = a + i as f64 * h;
            let coeff = if i % 2 == 0 { 2.0 } else { 4.0 };
            sum += coeff * f(x);
        }

        sum * h / 3.0
    }

    /// Passive heat per stage span [W/m].
    ///
    /// Q_span = ∫ G_eff(T) dT over the span
    /// Returns Q_span / length_span (per unit length).
    ///
    /// `stage_temps` ↔ `STAGE_TEMPS`, `t_ext` ↔ `T_EXT`.
    pub fn passive_heat_load(&self, stage_temps: &[f64], t_ext: f64) -> Vec<f64> {
        let mut loads = Vec::with_capacity(self.technology.len());

        for (i, tech) in self.technology.iter().enumerate() {
            let t_lo = stage_temps[i];
            let t_hi = if i < self.technology.len() - 1 {
                stage_temps[i + 1]
            } else {
                t_ext
            };

            let g = self.get_conductance_func(tech);
            let q = Self::integrate(&g, t_lo, t_hi, 1000); // adjust resolution as desired

            loads.push(q / self.lengths[i]);
        }

        loads
    }
}

/// Cryostat cabling model: stages, Carnot factors, and per-line data.
#[derive(Debug, Clone)]
pub struct Cabling {
    /// Stage names ordered cold → warm.
    pub stage_names: Vec<String>,
    /// Carnot factor per stage
    pub carnot_factors: Vec<f64>,
    /// Lines by logical type.
    pub lines: HashMap<LineType, Line>,
}

impl Default for Cabling {
    fn default() -> Self {
        // Python: ["MXC", "100mK", "Still", "4K", "50K"]
        let stage_names = vec![
            "MXC".to_string(),
            "100mK".to_string(),
            "Still".to_string(),
            "4K".to_string(),
            "50K".to_string(),
        ];

        let carnot_factors = STAGE_TEMPS
            .iter()
            .map(|&t| (T_EXT - t) / t)
            .collect::<Vec<f64>>();

        let mut lines = HashMap::new();
        lines.insert(LineType::Drive, Line::new(LineType::Drive, None, None, None, None));
        lines.insert(LineType::Pump,  Line::new(LineType::Pump,  None, None, None, None));
        lines.insert(LineType::Ffl,   Line::new(LineType::Ffl,   None, None, None, None));
        lines.insert(LineType::Dc,    Line::new(LineType::Dc,    None, None, None, None));

        Self {
            stage_names,
            carnot_factors,
            lines,
        }
    }
}

impl Cabling {
    /// Construct a `Cabling` object with optional overrides.
    pub fn new(
        stage_names: Option<Vec<String>>,
        carnot_factors: Option<Vec<f64>>,
        lines: Option<HashMap<LineType, Line>>,
    ) -> Self {
        let default = Cabling::default();

        Self {
            stage_names: stage_names.unwrap_or(default.stage_names),
            carnot_factors: carnot_factors.unwrap_or(default.carnot_factors),
            lines: lines.unwrap_or(default.lines),
        }
    }

    /// Cumulative attenuation at a given stage (dB).
    pub fn atilde(&self, stage: &str, line_type: LineType) -> f64 {
        let stage_index = self
            .stage_names
            .iter()
            .position(|s| s == stage)
            .unwrap_or_else(|| panic!("Unknown stage name: {stage:?}"));

        let line = self
            .lines
            .get(&line_type)
            .unwrap_or_else(|| panic!("Unknown line type in lines map"));

        let a_cum = line.cumulative_attenuation();
        a_cum[stage_index]
    }

    /// Linear attenuation factors per span, cold → warm.
    pub fn attenuation_factors(&self, line_type: LineType) -> Vec<f64> {
        let line = self
            .lines
            .get(&line_type)
            .unwrap_or_else(|| panic!("Unknown line type in lines map"));

        let a_cum = line.cumulative_attenuation(); // Vec<f64>
        let len = a_cum.len();

        let lin: Vec<f64> = a_cum.iter().map(|&a| db_to_val(a)).collect();

        let mut out = vec![0.0; len];
        if len > 0 {
            out[0] = lin[0];
        }
        for i in 1..len {
            out[i] = lin[i] - lin[i - 1];
        }

        out
    }

    /// Total 'macro' prefactor that weights chip power
    /// by the Carnot factors and per-stage linear attenuation contributions.
    pub fn m_prefactor(&self, line_type: LineType) -> f64 {
        let factors = self.attenuation_factors(line_type);

        self.carnot_factors
            .iter()
            .zip(factors.iter())
            .map(|(c, a)| c * a)
            .sum::<f64>()
    }
}

/// Type of interaction used when mapping `g` to `ε`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    /// Stabilizer-type interaction.
    Stab,
    /// CNOT-type interaction.
    Cnot,
    /// Longitudinal interaction.
    Longitudinal,
}

// --------------------- Hardware struct ---------------------

/// Container for cat-qubit hardware parameters and derived coefficients.
#[derive(Debug, Clone)]
pub struct Hardware {
    /// Dissipation/linewidth (angular units) for the single-photon loss channel.
    pub k_1: f64,
    /// Dissipation/linewidth (angular units) for mode `b`.
    pub k_b: f64,
    /// Dephasing rate (angular units).
    pub k_phi: f64,
    /// External coupling rate (angular units).
    pub k_ext: f64,

    /// Inductive coupling (H).
    pub m: f64,
    /// Line impedance (Ohm).
    pub z: f64,
    /// Josephson energy [J].
    pub e_j: f64,
    /// Inductive energy [J].
    pub e_l: f64,
    /// Angular frequency of mode `a`.
    pub omega_a: f64,
    /// Angular frequency of mode `b`.
    pub omega_b: f64,
    /// Flux quantum [Wb].
    pub phi_0: f64,

    /// Participation / lever arm for mode `a`.
    pub vphi_a: f64,
    /// Participation / lever arm for mode `b`.
    pub vphi_b: f64,
    /// Participation / lever arm for the coupler.
    pub vphi_c: f64,
    /// Participation / lever arm for the target.
    pub vphi_t: f64,

    /// Thermal occupancy of mode `a`.
    pub ntha: f64,
    /// Thermal occupancy of mode `b`.
    pub nthb: f64,

    /// Measurement efficiency.
    pub eta: f64,

    /// Cryostat cabling model.
    pub cabling: Cabling,

    /// Coherent state amplitude α (inferred via `get_alpha`).
    pub alpha: f64,
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            // Dissipation/linewidths (angular units)
            k_1: 6.97 * KHZ,
            k_b: 24.0 * MHZ,
            k_phi: 0.08 * MHZ,
            k_ext: 400.0 * 2.0 * PI * 1.0, // 400 Hz in angular units (adjust if you have a Hz unit)

            // Circuit parameters
            m: 2.0 * PH,                    // inductive coupling (H)
            z: 50.0,                        // Ohm
            e_j: 27.0 * GHZ * 1.0e-34,      // J = Hz * hbar (J·s / rad)
            e_l: 40.0 * GHZ * 1.0e-34,      // J = Hz * hbar (J·s / rad)
            omega_a: 4.29 * GHZ,
            omega_b: 7.21 * GHZ,
            phi_0: 2.0e-15,                 // Weber

            // Participation / lever arms
            vphi_a: 0.14,
            vphi_b: 0.19,
            vphi_c: 0.10,
            vphi_t: 0.20,

            // Thermal occupancies
            ntha: 0.02,
            nthb: 0.02,

            // Measurement efficiency
            eta: 0.4,

            // Cabling
            cabling: Cabling::default(),

            // alpha (will be inferred via get_alpha)
            alpha: 0.0,
        }
    }
}

impl Hardware {
    /// Construct a `Hardware` object, optionally overriding the cabling model.
    pub fn new(cabling: Option<Cabling>) -> Self {
        let mut hw = Hardware::default();
        if let Some(c) = cabling {
            hw.cabling = c;
        }
        hw
    }

    // --------------------- helpers / derived ---------------------

    /// Infer alpha from a target bit-flip time T_bf using a 1D root search.
    pub fn get_alpha(&mut self, t_bf: f64) -> f64 {
        let gamma_bf = 1.0 / t_bf;

        let gamma_fn = |alpha: f64, this: &Hardware| -> f64 {
            // Gamma_bf(alpha) = gamma_bf - k_1 * alpha^2 * exp(-4 alpha^2)
            //                    - k_1 * ntha * exp(-2 alpha^2)
            gamma_bf
                - this.k_1 * alpha * alpha * (-4.0 * alpha * alpha).exp()
                - this.k_1 * this.ntha * (-2.0 * alpha * alpha).exp()
        };

        // Grid search 0..10 with 200 points to mimic numpy linspace
        let n_grid = 200;
        let a_min = 0.0;
        let a_max = 10.0;
        let step = (a_max - a_min) / (n_grid as f64 - 1.0);

        let mut root = 0.0;

        // Helper: sign function like np.sign
        let sign = |x: f64| -> i32 {
            if x > 0.0 {
                1
            } else if x < 0.0 {
                -1
            } else {
                0
            }
        };

        // Simple bisection solver (instead of brentq) on bracket [a, b]
        fn bisect<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> Option<f64>
        where
            F: Fn(f64) -> f64,
        {
            let mut fa = f(a);
            let mut fb = f(b);
            if fa == 0.0 {
                return Some(a);
            }
            if fb == 0.0 {
                return Some(b);
            }
            if fa * fb > 0.0 {
                return None;
            }

            for _ in 0..max_iter {
                let mid = 0.5 * (a + b);
                let fm = f(mid);
                if fm == 0.0 || 0.5 * (b - a) < tol {
                    return Some(mid);
                }
                if fa * fm < 0.0 {
                    b = mid;
                    fb = fm;
                } else {
                    a = mid;
                    fa = fm;
                }
            }
            Some(0.5 * (a + b))
        }

        let mut prev_alpha = a_min;
        let mut prev_val = gamma_fn(prev_alpha, self);
        let mut prev_sign = sign(prev_val);

        for i in 1..n_grid {
            let alpha = a_min + step * i as f64;
            let val = gamma_fn(alpha, self);
            let s = sign(val);

            if s == 0 {
                root = alpha;
                break;
            }

            if prev_sign + s == 0 {
                // sign change: root is bracketed in [prev_alpha, alpha]
                if let Some(cand) =
                    bisect(|a| gamma_fn(a, self), prev_alpha, alpha, 1.0e-8, 100)
                {
                    if cand.is_finite() {
                        root = cand;
                        break;
                    }
                }
            }

            prev_alpha = alpha;
            prev_val = val;
            prev_sign = s;
        }

        self.alpha = root;
        root
    }

    // --------------------- macro prefactors from cabling ---------------------

   /// Macro prefactor for pump lines (from cabling).
    pub fn mp(&self) -> f64 {
        self.cabling.m_prefactor(LineType::Pump)
    }

    /// Macro prefactor for drive lines (from cabling).
    pub fn md(&self) -> f64 {
        self.cabling.m_prefactor(LineType::Drive)
    }

    /// Macro prefactor used for Zeno-type operations (drive lines).
    pub fn mz(&self) -> f64 {
        self.cabling.m_prefactor(LineType::Drive)
    }

    /// Shared ATS-cancellation coefficient independent of the interaction type.
    pub fn generic_ats_cancellation_coef(&self) -> f64 {
        self.z
            * (self.phi_0 / (2.0 * PI * self.m)).powi(2)
            * (1.0 + 4.0 * self.e_j.powi(2) / self.e_l.powi(2))
    }

    /// Map from interaction type to the `g → ε` conversion prefactor.
    pub fn g_to_eps_coef(&self, interaction: InteractionType) -> f64 {
        match interaction {
            InteractionType::Cnot => {
                HBAR / (self.e_j * self.vphi_c * self.vphi_t.powi(2))
            }
            InteractionType::Stab => {
                2.0 * HBAR / (self.e_j * self.vphi_b * self.vphi_a.powi(2))
            }
            InteractionType::Longitudinal => {
                HBAR / (self.e_j * self.vphi_b * self.vphi_a.powi(2))
            }
        }
    }

    /// ATS cancellation coefficient for a given interaction type.
    pub fn ats_cancellation_coef(&self, interaction: InteractionType) -> f64 {
        let g = self.g_to_eps_coef(interaction);
        g * g * self.generic_ats_cancellation_coef()
    }

    /// ATS cancellation coefficient for stabilizer-type interactions.
    pub fn p(&self) -> f64 {
        self.ats_cancellation_coef(InteractionType::Stab)
    }

    /// ATS cancellation coefficient for CNOT-type interactions.
    pub fn c(&self) -> f64 {
        self.ats_cancellation_coef(InteractionType::Cnot)
    }

    /// ATS cancellation coefficient for longitudinal interactions.
    pub fn l(&self) -> f64 {
        self.ats_cancellation_coef(InteractionType::Longitudinal)
    }

    /// Dimensionless ratio `d = ℏ ω_b / κ_b`.
    pub fn d(&self) -> f64 {
        HBAR * self.omega_b / self.k_b
    }

    /// Dimensionless ratio `z = ℏ ω_a / κ_ext`.
    pub fn z_factor(&self) -> f64 {
        HBAR * self.omega_a / self.k_ext
    }

    /// Bit-flip time for a given α.
    pub fn t_bf_of_alpha(&self, alpha: f64) -> f64 {
        let rate = self.k_1 * alpha * alpha * (-4.0 * alpha * alpha).exp()
            + self.k_1 * self.ntha * (-2.0 * alpha * alpha).exp();
        1.0 / rate
    }

    /// Bit-flip probability during a gate of duration `T_gate`.
    pub fn p_z(&self, alpha: f64, t_gate: f64) -> f64 {
        alpha * alpha * self.k_1 * t_gate * (1.0 + 2.0 * self.ntha)
    }
}
