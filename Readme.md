[![ci](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml/badge.svg)](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml)

# Q# Resource Estimator for Alice & Bob's Architecture

TL;DR Provide algorithmic resource counts from Q#, Qualtran, or explicit `(qubits, cx, ccx)` and get physical resource estimates for Alice & Bob's cat qubit + repetition code architecture.

This repository relies on the Microsoft Azure Resource Estimator for Alice & Bob's architecture, using it as a rust library.
It estimates physical resources from Q# or Qualtran programs or explicit algorithmic counts and exposes a Rust library, a command-line interface and a Python API for analysis workflows.

The implementation follows the Azure Resource Estimator described in [arXiv:2311.05801](https://arxiv.org/abs/2311.05801) and the Alice & Bob architecture assumptions detailed in [arXiv:2302.06639](https://arxiv.org/abs/2302.06639).
An elliptic-curve discrete logarithm example (from [arXiv:2302.06639](https://arxiv.org/abs/2302.06639)) is included, along with Qualtran-based tooling for deriving logical resources.

## Highlights
- Input algorithm description from Q#, Qualtran or `(qubits, cx, ccx)` + target total error rates.
- Cat qubit + repetition code architecture.
- Code parameters (distance and average number of photons) and magic state factory choice optimized.
- Rust library, command-line interface and Python bindings.

## Inputs and Model
The estimator consumes logical-level resources:
- `N_L`: logical qubits
- `D_L`: logical depth (cycles)
- `N_T`: magic-state demand (Toffoli/CCX count)

These are calculated internally from the algorithmic resource counts `(qubits, cx, ccx)` by adding routing qubits, factory qubits and calculating logical cyles following the explanation given in [arXiv:2302.06639](https://arxiv.org/abs/2302.06639).
You can provide `(qubits, cx, ccx)` in three ways:
1. **Qualtran Bloq**: Qualtran calculates `(qubits, cx, ccx)`
2. **Q# program**: the interpreter extracts `(qubits, cx, ccx)`
3. **Explicit resources**: you pass `(qubits, cx, ccx)` directly

## Outputs
The estimator returns physical-level estimates such as:
- Total physical qubits
- Runtime estimates
- Total error rate
- Code distance and cat-qubit parameters
- Magic state factory code distance and average photon number
- Number of magic state factories
- Percentage of qubits involved in magic state production

## Quickstart (Pixi)
This repo includes a `pixi.toml` for reproducible environments.

To run the setup commands, you’ll need Pixi installed on your machine. This can be easily done by following the installation guide [here](https://pixi.prefix.dev/latest/installation/). Pixi will create and manage the project environment for you, including installing Python, Rust, and any required dependencies defined in the project configuration. In addition, you’ll need a working native build toolchain for your operating system (Xcode Command Line Tools on macOS, build-essential on Linux, or Microsoft C++ Build Tools on Windows), since maturin develop compiles Rust extensions locally.

Install:
```bash
pixi install
pixi run python --version
```

## Installation
TODO: to review
This repository includes a pixi.toml file to enable reproducible environments.

To install the required environment and the Python extension, run the following commands:

To run the setup commands, you’ll need Pixi installed on your machine. This can be easily done by following the installation guide here. Pixi will create and manage the project environment for you, including installing Python, Rust, and any required dependencies defined in the project configuration. In addition, you’ll need a working native build toolchain for your operating system (Xcode Command Line Tools on macOS, build-essential on Linux, or Microsoft C++ Build Tools on Windows), since maturin develop compiles Rust extensions locally.

## Python Usage
See `getting_started.ipynb` for an end-to-end walkthrough.

The python package is named `anb_estimator`. The API exposes three main functions:
- `estimate_qsharp_file(...)`
- `estimate_from_qualtran(...)`
- `estimate_logical_counts(...)`

The additional arguments are
- `frontier` — If `true`, compute and return the Pareto frontier as structured objects.
- `error_total` — Overall error target; mutually exclusive with `error_budget`.
- `error_budget` — Tuple `(target, meas, routing)` for an explicit split.

### Example: estimate from a Q# file.
```python
import anb_estimator

filename = "../qsharp/Adder.qs"
single_qsharp, frontier, counts = anb_estimator.estimate_qsharp_file(
    filename, frontier=True, error_total=None, error_budget=(0.0005, 0.0005, 0.0)
)
```

### Example: ECC from Qualtran

Qualtran primitives for Shor/ECC resource derivations live in `ecc_primitives/`. A wrapper is provided that takes as an input any arbitrary Qualtran Bloq and outputs the resource estimation.
```python
import anb_estimator

# Import Qualtran primitives
from ecc_primitives.construction_helpers import (
    ECCInstance,
    create_ecc_circuit,
)

# Initialize secp256k1 curve parameters and window sizes for the ECC instance (see arXiv:2302.06639)
cfg = ECCInstance(
    n=256,  # bitsize
    p=2**256 - 2**32 - 977,  # prime
    wa=18,  # window size for elliptic curve multiplication
    wm=6,  # window size for the integer multiplication
    Gx=55066263022277343669578718895168534326250603453777594175500187360389116729240,  # x coordinate of the generator
    Gy=32670510020758816978083085130507043184471273380659243275938904335757337482424,  # y coordinate of the generator
    scalar=2026,  # private key to find
)

# Create Qualtran bloq
ecc_circuit = create_ecc_circuit(cfg)

# Perform resource estimation
ecc_result = anb_estimator.estimate_from_qualtran(
    ecc_circuit,
    frontier=False,
    error_total=None,
    error_budget=(0.333 * 0.5, 0.333 * 0.5, 0.0),
)
```

### Example: estimate from explicit logical counts.
```python
import anb_estimator

estimate, frontier = anb_estimator.estimate_logical_counts(
    qubits=100,
    cx=2000,
    ccx=300,
    frontier=True,
    error_total=0.333,
    error_budget=None,
)
```

## Assumptions
This estimator applies the logical-to-physical mapping described in:
- [arXiv:2311.05801](https://arxiv.org/abs/2311.05801) for the base Q# Resource Estimator model
- [arXiv:2302.06639](https://arxiv.org/abs/2302.06639) for Alice & Bob cat qubit architecture parameters

## Authors
- Mathias Soeken (initial version of the repository)
- Élie Gouzien
- Nicholas Gialouris
- Axel Pappalardo

## Acknowledgements
Thanks to Mathias Soeken for the initial repository and for rebuilding the Q# resource estimator to support any architecture, including Alice & Bob's one.

