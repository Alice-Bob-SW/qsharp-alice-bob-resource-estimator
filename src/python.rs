//! Python bindings for the Alice & Bob Q# resource estimator.
//! ! This module exposes functions to estimate quantum resources from Q# programs
//! ! and explicit logical counts, returning structured results suitable for Python consumers.
//! ! It leverages PyO3 to create Python-callable functions and classes.

use pyo3::prelude::*; // brings Python, PyResult, PyModule, Bound, etc.
use std::fmt;
use std::rc::Rc;

use num_format::{Locale, ToFormattedString};

use crate::estimates::make_budget;
use crate::{AliceAndBobEstimates, CatQubit, LogicalCounts, RepetitionCode, ToffoliBuilder};
use resource_estimator::estimates::PhysicalResourceEstimation;

/// Python-visible snapshot of logical counts extracted from a Q# program.
///
/// Exposes a minimal, read-only view sufficient for downstream analysis in Python.
/// Fields correspond to logical resources observed by the interpreter.
#[pyclass(frozen)]
pub struct LogicalCountsPy {
    /// Number of (algorithm) logical qubits allocated by the interpreter.
    #[pyo3(get)]
    qubit_count: u64,
    /// Number of logical CX-equivalent two-qubit gates (including CY/CZ and SWAP→3×CX).
    #[pyo3(get)]
    cx_count: u64,
    /// Number of logical CCX (Toffoli) gates.
    #[pyo3(get)]
    ccx_count: u64,
}

/// Converts an internal [`LogicalCounts`] reference into a Python-visible [`LogicalCountsPy`].
///
/// Copies only primitive fields; no heap sharing is required.
impl From<&LogicalCounts> for LogicalCountsPy {
    fn from(c: &LogicalCounts) -> Self {
        Self {
            qubit_count: c.qubit_count,
            cx_count: c.cx_count,
            ccx_count: c.ccx_count,
        }
    }
}

/// Estimate resources from a Q# file and return both the best estimate and, optionally,
/// a frontier of Pareto-optimal trade-offs, together with the parsed logical counts.
///
/// # Arguments
/// - `filename` — Path to a Q# source file to be parsed and interpreted for counts.
/// - `frontier` — If `true`, also compute a frontier of estimates (e.g., different distances/α).
/// - `error_total` — Overall error target `p_total`; mutually exclusive with `error_budget`.
/// - `error_budget` — Tuple `(logical, magic_state, rotation)` if an explicit split is desired.
///
/// # Returns
/// A 3-tuple:
/// 1. `EstimatesPy` — the single best estimate,
/// 2. `Vec<EstimatesPy>` — optionally, the frontier (empty if `frontier == false`),
/// 3. `LogicalCountsPy` — Python snapshot of the logical counts extracted from `filename`.
///
/// # Errors
/// - I/O or parsing failures when loading the Q# file,
/// - Failures during resource estimation.
///
#[pyfunction]
fn _estimate_qsharp_file(
    filename: &str,
    frontier: bool,
    error_total: Option<f64>,
    error_budget: Option<(f64, f64, f64)>,
) -> PyResult<(EstimatesPy, Vec<EstimatesPy>, LogicalCountsPy)> {
    // Build the estimation
    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let budget = make_budget(error_total, error_budget)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    // Put counts behind an Rc so we can both pass it into PRE and also derive a Python view
    let counts = std::rc::Rc::new(
        LogicalCounts::from_qsharp(filename)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.clone()))?,
    );
    let counts_py = LogicalCountsPy::from(counts.as_ref());

    let estimation = PhysicalResourceEstimation::new(
        qec,
        std::rc::Rc::new(qubit),
        builder,
        counts.clone(), // share with PRE
        budget,
    );

    // Single best estimate
    let single_est: AliceAndBobEstimates = estimation
        .estimate()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .into();
    let single_py = EstimatesPy::from(&single_est);

    // Optional frontier
    let mut frontier_py = Vec::new();
    if frontier {
        let results = estimation
            .build_frontier()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        frontier_py = results
            .into_iter()
            .map(|r| {
                let est: AliceAndBobEstimates = r.into();
                EstimatesPy::from(&est)
            })
            .collect();
    }

    Ok((single_py, frontier_py, counts_py))
}

