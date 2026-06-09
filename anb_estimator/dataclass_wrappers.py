from dataclasses import dataclass


from anb_estimator._native import LogicalCountsPy, EstimatesPy  # type: ignore[import-untyped]



@dataclass(frozen=True, slots=True)
class LogicalCounts:
    qubit_count: int
    cx_count: int
    ccx_count: int

    @classmethod
    def from_rust(cls, inner: LogicalCountsPy) -> "LogicalCounts":
        return cls(
            qubit_count=int(inner.qubit_count),
            cx_count=int(inner.cx_count),
            ccx_count=int(inner.ccx_count),
        )

    def to_dict(self) -> dict[str, int]:
        return {
            "qubit_count": self.qubit_count,
            "cx_count": self.cx_count,
            "ccx_count": self.ccx_count,
        }


@dataclass(frozen=True, slots=True)
class Estimates:
    physical_qubits: int
    runtime_seconds: float
    runtime_hours: float
    total_error: float

    code_distance: int
    code_alpha2: float

    factories: int
    factories_distance: int
    factories_alpha2: float

    factory_fraction_percent: float
    factory_fraction: float

    @classmethod
    def from_rust(cls, inner: EstimatesPy) -> "Estimates":
        return cls(
            physical_qubits=int(inner.physical_qubits),
            runtime_seconds=float(inner.runtime_seconds),
            runtime_hours=float(inner.runtime_hours),
            total_error=float(inner.total_error),
            code_distance=int(inner.code_distance),
            code_alpha2=float(inner.code_alpha2),
            factories=int(inner.factories),
            factories_distance=int(inner.factories_distance),
            factories_alpha2=float(inner.factories_alpha2),
            factory_fraction_percent=float(inner.factory_fraction_percent),
            factory_fraction=float(inner.factory_fraction),
        )

    def to_dict(self) -> dict[str, float | int]:
        return {
            "physical_qubits": self.physical_qubits,
            "runtime_seconds": self.runtime_seconds,
            "runtime_hours": self.runtime_hours,
            "total_error": self.total_error,
            "code_distance": self.code_distance,
            "code_alpha2": self.code_alpha2,
            "factories": self.factories,
            "factories_distance": self.factories_distance,
            "factories_alpha2": self.factories_alpha2,
            "factory_fraction_percent": self.factory_fraction_percent,
            "factory_fraction": self.factory_fraction,
        }


@dataclass(frozen=True)
class FullResults:
    estimates: Estimates
    frontier: list[Estimates] | None
    counts: LogicalCounts