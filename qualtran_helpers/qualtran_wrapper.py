from typing import Optional, Tuple, Union
import qsharp_alice_bob_resource_estimator as qre  # type: ignore[import-untyped]
from qualtran import Bloq  # type: ignore[import-untyped]
from qualtran_helpers.bloq_to_logical_counts import count_resources


ErrorBudget = Tuple[float, float, float]


def estimate_from_qualtran(
    bloq: Bloq,
    frontier: bool,
    error_total: Optional[float] = None,
    error_budget: Optional[ErrorBudget] = None,
) -> Union[qre.EstimatesPy, tuple[qre.EstimatesPy, list]]:  # type: ignore
    """
    Runs the Qualtran estimation and returns the results as an EstimatesPy class.

    Args:
        bloq (Bloq): The Bloq to be estimated.
        frontier (bool): If `true`, also return a list representing the frontier.
        error_total (float): Overall error target; mutually exclusive with `error_budget`.
        error_budget (Tuple): Tuple `(target, meas, routing)` for an explicit split.

    Returns:
        qre.EstimatesPy: The estimation results as an EstimatesPy class.
    """
    # --- validate bloq ---
    assert isinstance(bloq, Bloq), "bloq must be a qualtran Bloq"

    # Try resource counting early to ensure the bloq is well-formed
    try:
        num_qbits, num_cx, num_ccx = count_resources(bloq)
    except Exception as exc:
        raise AssertionError("bloq is not a valid qualtran Bloq") from exc

    # --- validate error inputs ---
    assert (error_total is None) ^ (error_budget is None), (
        "Exactly one of error_total or error_budget must be set"
    )

    if error_total is not None:
        assert error_total >= 0, "error_total must be >= 0"

    if error_budget is not None:
        assert len(error_budget) == 3, "error_budget must be a 3-tuple"
        assert all(x >= 0 for x in error_budget), "error_budget entries must be >= 0"

    estimate, frontier_data = qre.estimate_resources_struct(  # type: ignore
        num_qbits,
        num_cx,
        num_ccx,
        frontier=frontier,
        error_total=error_total,
        error_budget=error_budget,
    )

    if frontier:
        return estimate, frontier_data
    else:
        return estimate
