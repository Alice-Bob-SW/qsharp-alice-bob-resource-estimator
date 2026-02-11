from typing import Any, Tuple

# Import Qualtran tools
from qualtran import Bloq  # type: ignore[import-untyped]
from qualtran.resource_counting import get_cost_value, QubitCount  # type: ignore[import-untyped]
from qualtran.resource_counting.generalizers import (  # type: ignore[import-untyped]
    ignore_split_join,
    ignore_alloc_free,
    generalize_cvs,
)  # type: ignore[import-untyped]

default_generalizer = (ignore_alloc_free, ignore_split_join, generalize_cvs)


def count_resources(
    bloq: Bloq,
    graph_generalizer: Tuple[Any, ...] = default_generalizer,  # type: ignore
):
    """Counts the number of qubits, cx and ccx required for a given qualtran Bloq.
    -We count classicaly controlled CNOT as half a CNOT (approximation that we need
    them half of the time)
    -TwoBitCSwap are not native to A&B architectures
        and are decomposed in 2 CNOT + 1 Toffoli
    -And gate are counted as half a Toffoli (they require half as many T states),
        And.adjoint() are not counted
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

    num_qubits = get_cost_value(bloq, QubitCount())

    _, sigma = bloq.call_graph(graph_generalizer)
    L_k = [k for k in sigma.keys()]
    dict_sigma = {str(k): sigma[k] for k in L_k}

    num_cx = 0
    num_ccx = 0
    if "CNOT" in dict_sigma.keys():
        num_cx += dict_sigma["CNOT"]
    if "TwoBitCSwap" in dict_sigma.keys():  # needs to be decomposed on A&B architecture
        num_cx += 2 * dict_sigma["TwoBitCSwap"]
        num_ccx += dict_sigma["TwoBitCSwap"]
    if "C[CNOT]" in dict_sigma.keys():
        num_cx += 0.5 * dict_sigma["C[CNOT]"]

    if "Toffoli" in dict_sigma.keys():  # needs to be decomposed on A&B architecture
        num_ccx += dict_sigma["Toffoli"]
    if "And" in dict_sigma.keys():  # we count And as 0.5*Toffoli
        num_ccx += dict_sigma["And"]

    return num_qubits, int(num_cx), int(num_ccx)
