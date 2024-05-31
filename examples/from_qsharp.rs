// Copyright (c) Microsoft Corporation.
// Copyright (c) Alice & Bob
// Licensed under the MIT License.

#![warn(missing_docs)]
//! Estimate the ressources required for an adder, the adder being specified by a Q# file.

use std::rc::Rc;

use qsharp_alice_bob_resource_estimator::{
    AliceAndBobEstimates, CatQubit, LogicalCounts, RepetitionCode, ToffoliBuilder,
};
use resource_estimator::estimates::{ErrorBudget, PhysicalResourceEstimation};

fn main() -> Result<(), anyhow::Error> {
    // Resource estimation from Q#
    // ---------------------------

    let filename = format!("{}/qsharp/Adder.qs", env!("CARGO_MANIFEST_DIR"));

    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let count = LogicalCounts::from_qsharp(filename).map_err(anyhow::Error::msg)?;
    let budget = ErrorBudget::new(0.001 * 0.5, 0.001 * 0.5, 0.0);

    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, Rc::new(count), budget);
    let result: AliceAndBobEstimates = estimation.estimate()?.into();
    println!("Resource estimate from Q# code (ripple-carry adder):");
    println!("{result}");

    Ok(())
}
