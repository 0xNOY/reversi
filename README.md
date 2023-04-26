# Reversi / リバーシ

## Quick Start

```bash
git clone https://github.com/0xNOY/reversi.git
cd reversi
cargo run --release
```

## BOT/UIの作成

`reversi.rs`の`Player`トレイトを実装した構造体を作成することで、BOTや任意のプラットフォームで動作するUIを作成することができます。
デフォルトでは、`CliPlayer`と`WeekBot`が実装されています。
`CliPlayer`は、コマンドラインから入力を受け付けるプレイヤーです。`WeekBot`は、瞬間的な石の数の増加量が最も大きい手を選択するボットです。