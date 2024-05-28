// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//!
//! Compute:
//! <pre>
//! - Total number of physical qubits (data+ancilla+computation+factories+routing)
//! - Fraction of qubits allocated to the magic state factories
//! - Total failure probability
//! </pre>
//! and print the resource estimation results.

use std::{fmt::Display, ops::Deref};

use num_traits::{FromPrimitive, ToPrimitive};
use resource_estimator::estimates::{FactoryPart, Overhead, PhysicalResourceEstimationResult};

use crate::{code::RepetitionCode, counter::LogicalCounts, factories::ToffoliFactory};

/// Compute resources estimates for Alice & Bob's cat qubits
pub struct AliceAndBobEstimates(
    // compute a resource estimate
    PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>,
);

impl AliceAndBobEstimates {
    /// Store the number of factories
    pub fn toffoli_factory_part(&self) -> Option<&FactoryPart<ToffoliFactory>> {
        self.factory_parts()[0].as_ref()
    }

    /// Count the number of physical qubits
    pub fn physical_qubits(&self) -> u64 {
        // Routing qubits must be added to ensure all-to-all connectivity
        let additional_routing_qubits = 2
            * ((3 * self.layout_overhead().logical_qubits()
                + self.toffoli_factory_part().map_or(0, FactoryPart::copies) * 6)
                - 1);
        self.0.physical_qubits() + additional_routing_qubits
    }

    /// Compute the fraction of physical qubits allocated to the Toffoli magic states factories
    pub fn factory_fraction(&self) -> f64 {
        (self
            .physical_qubits_for_factories()
            .to_f64()
            .expect("can convert")
            / self.physical_qubits().to_f64().expect("can convert"))
            * 100.0
    }

    /// Compute the total error of the magic state preparation
    pub fn total_error(&self) -> f64 {
        // error is computed as "logical + magic" without the cross term since it is
        // largely sub-leading here, and negative anyway
        let logical = (self.num_cycles() * self.layout_overhead().logical_qubits())
            .to_f64()
            .expect("can convert volume as f64")
            * self.logical_patch().logical_error_rate();
        let magic_states = self.toffoli_factory_part().map_or(0.0, |p| {
            self.num_magic_states(0)
                .to_f64()
                .expect("can convert number of magic states as f64")
                * p.factory().error_probability()
        });

        logical + magic_states
    }
}

impl Deref for AliceAndBobEstimates {
    type Target = PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>>
    for AliceAndBobEstimates
{
    fn from(
        value: PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>,
    ) -> Self {
        Self(value)
    }
}

impl Display for AliceAndBobEstimates {
    // print the final estimates
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,)?;
        writeln!(f, "─────────────────────────────")?;
        writeln!(f, "#physical qubits:    {}", self.physical_qubits())?;
        writeln!(
            f,
            "runtime:             {:.2} hrs",
            f64::from_u64(self.runtime()).expect("runtime is not too large") / 1e9 / 3600.0
        )?;
        writeln!(f, "total error:         {:.5}", self.total_error())?;
        writeln!(f, "─────────────────────────────")?;
        writeln!(
            f,
            "code distance:       {}",
            self.logical_patch().code_parameter()
        )?;
        writeln!(
            f,
            "#factories:          {}",
            self.toffoli_factory_part().map_or(0, FactoryPart::copies)
        )?;
        writeln!(f, "factory fraction:    {:.2}%", self.factory_fraction())?;
        writeln!(f, "─────────────────────────────")
    }
}
