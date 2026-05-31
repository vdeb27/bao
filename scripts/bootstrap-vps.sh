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
# Many VPSes mount /tmp as a small tmpfs (RAM-backed). pip unpacks wheels
# there by default, so a big torch wheel blows the tmpfs limit ("Disk quota
# exceeded") even when $HOME has plenty of space. Point pip's tmp at a dir
# on the real disk under the repo.
PIP_TMP="$REPO_ROOT/.pip-tmp"
mkdir -p "$PIP_TMP"
export TMPDIR="$PIP_TMP"

bindings/py/.venv/bin/pip install --no-cache-dir --upgrade pip
# CPU-only torch is ~200 MB vs ~800 MB with CUDA wheels — we train CPU-only.
bindings/py/.venv/bin/pip install --no-cache-dir \
    --index-url https://download.pytorch.org/whl/cpu torch
bindings/py/.venv/bin/pip install --no-cache-dir maturin numpy
rm -rf "$PIP_TMP"

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
