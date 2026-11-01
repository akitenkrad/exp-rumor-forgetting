"""Shared plotting style."""

import matplotlib.pyplot as plt

PURPLE = "#534AB7"
TEAL = "#0F6E56"
GRAY = "#5F5E5A"


def apply_style() -> None:
    plt.rcParams.update({"figure.dpi": 140, "savefig.dpi": 180, "axes.spines.top": False, "axes.spines.right": False, "font.size": 10})
