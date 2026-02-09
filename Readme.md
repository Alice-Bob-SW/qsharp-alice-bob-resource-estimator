[![ci](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml/badge.svg)](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml)

# Q# Resource Estimator for Alice & Bob's Architecture

TL;DR Provide logical resource counts from Q#, Qualtran, or explicit `(qubits, cx, ccx)` and get physical resource estimates for Alice & Bob's cat-qubit + repetition-code architecture.

This repository adapts the Microsoft Q# Resource Estimator to Alice & Bob's architecture. It estimates physical resources from Q# programs or explicit logical counts and exposes a Python API for analysis workflows.

The implementation follows the Q# Resource Estimator described in [arXiv:2311.05801](https://arxiv.org/abs/2311.05801) and the Alice & Bob architecture assumptions detailed in [arXiv:2302.06639](https://arxiv.org/abs/2302.06639). An elliptic-curve discrete logarithm example (from [arXiv:2302.06639](https://arxiv.org/abs/2302.06639)) is included, along with Qualtran-based tooling for deriving logical resources.

## Highlights
- Q# -> logical counts -> physical resource estimates
- Direct ISA-level entry with `(qubits, cx, ccx)` logical counts
- Seamless Qualtran interface and Shor/ECC-style example
- Cat-qubit + repetition-code architecture parameters baked into the estimator
- Python bindings for integration in notebooks and analysis workflows

## Inputs and Model
The estimator consumes logical-level resources:
- `N_L`: logical qubits
- `D_L`: logical depth (cycles)
- `N_T`: magic-state demand (Toffoli/CCX count)

These are calculated internally from `(qubits, cx, ccx)` by adding routing qubits and calculating logical cyles following the explanation given in [arXiv:2302.06639](https://arxiv.org/abs/2302.06639). You can provide `(qubits, cx, ccx)` in three ways:

1. **Qualtran Bloq**: Qualtran calculates `(qubits, cx, ccx)`
2. **Q# program**: the interpreter extracts `(qubits, cx, ccx)`
3. **Explicit resources**: you pass `(qubits, cx, ccx)` directly

## Outputs
The estimator returns physical-level estimates such as:
- Total physical qubits
- Runtime estimates
- Error budget usage
- Code distance and cat-qubit parameters
- Magic-state factory sizing and allocation

## Quickstart (Pixi)
This repo includes a `pixi.toml` for reproducible environments.

Install:
```bash
pixi install
pixi run python --version
```

Build the Python extension in the Pixi environment:
```bash
pixi run maturin develop --uv
```

To run the setup commands, you need Pixi installed on your machine. Pixi will create and manage the project environment for you, including installing Python, Rust, and required dependencies. You will also need a working native build toolchain for your operating system (Xcode Command Line Tools on macOS, build-essential on Linux, or Microsoft C++ Build Tools on Windows), since `maturin develop` compiles Rust extensions locally.

## Python Usage
The module name is `qsharp_alice_bob_resource_estimator`. The API exposes three main functions:
- `estimate_file_struct(...)`
- `estimate_resources_struct(...)`
- `elliptic_curve_estimate_struct(...)`

The additional arguments are
- `frontier` — If `true`, compute and return the Pareto frontier as structured objects.
- `error_total` — Overall error target; mutually exclusive with `error_budget`.
- `error_budget` — Tuple `(target, meas, routing)` for an explicit split.

Example: estimate from a Q# file.
```python
import qsharp_alice_bob_resource_estimator as qre

estimate, frontier, counts = qre.estimate_file_struct(
    "qsharp/Adder.qs",
    frontier=False,
    error_total=0.333,
    error_budget=None,
)
```

Example: estimate from explicit logical counts.
```python
estimate, frontier = qre.estimate_resources_struct(
    qubits=100,
    cx=2000,
    ccx=300,
    frontier=True,
    error_total=0.333,
    error_budget=None,
)
```

Example: elliptic-curve discrete log estimates.
```python
estimate, frontier = qre.elliptic_curve_estimate_struct(
    bit_size=256,
    window_size=18,
    frontier=True,
)
```

## Qualtran Integration
Qualtran primitives for Shor/ECC resource derivations live in `ecc_primitives/`. A wrapper is provided that takes as an input any arbitrary Qualtran Bloq and outputs the resource estimation.

Example: ECC from Qualtran
```python
# Import Qualtran primitives
from ecc_primitives.construction_helpers import (
    ECCInstance,
    create_ecc_circuit,
)
from qualtran_helpers.qualtran_wrapper import estimate_from_qualtran

# Initialize configuration for secp256k1 curve
cfg = ECCInstance(
    n=256,  # bitsize
    p=2**256 - 2**32 - 977,  # prime
    wa=18,  # window size for elliptic curve multiplication
    wm=6,  # window size for the integer multiplication
    Gx=55066263022277343669578718895168534326250603453777594175500187360389116729240,  # x coordinate of the generator
    Gy=32670510020758816978083085130507043184471273380659243275938904335757337482424,  # y coordinate of the generator
    scalar=2026,  # private key to find
)

# Create bloq
ecc_circuit = create_ecc_circuit(cfg)

# Perform resource estimation
ecc_result = estimate_from_qualtran(
    ecc_circuit,
    frontier=False,
    error_total=None,
    error_budget=(0.333 * 0.5, 0.333 * 0.5, 0.0),
)
```

## Notebook
See `getting_started.ipynb` for an end-to-end walkthrough.

## Assumptions
This estimator applies the logical-to-physical mapping described in:
- [arXiv:2311.05801](https://arxiv.org/abs/2311.05801) for the base Q# Resource Estimator model
- [arXiv:2302.06639](https://arxiv.org/abs/2302.06639) for Alice & Bob cat-qubit architecture parameters

## Acknowledgements
Thanks to Mathias Soeken for the initial repository and for rebuilding the Q# resource estimator to support Alice & Bob's architecture.

## Contact
For questions not addressed here, please contact Élie Gouzien.
