from typing import Optional
import math
import numpy as np
import hardware
from hardware import us, ns, MHz, GHz, HBAR
import logical_utils as lu
import copy

# Helper functions
def print_summary(summary) -> None:
    """Pretty-print a resource estimate summary (no side effects beyond stdout)."""
    lines = [
        "Parameters obtained from the Rust resource estimator",
        "─────────────────────────────",
        f"# physical qubits:    {summary.physical_qubits:,}",
        f"runtime:             {summary.runtime_hours:.2f} hrs",
        f"total error:         {summary.total_error:.5f}",
        "─────────────────────────────",
        f"code distance:       {summary.code_distance} (|α|² = {summary.code_alpha2:.2f})",
        f"#factories:          {summary.factories}",
        f"factories distance:  {summary.factories_distance} (|α|² = {summary.factories_alpha2:.2f})",
        f"factory fraction:    {summary.factory_fraction_percent:.2f}%",
        "─────────────────────────────",
    ]
    print("\n".join(lines))


def format_energy(E: Optional[float], *, unicode: bool = False) -> str:
    """Format energy with engineering units (J, kJ, MJ, GJ)."""
    if E is None:
        return "None"
    if math.isnan(E):
        return "nan"
    if math.isinf(E):
        return "∞" if unicode else r"\infty"

    # choose unit
    absE = abs(E)
    if absE < 1e3:
        val, unit = E, "J"
    elif absE < 1e6:
        val, unit = E / 1e3, "kJ"
    elif absE < 1e9:
        val, unit = E / 1e6, "MJ"
    else:
        val, unit = E / 1e9, "GJ"

    return f"{round(val)} {unit}" if unicode else rf"{round(val)}{unit}"

def hardware_params():
    """Define Hardware

    Returns:
        Hardware
    """
    # Hardware

    k_1 = 100
    k_b = 40 * MHz


    H = hardware.Hardware(
        k_1=k_1,
        k_b=k_b,
        ntha=0.02,
        nthb=0.02,
        eta=0.4,
        E_J=40 * GHz * HBAR,
        vphi_a=0.11,
        vphi_b=0.20,
    )
    return H

def estimate_total_energy(summary, H: hardware, n_logical:int,macro: bool, k1_over_k2: float = 1e-5, display: bool = True) -> dict:
    """
    Calculates the approximate energy consumption based on the resource estimation provided by the rust module.
    
    Args:
        summary: Summary containing the results from the resource estimator
            - `physical_qubits`
            - `runtime_seconds`
            - `runtime_hours`
            - `total_error`
            - `code_distance`
            - `code_alpha2`
            - `factories`
            - `factories_distance`
            - `factories_alpha2`
            - `factory_fraction_percent`
            - `factory_fraction`
        H: Hardware class containing hardware [parameters]
            - `k_1`
            - `k_b`
            - `k_phi`
            - `k_ext`
            - `M`
            - `Z`
            - `E_J`
            - `E_L`
            - `omega_a`
            - `omega_b`
            -`Phi_0`
            - `vphi_a`
            - `vphi_b`
            - `vphi_C`
            - `vphi_T`
            - `ntha`
            - `nthb`
            - `eta`
            - `cabling`
        n_logical: Number of logical qubits estimated from Qsharp file or calculated according to arXiv:2302.06639 (p. 22, app C.11)
        macro: True -> Macroscopic Energy consumption
        K1ONK2: Set to 1e-5 by default
        display: Pretty print results, set to True by default
    """
    K2 =H.k_1/k1_over_k2
    
    # 2) Cycle time from current alpha (data first)
    H_data = copy.deepcopy(H)
    H_data.alpha = np.sqrt(summary.code_alpha2)
    T_cycle_data = lu.duration_cycle(K2, H_data)[-1]
    #print("The cycle time is: ",T_cycle_data)
    #print((T_cycle_data)/(500e-9)) 
    num_rounds_data = int(np.ceil(summary.runtime_seconds / T_cycle_data))

    # 3) Data energy
    E_data_patch = lu.E_tot(k1_over_k2, H_data, dist=summary.code_distance, num_round=num_rounds_data,
                     macro=macro)

    # 4) Factory energy (switch alpha)
    H_fac = copy.deepcopy(H)
    H_fac.alpha = np.sqrt(summary.factories_alpha2)
    T_cycle_factory = lu.duration_cycle(K2, H_fac)[-1]
    num_rounds_factory = int(np.ceil(summary.runtime_seconds / T_cycle_factory))
    E_one_factory = lu.E_tot(k1_over_k2, H_fac, dist=summary.factories_distance, num_round=num_rounds_factory,
                     macro=macro)
    
    E_factories = summary.factories * E_one_factory

    # 5) Totals
    E_total = E_data_patch*n_logical + E_factories
    
    if display == True:
        print_summary(summary)
        print(f"Number of logical qubits: {n_logical} ")
        print("Energy consumption estimation")
        print(f"T_cycle:       {T_cycle_data:.6e} s")
        print(f"Number of rounds:      {num_rounds_data}")
        print(f"E_data_patch:  {format_energy(E_data_patch, unicode=True)}")
        print(f"E_factory(one):{format_energy(E_one_factory, unicode=True)}")
        print(f"E_factories:   {format_energy(E_factories, unicode=True)}")
        print(f"E_total:       {format_energy(E_total)}")

    return {
        "T_cycle_s": float(T_cycle_data),
        "num_rounds": num_rounds_data,
        "E_data_patch_J": format_energy(E_data_patch),
        "E_one_factory_J": format_energy(E_one_factory),
        "E_factories_J": format_energy(E_factories),
        "E_total": format_energy(E_total),
    }