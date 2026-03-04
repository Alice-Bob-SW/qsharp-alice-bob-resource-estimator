// Copyright (c) Alice & Bob.
// Licensed under the MIT License.

//! Convenience structure to display LDPC resource estimation results.

use std::{fmt::Display, ops::Deref};

use num_traits::{FromPrimitive, ToPrimitive};
use resource_estimator::estimates::{FactoryPart, Overhead, PhysicalResourceEstimationResult};

use crate::{
    factories::LdpcToffoliFactory, ldpc_code::LdpcCode, ldpc_overhead::LdpcOverhead,
};

/// Represents a physical resources estimate for LDPC code architecture.
pub struct LdpcEstimates(
    PhysicalResourceEstimationResult<LdpcCode, LdpcToffoliFactory, LdpcOverhead>,
);

impl LdpcEstimates {
    #[must_use]
    fn toffoli_factory_part(&self) -> Option<&FactoryPart<LdpcToffoliFactory>> {
        self.factory_parts()[0].as_ref()
    }

    #[must_use]
    /// Count the number of physical qubits.
    ///
    /// For LDPC codes, routing is handled within the computing layer, so no
    /// additional vertical routing qubits are needed. However, we still need
    /// to account for the factory qubits having their own routing overhead.
    pub fn physical_qubits(&self) -> u64 {
        // LDPC blocks already include ancilla qubits for routing/syndrome extraction.
        // Factory routing: each factory copy needs 1 routing qubit worth of overhead
        // using repetition code internally.
        let factory_routing = self
            .toffoli_factory_part()
            .map_or(0, |p| p.copies() * 2);

        self.0.physical_qubits() + factory_routing
    }

    #[must_use]
    /// Compute the percentage of physical qubits allocated to factories.
    pub fn factory_fraction(&self) -> f64 {
        (self
            .physical_qubits_for_factories()
            .to_f64()
            .expect("can't convert")
            / self.physical_qubits().to_f64().expect("can't convert"))
            * 100.0
    }

    #[must_use]
    /// Compute the total error of the computation.
    pub fn total_error(&self) -> f64 {
        let logical = (self.num_cycles() * self.layout_overhead().logical_qubits())
            .to_f64()
            .expect("can't convert volume as f64")
            * self.logical_patch().logical_error_rate();
        let magic_states = self.toffoli_factory_part().map_or(0.0, |p| {
            self.num_magic_states(0)
                .to_f64()
                .expect("can't convert number of magic states as f64")
                * p.factory().error_probability()
        });

        logical + magic_states
    }
}

impl Deref for LdpcEstimates {
    type Target = PhysicalResourceEstimationResult<LdpcCode, LdpcToffoliFactory, LdpcOverhead>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PhysicalResourceEstimationResult<LdpcCode, LdpcToffoliFactory, LdpcOverhead>>
    for LdpcEstimates
{
    fn from(
        value: PhysicalResourceEstimationResult<LdpcCode, LdpcToffoliFactory, LdpcOverhead>,
    ) -> Self {
        Self(value)
    }
}

impl Display for LdpcEstimates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "─────────────────────────────")?;
        writeln!(f, "# physical qubits:    {}", self.physical_qubits())?;
        writeln!(
            f,
            "runtime:             {:.2} hrs",
            f64::from_u64(self.runtime()).expect("runtime is too large") / 1e9 / 3600.0
        )?;
        writeln!(f, "total error:         {:.5}", self.total_error())?;
        writeln!(f, "─────────────────────────────")?;
        writeln!(
            f,
            "code:                LDPC {}",
            self.logical_patch().code_parameter()
        )?;
        writeln!(
            f,
            "#factories:          {}",
            self.toffoli_factory_part().map_or(0, FactoryPart::copies)
        )?;
        writeln!(
            f,
            "factories distance:  {}",
            self.toffoli_factory_part()
                .expect("No factory part")
                .factory()
        )?;
        writeln!(f, "factory fraction:    {:.2}%", self.factory_fraction())?;
        writeln!(f, "─────────────────────────────")
    }
}
