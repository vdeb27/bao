#!/usr/bin/env bash
# Bootstrap a Debian/Ubuntu VPS for Bao training.
# Idempotent: re-run safely.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> apt deps"
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev curl git \
    python3 python3-venv python3-pip python3-dev

# Use whatever python3 is on PATH; require >= 3.10 (PyTorch + our code).
PY=python3
PY_VERSION="$("$PY" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
PY_MAJOR="${PY_VERSION%%.*}"
PY_MINOR="${PY_VERSION##*.}"
if [ "$PY_MAJOR" -lt 3 ] || { [ "$PY_MAJOR" -eq 3 ] && [ "$PY_MINOR" -lt 10 ]; }; then
    echo "ERROR: need Python >= 3.10, found $PY_VERSION" >&2
    exit 1
fi
echo "    using Python $PY_VERSION"

echo "==> rustup (skipped if already installed)"
if ! command -v cargo >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi

echo "==> python venv for PyO3 binding"
if [ ! -d bindings/py/.venv ]; then
    "$PY" -m venv bindings/py/.venv
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
