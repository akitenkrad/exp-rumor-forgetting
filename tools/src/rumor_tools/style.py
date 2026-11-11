"""Editorial-minimal matplotlib styling.

The colors, fonts and rcParams come from the shared blog-figure-style package
(https://github.com/akitenkrad/blog-figure-style). This module only re-exports
the names that plot.py and blog_figures.py have always used.
"""
from blog_figure_style.compat import *  # noqa: F401,F403
from blog_figure_style.compat import apply_editorial_minimal, apply_style  # noqa: F401
from blog_figure_style.mpl import JAPANESE_FONT_FALLBACKS, select_japanese_font  # noqa: F401
from matplotlib import font_manager, rcParams  # noqa: F401  (tests may patch style.font_manager)