/// Estimate resources from explicit logical counts and return typed results,
/// optionally including a frontier of trade-offs.
///
/// # Arguments
/// - `qubits` — Logical (algorithm) qubit count.
/// - `cx` — Logical CX-equivalent two-qubit gate count.
/// - `ccx` — Logical CCX (Toffoli) gate count.
/// - `frontier` — If `true`, compute and return the frontier as structured objects.
/// - `error_total` — Overall error target; mutually exclusive with `error_budget`.
/// - `error_budget` — Tuple `(topological error budget, magic state error budget,
///    rotation error budget)` for an explicit split.
///
/// # Returns
/// A tuple:
/// 1. `EstimatesPy` — single best estimate,
/// 2. `Vec<EstimatesPy>` — frontier (empty if `frontier == false`).
///
/// # Errors
/// Propagates errors from the physical resource estimator.
#[pyfunction]
fn _estimate_logical_counts(
    // TODO: remove duplication between here and main.rs.
    qubits: u64,
    cx: u64,
    ccx: u64,
    frontier: bool,
    error_total: Option<f64>,
    error_budget: Option<(f64, f64, f64)>,
) -> PyResult<(EstimatesPy, Vec<EstimatesPy>)> {
    // Build the estimation
    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let budget = make_budget(error_total, error_budget)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let counts = LogicalCounts::new(qubits, cx, ccx);
    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, Rc::new(counts), budget);

    // Single best estimate
    let single_est: AliceAndBobEstimates = estimation
        .estimate()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .into();
    let single_py = EstimatesPy::from(&single_est);

    // Optional frontier
    let mut frontier_py = Vec::new();
    if frontier {
        let results = estimation
            .build_frontier()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        for r in results {
            let est: AliceAndBobEstimates = r.into();
            frontier_py.push(EstimatesPy::from(&est));
        }
    }

    Ok((single_py, frontier_py))
}

/// Python class that encapsulates a single resource-estimation result.
///
/// Exposed fields:
/// - `physical_qubits`
/// - `runtime_seconds`
/// - `runtime_hours`
/// - `total_error`
/// - `code_distance`
/// - `code_alpha2`
/// - `factories`
/// - `factories_distance`
/// - `factories_alpha2`
/// - `factory_fraction_percent`
/// - `factory_fraction`
#[pyclass(frozen)]
#[derive(Clone)]
pub struct EstimatesPy {
    // TODO: pythonyse AliceAndBobEstimates rather than making a new one.
    #[pyo3(get)]
    pub physical_qubits: u64,
    #[pyo3(get)]
    pub runtime_seconds: f64,
    #[pyo3(get)]
    pub runtime_hours: f64,
    #[pyo3(get)]
    pub total_error: f64,

    // logical patch
    #[pyo3(get)]
    pub code_distance: u64,
    #[pyo3(get)]
    pub code_alpha2: f64,

    // factories
    #[pyo3(get)]
    pub factories: u64,
    #[pyo3(get)]
    pub factories_distance: u64,
    #[pyo3(get)]
    pub factories_alpha2: f64,

    // fractions
    #[pyo3(get)]
    pub factory_fraction_percent: f64,
    #[pyo3(get)]
    pub factory_fraction: f64,
}

impl fmt::Display for EstimatesPy {
    // TODO: use the same printing for rust and python.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // matches Python's "{:,}" behavior
        let qubits = self.physical_qubits.to_formatted_string(&Locale::fr);

        // Use separator line
        let sep = "─────────────────────────────";

        // Write the full multi-line string
        write!(
            f,
            "Parameters obtained from the Rust resource estimator\n{}\n\
             # physical qubits:    {}\n\
             runtime:             {:.2} hrs\n\
             total error:         {:.5}\n\
             {}\n\
             code distance:       {} (|α|² = {:.2})\n\
             #factories:          {}\n\
             factories distance:  {} (|α|² = {:.2})\n\
             factory fraction:    {:.2}%\n\
             {}\n",
            sep,
            qubits,
            self.runtime_hours,
            self.total_error,
            sep,
            self.code_distance,
            self.code_alpha2,
            self.factories,
            self.factories_distance,
            self.factories_alpha2,
            self.factory_fraction_percent,
            sep,
        )
    }
}

#[pymethods]
impl EstimatesPy {
    fn __str__(&self) -> String {
        self.to_string()
    }
}

/// Converts a core `AliceAndBobEstimates` value into a Python-friendly [`EstimatesPy`].
impl From<&crate::AliceAndBobEstimates> for EstimatesPy {
    fn from(e: &crate::AliceAndBobEstimates) -> Self {
        Self {
            physical_qubits: e.physical_qubits(),
            runtime_seconds: e.runtime_seconds(),
            runtime_hours: e.runtime_hours(),
            total_error: e.total_error(),
            code_distance: e.code_distance(),
            code_alpha2: e.code_alpha2(),
            factories: e.factories(),
            factories_distance: e.factories_distance(),
            factories_alpha2: e.factories_alpha2(),
            factory_fraction_percent: e.factory_fraction(),
            factory_fraction: e.factory_fraction_ratio(),
        }
    }
}

/// Python module entry point for the Alice & Bob Q# resource estimator bindings.
///
/// Registers user-facing functions that load Q# programs, accept explicit logical counts,
/// in both pretty-printed and structured forms.
///
/// # Exposed callables
/// - `_estimate_qsharp_file(...)`
/// - `_estimate_logical_counts(...)`
///
/// # Errors
/// Any initialization failure is surfaced as a Python `RuntimeError`.
#[pymodule]
#[pyo3(name = "_native")]
fn qsharp_alice_bob_resource_estimator(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // functions
    m.add_function(wrap_pyfunction!(_estimate_qsharp_file, m)?)?;
    m.add_function(wrap_pyfunction!(_estimate_logical_counts, m)?)?;

    // classes
    m.add_class::<EstimatesPy>()?;
    m.add_class::<LogicalCountsPy>()?; // optional, but useful since you return it too

    Ok(())
}
