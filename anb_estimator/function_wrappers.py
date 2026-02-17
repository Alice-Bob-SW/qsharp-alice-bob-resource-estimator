from typing import Optional, Tuple, Union

try:
    from ._native import estimate_qsharp_file_rust, estimate_logical_counts_rust, estimate_ecc_example_rust, EstimatesPy  # type: ignore[import-untyped]
except ImportError:
    from _native import estimate_qsharp_file_rust, estimate_logical_counts_rust, estimate_ecc_example_rust, EstimatesPy  # type: ignore[import-untyped]
from qualtran import Bloq  # type: ignore[import-untyped]

try:
    from .bloq_to_logical_counts import count_resources
    from .dataclass_wrappers import Estimates
except ImportError:
    from bloq_to_logical_counts import count_resources
    from dataclass_wrappers import Estimates


ErrorBudget = Tuple[float, float, float]


def estimate_from_qualtran(
    bloq: Bloq,
    frontier: bool,
    error_total: Optional[float] = None,
    error_budget: Optional[ErrorBudget] = None,
) -> Union[EstimatesPy, tuple[EstimatesPy, list]]:  # type: ignore
    """
    Runs the Qualtran estimation and returns the results as an EstimatesPy class.

    Args:
        bloq (Bloq): The Bloq to be estimated.
        frontier (bool): If `true`, also return a list representing the frontier.
        error_total (float): Overall error target; mutually exclusive with `error_budget`.
        error_budget (Tuple): Tuple `(target, meas, routing)` for an explicit split.

    Returns:
        Union[Estimates, tuple[Estimates, list]]: The estimation results as an Estimates dataclass or a tuple with the frontier.
    """
    # --- validate bloq ---
    assert isinstance(bloq, Bloq), "bloq must be a qualtran Bloq"

    # Try resource counting early to ensure the bloq is well-formed
    try:
        num_qbits, num_cx, num_ccx = count_resources(bloq)
    except Exception as exc:
        raise AssertionError("bloq is not a valid qualtran Bloq") from exc

    # --- validate error inputs ---
    if error_total is None and error_budget is None:
        print("No error budget provided. Falling back to default error budget (0.333 * 0.5, 0.333 * 0.5, 0.0).")
    
    if error_total is not None and error_budget is not None:
        raise ValueError("Exactly one of error_total or error_budget must be set")

    if error_total is not None:
        assert error_total >= 0, "error_total must be >= 0"

    if error_budget is not None:
        assert len(error_budget) == 3, "error_budget must be a 3-tuple"
        assert all(x >= 0 for x in error_budget), "error_budget entries must be >= 0"

    estimate, frontier_data = estimate_logical_counts(  # type: ignore
        num_qbits, #type: ignore
        num_cx,
        num_ccx,
        frontier=frontier,
        error_total=error_total,
        error_budget=error_budget,
    )

    if frontier:
        frontier_converted = [Estimates.from_rust(e) for e in frontier_data]
        return Estimates.from_rust(estimate), frontier_converted
    else:
        return Estimates.from_rust(estimate)
    
