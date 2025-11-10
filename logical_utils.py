from __future__ import annotations

from contextlib import contextmanager
from typing import Iterable, Tuple, List, Optional
from jax.scipy.special import erf

import numpy as np
import gates  # your module that exposes e_d, drive_opt, P_* , T_* , g_l, etc.
from rescat import ResCat, HBAR

# ---------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------

TG2_FIZZ: float = 2.0
TG2_HOLO: float = 5.0
DEFAULT_MARGIN: float = 5.0  # k_b = margin * k2 for adiabatic elimination


# ---------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------

def kb_adia(k2: float, margin: float = DEFAULT_MARGIN) -> float:
    """Adiabatic elimination rule: set k_b = margin * k2."""
    return margin * k2


@contextmanager
def _adia_kb(hard_param, k2: float, margin: float = DEFAULT_MARGIN):
    """
    Temporarily set `hard_param.k_b` to the adiabatic value, then restore it.

    Usage:
        with _adia_kb(H, k2):
            ...  # inside, H.k_b = margin*k2
        # back to original H.k_b outside the block
    """
    kb_old = getattr(hard_param, "k_b")
    try:
        hard_param.k_b = kb_adia(k2, margin)
        yield
    finally:
        hard_param.k_b = kb_old

@contextmanager
def _adia_kb_rescat(auto: ResCat, k2: float, margin: float = DEFAULT_MARGIN):
    """
    Same idea as _adia_kb but for the ResCat ancilla object: temporarily set
    `auto.k_b = margin * k2` (local adiabatic choice), then restore.
    """
    kb_old = getattr(auto, "k_b")
    try:
        auto.k_b = kb_adia(k2, margin)
        yield
    finally:
        auto.k_b = kb_old

# ---------------------------------------------------------------------
# Autoparametric-cat specific CNOT power pieces
# ---------------------------------------------------------------------

def e_d_rescat(alpha: float, auto: ResCat) -> float:
    """
    Buffer drive rate ε for autoparametric cat (assumes resonance).
    Units: if auto.E_W / HBAR is [s^-1], ε is [s^-1].
    """
    g2 = float(auto.E_W / HBAR * auto.phi_zpf_m**2 * auto.phi_zpf_b)
    return (np.abs(alpha) ** 2) * g2

def P_buffer_drive_rescat(ed: float, auto: ResCat, macro: bool = True) -> float:
    """Input power needed to sustain buffer drive ε on resonance."""
    # Prefactor mirrors your original style: HBAR*omega_b/k_b times drive^2
    pref = HBAR * auto.omega_b / auto.k_b
    base = (ed ** 2) * pref
    return (auto.Md * float(base)) if macro else float(base)

def P_CNOT_total(target_hw, auto: ResCat, gcnot: float, macro: bool = True) -> float:
    """
    Total CNOT power when using an autoparametric cat ancilla:
      P_total = P_buf (autocat buffer drive) + P_pump (CNOT pump on target HW).
    """
    # Use the ancilla's own alpha for its buffer drive
    ed = e_d_rescat(getattr(auto, "alpha", getattr(target_hw, "alpha", 0.0)), auto)
    P_buf  = P_buffer_drive_rescat(ed, auto, macro)
    P_pump = gates.P_CNOT_pump(target_hw, gcnot, macro)
    return P_buf + P_pump

# ---------------------------------------------------------------------
# Power (macro flag preserved)
# ---------------------------------------------------------------------

def power_stab(k2: float, hard_param, macro: bool = False) -> float:
    """Stabilization (Id) power."""
    with _adia_kb(hard_param, k2):
        ed = gates.e_d(hard_param, hard_param.alpha, k2)
        return gates.P_Idgate(hard_param, ed, k2, macro)


def power_CNOT(k2: float, hard_param, macro: bool = False, rescat: Optional[ResCat] = None) -> float:
    """
    CNOT power (both qubits).
    - If `rescat` is None: original (non-autocat) model.
    - If `rescat` is a ResCat: autocat ancilla model (buffer drive + pump).
    """
    if rescat is None:
        with _adia_kb(hard_param, k2):
            ed = gates.e_d(hard_param, hard_param.alpha, k2)
            gcnot = gates.drive_opt(hard_param, hard_param.alpha, k2, "CNOT")
            return gates.P_Cnotgate(hard_param, ed, gcnot, k2, macro)
    else:
        # Use *both* adiabatic contexts so k_b is consistent in each object
        with _adia_kb(hard_param, k2), _adia_kb_rescat(rescat, k2):
            gcnot = gates.drive_opt(hard_param, hard_param.alpha, k2, "CNOT")
            return P_CNOT_total(hard_param, rescat, gcnot, macro)


