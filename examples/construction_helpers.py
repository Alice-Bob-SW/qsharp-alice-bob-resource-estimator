from dataclasses import dataclass

# Import Qualtran tools
from qualtran.resource_counting.generalizers import (  # type: ignore[import-untyped]
    ignore_split_join,
    ignore_alloc_free,
    generalize_cvs,
)  # type: ignore[import-untyped]

from ecc_primitives.ec_arithmetic.ec_find_key import FindECCPrivateKey_anb, ECPoint

default_generalizer = (ignore_alloc_free, ignore_split_join, generalize_cvs)


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


def create_ecc_circuit(cfg: ECCInstance):
    G, P = build_instance(cfg)
    return FindECCPrivateKey_anb(cfg.n, G, P, cfg.wa, cfg.wm)
