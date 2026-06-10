from math import floor
from typing import Any, Tuple

# Import Qualtran tools
from qualtran import Bloq  # type: ignore[import-untyped]
from qualtran.resource_counting import get_cost_value, QubitCount  # type: ignore[import-untyped]
from qualtran.resource_counting.generalizers import (  # type: ignore[import-untyped]
    ignore_split_join,
    ignore_alloc_free,
    generalize_cvs,
)
from sympy import Expr

from anb_estimator.dataclass_wrappers import LogicalCounts  # type: ignore[import-untyped]

default_generalizer = (ignore_alloc_free, ignore_split_join, generalize_cvs)


def _as_exact_int(value: Any, name: str) -> int:
    """
    Convert a value (typically a sympy expression) to an integer, ensuring that it is a concrete integer (not symbolic).
    """
    if isinstance(value, Expr):
        if value.free_symbols or getattr(value, "is_integer", None) is not True:
            raise ValueError(f"{name} must be a concrete integer, got {value!r}")
        return int(value)

    return int(value)


def count_resources(
    bloq: Bloq,
    graph_generalizer: Tuple[Any, ...] = default_generalizer,  # type: ignore
) -> LogicalCounts:
    """Count the number of qubits, cx and ccx required for a given qualtran Bloq.

    -We count classically controlled CNOT as half a CNOT 
    -TwoBitCSwap are not native to A&B architectures
        and are decomposed in 2 CNOT + 1 Toffoli
    -And gates are counted as a Toffoli,
        And.adjoint() are not counted, so the pair of Ands counts as 1 Toffoli in Gitney's adder, see 2302.06639 G.2
    -Single qubit gates are not counted

    Parameters
    ----------
    bloq : Bloq
        Bloq of the circuit for which we want to count resources

    Returns
    -------
    num_qubits : int
        number of qubits needed for the Bloq
    num_cx : int
        number of cx needed for the Bloq
    num_ccx : int
        number of ccx needed for the Bloq
    """
    num_qubits = _as_exact_int(get_cost_value(bloq, QubitCount()), "Bloq qubit count")
    

    _, sigma = bloq.call_graph(graph_generalizer)
    L_k = [k for k in sigma.keys()]
    dict_sigma = {str(k): sigma[k] for k in L_k}

    num_cx = 0
    num_ccx = 0
    if "CNOT" in dict_sigma:
        num_cx += dict_sigma["CNOT"]  # type: ignore
    if "TwoBitCSwap" in dict_sigma:  # needs to be decomposed on A&B architecture
        num_cx += 2 * dict_sigma["TwoBitCSwap"]  # type: ignore
        num_ccx += dict_sigma["TwoBitCSwap"]  # type: ignore
    if "C[CNOT]" in dict_sigma:
        num_cx += 0.5 * dict_sigma["C[CNOT]"]  # type: ignore

    if "Toffoli" in dict_sigma:  # needs to be decomposed on A&B architecture
        num_ccx += dict_sigma["Toffoli"]  # type: ignore
    if "And" in dict_sigma:  # 
        num_ccx += dict_sigma["And"]  # type: ignore

    return LogicalCounts(
        qubit_count=num_qubits,
        cx_count=floor(num_cx),
        ccx_count=floor(num_ccx)
    )
