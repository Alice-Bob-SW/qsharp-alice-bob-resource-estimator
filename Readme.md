[![ci](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml/badge.svg)](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml)

Q# resource estimator for Alice & Bob's architecture
=====================================================

This project contains the code for using [Microsoft Q# resource estimator](https://github.com/microsoft/qsharp/tree/main/resource_estimator) (presented in [this paper](https://arxiv.org/abs/2311.05801)) for [Alice & Bob](https://alice-bob.com)'s architecture, using cat qubits and repetition code (LDPC codes might be added in the future).

Shor's algorithm for solving the elliptic curve discrete logarithm problem is used as an example, as in the paper [Phys. Rev. Lett. 131, 040602](https://dx.doi.org/10.1103/PhysRevLett.131.040602) ([arXiv: 2302.06639](https://arxiv.org/abs/2302.06639)).
Results from the resource estimator can be compared with the one of [the code coming with the paper](https://github.com/ElieGouzien/elliptic_log_cat).

Big thanks to Mathias Soeken for having written the initial version of this repository, and rebuilt [Microsoft Q# resource estimator](https://github.com/microsoft/qsharp/tree/main/resource_estimator) to allow our architecture to be handled.

Installation
------------
This is a standard Cargo crate.
Once rust is installed, `cargo build --release` will do its magic and build the estimator. The executable files are also available from the CI artifacts.

Usage
-----
This crate is designed as a library, and also contains a standalone executable that estimates resources from either a Q# file or from numbers of logical qubits, CX and CCX.
Use the subcommand `help` to have the documentation of the executable.

Examples can be run with `cargo run --example=elliptic_log` and `cargo run --example=from_qsharp`.

