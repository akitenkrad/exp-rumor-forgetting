"""Load aggregate rumor results from CSV files or runvault run directories."""

from pathlib import Path
import pandas as pd


def load_aggregates(source: str | Path) -> pd.DataFrame:
    path = Path(source)
    if path.is_dir():
        candidates = [path / "artifacts" / "aggregates.csv", path / "aggregates.csv"]
        path = next((candidate for candidate in candidates if candidate.is_file()), candidates[0])
    frame = pd.read_csv(path)
    required = {"lambda", "delta", "p_survive", "s_qs", "known_qs"}
    missing = required - set(frame.columns)
    if missing:
        raise ValueError(f"aggregate input is missing columns: {', '.join(sorted(missing))}")
    return frame
