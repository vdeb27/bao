#!/usr/bin/env bash
# Bootstrap a Debian/Ubuntu VPS for Bao training.
# Idempotent: re-run safely.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> apt deps"
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev curl git \
    python3.12 python3.12-venv python3-pip

echo "==> rustup (skipped if already installed)"
if ! command -v cargo >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi

echo "==> python venv for PyO3 binding"
if [ ! -d bindings/py/.venv ]; then
    python3.12 -m venv bindings/py/.venv
fi
bindings/py/.venv/bin/pip install --upgrade pip
bindings/py/.venv/bin/pip install maturin torch numpy

echo "==> build engine (release)"
cargo build --release -p bao-engine
cargo build --release --example generate_positions -p bao-engine

echo "==> build PyO3 binding"
cd bindings/py
VIRTUAL_ENV="$(pwd)/.venv" .venv/bin/maturin develop --release
cd "$REPO_ROOT"

echo "==> done"
echo
echo "Next steps:"
echo "  - rsync shards and checkpoints from local (see docs/vps-setup.md if present)"
echo "  - install Claude Code: https://docs.claude.com/claude-code"
echo "  - run training/scripts/heuristic_vs_random.py to verify the engine"
