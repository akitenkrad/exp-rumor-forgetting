<p align="center"><img src="docs/assets/hero.svg" width="100%"></p>

[English](README.md) | **日本語**

# Rumor Forgetting

忘却を伴う連続時間 I/S/R 噂モデルを調べる Zenn 記事のコンパニオンコードです．Rust の Gillespie シミュレーション，独立な Python RK4 基準実装，検証，集計，可視化を収録しています．

## クイックスタート

Rust と [uv](https://docs.astral.sh/uv/) が必要です．開発中の run は必ず `--scratch` 付きで作成してください．

```bash
uv sync
uv run rumor-tools ode --out reference.csv
cargo test
cargo run --release -- check --scratch
cargo run --release -- run --config configs/main.toml --seeds 0 --scratch
```

詳細は [モデルと実装](docs/model.ja.md) および [CLI と再現手順](docs/cli.ja.md) を参照してください．

## ライセンス

[MIT License](LICENSE)
