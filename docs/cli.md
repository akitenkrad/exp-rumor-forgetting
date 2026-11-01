**English** | [日本語](cli.ja.md)

# CLI and reproduction

Run all commands from the repository root. During development, always create scratch runs:

```bash
cargo run --release -- run --config configs/main.toml --seeds 0 --scratch
cargo run --release -- check --scratch
```

`run` accepts `--seeds 0,1`, `--name LABEL`, and `--threads N`. The TOML files in `configs/` define the main grid, complete mixing, and sensitivity grids.

Generate independent reference values and run the test suites with:

```bash
uv run rumor-tools ode --out reference.csv
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -p rumor-simulation --check
uv run pytest tools/tests
```

After production runs exist, draw both planned figure alternatives with:

```bash
uv run rumor-tools plot --run RUN_DIR --complete COMPLETE_RUN_DIR --out fig
```

The plot command writes `survival-boundary.png`, including the interpolated `p_survive = 0.5` contour and the theoretical `lambda = delta` line, plus `prevalence-panels.png`.
