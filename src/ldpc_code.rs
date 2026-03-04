// Copyright (c) Alice & Bob.
// Licensed under the MIT License.

//! LDPC code for biased error correction with cat qubits.
//!
//! The code and its performances are described in
//! [arXiv:2401.09541](https://arxiv.org/abs/2401.09541).
//!
//! Code parameters:
//! - family index (selects one of 5 parametric code families)
//! - elongation parameter ℓ (scales code size within a family)
//! - average number of photons |α|²
//!
//! Hard-coded values:
//! - 1/κ₂ = 100 ns (sets the gates speed)
//! - (κ₁/κ₂)_th ≈ 0.0065 (approximately half of the repetition code threshold)
//! - max ℓ (for iteration) = 50
//! - max |α|² (for iteration) = 30.0

use num_traits::FromPrimitive;
use std::{cmp::Ordering, fmt::Display};

use resource_estimator::estimates::ErrorCorrection;

use crate::qubit::CatQubit;

/// Defines a parametric family of LDPC codes `[n0 + delta_n*ℓ, k0 + 2*ℓ, d]`.
///
/// From [arXiv:2401.09541](https://arxiv.org/abs/2401.09541), Table 1.
struct LdpcFamily {
    n0: u64,
    delta_n: u64,
    k0: u64,
    d: u64,
}

/// The five LDPC code families from arXiv:2401.09541.
const LDPC_FAMILIES: [LdpcFamily; 5] = [
    LdpcFamily {
        n0: 20,
        delta_n: 4,
        k0: 10,
        d: 5,
    },
    LdpcFamily {
        n0: 55,
        delta_n: 5,
        k0: 22,
        d: 9,
    },
    LdpcFamily {
        n0: 78,
        delta_n: 6,
        k0: 26,
        d: 12,
    },
    LdpcFamily {
        n0: 119,
        delta_n: 7,
        k0: 34,
        d: 16,
    },
    LdpcFamily {
        n0: 136,
        delta_n: 8,
        k0: 34,
        d: 22,
    },
];

impl LdpcFamily {
    /// Number of physical qubits per block: n = n0 + delta_n * ℓ
    fn n(&self, ell: u64) -> u64 {
        self.n0 + self.delta_n * ell
    }

    /// Number of logical qubits per block: k = k0 + 2 * ℓ
    fn k(&self, ell: u64) -> u64 {
        self.k0 + 2 * ell
    }
}

/// Store the LDPC code parameters: family index, elongation ℓ, and |α|².
#[derive(Clone)]
pub struct LdpcCodeParameter {
    family_idx: usize,
    ell: u64,
    alpha_sq: f64,
}

impl LdpcCodeParameter {
    #[must_use]
    /// Create a new LDPC code parameter set.
    pub fn new(family_idx: usize, ell: u64, alpha_sq: f64) -> Self {
        Self {
            family_idx,
            ell,
            alpha_sq,
        }
    }

    fn family(&self) -> &LdpcFamily {
        &LDPC_FAMILIES[self.family_idx]
    }

    /// Block length n.
    fn n(&self) -> u64 {
        self.family().n(self.ell)
    }

    /// Number of logical qubits k per block.
    fn k(&self) -> u64 {
        self.family().k(self.ell)
    }

    /// Code distance d.
    fn d(&self) -> u64 {
        self.family().d
    }
}

impl Display for LdpcCodeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{},{},{}] (|ɑ|² = {})",
            self.n(),
            self.k(),
            self.d(),
            self.alpha_sq
        )
    }
}

/// Represents an LDPC code for cat qubits.
pub struct LdpcCode {
    p_threshold: f64,
}

impl LdpcCode {
    #[must_use]
    /// Default initialization, with threshold at 0.0065
    /// (approximately half the repetition code threshold of 0.013).
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    /// Logical phase-flip probability per round per logical qubit.
    ///
    /// Approximated from arXiv:2401.09541 data points and the observation that
    /// the LDPC threshold is about half the repetition code threshold.
    fn logical_phaseflip_probability(
        &self,
        physical_qubit: &CatQubit,
        parameter: &LdpcCodeParameter,
    ) -> Option<f64> {
        let prefactor = 0.1;
        let d = parameter.d();
        let exponent = (i32::from_u64(d)? + 1) / 2;

        Some(
            prefactor
                * ((parameter.alpha_sq.powf(0.86) * physical_qubit.k1_k2) / self.p_threshold)
                    .powi(exponent),
        )
    }

