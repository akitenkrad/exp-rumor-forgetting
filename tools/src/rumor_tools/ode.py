"""Independent mean-field reference solver for the rumor model."""

from __future__ import annotations

import argparse
import csv
import math
from pathlib import Path


def _derivative(i: float, s: float, r: float, lambda_: float, sigma: float, delta: float) -> tuple[float, float, float]:
    infection = lambda_ * i * s
    stifling = sigma * s * (s + r)
    return (-infection + delta * (s + r), infection - stifling - delta * s, stifling - delta * r)


def _rk4_step(state: tuple[float, float, float], dt: float, lambda_: float, sigma: float, delta: float) -> tuple[float, float, float]:
    def add(base: tuple[float, ...], slope: tuple[float, ...], scale: float) -> tuple[float, float, float]:
        return tuple(base[j] + scale * slope[j] for j in range(3))  # type: ignore[return-value]

    k1 = _derivative(*state, lambda_, sigma, delta)
    k2 = _derivative(*add(state, k1, dt / 2), lambda_, sigma, delta)
    k3 = _derivative(*add(state, k2, dt / 2), lambda_, sigma, delta)
    k4 = _derivative(*add(state, k3, dt), lambda_, sigma, delta)
    return tuple(state[j] + dt * (k1[j] + 2 * k2[j] + 2 * k3[j] + k4[j]) / 6 for j in range(3))  # type: ignore[return-value]


def integrate(*, lambda_: float, sigma: float, delta: float, s0: float = 0.01, t_max: float = 1000.0, dt: float = 0.01) -> tuple[float, float, float]:
    """Integrate the plan's mean-field ODE with fixed-step RK4."""
    state = (1.0 - s0, s0, 0.0)
    for _ in range(math.ceil(t_max / dt)):
        state = _rk4_step(state, dt, lambda_, sigma, delta)
        if delta == 0.0 and state[1] < 1e-9:
            break
    total = sum(state)
    return tuple(value / total for value in state)  # type: ignore[return-value]


def final_ignorant(lambda_: float, sigma: float) -> float:
    """Solve A-1 for the non-trivial root in (0, 1)."""
    ratio = sigma / lambda_

    def equation(i: float) -> float:
        return (1.0 - i) * (1.0 + ratio) + ratio * math.log(i)

    lo, hi = 1e-15, 1.0 - 1e-12
    for _ in range(200):
        mid = (lo + hi) / 2
        if equation(mid) > 0:
            hi = mid
        else:
            lo = mid
    return (lo + hi) / 2


def _label(value: float) -> str:
    return f"{value:.2f}".replace(".", "p")


def generate_reference(path: Path) -> None:
    """Generate all A-1 and V-1--V-3 reference targets."""
    rows: list[dict[str, str | float]] = [{"target_id": "a1.l1p00_s1p00", "value": final_ignorant(1.0, 1.0), "source": "a1_equation_s0_limit"}]
    for validation, lambda_, sigma in [("v1", 1.0, 1.0), ("v2", 0.5, 1.0), ("v2", 1.0, 0.5)]:
        i, _, _ = integrate(lambda_=lambda_, sigma=sigma, delta=0.0)
        rows.append({"target_id": f"{validation}.ode.l{_label(lambda_)}_s{_label(sigma)}", "value": i, "source": "rk4_s0_0p01"})
    for lambda_, delta in [(0.8, 0.2), (0.6, 0.3), (1.0, 0.5)]:
        i, s, r = integrate(lambda_=lambda_, sigma=1.0, delta=delta)
        prefix = f"l{_label(lambda_)}_d{_label(delta)}"
        for quantity, value in [("i", i), ("s", s), ("known", s + r)]:
            rows.append({"target_id": f"a3.{quantity}.{prefix}", "value": value, "source": "rk4_s0_0p01"})
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=["target_id", "value", "source"])
        writer.writeheader()
        writer.writerows(rows)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="Generate independent ODE reference values")
    parser.add_argument("--out", type=Path, default=Path("reference.csv"))
    args = parser.parse_args(argv)
    generate_reference(args.out)
    print(args.out)


if __name__ == "__main__":
    main()
