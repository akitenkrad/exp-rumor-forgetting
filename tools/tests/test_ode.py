from __future__ import annotations

import csv
from pathlib import Path

import pytest

from rumor_tools.ode import final_ignorant, generate_reference, integrate


def test_a1_limit_is_about_0p2032() -> None:
    assert final_ignorant(1.0, 1.0) == pytest.approx(0.2032, abs=5e-4)


def test_rk4_conserves_population_and_approaches_a3() -> None:
    i, s, r = integrate(lambda_=0.8, sigma=1.0, delta=0.2, s0=0.01, t_max=1000.0, dt=0.01)
    assert i + s + r == pytest.approx(1.0, abs=1e-10)
    assert i == pytest.approx(2.0 / 3.0, rel=1e-4)
    assert s == pytest.approx(0.125, rel=1e-4)


def test_reference_csv_has_all_v1_to_v3_targets(tmp_path: Path) -> None:
    out = tmp_path / "reference.csv"
    generate_reference(out)
    with out.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    ids = {row["target_id"] for row in rows}
    assert "a1.l1p00_s1p00" in ids
    assert "v1.ode.l1p00_s1p00" in ids
    assert "v2.ode.l0p50_s1p00" in ids
    assert "a3.known.l1p00_d0p50" in ids
