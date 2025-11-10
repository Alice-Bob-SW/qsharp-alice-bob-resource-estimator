// Copyright (c) Microsoft Corporation.
// Copyright (c) Alice & Bob
// Licensed under the MIT License.

#![warn(missing_docs)]
//! Estimate the resources required for Elliptic Curve Cryptography (ECC) on a
//! cat-based quantum processor.
//!
//! Based on É. Gouzien et al.'s article (<https://arxiv.org/abs/2302.06639>)
//! and code (<https://github.com/ElieGouzien/elliptic_log_cat/tree/master>).

// src/examples.rs

use std::rc::Rc;
use anyhow::Result;
use crate::{AliceAndBobEstimates, CatQubit, LogicalCounts, RepetitionCode, ToffoliBuilder};
use resource_estimator::estimates::{ErrorBudget, PhysicalResourceEstimation};

#[allow(clippy::similar_names)]
fn elliptic_curve_crypto_count(bit_size: u64, window_size: u64) -> LogicalCounts {
    let qubit_count = 9 * bit_size + window_size + 4;
    let cx_count = (448 * bit_size.pow(3)).div_ceil(window_size);
    let ccx_count = (348 * bit_size.pow(3)).div_ceil(window_size);
    LogicalCounts::new(qubit_count, cx_count, ccx_count)
}

/// Run the ECC example and return the output lines (instead of printing).
pub fn run_ecc_example(bit_size: u64, window_size: u64) -> Result<Vec<String>> {
    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let count = elliptic_curve_crypto_count(bit_size, window_size);
    let budget = ErrorBudget::new(0.333 * 0.5, 0.333 * 0.5, 0.0);

    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, Rc::new(count), budget);

    let mut out = Vec::new();

    let result: AliceAndBobEstimates = estimation.estimate()?.into();
    out.push("Estimates from pre-computed logical count (elliptic curve discrete logarithm):".into());
    out.push(format!("{result}"));

    out.push("----------------------------------------".into());
    out.push("Exploration of good estimates from pre-computed logical count (elliptic curve discrete logarithm):".into());

    let results = estimation.build_frontier()?;
    for r in results {
        out.push(AliceAndBobEstimates::from(r).to_string());
    }

    Ok(out)
}

/// Compute ECC estimates and return the core result(s), not strings.
/// If `frontier == false`, the frontier vec is empty.
pub fn run_ecc_example_struct(
    bit_size: u64,
    window_size: u64,
    frontier: bool,
) -> Result<(AliceAndBobEstimates, Vec<AliceAndBobEstimates>)> {
    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let count = elliptic_curve_crypto_count(bit_size, window_size);
    let budget = ErrorBudget::new(0.333 * 0.5, 0.333 * 0.5, 0.0);

    let est = PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, Rc::new(count), budget);

    let single: AliceAndBobEstimates = est.estimate()?.into();

    let frontier_vec = if frontier {
        est.build_frontier()?.into_iter().map(Into::into).collect()
    } else {
        Vec::new()
    };

    Ok((single, frontier_vec))
}
