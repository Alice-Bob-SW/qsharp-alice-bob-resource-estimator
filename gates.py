"""
Cat-qubit power, cabling, and noise utilities.

This module provides:
- Cryostat cabling models (attenuation, thermal conductance, passive heat loads)
- Hardware parameter container with derived coefficients
- Drive/interaction strength helpers and gate-time formulas
- Power accounting for pump/drive/readout
- Stim circuits for a repetition code and a noise injection pass

All temperatures are in Kelvin, frequencies in angular units (rad/s),
and powers/energies follow the conventions in the associated report/notes.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable, Dict, Iterable, List, Literal, Tuple

import numpy as np
import scipy.optimize as scopt
from numpy.typing import NDArray
from scipy.constants import hbar, Boltzmann
from jax.scipy.special import erf

from hardware import Hardware  # Hardware container
from hardware import Hz, kHz, MHz, GHz, us, ns, pH


# ---------------------------------------------------------------------
# Drive amplitude helpers & gate durations
# ---------------------------------------------------------------------

def e_d(H: Hardware, alpha: float, k_2: float) -> float:
    """Buffer drive formula."""
    return np.sqrt(H.k_b * k_2) / 2.0 * (alpha ** 2 + H.k_1 / (2.0 * k_2))


def g_l(H: Hardware) -> float:
    """
    Longitudinal coupling for FLoR (bifurcation margin of 1.2 applied).
    """
    gl = 1.0 / (2 * np.sqrt(2)) * H.vphi_a / H.vphi_b * H.k_b
    return gl / 1.2


def e_z(H: Hardware, alpha: float, p_Z: float, k_2: float) -> float:
    """Zeno drive to reach a given fidelity (report formula after Table 2)."""
    return (
        2.0
        / np.pi
        * alpha ** 3
        * p_Z
        * k_2
        / (1 + 2 * H.nthb)
        * (1 - np.sqrt(1 - np.pi ** 2 * H.k_1 / (4 * alpha ** 2 * p_Z ** 2 * k_2) * (1 + 2 * H.ntha) * (1 + 2 * H.nthb)))
    )


def g_CNOT(H: Hardware, alpha: float, p_ZC: float, k_2: float) -> float:
    """Pump drive for a CNOT (report formula after Table 2)."""
    return (
        2.0
        / np.pi
        * alpha
        * p_ZC
        * k_2
        / (1 + 2 * H.nthb)
        * (1 - np.sqrt(1 - np.pi ** 2 * H.k_1 / (4 * p_ZC ** 2 * k_2) * (1 + 2 * H.ntha) * (1 + 2 * H.nthb)))
    )


def drive_opt(H: Hardware, alpha: float, k_2: float, interaction: Literal["CNOT", "Z"] = "CNOT") -> float:
    """Drive required for maximum-fidelity gates (Table 3)."""
    if interaction == "CNOT":
        return alpha * np.sqrt(H.k_1 * k_2) * np.sqrt((1 + 2 * H.ntha) / (1 + 2 * H.nthb))
    if interaction == "Z":
        return alpha ** 2 * np.sqrt(H.k_1 * k_2) * np.sqrt((1 + 2 * H.ntha) / (1 + 2 * H.nthb))
    raise ValueError(f"Unsupported interaction: {interaction!r}")


def e_holo(alpha: float, k2: float) -> float:
    """Holonomic: e_holo = alpha**2 * g2."""
    return alpha ** 3 * k2


def e_m(alpha: float, k2: float) -> float:
    """Measurement: e_m = 2 * alpha**2 * k2."""
    return 2.0 * alpha ** 2 * k2


def T_Zgate(alpha: float, ez: float) -> float:
    return np.pi / (4.0 * alpha * ez)


def T_CNOT(alpha: float, gcnot: float) -> float:
    return np.pi / (4.0 * alpha * gcnot)


def T_holo(alpha: float, k2: float, tg2: float = 5.0) -> float:
    return tg2 / (alpha * k2)


def T_FIZZ(alpha: float, k2: float, tg2: float = 2.0) -> float:
    return tg2 / (alpha * k2)


# ---------------------------------------------------------------------
# Power accounting
# ---------------------------------------------------------------------

def P_pump(H: Hardware, k_2: float, macro: bool = True) -> float:
    """Power to drive two-photon dissipation with ATS."""
    base = H.p * H.k_b * k_2 / 4.0
    return H.Mp * base if macro else base


def P_buffer_drive(H: Hardware, ed: float, macro: bool = True) -> float:
    base = H.d * ed ** 2
    return H.Md * base if macro else base

def P_zeno_drive(H: Hardware, ez: float, macro: bool = True) -> float:
    base = H.z * ez ** 2
    return H.Mz * base if macro else base


def P_CNOT_pump(H: Hardware, gcnot: float, macro: bool = True) -> float:
    base = H.c * gcnot ** 2
    return H.Mp * base if macro else base


def P_Idgate(H: Hardware, ed: float, k_2: float, macro: bool = True) -> float:
    """Power for stabilisation-only (Id gate)."""
    return P_pump(H, k_2, macro) + P_buffer_drive(H, ed, macro)


def P_Zgate(H: Hardware, ed: float, ez: float, k_2: float, macro: bool = True) -> float:
    """Power for Zeno gate (includes stabilisation)."""
    return P_pump(H, k_2, macro) + P_buffer_drive(H, ed, macro) + P_zeno_drive(H, ez, macro)


def P_Cnotgate(H: Hardware, ed: float, gcnot: float, k_2: float, macro: bool = True) -> float:
    """Power for both qubits in a CNOT (control stabilised + target ATS)."""
    return P_pump(H, k_2, macro) + P_buffer_drive(H, ed, macro) + P_CNOT_pump(H, gcnot, macro)


def P_FLoR(H: Hardware, gl: float, macro: bool = True) -> float:
    """Power for Fock-state longitudinal readout."""
    base = H.l * gl ** 2
    return H.Mp * base if macro else base

# ---------------------------------------------------------------------
# Noise model & circuit helpers
# ---------------------------------------------------------------------

def SNR_FLoR(t: float | NDArray[np.float64], gl: float, H: Hardware) -> float | NDArray[np.float64]:
    return np.sqrt(
        4 * H.eta * gl ** 2 / H.k_b ** 2 * (H.k_b * t - 4 * (1 - np.exp(-0.5 * H.k_b * t)) + (1 - np.exp(-H.k_b * t)))
    )


def fid_FLoR(t: float | NDArray[np.float64], gl: float, H: Hardware) -> float | NDArray[np.float64]:
    """Analytical FLoR fidelity (Tech Wiki / FLoR study)."""
    return np.exp(-H.k_1 * t) * erf(0.5 * SNR_FLoR(t, gl, H))


def max_fid_Flor(L_t: NDArray[np.float64], H: Hardware) -> Tuple[float, float]:
    """Return (max fidelity, optimal duration) scanning durations in L_t."""
    gl = g_l(H)
    L_fid = fid_FLoR(L_t, gl, H)
    idx = int(np.argmax(L_fid))
    return float(np.max(L_fid)), float(L_t[idx])


def noise_FLoR_meas(
    k1onk2: float, H: Hardware, alpha: float, T_max_FLoR: float = 5.0, param_fit_deflate: List[float] = [2.57779357, 1.0]
) -> Tuple[float, float]:
    half_Z = np.pi / (4 * alpha) * np.sqrt(k1onk2) * np.sqrt((1 + 2 * H.ntha) * (1 + 2 * H.nthb))
    deflate = param_fit_deflate[0] * k1onk2 + param_fit_deflate[1] * H.nthb
    inflate = alpha ** 2 * k1onk2 * (1 + 2 * H.ntha)

    fid_F, dur = max_fid_Flor(np.linspace(0.0, T_max_FLoR * us, 10_000), H)

    fid_tot = (1 - 2 * half_Z) * (1 - 2 * deflate) * (1 - 2 * inflate) * fid_F
    error_tot_meas = (1 - fid_tot) / 2.0

    stab_halfZ = np.pi / (8 * alpha) * np.sqrt(k1onk2) * np.sqrt((1 + 2 * H.ntha) * (1 + 2 * H.nthb))
    stab_remaining_meas = (4.2) * alpha ** 2 * k1onk2 * (1 + 2 * H.ntha)  # 3.2/k2 + 1/k2
    stab_FLoR = alpha ** 2 * H.k_1 * dur * (1 + 2 * H.ntha)

    error_stab_mx = stab_halfZ + stab_FLoR + stab_remaining_meas
    return float(error_tot_meas), float(error_stab_mx)


def fid_holo(
    k_2: float, H: Hardware, alpha: float, param_fit_holo: List[float] = [0.2, 2.6, 1.2], Tg2: float = 5.0
) -> Tuple[float, float]:
    """Holonomic gate fidelity and duration (Eq. S6, Gautier & Ruiz note)."""
    a, b, c = param_fit_holo
    g2 = alpha * k_2
    T = Tg2 / g2
    err = a * alpha ** b * np.exp(-c * g2 * T) + 0.5 * H.k_1 * alpha ** 2 * T
    return float(1 - 2 * err), float(T)


def fid_FIZZ(k_2: float, H: Hardware, alpha: float, Tg2: float = 2.0) -> Tuple[float, float]:
    g2 = alpha * k_2
    T = Tg2 / g2
    snr = np.sqrt(16 * H.eta * alpha * g2 * T)
    return float(erf(0.5 * snr)), float(T)


def noise_holo_meas(
    k1onk2: float,
    k_2: float,
    H: Hardware,
    alpha: float,
    param_fit_holo: List[float] = [0.2, 2.6, 1.2],
    Tg2_holo: float = 5.0,
    Tg2_FIZZ: float = 2.0,
) -> Tuple[float, float]:
    half_Z = np.pi / (4 * alpha) * np.sqrt(k1onk2) * np.sqrt((1 + 2 * H.ntha) * (1 + 2 * H.nthb))
    holo, T_h = fid_holo(k_2, H, alpha, param_fit_holo=param_fit_holo, Tg2=Tg2_holo)
    fizz, T_f = fid_FIZZ(k_2, H, alpha, Tg2=Tg2_FIZZ)

    fid_tot = (1 - 2 * half_Z) * holo * fizz
    error_tot_meas = (1 - fid_tot) / 2.0

    stab_halfZ = np.pi / (8 * alpha) * np.sqrt(k1onk2) * np.sqrt((1 + 2 * H.ntha) * (1 + 2 * H.nthb))
    error_stab_mx = stab_halfZ + alpha ** 2 * H.k_1 * (T_h + T_f) * (1 + 2 * H.ntha)

    return float(error_tot_meas), float(error_stab_mx)
