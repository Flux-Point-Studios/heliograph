#!/usr/bin/env bash
# The session gate (HANDOFF §7/§10) — the single source of truth `make gate` and CI both run.
# Phase 0: scaffold integrity. The gate GROWS with the code (fmt + clippy -D warnings + tests +
# equality invariant + journal zoo + golden journals land with the workspace) and never shrinks.
set -euo pipefail
cd "$(dirname "$0")/.."

test -f HANDOFF.md || { echo "gate: HANDOFF.md missing" >&2; exit 1; }
test -f STATUS.md || { echo "gate: STATUS.md missing" >&2; exit 1; }
test -f LICENSE || { echo "gate: LICENSE missing" >&2; exit 1; }

if [ -f Cargo.toml ]; then
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
else
  echo "gate: phase-0 scaffold OK (no workspace yet — gate grows with the code)"
fi
