// Copyright (c) Alice & Bob.
// Licensed under the MIT License.

//! Overhead model for LDPC codes.
//!
//! The LDPC architecture differs from the repetition code architecture in:
//! - No horizontal routing qubits needed (routing handled by the computing layer)
//! - Slightly higher gate cycle costs due to weight-4 stabilizers

use num_traits::ToPrimitive;
use resource_estimator::estimates::{ErrorBudget, Overhead};

use crate::counter::LogicalCounts;

/// Overhead wrapper for LDPC code architecture.
///
/// Adapts [`LogicalCounts`] to the LDPC architecture where routing is handled
/// within the computing layer rather than requiring dedicated routing qubits.
pub struct LdpcOverhead(LogicalCounts);

impl From<LogicalCounts> for LdpcOverhead {
    fn from(counts: LogicalCounts) -> Self {
        Self(counts)
    }
}

impl Overhead for LdpcOverhead {
    /// Number of logical qubits for the LDPC architecture.
    ///
    /// Unlike the repetition code, no horizontal routing qubits are needed:
    /// the LDPC computing layer handles routing natively.
    fn logical_qubits(&self) -> u64 {
        self.0.qubit_count
    }

    /// Logical depth in cycles for the LDPC architecture.
    ///
    /// Gate costs are slightly higher than repetition code due to weight-4
    /// stabilizers: CX = 2.5 cycles, CCX = 11.5 cycles.
    fn logical_depth(&self, _: &ErrorBudget) -> u64 {
        let cx_f = self
            .0
            .cx_count
            .to_f64()
            .expect("#CX didn't convert to f64");
        let ccx_f = self
            .0
            .ccx_count
            .to_f64()
            .expect("#CCX didn't convert to f64");

        // Slightly higher cycle costs than repetition code (2.2/10.1)
        // due to weight-4 stabilizer measurements
        let cx_cycles = 2.5;
        let ccx_cycles = 11.5;

        ((cx_f * cx_cycles) + (ccx_f * ccx_cycles))
            .ceil()
            .to_u64()
            .expect("logical depth is too large")
    }

    fn num_magic_states(&self, _: &ErrorBudget, _: usize) -> u64 {
        self.0.ccx_count
    }
}