def power_Z(k2: float, hard_param, macro: bool = False) -> float:
    """Z (Zeno) gate power (includes stabilization power for that step)."""
    with _adia_kb(hard_param, k2):
        ed = gates.e_d(hard_param, hard_param.alpha, k2)
        ez = gates.drive_opt(hard_param, hard_param.alpha, k2, interaction="Z")
        return gates.P_Zgate(hard_param, ed, ez, k2, macro)


def power_atspump(k2: float, hard_param, macro: bool = False) -> float:
    """ATS pump power (two-photon dissipation)."""
    with _adia_kb(hard_param, k2):
        return gates.P_pump(hard_param, k2, macro)


def power_FLoR(k2: float, hard_param, macro: bool = False) -> float:
    """FLoR (longitudinal readout) power."""
    with _adia_kb(hard_param, k2):
        gl = gates.g_l(hard_param)
        return gates.P_FLoR(hard_param, gl, macro)


# ---------------------------------------------------------------------
# Durations & Energies
# ---------------------------------------------------------------------

def duration_meas(k2: float, ez: float, hard_param) -> List[float]:
    """
    Durations of the measurement composite step as:
      [T_halfZ (π/2 Z-rotation), T_holo, T_FIZZ]
    """
    # Z π/2 rotation uses the same Z drive; half the full Z gate time.
    T_halfZ = gates.T_Zgate(hard_param.alpha, ez) / 2.0
    T_holo = gates.T_holo(hard_param.alpha, k2, tg2=TG2_HOLO)
    T_FIZZ = gates.T_FIZZ(hard_param.alpha, k2, tg2=TG2_FIZZ)
    return [T_halfZ, T_holo, T_FIZZ]


def E_meas(k2: float, hard_param, macro: bool) -> List[float]:
    """
    Energy of the three measurement sub-steps:
      [E_halfZ, E_holo, E_FIZZ]
    """
    with _adia_kb(hard_param, k2):
        # Drives needed
        ed = gates.e_d(hard_param, hard_param.alpha, k2)
        ez = gates.drive_opt(hard_param, hard_param.alpha, k2, interaction="Z")
        e_holo = gates.e_holo(hard_param.alpha, k2)
        e_m = gates.e_m(hard_param.alpha, k2)

        # Powers for each sub-step
        P_halfZ = gates.P_Zgate(hard_param, ed, ez, k2, macro)
        P_holo = gates.P_pump(hard_param, k2, macro) + gates.P_buffer_drive(hard_param, e_holo, macro)
        P_FIZZ = gates.P_Zgate(hard_param, ed, e_m, k2, macro)

        # Durations
        T_halfZ, T_holo, T_FIZZ = duration_meas(k2, ez, hard_param)
        return [P_halfZ * T_halfZ, P_holo * T_holo, P_FIZZ * T_FIZZ]


def duration_cycle(k2: float, hard_param) -> List[float]:
    """
    Compute durations (in seconds) for one repetition-code cycle:
      [T_prep, T_CNOT, T_meas, T_cycle]
    where:
      - T_prep = 1/k2
      - T_CNOT is for a single CNOT
      - T_meas is composed of the three measurement sub-steps
      - T_cycle = T_prep + 2*T_CNOT + T_meas
    """
    with _adia_kb(hard_param, k2):
        ez = gates.drive_opt(hard_param, hard_param.alpha, k2, interaction="Z")
        gcnot = gates.drive_opt(hard_param, hard_param.alpha, k2, "CNOT")

        T_meas = sum(duration_meas(k2, ez, hard_param))
        T_prep = 1.0 / k2
        T_CNOT = gates.T_CNOT(hard_param.alpha, gcnot)
        T_cycle = T_prep + 2.0 * T_CNOT + T_meas

        return [T_prep, T_CNOT, T_meas, T_cycle]


