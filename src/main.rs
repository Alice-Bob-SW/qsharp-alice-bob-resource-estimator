// Copyright (c) Microsoft Corporation.
// Copyright (c) Alice & Bob
// Licensed under the MIT License.

#![warn(missing_docs)]
//! Estimate the ressources required for Elliptic Curve Cryptography (ECC) on a cat-based quantum
//! processor.
//!
//! Author: Mathias Soeken
//!
//! Based on É. Gouzien et al.'s article (<https://arxiv.org/abs/2302.06639>) and code
//! (<https://github.com/ElieGouzien/elliptic_log_cat/tree/master>).
//!
//! <b>Inputs:</b><br>
//! <pre>
//! - Qubit parameters (qubit.rs):
//!      * k₁_k₂ = ratio one photon/two photon losses (1e-5 hardcoded)
//! - Gates parameters (factories.rs):
//!      * t = single physical gate time (100 ns hardcoded). Same value assumed for state preparation, measurement, CNOT and Toffoli
//!      * gate_time ∝ time steps (89.2 time steps hardcoded)
//! - Repetition code parameters (code.rs):
//!      * (κ₁/κ₂)_th: fault tolerance threshold (0.013 hardcoded)
//! </pre>
//! <b>Outputs:</b>
//! <pre>
//! - # of physical cat qubits
//! - Runtime
//! - Total error probability
//! - Repetition code distance & # of photons
//! - Ffraction of qubits assigned to the magic state factory
//! </pre>

use std::rc::Rc;

use code::RepetitionCode;
use counter::LogicalCounts;
use estimates::AliceAndBobEstimates;
use factories::ToffoliBuilder;
use qubit::CatQubit;
use resource_estimator::estimates::{ErrorBudget, PhysicalResourceEstimation};

/// Repetition code for biased error correction with a focus on phase flips
mod code;
/// Computes logical space-time volume overhead for resource estimation from Q#
/// files or formulas for ECC application
mod counter;
/// Convenience structure to display resource estimation results
mod estimates;
/// Toffoli magic state factories
mod factories;
/// Model for cat qubits
mod qubit;

fn main() -> Result<(), anyhow::Error> {
    // ECC pre-computed counts
    // -----------------------

    // This value can be changed to investigate other key sizes, e.g., those in
    // arXiv:2302.06639 (Table IV, p. 37)
    let bit_size = 256;
    // Window size for modular exponentiation (arXiv:2001.09580, sec 4.1, p. 6)
    // Value w_e as reported in arXiv:2302.06639 (Table IV, p. 37)
    let window_size = 18;

    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let overhead = Rc::new(LogicalCounts::from_elliptic_curve_crypto(
        bit_size,
        window_size,
    ));
    let budget = ErrorBudget::new(0.333 * 0.5, 0.333 * 0.5, 0.0);

    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, overhead, budget);
    let result: AliceAndBobEstimates = estimation.estimate()?.into();
    println!("Estimates from precomputed logical count (elliptic curve discrete logarithm):");
    println!("{result}");

    println!("----------------------------------------");
    println!("Exploration of good estimates from precomputed logical count (elliptic curve discrete logarithm):");
    let results = estimation.build_frontier()?;

    for r in results {
        println!("{}", AliceAndBobEstimates::from(r));
    }
    println!("----------------------------------------");

    // Resource estimation from Q#
    // ---------------------------

    let filename = format!("{}/qsharp/Adder.qs", env!("CARGO_MANIFEST_DIR"));

    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let overhead = Rc::new(LogicalCounts::from_qsharp(filename).map_err(anyhow::Error::msg)?);
    let budget = ErrorBudget::new(0.001 * 0.5, 0.001 * 0.5, 0.0);

    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, overhead, budget);
    let result: AliceAndBobEstimates = estimation.estimate()?.into();
    println!("Resource estimate from Q# code (ripple-carry adder):");
    println!("{result}");

    Ok(())
}
