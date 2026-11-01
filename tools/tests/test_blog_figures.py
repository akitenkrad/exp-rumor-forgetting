from __future__ import annotations

import json
import struct
from pathlib import Path

from matplotlib.figure import Figure
import pandas as pd
import pytest

from rumor_tools.blog_figures import select_latest_runs
from rumor_tools.cli import main


REQUIRED_CONFIGS = [
    "complete",
    "main",
    "sens_k4",
    "sens_k16",
    "sens_n2000",
    "sens_t4000",
    "sens_sigma0p5",
    "sens_stifler_only",
]


def _write_run(
    root: Path,
    run_name: str,
    config_name: str,
    *,
    state: str = "finished",
    finished_at: str | None = "2026-01-01T00:00:00+09:00",
) -> Path:
    run = root / run_name
    artifacts = run / "artifacts"
    artifacts.mkdir(parents=True)
    (run / "config.json").write_text(
        json.dumps({"parameters": {"config": f"configs/{config_name}.toml"}}),
        encoding="utf-8",
    )
    (run / "status.json").write_text(
        json.dumps({"state": state, "finished_at": finished_at}),
        encoding="utf-8",
    )

    aggregate_rows = []
    trial_rows = []
    for transmission in (0.25, 1.0):
        for forgetting in (0.0, 0.05, 0.5, 1.0):
            survives = transmission == 1.0 and forgetting in (0.05, 0.5)
            aggregate_rows.append(
                {
                    "lambda": transmission,
                    "delta": forgetting,
                    "trials": 2,
                    "surviving_trials": 2 if survives else 0,
                    "p_survive": 1.0 if survives else 0.0,
                    "i_qs": 0.6 if survives else None,
                    "s_qs": 0.2 if survives else None,
                    "known_qs": 0.4 if survives else None,
                    "ever": 0.5,
                    "ignorant_final": 0.5,
                }
            )
            for seed, spreaders, known in ((0, 0.15, 0.35), (1, 0.25, 0.45)):
                trial_rows.append(
                    {
                        "seed": seed,
                        "lambda": transmission,
                        "sigma": 1.0,
                        "delta": forgetting,
                        "survived": survives,
                        "s_qs": spreaders if survives else None,
                        "known_qs": known if survives else None,
                    }
                )
    pd.DataFrame(aggregate_rows).to_csv(artifacts / "aggregates.csv", index=False)
    pd.DataFrame(trial_rows).to_csv(artifacts / "trials.csv", index=False)
    return run


def _png_size(path: Path) -> tuple[int, int]:
    with path.open("rb") as png:
        assert png.read(8) == b"\x89PNG\r\n\x1a\n"
        length = struct.unpack(">I", png.read(4))[0]
        assert png.read(4) == b"IHDR"
        width, height = struct.unpack(">II", png.read(length)[:8])
    return width, height


def test_select_latest_finished_run_for_each_config(tmp_path: Path) -> None:
    old = _write_run(
        tmp_path,
        "run_old_main",
        "main",
        finished_at="2026-01-01T00:00:00+09:00",
    )
    newest_finished = _write_run(
        tmp_path,
        "run_new_main",
        "main",
        finished_at="2026-01-02T00:00:00+09:00",
    )
    _write_run(
        tmp_path,
        "run_newer_but_running_main",
        "main",
        state="running",
        finished_at=None,
    )

    selected = select_latest_runs(tmp_path)

    assert selected["configs/main.toml"] == newest_finished
    assert selected["configs/main.toml"] != old


def test_blog_figures_cli_writes_three_png_files(tmp_path: Path) -> None:
    runs_root = tmp_path / "runs"
    for index, config_name in enumerate(REQUIRED_CONFIGS):
        _write_run(
            runs_root,
            f"run_{index:02d}_{config_name}",
            config_name,
            finished_at=f"2026-01-{index + 1:02d}T00:00:00+09:00",
        )
    out = tmp_path / "figures"

    main(["blog-figures", "--runs-root", str(runs_root), "--out", str(out)])

    assert _png_size(out / "boundary-map.png") == (1600, 1100)
    assert _png_size(out / "boundary-sensitivity.png") == (1800, 800)
    assert _png_size(out / "levels-lambda1.png") == (1600, 900)


@pytest.fixture
def figure_contracts(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> dict[str, object]:
    runs_root = tmp_path / "runs"
    for index, config_name in enumerate(REQUIRED_CONFIGS):
        _write_run(
            runs_root,
            f"run_{index:02d}_{config_name}",
            config_name,
            finished_at=f"2026-01-{index + 1:02d}T00:00:00+09:00",
        )

    captured: dict[str, object] = {}

    def capture_savefig(figure: Figure, path: Path, **_: object) -> None:
        name = Path(path).name
        primary_ax = figure.axes[0]
        texts = {text.get_text(): text.get_position() for text in primary_ax.texts}
        if name == "boundary-map.png":
            captured["map_texts"] = texts
        elif name == "levels-lambda1.png":
            captured["level_texts"] = texts
            captured["level_ylim"] = primary_ax.get_ylim()

    monkeypatch.setattr(Figure, "savefig", capture_savefig)
    main(
        [
            "blog-figures",
            "--runs-root",
            str(runs_root),
            "--out",
            str(tmp_path / "figures"),
        ]
    )
    return captured


def test_boundary_map_theory_label_is_below_theory_line(
    figure_contracts: dict[str, object],
) -> None:
    texts = figure_contracts["map_texts"]
    x, y = texts["理論 λ = δ"]

    assert x == pytest.approx(0.85)
    assert y == pytest.approx(0.72)
    assert y < x - 0.05


def test_levels_has_four_left_labels_and_compact_axis(
    figure_contracts: dict[str, object],
) -> None:
    texts = figure_contracts["level_texts"]
    series_labels = {
        "知っている人 (ネットワーク)",
        "拡散者 (ネットワーク)",
        "知っている人 (完全混合)",
        "拡散者 (完全混合)",
    }

    assert series_labels <= texts.keys()
    assert all(texts[label][0] <= 0.1 for label in series_labels)
    assert len({texts[label][1] for label in series_labels}) == 4
    assert texts["破線: 完全混合の理論値"][0] >= 0.7
    assert texts["破線: 完全混合の理論値"][1] >= 0.45
    assert "理論" not in texts
    assert figure_contracts["level_ylim"] == pytest.approx((0.0, 0.55))