def E_CNOT(k2: float, hard_param, macro: bool, rescat: Optional[ResCat] = None) -> float:
    """Energy of one CNOT (two-qubit gate)."""
    if rescat is None:
        with _adia_kb(hard_param, k2):
            gcnot = gates.drive_opt(hard_param, hard_param.alpha, k2, "CNOT")
            P_CNOT = power_CNOT(k2, hard_param, macro)
            T_CNOT = gates.T_CNOT(hard_param.alpha, gcnot)
            return P_CNOT * T_CNOT
    else:
        # Autocat ancilla: power comes from autocat model; duration from target HW
        with _adia_kb(hard_param, k2), _adia_kb_rescat(rescat, k2):
            gcnot = gates.drive_opt(hard_param, hard_param.alpha, k2, "CNOT")
            P_CNOT = power_CNOT(k2, hard_param, macro, rescat=rescat)
            # Keep your existing duration model (depends on target alpha and gcnot)
            T_CNOT = gates.T_CNOT(hard_param.alpha, gcnot)
            return P_CNOT * T_CNOT


def E_prep(k2: float, hard_param, macro: bool) -> float:
    """Energy of preparation (stabilization-only step)."""
    P_prep = power_stab(k2, hard_param, macro)
    T_prep = 1.0 / k2
    return P_prep * T_prep


def E_stab(k2: float, hard_param, macro: bool) -> float:
    """
    Stabilization energy during one *unit cell* of the repetition code:
    we stabilize during preparation and during measurement (control CNOT
    stabilization is included in the CNOT energy).
    """
    P_stab = power_stab(k2, hard_param, macro)
    T_prep, _, T_meas, _ = duration_cycle(k2, hard_param)
    return P_stab * (T_prep + T_meas)


def E_stab_edge(k2: float, hard_param, num_round: int, macro: bool) -> float:
    """
    Additional stabilization energy for edge cases:
    - no ancilla prep at the very end,
    - 1-qubit stabilization for the whole correction duration
    (modeled here as stab power over T_cycle * num_round).
    """
    _, _, _, T_cycle = duration_cycle(k2, hard_param)
    P_stab = power_stab(k2, hard_param, macro)
    return P_stab * T_cycle * num_round


def E_unit_cell(k2: float, hard_param, macro: bool, rescat: Optional[ResCat] = None) -> float:
    """Energy of a full unit cell (two CNOTs + prep + measurement + stabilization)."""
    return (
        2.0 * E_CNOT(k2, hard_param, macro, rescat=rescat)
        + E_prep(k2, hard_param, macro)
        + sum(E_meas(k2, hard_param, macro))
        + E_stab(k2, hard_param, macro)
    )


def E_tot(k1onk2: float, hard_param, dist: int, num_round: int, macro: bool, rescat: Optional[ResCat] = None) -> float:
    """
    Total energy for a repetition code run:
      E_total = unit_cell * (dist - 1) * num_round + edge_stab
    Assumes data qubits are already prepared and their measurement
    cost is handled elsewhere.

    If `rescat` is provided, uses the autocat CNOT power model.
    """
    k2 = hard_param.k_1 / k1onk2
    unit_cell = E_unit_cell(k2, hard_param, macro, rescat=rescat)
    stab_edge = E_stab_edge(k2, hard_param, num_round, macro)
    return unit_cell * (dist - 1) * num_round + stab_edge

# ---------------------------------------------------------------------
# FLoR measurement scheme
# ---------------------------------------------------------------------
def SNR_FLoR(t: np.ndarray, gl: float, hard_param) -> np.ndarray:
    kb = hard_param.k_b
    # group exponents for clarity & numeric stability
    term1 = kb * t
    term2 = 4.0 * (1.0 - np.exp(-0.5 * kb * t))
    term3 = 1.0 - np.exp(-kb * t)
    pref = 4.0 * hard_param.eta * gl**2 / kb**2
    return np.sqrt(pref * (term1 - term2 + term3))

def fid_FLoR(t: np.ndarray, gl: float, hard_param) -> np.ndarray:
    return np.exp(-hard_param.k_1 * t) * erf(0.5 * SNR_FLoR(t, gl, hard_param))

def Duration_FLoR(t_grid: np.ndarray, hard_param, gl: float) -> float:
    L_fid = fid_FLoR(t_grid, gl, hard_param)
    return float(t_grid[np.argmax(L_fid)])

