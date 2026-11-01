"""Draw the three editorial figures used by the blog article."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import matplotlib.pyplot as plt
from matplotlib.axes import Axes
from matplotlib.colors import LinearSegmentedColormap
import numpy as np
import pandas as pd

from rumor_tools.style import (
    ACCENT,
    ACCENT_FILL,
    ACCENT_TEXT,
    BACKGROUND,
    HAIRLINE,
    INK,
    MUTED,
    apply_editorial_minimal,
)

CONFIG_PATHS = {
    "main": "configs/main.toml",
    "complete": "configs/complete.toml",
    "sens_k4": "configs/sens_k4.toml",
    "sens_k16": "configs/sens_k16.toml",
    "sens_n2000": "configs/sens_n2000.toml",
    "sens_t4000": "configs/sens_t4000.toml",
    "sens_sigma0p5": "configs/sens_sigma0p5.toml",
    "sens_stifler_only": "configs/sens_stifler_only.toml",
}

SURVIVAL_CMAP = LinearSegmentedColormap.from_list(
    "rumor_survival", [BACKGROUND, ACCENT_FILL, ACCENT]
)


def _read_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def select_latest_runs(runs_root: str | Path) -> dict[str, Path]:
    """Return the latest finished run for every ``parameters.config`` value."""
    candidates: dict[str, tuple[str, str, Path]] = {}
    for run in sorted(Path(runs_root).glob("run_*")):
        config_path = run / "config.json"
        status_path = run / "status.json"
        if not config_path.is_file() or not status_path.is_file():
            continue
        status = _read_json(status_path)
        if status.get("state") != "finished":
            continue
        config = _read_json(config_path).get("parameters", {}).get("config")
        if not isinstance(config, str):
            continue
        finished_at = status.get("finished_at")
        timestamp = finished_at if isinstance(finished_at, str) else ""
        key = (timestamp, run.name, run)
        previous = candidates.get(config)
        if previous is None or key[:2] > previous[:2]:
            candidates[config] = key
    return {config: item[2] for config, item in candidates.items()}


def _required_runs(runs_root: str | Path) -> dict[str, Path]:
    selected = select_latest_runs(runs_root)
    missing = [path for path in CONFIG_PATHS.values() if path not in selected]
    if missing:
        raise ValueError("finished runs are missing for: " + ", ".join(missing))
    return {name: selected[path] for name, path in CONFIG_PATHS.items()}


def _load_aggregates(run: Path) -> pd.DataFrame:
    frame = pd.read_csv(run / "artifacts" / "aggregates.csv")
    required = {"lambda", "delta", "p_survive", "s_qs", "known_qs"}
    missing = required - set(frame.columns)
    if missing:
        raise ValueError(
            f"{run} aggregates are missing columns: {', '.join(sorted(missing))}"
        )
    return frame


def _load_trials(run: Path) -> pd.DataFrame:
    frame = pd.read_csv(run / "artifacts" / "trials.csv")
    required = {"lambda", "delta", "s_qs", "known_qs"}
    missing = required - set(frame.columns)
    if missing:
        raise ValueError(
            f"{run} trials are missing columns: {', '.join(sorted(missing))}"
        )
    return frame


def _survival_grid(frame: pd.DataFrame) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    pivot = (
        frame.pivot(index="lambda", columns="delta", values="p_survive")
        .sort_index()
        .sort_index(axis=1)
    )
    return pivot.columns.to_numpy(), pivot.index.to_numpy(), pivot.to_numpy()


def _draw_boundary(
    ax: Axes,
    frame: pd.DataFrame,
    *,
    color: str,
    linewidth: float,
    linestyle: str = "-",
) -> Any | None:
    deltas, lambdas, survival = _survival_grid(frame)
    finite = survival[np.isfinite(survival)]
    if (
        deltas.size < 2
        or lambdas.size < 2
        or finite.size == 0
        or finite.min() > 0.5
        or finite.max() < 0.5
        or finite.min() == finite.max()
    ):
        return None
    return ax.contour(
        deltas,
        lambdas,
        survival,
        levels=[0.5],
        colors=[color],
        linewidths=[linewidth],
        linestyles=[linestyle],
        zorder=4,
    )


def _label_contour(
    ax: Axes,
    contour: Any | None,
    label: str,
    *,
    color: str,
    position: float,
    offset: tuple[float, float] = (0.012, 0.012),
    size: float = 10,
) -> None:
    if contour is None:
        return
    segments = [segment for segment in contour.allsegs[0] if len(segment)]
    if not segments:
        return
    segment = max(segments, key=len)
    point = segment[min(round((len(segment) - 1) * position), len(segment) - 1)]
    ax.text(
        point[0] + offset[0],
        point[1] + offset[1],
        label,
        color=color,
        fontsize=size,
        ha="left",
        va="center",
        zorder=7,
        bbox={"facecolor": BACKGROUND, "edgecolor": "none", "pad": 1.2, "alpha": 0.88},
    )


def _format_axes(ax: Axes, *, xlabel: bool = True, ylabel: bool = True) -> None:
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.set_xticks(np.linspace(0, 1, 6))
    ax.set_yticks(np.linspace(0, 1, 6))
    ax.grid(color=HAIRLINE, linewidth=0.55, alpha=0.55, zorder=0)
    ax.set_axisbelow(True)
    if xlabel:
        ax.set_xlabel("忘却率 δ")
    if ylabel:
        ax.set_ylabel("伝播率 λ")


def _draw_theory_boundary(
    ax: Axes,
    *,
    label: bool = True,
    label_position: tuple[float, float] = (0.67, 0.72),
    label_alignment: tuple[str, str] = ("left", "bottom"),
) -> None:
    ax.plot([0, 1], [0, 1], color=MUTED, linestyle=(0, (5, 4)), linewidth=1.4, zorder=3)
    if label:
        ax.text(
            *label_position,
            "理論 λ = δ",
            color=MUTED,
            fontsize=10,
            rotation=35,
            ha=label_alignment[0],
            va=label_alignment[1],
            bbox={"facecolor": BACKGROUND, "edgecolor": "none", "pad": 1.0, "alpha": 0.88},
            zorder=7,
        )


def _boundary_map(frames: dict[str, pd.DataFrame], out: Path) -> None:
    fig, ax = plt.subplots(figsize=(8, 5.5), constrained_layout=True)
    deltas, lambdas, survival = _survival_grid(frames["main"])
    mesh = ax.pcolormesh(
        deltas,
        lambdas,
        survival,
        shading="nearest",
        cmap=SURVIVAL_CMAP,
        vmin=0,
        vmax=1,
        rasterized=True,
        zorder=1,
    )
    network = _draw_boundary(
        ax, frames["main"], color=ACCENT_TEXT, linewidth=3.0
    )
    complete = _draw_boundary(ax, frames["complete"], color=INK, linewidth=1.9)
    _draw_theory_boundary(
        ax,
        label_position=(0.85, 0.72),
        label_alignment=("center", "center"),
    )
    _label_contour(
        ax,
        network,
        "ネットワーク (平均 8 人とつながる)",
        color=ACCENT_TEXT,
        position=0.62,
        offset=(0.015, 0.025),
    )
    _label_contour(
        ax,
        complete,
        "完全混合",
        color=INK,
        position=0.35,
        offset=(0.015, -0.028),
    )
    _format_axes(ax)
    colorbar = fig.colorbar(mesh, ax=ax, pad=0.025, fraction=0.05)
    colorbar.set_label("噂が残った試行の割合")
    colorbar.outline.set_edgecolor(HAIRLINE)
    colorbar.outline.set_linewidth(0.6)
    fig.savefig(out / "boundary-map.png", dpi=200)
    plt.close(fig)


def _sensitivity_panel(
    ax: Axes,
    frames: dict[str, pd.DataFrame],
    curves: list[tuple[str, str, str, float, str, float, tuple[float, float]]],
    heading: str,
) -> None:
    for name, label, color, width, linestyle, position, offset in curves:
        contour = _draw_boundary(
            ax,
            frames[name],
            color=color,
            linewidth=width,
            linestyle=linestyle,
        )
        _label_contour(
            ax,
            contour,
            label,
            color=color,
            position=position,
            offset=offset,
            size=9,
        )
    _draw_theory_boundary(ax)
    _format_axes(ax)
    ax.text(
        0.02,
        0.97,
        heading,
        transform=ax.transAxes,
        color=INK,
        fontsize=12,
        fontweight="semibold",
        ha="left",
        va="top",
        bbox={"facecolor": BACKGROUND, "edgecolor": "none", "pad": 1.5, "alpha": 0.9},
        zorder=8,
    )


def _boundary_sensitivity(frames: dict[str, pd.DataFrame], out: Path) -> None:
    fig, axes = plt.subplots(1, 2, figsize=(9, 4), constrained_layout=True)
    left_curves = [
        ("complete", "完全混合", INK, 1.8, "-", 0.78, (0.012, -0.035)),
        ("sens_k16", r"$\langle k\rangle$=16", MUTED, 1.6, "-.", 0.66, (0.012, 0.025)),
        ("main", r"$\langle k\rangle$=8 (主)", ACCENT_TEXT, 2.8, "-", 0.48, (0.012, 0.025)),
        ("sens_k4", r"$\langle k\rangle$=4", MUTED, 1.5, (0, (2, 2)), 0.30, (0.012, -0.025)),
    ]
    right_curves = [
        ("main", r"主 ($\langle k\rangle$=8)", ACCENT_TEXT, 2.8, "-", 0.80, (0.012, 0.025)),
        ("sens_n2000", "N = 2,000", INK, 1.5, "-", 0.66, (0.012, -0.032)),
        ("sens_t4000", "T = 4,000", MUTED, 1.6, "-.", 0.52, (0.012, 0.035)),
        ("sens_sigma0p5", "σ = 0.5", MUTED, 1.5, (0, (2, 2)), 0.38, (0.012, -0.035)),
        (
            "sens_stifler_only",
            "抑制者だけが忘れる",
            INK,
            1.5,
            (0, (6, 3)),
            0.22,
            (0.012, 0.032),
        ),
    ]
    _sensitivity_panel(axes[0], frames, left_curves, "つながりの数")
    _sensitivity_panel(axes[1], frames, right_curves, "その他の条件")
    axes[1].set_ylabel("")
    fig.savefig(out / "boundary-sensitivity.png", dpi=200)
    plt.close(fig)


def _lambda_one(frame: pd.DataFrame) -> pd.DataFrame:
    return frame.loc[np.isclose(frame["lambda"], 1.0)].sort_values("delta")


def _plot_level_series(
    ax: Axes,
    aggregates: pd.DataFrame,
    trials: pd.DataFrame,
    quantity: str,
    *,
    color: str,
    marker: str,
    filled: bool,
    linewidth: float,
    zorder: int,
) -> Any | None:
    values = _lambda_one(aggregates).dropna(subset=[quantity])
    if "surviving_trials" in values:
        values = values.loc[values["surviving_trials"] > 0]
    if values.empty:
        return None
    trial_values = _lambda_one(trials).dropna(subset=[quantity])
    ranges = trial_values.groupby("delta")[quantity].agg(["min", "max"])
    selected_ranges = ranges.reindex(values["delta"].to_numpy())
    y = values[quantity].to_numpy()
    lower = np.maximum(0, y - selected_ranges["min"].to_numpy())
    upper = np.maximum(0, selected_ranges["max"].to_numpy() - y)
    marker_face = color if filled else BACKGROUND
    return ax.errorbar(
        values["delta"],
        y,
        yerr=np.vstack([lower, upper]),
        color=color,
        linewidth=linewidth,
        marker=marker,
        markersize=6,
        markerfacecolor=marker_face,
        markeredgecolor=color,
        markeredgewidth=1.4,
        capsize=2.5,
        elinewidth=0.9,
        alpha=0.96,
        zorder=zorder,
    )


def _label_line(
    ax: Axes,
    artist: Any | None,
    text: str,
    *,
    color: str,
    index: int = -1,
    offset: tuple[float, float] = (0.015, 0),
    position: tuple[float, float] | None = None,
) -> None:
    if artist is None:
        return
    line = artist.lines[0]
    x, y = line.get_data()
    if len(x) == 0:
        return
    label_position = position or (
        float(x[index]) + offset[0],
        float(y[index]) + offset[1],
    )
    ax.text(
        *label_position,
        text,
        color=color,
        fontsize=10,
        ha="left",
        va="center",
        bbox={"facecolor": BACKGROUND, "edgecolor": "none", "pad": 1.0, "alpha": 0.88},
        zorder=8,
    )


def _levels_lambda_one(
    frames: dict[str, pd.DataFrame], runs: dict[str, Path], out: Path
) -> None:
    fig, ax = plt.subplots(figsize=(8, 4.5), constrained_layout=True)
    main_trials = _load_trials(runs["main"])
    complete_trials = _load_trials(runs["complete"])
    network_s = _plot_level_series(
        ax,
        frames["main"],
        main_trials,
        "s_qs",
        color=ACCENT_TEXT,
        marker="o",
        filled=True,
        linewidth=2.3,
        zorder=6,
    )
    network_known = _plot_level_series(
        ax,
        frames["main"],
        main_trials,
        "known_qs",
        color=ACCENT_TEXT,
        marker="o",
        filled=False,
        linewidth=2.3,
        zorder=6,
    )
    complete_s = _plot_level_series(
        ax,
        frames["complete"],
        complete_trials,
        "s_qs",
        color=INK,
        marker="s",
        filled=True,
        linewidth=1.5,
        zorder=5,
    )
    complete_known = _plot_level_series(
        ax,
        frames["complete"],
        complete_trials,
        "known_qs",
        color=INK,
        marker="s",
        filled=False,
        linewidth=1.5,
        zorder=5,
    )

    delta = np.linspace(0.001, 0.999, 500)
    spreaders = delta * (1 - delta) / (1 + delta)
    known = (1 - delta) / 2
    ax.plot(delta, spreaders, color=MUTED, linestyle=(0, (5, 4)), linewidth=1.4, zorder=3)
    ax.plot(delta, known, color=MUTED, linestyle=(0, (5, 4)), linewidth=1.4, zorder=3)

    _label_line(
        ax,
        network_s,
        "拡散者 (ネットワーク)",
        color=ACCENT_TEXT,
        position=(0.055, 0.155),
    )
    _label_line(
        ax,
        network_known,
        "知っている人 (ネットワーク)",
        color=ACCENT_TEXT,
        position=(0.055, 0.365),
    )
    _label_line(
        ax,
        complete_s,
        "拡散者 (完全混合)",
        color=INK,
        position=(0.055, 0.215),
    )
    _label_line(
        ax,
        complete_known,
        "知っている人 (完全混合)",
        color=INK,
        position=(0.055, 0.525),
    )
    ax.text(
        0.97,
        0.515,
        "破線: 完全混合の理論値",
        color=MUTED,
        fontsize=9,
        ha="right",
        va="top",
        bbox={"facecolor": BACKGROUND, "edgecolor": "none", "pad": 1.0, "alpha": 0.88},
        zorder=8,
    )
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 0.55)
    ax.set_xticks(np.linspace(0, 1, 6))
    ax.set_yticks(np.arange(0, 0.51, 0.1))
    ax.set_xlabel("忘却率 δ")
    ax.set_ylabel("人口に占める割合")
    ax.grid(color=HAIRLINE, linewidth=0.55, alpha=0.55)
    ax.set_axisbelow(True)
    fig.savefig(out / "levels-lambda1.png", dpi=200)
    plt.close(fig)


def draw_blog_figures(runs_root: str | Path, out: str | Path) -> None:
    """Select production runs and write the three requested PNG figures."""
    apply_editorial_minimal()
    runs = _required_runs(runs_root)
    frames = {name: _load_aggregates(run) for name, run in runs.items()}
    output = Path(out)
    output.mkdir(parents=True, exist_ok=True)
    _boundary_map(frames, output)
    _boundary_sensitivity(frames, output)
    _levels_lambda_one(frames, runs, output)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="Draw editorial blog figures")
    parser.add_argument("--runs-root", type=Path, default=Path("results/rumor"))
    parser.add_argument("--out", type=Path, default=Path("fig/blog"))
    args = parser.parse_args(argv)
    draw_blog_figures(args.runs_root, args.out)


if __name__ == "__main__":
    main()