    #[must_use]
    /// Logical bit-flip probability per round per logical qubit.
    ///
    /// Each of the n-k stabilizer measurements involves weight-4 checks,
    /// contributing bit-flip errors.
    fn logical_bitflip_probability(parameter: &LdpcCodeParameter) -> Option<f64> {
        let n = parameter.n();
        let k = parameter.k();

        // Bit-flip error probability of a CX gate
        let pcx = 0.5 * (-2.0 * parameter.alpha_sq).exp();

        // 4*(n-k) CX gates per syndrome extraction cycle, distributed over k logical qubits
        Some(4.0 * f64::from_u64(n - k)? * pcx / f64::from_u64(k)?)
    }
}

impl Default for LdpcCode {
    /// Create LDPC code with threshold (κ₁/κ₂)_th ≈ 0.0065.
    ///
    /// The threshold is approximately half that of the repetition code,
    /// as noted in [arXiv:2401.09541](https://arxiv.org/abs/2401.09541).
    fn default() -> Self {
        let p_threshold = 0.0065;
        Self { p_threshold }
    }
}

/// Iterator over LDPC code parameters, ordered by physical qubits per logical qubit.
struct LdpcCodeParameterRange {
    /// Pre-sorted list of (family_idx, ell, alpha_sq) triples.
    params: Vec<(usize, u64, u64)>,
    index: usize,
}

impl LdpcCodeParameterRange {
    fn new(lower_bound: Option<&LdpcCodeParameter>) -> Self {
        let max_ell: u64 = 50;
        let max_alpha_sq: u64 = 30;
        let min_alpha_sq: u64 = 1;

        let mut params: Vec<(usize, u64, u64)> = Vec::new();

        for (family_idx, _family) in LDPC_FAMILIES.iter().enumerate() {
            for ell in 0..=max_ell {
                for alpha_sq in min_alpha_sq..=max_alpha_sq {
                    // Apply lower bound filtering if provided
                    if let Some(lb) = lower_bound {
                        if family_idx == lb.family_idx
                            && ell == lb.ell
                            && (alpha_sq as f64) < lb.alpha_sq
                        {
                            continue;
                        }
                    }
                    params.push((family_idx, ell, alpha_sq));
                }
            }
        }

        // Sort by physical qubits per logical qubit (ascending), then by total physical qubits
        params.sort_by(|a, b| {
            let fam_a = &LDPC_FAMILIES[a.0];
            let fam_b = &LDPC_FAMILIES[b.0];
            let na = fam_a.n(a.1);
            let ka = fam_a.k(a.1);
            let nb = fam_b.n(b.1);
            let kb = fam_b.k(b.1);
            // Physical qubits per block = 2n - k; logical qubits per block = k
            let phys_per_logical_a = (2 * na - ka) as f64 / ka as f64;
            let phys_per_logical_b = (2 * nb - kb) as f64 / kb as f64;
            phys_per_logical_a
                .partial_cmp(&phys_per_logical_b)
                .unwrap_or(Ordering::Equal)
                .then((2 * na - ka).cmp(&(2 * nb - kb)))
        });

        Self { params, index: 0 }
    }
}

impl Iterator for LdpcCodeParameterRange {
    type Item = LdpcCodeParameter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.params.len() {
            return None;
        }

        let (family_idx, ell, alpha_sq) = self.params[self.index];
        self.index += 1;

        Some(LdpcCodeParameter::new(family_idx, ell, alpha_sq as f64))
    }
}

impl ErrorCorrection for LdpcCode {
    type Qubit = CatQubit;
    type Parameter = LdpcCodeParameter;

    fn code_parameter_range(
        &self,
        lower_bound: Option<&Self::Parameter>,
    ) -> impl Iterator<Item = Self::Parameter> {
        LdpcCodeParameterRange::new(lower_bound)
    }

    fn physical_qubits(&self, parameter: &Self::Parameter) -> Result<u64, String> {
        let n = parameter.n();
        let k = parameter.k();
        // n data qubits + (n - k) ancilla qubits for syndrome extraction
        Ok(2 * n - k)
    }

    fn logical_qubits(&self, parameter: &Self::Parameter) -> Result<u64, String> {
        Ok(parameter.k())
    }

    fn logical_cycle_time(
        &self,
        _qubit: &Self::Qubit,
        parameter: &Self::Parameter,
    ) -> Result<u64, String> {
        // LDPC codes have weight-4 stabilizers (vs weight-2 for repetition code),
        // roughly doubling the cycle time: 10/κ₂ per round × d rounds
        // With 1/κ₂ = 100 ns: 1000 * d ns
        Ok(1000 * parameter.d())
    }

    fn logical_error_rate(
        &self,
        qubit: &Self::Qubit,
        parameter: &Self::Parameter,
    ) -> Result<f64, String> {
        if let (Some(d_f64), Some(lzp), Some(lxp)) = (
            f64::from_u64(parameter.d()),
            self.logical_phaseflip_probability(qubit, parameter),
            Self::logical_bitflip_probability(parameter),
        ) {
            // Error rate per logical qubit per cycle.
            // Factor d accounts for d rounds per cycle (analogous to repetition code).
            // lzp and lxp are per-round per-logical-qubit rates.
            Ok(d_f64 * (lzp + lxp))
        } else {
            Err("cannot compute logical failure probability".into())
        }
    }

