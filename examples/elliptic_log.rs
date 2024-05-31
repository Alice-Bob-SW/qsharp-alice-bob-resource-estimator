// Copyright (c) Microsoft Corporation.
// Copyright (c) Alice & Bob
// Licensed under the MIT License.

#![warn(missing_docs)]
//! Estimate the ressources required for Elliptic Curve Cryptography (ECC) on a cat-based quantum
//! processor.
//!
//! Based on É. Gouzien et al.'s article (<https://arxiv.org/abs/2302.06639>) and code
//! (<https://github.com/ElieGouzien/elliptic_log_cat/tree/master>).

use std::rc::Rc;

use qsharp_alice_bob_resource_estimator::{RepetitionCode, LogicalCounts, AliceAndBobEstimates, ToffoliBuilder, CatQubit};
use resource_estimator::estimates::{ErrorBudget, PhysicalResourceEstimation};

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

    Ok(())
}
