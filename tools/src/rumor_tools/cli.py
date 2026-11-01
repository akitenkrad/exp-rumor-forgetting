"""Command-line dispatcher for reference generation and plotting."""
from __future__ import annotations

import argparse
import sys


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        prog="rumor-tools",
        description="Rumor-forgetting analysis tools",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("ode", help="generate independent ODE references", add_help=False)
    subparsers.add_parser("plot", help="plot runvault aggregates", add_help=False)
    subparsers.add_parser("blog-figures", help="draw editorial blog figures", add_help=False)

    argv = sys.argv[1:] if argv is None else argv
    if not argv or argv[0] in {"-h", "--help"}:
        parser.parse_args(argv)
        return

    command = argv[0]
    rest = argv[1:]
    if command == "ode":
        from rumor_tools.ode import main as run_main
        run_main(rest)
    elif command == "plot":
        from rumor_tools.plot import main as run_main
        run_main(rest)
    elif command == "blog-figures":
        from rumor_tools.blog_figures import main as run_main
        run_main(rest)
    else:
        parser.parse_args(argv)  # 不正なサブコマンドはここでエラーになる


if __name__ == "__main__":
    main()
