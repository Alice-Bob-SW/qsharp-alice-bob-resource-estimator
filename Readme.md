[![ci](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml/badge.svg)](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml)

# Q# Resource Estimator for Alice & Bob's Architecture

This repository adapts the Microsoft Q# Resource Estimator to Alice & Bob's cat-qubit architecture with repetition code. It estimates physical resources from either Q# programs or explicit logical counts, and can be used from a Python API.

The implementation follows the Q# Resource Estimator described in [arXiv:2311.05801](https://arxiv.org/abs/2311.05801) and the Alice & Bob architecture assumptions detailed in [arXiv:2302.06639](https://arxiv.org/abs/2302.06639). An elliptic-curve discrete logarithm example (from [arXiv:2302.06639](https://arxiv.org/abs/2302.06639)) is included, along with Qualtran-based tooling for deriving logical resources.

## Highlights
- Q# -> logical counts -> physical resource estimates
- Direct ISA-level entry with `(N_L, D_L, N_T)` logical counts
- Cat-qubit + repetition-code architecture parameters baked into the estimator
- Python bindings for integration in notebooks and analysis workflows
- Qualtran primitives and tests for Shor/ECC-style resource derivations

## Architecture and Inputs
The estimator consumes logical-level resources:
- `N_L`: logical qubits
- `D_L`: logical depth (cycles)
- `N_T`: magic-state demand (Toffoli/CCX count)

You can provide these in two ways:
1. **Q# program**: the interpreter extracts logical counts.
2. **Explicit resources**: you pass `(qubits, cx, ccx)` directly.

It returns physical-level estimates such as total qubits, runtime, error budget usage, code distance, cat parameters, and factory sizing/fraction.

## Quickstart (Pixi)
This repo includes a `pixi.toml` for reproducible environments.

```bash
pixi install
pixi run python --version
```

To build the Python extension in the Pixi environment:

```bash
pixi run maturin develop --uv
```

## Python Usage
The module name is `qsharp_alice_bob_resource_estimator`. The API exposes three main functions:
- `estimate_file_struct(...)`
- `estimate_resources_struct(...)`
- `elliptic_curve_estimate_struct(...)`

Example: estimate from a Q# file.

```python
import qsharp_alice_bob_resource_estimator as qre

estimate, frontier, counts = qre.estimate_file_struct(
    "qsharp/Adder.qs",
    frontier=False,
    error_total=0.333,
    error_budget=None,
)

print(estimate)
print(counts.qubit_count, counts.cx_count, counts.ccx_count)
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
Qualtran primitives for Shor/ECC resource derivations live in `qualtran_primitives/`. These can be used to derive logical counts at the ISA level, then fed into the estimator.

## Notebook
See `getting_started.ipynb` for an end-to-end walkthrough.

## Acknowledgements
Thanks to Mathias Soeken for the initial repository and for rebuilding the Q# resource estimator to support Alice & Bob's architecture.

## Contact
For questions not addressed here, please contact Élie Gouzien.
