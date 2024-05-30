// Copyright (c) Microsoft Corporation.
// Copyright (c) Alice & Bob
// Licensed under the MIT License.

#![warn(missing_docs)]
//! Estimate the resources required for Elliptic Curve Cryptography (ECC) on a cat-based quantum
//! processor.
//!
//! Author: Mathias Soeken
//!
//! Based on É. Gouzien et al.'s article (<https://arxiv.org/abs/2302.06639>) and code
//! (<https://github.com/ElieGouzien/elliptic_log_cat/tree/master>).
//!
//! <b>Takes:</b><br>
//! <pre>
//! - A Q# file giving the required number of logical qubits for the algorithm (qsharp/Adder.qs)
//! - Qubit parameters (qubit.rs):
//!      * k₁_k₂ = ratio one photon/two photon losses (1e-5 hardcoded)
//! - Gates parameters (factories.rs):
//!      * t = single physical gate time (100 ns hardcoded). Same value assumed for state preparation, measurement, CNOT and Toffoli
//!      * gate_time ∝ time steps (89.2 time steps hardcoded)
//! - Repetition code parameters (code.rs):
//!      * (κ₁/κ₂)_th: fault tolerance threshold (0.013 hardcoded)
//! </pre>
//! <b>Provides:</b>
//! <pre>
//! - # of physical cat qubits
//! - Runtime
//! - Total error probability
//! - Repetition code distance & # of photons
//! - Fraction of qubits assigned to the magic state factory
//! </pre>

pub use code::RepetitionCode;
pub use counter::LogicalCounts;
pub use estimates::AliceAndBobEstimates;
pub use factories::ToffoliBuilder;
pub use qubit::CatQubit;

/// Repetition code for biased error correction with a focus on phase flips
pub mod code;
/// Computes logical space-time volume overhead for resource estimation from Q#
/// files or formulas for ECC application
pub mod counter;
/// Convenience structure to display resource estimation results
pub mod estimates;
/// Toffoli magic state factories
pub mod factories;
/// Model for cat qubits
pub mod qubit;