    fn compute_code_parameter(
        &self,
        qubit: &Self::Qubit,
        required_logical_error_rate: f64,
    ) -> Result<Self::Parameter, String> {
        self.compute_code_parameter_for_smallest_size(qubit, required_logical_error_rate)
    }

    fn code_parameter_cmp(
        &self,
        qubit: &Self::Qubit,
        p1: &Self::Parameter,
        p2: &Self::Parameter,
    ) -> Ordering {
        if let (
            Ok(num_qubits1),
            Ok(logical_cycle_time1),
            Ok(num_qubits2),
            Ok(logical_cycle_time2),
        ) = (
            self.physical_qubits(p1),
            self.logical_cycle_time(qubit, p1),
            self.physical_qubits(p2),
            self.logical_cycle_time(qubit, p2),
        ) {
            num_qubits1
                .cmp(&num_qubits2)
                .then(logical_cycle_time1.cmp(&logical_cycle_time2))
        } else {
            Ordering::Equal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_parameters() {
        // Family 5: [136+8ℓ, 34+2ℓ, 22]
        let param = LdpcCodeParameter::new(4, 33, 11.0);
        assert_eq!(param.n(), 136 + 8 * 33); // 400
        assert_eq!(param.k(), 34 + 2 * 33); // 100
        assert_eq!(param.d(), 22);
    }

    #[test]
    fn test_physical_qubits_per_block() {
        let code = LdpcCode::new();
        // [400, 100, 22]: physical = 2*400 - 100 = 700
        let param = LdpcCodeParameter::new(4, 33, 11.0);
        assert_eq!(code.physical_qubits(&param).unwrap(), 700);
    }

    #[test]
    fn test_logical_qubits_per_block() {
        let code = LdpcCode::new();
        let param = LdpcCodeParameter::new(4, 33, 11.0);
        assert_eq!(code.logical_qubits(&param).unwrap(), 100);
    }

    #[test]
    fn test_cycle_time() {
        let code = LdpcCode::new();
        let qubit = CatQubit::new();
        let param = LdpcCodeParameter::new(4, 33, 11.0);
        // 1000 * 22 = 22000 ns
        assert_eq!(code.logical_cycle_time(&qubit, &param).unwrap(), 22000);
    }

    #[test]
    fn test_phase_flip_error_order_of_magnitude() {
        // Validate against the paper's data point:
        // [429,100,22] code with k1/k2=1e-4, nbar=11 gives p_Z_L ≈ 6.4e-10
        // Our [400,100,22] approximation with same parameters should be similar order
        let code = LdpcCode::new();
        let qubit = CatQubit { k1_k2: 1e-4 };
        let param = LdpcCodeParameter::new(4, 33, 11.0);
        let p_z = code
            .logical_phaseflip_probability(&qubit, &param)
            .unwrap();

        // Should be in the ballpark of 6.4e-10 (within a few orders of magnitude
        // given our approximations)
        assert!(p_z < 1e-6, "Phase-flip error too high: {p_z}");
        assert!(p_z > 1e-15, "Phase-flip error too low: {p_z}");
    }

    #[test]
    fn test_error_rate_decreases_with_distance() {
        let code = LdpcCode::new();
        let qubit = CatQubit::new();

        // Compare family 1 (d=5) vs family 3 (d=12) at same alpha_sq
        let param_low_d = LdpcCodeParameter::new(0, 10, 10.0);
        let param_high_d = LdpcCodeParameter::new(2, 10, 10.0);

        let err_low = code.logical_error_rate(&qubit, &param_low_d).unwrap()
            / param_low_d.k() as f64;
        let err_high = code.logical_error_rate(&qubit, &param_high_d).unwrap()
            / param_high_d.k() as f64;

        assert!(
            err_high < err_low,
            "Higher distance should have lower error per logical qubit"
        );
    }

    #[test]
    fn test_qubits_per_logical_qubit_ratio() {
        let code = LdpcCode::new();
        // At large ℓ, physical/logical ratio should approach ~2*delta_n/2 = delta_n
        // For family 1: delta_n=4, so ratio → (2*4)/2 = 4 physical per logical
        // But at ℓ=0: (2*20-10)/10 = 3.0
        let param = LdpcCodeParameter::new(0, 0, 10.0);
        let phys = code.physical_qubits(&param).unwrap() as f64;
        let logi = code.logical_qubits(&param).unwrap() as f64;
        let ratio = phys / logi;
        assert!(ratio < 10.0, "LDPC should have low qubit overhead, got {ratio}");
    }
}
