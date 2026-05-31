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
# Many VPSes mount /tmp as a small tmpfs (RAM-backed). pip and maturin unpack
# to $TMPDIR by default, so a big torch wheel (or a maturin build dir) blows
# the tmpfs limit ("Disk quota exceeded") even when $HOME has plenty of space.
# Point the temp dir at the real disk under the repo for the whole build, and
# clean it up at the very end (cleaning it mid-script would pull TMPDIR out
# from under maturin).
LOCAL_TMP="$REPO_ROOT/.build-tmp"
mkdir -p "$LOCAL_TMP"
export TMPDIR="$LOCAL_TMP"
trap 'rm -rf "$LOCAL_TMP"' EXIT

bindings/py/.venv/bin/pip install --no-cache-dir --upgrade pip
# CPU-only torch is ~200 MB vs ~800 MB with CUDA wheels — we train CPU-only.
bindings/py/.venv/bin/pip install --no-cache-dir \
    --index-url https://download.pytorch.org/whl/cpu torch
bindings/py/.venv/bin/pip install --no-cache-dir maturin numpy

echo "==> build engine (release)"
cargo build --release -p bao-engine
cargo build --release --example generate_positions -p bao-engine

echo "==> build PyO3 binding"
# If the system Python is newer than PyO3's max supported version (e.g. 3.14
# vs PyO3 0.22's 3.13), build against the stable ABI via forward-compat.
cd bindings/py
VIRTUAL_ENV="$(pwd)/.venv" PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 \
    .venv/bin/maturin develop --release
cd "$REPO_ROOT"

echo "==> done"
echo
echo "Next steps:"
echo "  - rsync shards and checkpoints from local (see docs/vps-setup.md if present)"
echo "  - install Claude Code: https://docs.claude.com/claude-code"
echo "  - run training/scripts/heuristic_vs_random.py to verify the engine"
