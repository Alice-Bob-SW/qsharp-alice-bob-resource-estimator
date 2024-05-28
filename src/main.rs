// Copyright (c) Alice & Bob
// Licensed under the Apache License.

use clap::{Args, Parser, Subcommand};
use std::rc::Rc;

use qsharp_alice_bob_resource_estimator::{
    AliceAndBobEstimates, CatQubit, LogicalCounts, RepetitionCode, ToffoliBuilder,
};
use resource_estimator::estimates::{ErrorBudget, PhysicalResourceEstimation};

/// Resource estimator for Alice & Bob's architecture (cats + repetition code).
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Show the frontier of reasonable parameter sets instead of a single result
    #[arg(short, long)]
    frontier: bool,

    #[command(flatten)]
    budget: Budget,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
struct Budget {
    /// Overall error budget (equally split between topological and magic state errors)
    #[arg(long, value_name = "ERROR_PROBA")]
    error_total: Option<f64>,

    /// Detailled error budget
    #[arg(long, num_args = 3, value_names = ["TOPOLOGICAL_ERROR", "MAGIC_ERROR", "ROTATION_ERROR"])]
    error_budget: Option<Vec<f64>>,
}

#[derive(Subcommand)]
enum Commands {
    /// Reads a Q# file
    File {
        /// Path to the Q# file
        filename: String,
    },
    /// Computes from listed resources
    Resources {
        /// Logical qubit number
        qubits: u64,
        /// Number of controlled-not gates
        cx: u64,
        /// Number of Toffoli gates
        ccx: u64,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let budget = match (args.budget.error_total, args.budget.error_budget) {
        (Some(proba), None) => ErrorBudget::new(proba * 0.5, proba * 0.5, 0.0),
        (None, Some(vec)) => ErrorBudget::new(vec[0], vec[1], vec[2]),
        (None, None) => ErrorBudget::new(0.333 * 0.5, 0.333 * 0.5, 0.0),
        _ => unreachable!("Clap should have catched that!"),
    };

    let overhead = Rc::new(match args.command {
        Commands::File { filename } => {
            LogicalCounts::from_qsharp(filename).map_err(anyhow::Error::msg)?
        }
        Commands::Resources { qubits, cx, ccx } => LogicalCounts::new(qubits, cx, ccx),
    });
    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, overhead, budget);

    if args.frontier {
        let results = estimation.build_frontier()?;
        for r in results {
            println!("{}", AliceAndBobEstimates::from(r));
        }
    } else {
        let result: AliceAndBobEstimates = estimation.estimate()?.into();
        println!("{result}");
    }

    Ok(())
}
