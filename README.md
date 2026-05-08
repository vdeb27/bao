# Bao la Kiswahili

A playable implementation of **Bao la Kiswahili** — a complex traditional East African mancala game played on a 4×8 board — together with a self-trained AlphaZero-style AI. Bao la Kujifunza, the simplified teaching variant, is supported via a feature flag on the same engine.

See `CLAUDE.md` for project conventions and Swahili terminology, `RULES.md` for the rules of play (consolidated from multiple sources), and `/home/johan/.claude/plans/implementeer-al-deze-edits-fizzy-kitten.md` for the architecture and implementation plan.

## Workspace layout

| Path | Contents |
|------|----------|
| `crates/bao-engine/` | Rust rules engine, MCTS, and event stream (single source of truth) |
| `bindings/py/` | PyO3 wrapper exposing the engine to the training pipeline |
| `bindings/wasm/` | wasm-bindgen wrapper exposing the engine to the browser |
| `training/` | PyTorch self-play training (AlphaZero-style) |
| `ui/` | React + Canvas front-end |
| `golden/` | Golden game records used as test fixtures |
| `data/` | ELO export consumed by the browser bundle |
| `docs/` | Architecture notes, BAN spec, training guide |

## Development phases

The project is staged through fase 0 (rules consolidation) → fase 1 (engine) → fase 2 (UI) → fase 3 (alpha-beta baseline AI) → fases 4–6 (AlphaZero training, ELO, polish) → fase 7 (release). The current state is **fase 0**.
