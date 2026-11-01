[English](cli.md) | **日本語**

# CLI と再現手順

全コマンドをリポジトリのルートから実行します．開発中は必ず scratch run を作成します．

```bash
cargo run --release -- run --config configs/main.toml --seeds 0 --scratch
cargo run --release -- check --scratch
```

`run` は `--seeds 0,1`，`--name LABEL`，`--threads N` を受け付けます．`configs/` の TOML ファイルが主格子，完全混合，感度分析の各格子を定義します．

独立な基準値の生成とテストは次の通りです．

```bash
uv run rumor-tools ode --out reference.csv
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -p rumor-simulation --check
uv run pytest tools/tests
```

本番 run の作成後，次のコマンドで 2 案の図を描きます．

```bash
uv run rumor-tools plot --run RUN_DIR --complete COMPLETE_RUN_DIR --out fig
uv run rumor-tools blog-figures --runs-root results/rumor --out fig/blog
```

`survival-boundary.png` には，線形補間した `p_survive = 0.5` の等高線と理論境界 `lambda = delta` を描きます．もう一方の `prevalence-panels.png` は準定常量を示します．
