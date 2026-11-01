"""Draw the two planned figure alternatives from aggregate results."""

from __future__ import annotations

import argparse
from pathlib import Path
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from rumor_tools.load import load_aggregates
from rumor_tools.style import GRAY, PURPLE, TEAL, apply_style


def _grid(frame: pd.DataFrame, value: str) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    pivot = frame.pivot(index="delta", columns="lambda", values=value).sort_index().sort_index(axis=1)
    return pivot.columns.to_numpy(), pivot.index.to_numpy(), pivot.to_numpy()


def plot_results(frame: pd.DataFrame, complete: pd.DataFrame | None, out: Path) -> None:
    apply_style()
    out.mkdir(parents=True, exist_ok=True)
    lambdas, deltas, survival = _grid(frame, "p_survive")
    fig, ax = plt.subplots(figsize=(6.4, 4.8), constrained_layout=True)
    mesh = ax.pcolormesh(lambdas, deltas, survival, shading="nearest", vmin=0, vmax=1, cmap="viridis")
    if lambdas.size >= 2 and deltas.size >= 2 and np.nanmin(survival) <= 0.5 <= np.nanmax(survival):
        ax.contour(lambdas, deltas, survival, levels=[0.5], colors=[PURPLE], linewidths=2)
    limit = min(float(lambdas.max()), float(deltas.max()))
    ax.plot([0, limit], [0, limit], linestyle="--", color=GRAY, label=r"$\lambda=\delta$")
    ax.set(xlabel=r"Transmission probability $\lambda$", ylabel=r"Forgetting rate $\delta$", title="Survival probability and boundary")
    ax.legend(frameon=False)
    fig.colorbar(mesh, ax=ax, label="Survival probability")
    fig.savefig(out / "survival-boundary.png", bbox_inches="tight")
    plt.close(fig)

    fig, axes = plt.subplots(1, 2, figsize=(11, 4.4), constrained_layout=True)
    for ax, value, title, color in [(axes[0], "s_qs", "Quasi-stationary spreaders", PURPLE), (axes[1], "known_qs", "Quasi-stationary informed", TEAL)]:
        x, y, z = _grid(frame, value)
        mesh = ax.pcolormesh(x, y, z, shading="nearest", cmap="magma")
        limit = min(float(x.max()), float(y.max()))
        ax.plot([0, limit], [0, limit], linestyle="--", color=color)
        if complete is not None:
            cx, cy, cz = _grid(complete, "p_survive")
            if cx.size >= 2 and cy.size >= 2 and np.nanmin(cz) <= 0.5 <= np.nanmax(cz):
                ax.contour(cx, cy, cz, levels=[0.5], colors=[GRAY], linewidths=1.5)
        ax.set(xlabel=r"$\lambda$", ylabel=r"$\delta$", title=title)
        fig.colorbar(mesh, ax=ax)
    fig.savefig(out / "prevalence-panels.png", bbox_inches="tight")
    plt.close(fig)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="Plot rumor-forgetting runvault results")
    parser.add_argument("--run", required=True)
    parser.add_argument("--complete")
    parser.add_argument("--out", type=Path, default=Path("fig"))
    args = parser.parse_args(argv)
    plot_results(load_aggregates(args.run), load_aggregates(args.complete) if args.complete else None, args.out)


if __name__ == "__main__":
    main()
