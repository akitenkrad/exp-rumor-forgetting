from __future__ import annotations

from pathlib import Path

import pandas as pd

from rumor_tools.plot import plot_results


def test_plot_writes_both_figure_options(tmp_path: Path) -> None:
    frame = pd.DataFrame(
        {
            "lambda": [0.2, 0.4, 0.2, 0.4],
            "delta": [0.1, 0.1, 0.3, 0.3],
            "p_survive": [0.2, 0.8, 0.0, 0.6],
            "s_qs": [0.01, 0.08, 0.0, 0.03],
            "known_qs": [0.1, 0.3, 0.0, 0.2],
        }
    )
    plot_results(frame, None, tmp_path)
    assert (tmp_path / "survival-boundary.png").is_file()
    assert (tmp_path / "prevalence-panels.png").is_file()
