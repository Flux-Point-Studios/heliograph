# S-0002 — SP1 guest compile bisect (Sextant `mithril` leg)

The SP1 arm of the M0 compile question (BENCH.md §6.1, ADR-001 context): does
Sextant's Mithril standard-certificate verification **compile** to an SP1 guest
ELF? The guest (`src/main.rs`) reads certificate JSON via `sp1_zkvm::io`, runs
`sextant::mithril::Certificate::from_json` → content-hash recompute →
`verify_standard`, and commits a `u32` verdict. `cargo prove build` producing
the ELF is the win condition; nothing is proven or executed here.

## Reproduction

`./build.sh` from Git Bash on the Windows host (Docker Desktop running). It
reuses the named container `hg-sp1-spike` (`rust:1.93-bookworm`) with named
volumes for the cargo registry and `/root/.sp1`, installs the SP1 toolchain via
`sp1up` and Debian's `gcc-riscv64-unknown-elf`, then runs the three bisect
layers (purging blst's build cache between mithril layers so the compiler
choice is exercised, not replayed). Logs land in `logs/` (untracked); curated
evidence is committed at `fixtures/bench/evidence/S-0002/`.

## Outcome (2026-07-14) — mithril guest ELF COMPILES

| layer | configuration | result |
|---|---|---|
| 1 | sextant `default-features = false` | **compiles** (30 s cold); ELF sha256 `0db3a6e3…7435977` |
| 2 | + `mithril`, stock toolchain | **fails in `blst 0.3.16`'s build script**: cc-rs finds no compiler for `riscv64im-succinct-zkvm-elf`, falls back to host x86-64 `cc` with `-march=rv64im -mabi=lp64 … -D__BLST_NO_ASM__ -D__BLST_PORTABLE__` → `cc: error: unrecognized argument in option '-mabi=lp64'`. The **only** failing crate — rayon-core, crossbeam, subtle, getrandom 0.2.17, num-bigint, ff, chrono all compile for the SP1 std target |
| 3 | + `mithril`, `CC_riscv64im_succinct_zkvm_elf=riscv64-unknown-elf-gcc` (Debian gcc 12.2.0) | **compiles and links**: ELF sha256 `6aae24ad…6a0a671` (1.4 MB, 39 `blst_*` text symbols); vkey `0x004c3360d58ac7e46b33b44095f81524c3aada3eb4433235dc8b693cd98f5f7d`; rebuild after cache purge → identical sha256 |

The recon's expected hard wall (sp1.md §2: blst "cannot ride SP1's patch
mechanism", pure-Rust backend seam required) is narrower than predicted: the
wall is *only* the missing C cross-compiler, and a stock Debian bare-metal
RISC-V gcc clears it. mithril-stm's rayon does not block compilation.

## What compiling does NOT establish (M0-proper work)

- **Runs/proves.** Untested here: rayon will try to spawn std threads and
  `blst` C runs as plain RV64IM guest code (zero SP1 syscall acceleration —
  the software-pairing cycle count is exactly what M0 measures). First
  execution may still fail at runtime; that is a run question, not a compile
  question.
- **Canonical identity.** This is a container build, not
  `cargo prove build --docker` (ADR-004 D1 non-canonical). The pinned SP1
  build image must be checked for — and, if absent, extended with — a RISC-V C
  cross-compiler before a citable vkey exists; the cross-gcc version then
  becomes a `build-recipe.toml` pin (it shapes the ELF bytes).
- **Semantics of gcc-compiled C under the prover.** The `-march=rv64im
  -mabi=lp64` flags match the target, but vendor-blessed support for foreign
  C objects in guests should be confirmed upstream before this path carries a
  threshold-bearing row (vs. the sp1-patches/bls12_381 pure-Rust seam, which
  remains the vendor-documented primary).
