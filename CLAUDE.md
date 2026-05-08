# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A fully featured, playable implementation of **Bao la Kiswahili** — a complex traditional East African mancala game played on a 4×8 board. The project also supports **Bao la Kujifunza**, the simplified teaching variant.

## Game terminology

Use the correct Swahili terms throughout the codebase:

| Term | Meaning |
|------|---------|
| kete | seeds/counters (game pieces) |
| shimo / vichwa (pl.) | pit/hole |
| kichwa / vichwa (pl.) | first or last hole in the front row (positions 1 and 8) |
| kimbi | the four flank holes of the front row, including the kichwa: positions 1, 2, 7, and 8. Kimbi is a superset of kichwa |
| nyumba | the special square hole |
| mbele | front row |
| nyuma | back row |
| ghala | storage hole for kete during namu stage |
| namu | first stage of the game |
| mtaji | second stage of the game; also: a capturing move |
| kula | to capture seeds |
| takata / kutakata | a move without capture |
| endelea | relay sowing (continuing a move into another lap) |
| hamna | winning condition: capturing all opponent's kete from mbele |
| mkononi | victory achieved during the namu stage |
| zamu | turn |
| mchezaji | player |
| kunamua | the act of playing a namu move (verb form of namu) |
| safari | during a capture-sow, the player's choice to empty their own functional nyumba in order to continue the sow |
| kichwa-selectie | player choice of which kichwa pit (left or right) is the start of a capture-sow; only a real choice when the capture happens in middle mbele with no prior direction |
| kutakatia | mtaji-phase blocking mechanism: after specific takata moves, an opponent pit is marked so it can only be captured (not sown into) for the next few turns |

## Architecture

The codebase is split into three decoupled layers that all share the same rules engine:

1. **Rules engine** (`engine/`) — Pure game logic with no UI or ML dependencies. Handles all rules for both variants, move generation, and win detection. This is the single source of truth used by the UI, the training pipeline, and ELO matches.

2. **UI** (`ui/`) — Renders the board, handles input, animations (kete moving step by step via endelea, or instant), legal move highlighting, advantage bar, and ELO opponent selection.

3. **Training pipeline** (`training/`) — Self-play training loop (AlphaZero-style: MCTS + neural network policy/value heads). Saves model checkpoints with associated ELO ratings. May use different tech or run headlessly.

## ELO system

- Single ELO pool, scoped to Bao la Kiswahili. Bao la Kujifunza is a feature-flag variant of the same engine and shares no separate ratings initially; a per-variant pool may be added later if Kujifunza moves out of teaching-only scope.
- Human player ELO and AI checkpoint ELO are stored in `data/elo.json`.
- AI checkpoints are stored in `models/` and labeled with their ELO at save time.