def estimate_logical_counts(
    num_qbits: int,
    num_cx: int,
    num_ccx: int,
    frontier: bool,
    error_total: Optional[float] = None,
    error_budget: Optional[ErrorBudget] = None,
) -> Union[EstimatesPy, tuple[EstimatesPy, list]]:  # type: ignore
    """
    Runs the estimation and returns the results as an EstimatesPy class.

    Args:
        num_qbits (int): Logical (algorithm) qubit count.
        num_cx (int): Logical CX-equivalent two-qubit gate count.
        num_ccx (int): Logical CCX (Toffoli) gate count.
        frontier (bool): If `true`, also return a list representing the frontier.
        error_total (float): Overall error target; mutually exclusive with `error_budget`.
        error_budget (Tuple): Tuple `(topological error budget, magic state error budget, rotation error budget)` if an explicit split is desired.

    Returns:
        Union[Estimates, tuple[Estimates, list]]: The estimation results as an Estimates dataclass or a tuple with the frontier.
    """
    # --- validate inputs ---
    assert num_qbits >= 0, "num_qbits must be >= 0"
    assert num_cx >= 0, "num_cx must be >= 0"
    assert num_ccx >= 0, "num_ccx must be >= 0"
    
    if error_total is None and error_budget is None:
        print("No error budget provided. Falling back to default error budget (0.333 * 0.5, 0.333 * 0.5, 0.0).")
    
    if error_total is not None and error_budget is not None:
        raise ValueError("Exactly one of error_total or error_budget must be set")

    if error_total is not None:
        assert error_total >= 0, "error_total must be >= 0"

    if error_budget is not None:
        assert len(error_budget) == 3, "error_budget must be a 3-tuple"
        assert all(x >= 0 for x in error_budget), "error_budget entries must be >= 0"

    estimate, frontier_data = estimate_logical_counts_rust(  # type: ignore
        num_qbits,
        num_cx,
        num_ccx,
        frontier=frontier,
        error_total=error_total,
        error_budget=error_budget,
    )

    if frontier:
        frontier_converted = [Estimates.from_rust(e) for e in frontier_data]
        return Estimates.from_rust(estimate), frontier_converted
    else:
        return Estimates.from_rust(estimate)
    
def estimate_ecc_example(bit_size: int, window_size: int, frontier: bool) -> Union[EstimatesPy, tuple[EstimatesPy, list]]:  # type: ignore
    """
    Runs the estimation for the ECC example and returns the results as an EstimatesPy class.

    Args:
        bit_size (int): ECC modulus bit size.
        window_size (int): Window size used by the example algorithm.
        frontier (bool): If `true`, also return a list representing the Pareto frontier.

    Returns:
        Union[Estimates, tuple[Estimates, list]]: The estimation results as an Estimates dataclass or a tuple with the frontier.
    """
    assert bit_size > 0, "bit_size must be > 0"
    assert window_size > 0, "window_size must be > 0"

    estimate, frontier_data = estimate_ecc_example_rust(  # type: ignore
        bit_size,
        window_size,
        frontier=frontier,
    )

    if frontier:
        frontier_converted = [Estimates.from_rust(e) for e in frontier_data]
        return Estimates.from_rust(estimate), frontier_converted
    else:
        return Estimates.from_rust(estimate)

def estimate_qsharp_file(file_path: str, frontier: bool, error_total: Optional[float] = None, error_budget: Optional[ErrorBudget] = None) -> Union[EstimatesPy, tuple[EstimatesPy, list]]:  # type: ignore
    """
    Runs the estimation for a Q# file and returns the results as a dataclass.

    Args:
        file_path (str): The path to the Q# file to be estimated.
        frontier (bool): If `true`, also compute a frontier of estimates (e.g., different distances/α).
        error_total (float): Overall error target `p_total`; mutually exclusive with `error_budget`.
        error_budget (Tuple): Tuple `(topological error budget, magic state error budget, rotation error budget)` if an explicit split is desired.

    Returns:
        Union[Estimates, tuple[Estimates, list]]: The estimation results as an Estimates dataclass or a tuple with the frontier.
    """
    # --- validate inputs ---
    assert isinstance(file_path, str), "file_path must be a string"
    assert file_path.endswith(".qs"), "file_path must point to a Q# file"
    assert isinstance(frontier, bool), "frontier must be a boolean"
    if error_total is not None:
        assert error_total >= 0, "error_total must be >= 0"
    if error_budget is not None:
        assert len(error_budget) == 3, "error_budget must be a 3-tuple"
        assert all(x >= 0 for x in error_budget), "error_budget entries must be >= 0"
    if error_total is not None and error_budget is not None:
        raise ValueError("Exactly one of error_total or error_budget must be set")
    if error_total is None and error_budget is None:
        print("No error budget provided. Falling back to default error budget (0.333 * 0.5, 0.333 * 0.5, 0.0).")

    estimate, frontier_data = estimate_qsharp_file_rust(  # type: ignore
        file_path,
        frontier=frontier,
        error_total=error_total,
        error_budget=error_budget,
    )

    frontier_converted = [Estimates.from_rust(e) for e in frontier_data]

    if frontier:
        return Estimates.from_rust(estimate), frontier_converted
    else:
        return Estimates.from_rust(estimate)
