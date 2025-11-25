[![ci](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml/badge.svg)](https://github.com/Alice-Bob-SW/qsharp-alice-bob-resource-estimator/actions/workflows/ci.yml)

Q# resource estimator for Alice & Bob's architecture
=====================================================

This project contains the code for using [Microsoft Q# resource estimator](https://github.com/microsoft/qsharp/tree/main/resource_estimator) (presented in [this paper](https://arxiv.org/abs/2311.05801)) for [Alice & Bob](https://alice-bob.com)'s architecture, using cat qubits and repetition code (LDPC codes might be added in the future).

Shor's algorithm for solving the elliptic curve discrete logarithm problem is used as an example, as in the paper [Phys. Rev. Lett. 131, 040602](https://dx.doi.org/10.1103/PhysRevLett.131.040602) ([arXiv: 2302.06639](https://arxiv.org/abs/2302.06639)).
Results from the resource estimator can be compared with the one of [the code coming with the paper](https://github.com/ElieGouzien/elliptic_log_cat).

Big thanks to Mathias Soeken for having written the initial version of this repository, and rebuilt [Microsoft Q# resource estimator](https://github.com/microsoft/qsharp/tree/main/resource_estimator) to allow our architecture to be handled.

## Q# Resource Estimator Python Interface
The resource estimator can now be imported into Python as a module using the functionality provided by [PyO3](https://pyo3.rs/v0.27.1/getting-started.html). To set it up, first install maturin in your virtual environment:

`pip install maturin`

Then build and install the module locally by running:

`run maturin develop`

After this, you should be good to go and you can import the resource estimator directly from Python.

## Functionality
The python interface offers two main functionalities:
- The first option is to input a tuple ```(#Qubits, #CX, #CCX)``` for which you want to estimate the physical resources. These values can be obtained for example from Qualtran so this provides a seemless interface between the two tools.
- The second option is to provide a Q# file which specificies a concrete algorithm for which you want to estimate the physical resources. The tool will then calculate ```(#Qubits, #CX, #CCX)``` from this Q# sharp file and perform the resource estimation from there.