def duration_meas_FLoR(k2: float, ez: float, hard_param) -> List[float]:
    """
    FLoR measurement composite step durations:
      [T_halfZ, T_deflate, T_inflate, T_FLoR]
    """
    # Use the same source of truth for g_l everywhere:
    gl = gates.g_l(hard_param)

    T_halfZ = gates.T_Zgate(hard_param.alpha, ez) / 2.0
    T_deflate = 3.2 / k2            # from fit (your comment)
    T_inflate = 1.0 / k2
    # 5 microseconds scan window (assuming gates.us is seconds)
    T_FLoR = Duration_FLoR(np.linspace(0.0, 5.0 * gates.us, 10_000), hard_param, gl)
    return [T_halfZ, T_deflate, T_inflate, T_FLoR]

def E_meas_FLoR(k2: float, hard_param, macro: bool) -> List[float]:
    """Energy of the four FLoR measurement sub-steps."""
    with _adia_kb(hard_param, k2):  # avoid leaking k_b side-effects
        # drives
        ed = gates.e_d(hard_param, hard_param.alpha, k2)
        ez = gates.drive_opt(hard_param, hard_param.alpha, k2, interaction="Z")
        gl = gates.g_l(hard_param)

        # powers
        P_halfZ   = gates.P_Zgate(hard_param, ed, ez, k2, macro)
        P_deflate = gates.P_pump(hard_param, k2, macro)
        P_inflate = gates.P_Idgate(hard_param, ed, k2, macro)
        P_FLoR    = gates.P_FLoR(hard_param, gl, macro)

        # durations (FLoR version!)
        T_halfZ, T_deflate, T_inflate, T_FLoR = duration_meas_FLoR(k2, ez, hard_param)

        return [
            P_halfZ * T_halfZ,
            P_deflate * T_deflate,
            P_inflate * T_inflate,
            P_FLoR * T_FLoR,
        ]

def duration_cycle_FLoR(k2: float, hard_param) -> List[float]:
    """
    Durations for one repetition-code cycle with FLoR readout:
      [T_prep, T_CNOT, T_meas, T_cycle]
    """
    with _adia_kb(hard_param, k2):
        ez    = gates.drive_opt(hard_param, hard_param.alpha, k2, interaction="Z")
        gcnot = gates.drive_opt(hard_param, hard_param.alpha, k2, "CNOT")

        T_meas = sum(duration_meas_FLoR(k2, ez, hard_param))
        T_prep = 1.0 / k2
        T_CNOT = gates.T_CNOT(hard_param.alpha, gcnot)
        T_cycle = T_prep + 2.0 * T_CNOT + T_meas
        return [T_prep, T_CNOT, T_meas, T_cycle]

def E_stab_edge_FLoR(k2: float, hard_param, num_round: int, macro: bool) -> float:
    """Edge-case stabilization energy under FLoR timing."""
    _, _, _, T_cycle = duration_cycle_FLoR(k2, hard_param)
    P_stab = power_stab(k2, hard_param, macro)
    return P_stab * T_cycle * num_round

def E_stab_FLoR(k2: float, hard_param, macro: bool) -> float:
    P_stab = power_stab(k2, hard_param, macro)
    T_prep, _, T_meas, _ = duration_cycle_FLoR(k2, hard_param)
    return P_stab * (T_prep + T_meas)

def E_unit_cell_FLoR(k2: float, hard_param, macro: bool, rescat: Optional[ResCat] = None) -> float:
    """Energy of a full unit cell under FLoR readout."""
    return (
        2.0 * E_CNOT(k2, hard_param, macro, rescat=rescat)
        + E_prep(k2, hard_param, macro)
        + sum(E_meas_FLoR(k2, hard_param, macro))
        + E_stab_FLoR(k2, hard_param, macro)  # uses T_prep + T_meas_FLoR via duration_cycle_FLoR if you prefer to mirror
    )


def E_tot_FLoR(k1onk2: float, hard_param, dist: int, num_round: int, macro: bool, rescat: Optional[ResCat] = None) -> float:
    """
    Total energy for a repetition code run with FLoR readout.
    If `rescat` is provided, uses the autocat CNOT power model.
    """
    k2 = hard_param.k_1 / k1onk2
    with _adia_kb(hard_param, k2):
        unit_cell = E_unit_cell_FLoR(k2, hard_param, macro, rescat=rescat)
        stab_edge = E_stab_edge_FLoR(k2, hard_param, num_round, macro)
        return unit_cell * (dist - 1) * num_round + stab_edge