from dataclasses import dataclass
from typing import Any, Tuple

# Import Qualtran tools
from qualtran.drawing import show_counts_sigma  # type: ignore[import-untyped]
from qualtran.resource_counting import get_cost_value, QubitCount, QECGatesCost  # type: ignore[import-untyped]
from qualtran.resource_counting.generalizers import (  # type: ignore[import-untyped]
    ignore_split_join,
    ignore_alloc_free,
    generalize_cvs,
)  # type: ignore[import-untyped]
from qualtran_primitives.utilities import ignore_classical_control

from qualtran_primitives.ec_arithmetic.ec_find_key import FindECCPrivateKey_anb, ECPoint


@dataclass(frozen=True)
class ECCInstance:
    n: int
    p: int
    wa: int
    wm: int
    Gx: int
    Gy: int
    scalar: int = 2026  # P = scalar * G by default


def build_instance(cfg: ECCInstance):
    G = ECPoint(cfg.Gx, cfg.Gy, mod=cfg.p)
    P = cfg.scalar * G
    return G, P


def analyze_ecc_private_key_circuit(
    cfg: ECCInstance,
    graph_generalizer: Tuple[Any, ...] = (
        ignore_split_join,
        ignore_alloc_free,
        generalize_cvs,
    ),
    cost_generalizer: Tuple[Any, ...] = (
        ignore_split_join,
        ignore_alloc_free,
        ignore_classical_control,
    ),
):
    """
    Builds (G, P), runs FindECCPrivateKey_anb, computes call graph + sigma,
    gate costs, qubit count, and derived CX/CCX counts.

    Returns a dictionary with all useful artifacts.
    """
    G, P = build_instance(cfg)

    findecc = FindECCPrivateKey_anb(cfg.n, G, P, cfg.wa, cfg.wm)

    graph, sigma = findecc.call_graph(graph_generalizer)

    show_counts_sigma(sigma)

    gate_cost = get_cost_value(
        findecc,
        QECGatesCost(),
        generalizer=cost_generalizer,
    )

    qubits = get_cost_value(findecc, QubitCount())

    # Robustly read sigma regardless of whether keys are strings or objects
    L_k = [k for k in sigma.keys()]
    dict_sigma = {str(k): sigma[k] for k in L_k}

    num_cx = int(
        dict_sigma.get("CNOT", 0)
        + 2 * dict_sigma.get("TwoBitCSwap", 0)  # type: ignore
        + 0.5 * dict_sigma.get("C[CNOT]", 0)  # type: ignore
    )  # type: ignore
    num_ccx = int(
        dict_sigma.get("Toffoli", 0)
        + dict_sigma.get("TwoBitCSwap", 0)  # type: ignore
        + 0.5 * dict_sigma.get("And", 0)  # type: ignore
    )  # type: ignore

    return {
        "G": G,
        "P": P,
        "findecc": findecc,
        "graph": graph,
        "sigma": sigma,
        "gate_cost": gate_cost,
        "qubits": qubits,
        "num_cx": num_cx,
        "num_ccx": num_ccx,
    }
