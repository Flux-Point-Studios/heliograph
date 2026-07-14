# S-0001 — RISC Zero × Sextant mithril compile spike

**Question (BENCH.md §6.2, §8.6; ADR-001 "the crux"):** does Sextant's Mithril
STM leg — `mithril-stm` 0.10.5 → `blst` 0.3.16 C + `rayon` — compile for the
RISC Zero guest target `riscv32im-risc0-zkvm-elf`, and does the vendor's
accelerated blst fork (`v0.3.16-risczero.0`) slot in via a single
`[patch.crates-io]` line with zero Sextant changes?

**Answer: YES — all three bisect layers compile and link.** Compile-only spike;
no proving, no host run (BENCH.md branches 1–2 of §6.2 are both *buildable*;
runtime behavior is M0 proper).

## Outcome per bisect layer

Guest: `guest/` — `env::read()`s certificate JSON bytes, runs
`sextant::mithril::Certificate::from_json` + `verify_standard`, `env::commit()`s
a `u32` verdict code (codes documented in `guest/src/main.rs`). Sextant consumed
as the pinned git dependency `90b2672aa3ee7a957ecaee0e33026337825efafa`, never
vendored (HANDOFF §0).

| layer | configuration | result | user ELF bytes |
|---|---|---|---|
| 1 | `sextant` `default-features = false` only | **compiles** (52s cold) | 341,596 |
| 2 | + `features = ["mithril"]` — vanilla `blst` 0.3.16 `portable` C, cross-compiled by `riscv32-unknown-elf-gcc` 13.2.0 | **compiles + links** — the *unaccelerated control* (BENCH.md §6.2 branch 2) is buildable | 1,288,828 |
| 3 | + `[patch.crates-io] blst = risc0/blst tag v0.3.16-risczero.0` (via `--config risc0-blst-patch.toml`) | **compiles + links** — the *accelerated path* (branch 1); pulls `risc0-bigint2` 1.4.13; ELF shrinks vs layer 2 (C field arithmetic remapped onto bigint2 384-bit ops) | 1,199,904 |

`rayon` 1.12.0 / `rayon-core` 1.13.0 (mithril-stm's unconditional dep) and
`chrono` 0.4.45 compile **and link** for the target — the risc0 toolchain ships
`std` for the guest, so `rayon`'s thread machinery type-checks and links. No
rayon-shedding feature was needed at compile time. Whether mithril-stm's
*verify* path spawns threads at runtime (a single-threaded guest would trap) is
deliberately out of scope here and is the first M0-proper question.

## Layer-3 identity (the M0-relevant configuration)

Derived exactly as risc0-build v3.0.5 does (`imageid/`: user ELF + v1compat
kernel → `ProgramBinary` → `compute_image_id`):

| artifact | value |
|---|---|
| user ELF SHA-256 | `15735607ba22a2cfad3478ddf4bb954471e7e5143c63948a1ba55b8a9d573329` |
| combined (user+kernel) SHA-256 | `975382e8314f445d35bbd4c9a400b9b65ca4ed0e56cd5bc13e3517f36b9ee99f` |
| image ID | `5515edd69e5077b4044cd432a54d39833b52a2a270da2ab2b65181ba990e3080` |

Two independent fresh-container runs (`build.sh`, clean target dirs) produced
**byte-identical user ELF, combined binary, and image ID** — a same-host
reproducibility check (ADR-004 T-A6-1 proper needs divergent-OS/arch runners;
this is necessary-not-sufficient evidence).

**Canonicality caveat (ADR-004 D1):** builds ran *inside* the vendor's own
guest-builder container (docker-in-docker is unavailable), replicating
risc0-build v3.0.5's exact environment (`cargo_command_internal` +
`encode_rust_flags`: `CARGO_ENCODED_RUSTFLAGS`, `CC_riscv32im_risc0_zkvm_elf`,
`CFLAGS_riscv32im_risc0_zkvm_elf="-march=rv32im -nostdlib"`,
`RISC0_FEATURE_bigint2`). This is compile evidence, not a canonical registry
image ID — M0 proper re-derives identity via host-side `cargo risczero build`
(the docker pipeline) per ADR-004.

## Version pins (ADR-004 record)

| component | pin |
|---|---|
| build container | `risczero/risc0-guest-builder:r0.1.88.0@sha256:3e12f71bacd27527a61dea96fa0e53e468c99aa261d3a1019b593f6dbd943eb3` (Ubuntu 20.04.6; risc0-build v3.0.5 `DEFAULT_DOCKER_TAG`) |
| guest rustc / cargo | `rustc 1.88.0-dev (de85b1d3d 2025-06-26)` / `cargo 1.88.0-dev (873a06493)` (rustup toolchain `risc0`) |
| C cross-compiler | `riscv32-unknown-elf-gcc (gc891d8dc23e) 13.2.0` |
| `risc0-zkvm` (guest dep) | `=3.0.5`, `default-features = false, features = ["std"]` |
| `risc0-bigint2` | 1.4.13 (pulled by the blst fork) |
| `blst` (layer 3) | `git+https://github.com/risc0/blst?tag=v0.3.16-risczero.0#7d1fc3e6d8cf0d52a291f52455233bf8d6432ed2` |
| `mithril-stm` | 0.10.5 (`default-features = false, features = ["num-integer-backend"]`, via sextant) |
| `sextant` | `git rev 90b2672aa3ee7a957ecaee0e33026337825efafa`, `default-features = false` (+`mithril` in layers 2–3) |
| key transitives | rayon 1.12.0, rayon-core 1.13.0, chrono 0.4.45, serde_json 1.0.150, subtle 2.6.1, getrandom 0.2.17, num-bigint 0.4.8 |
| imageid helper | `risc0-binfmt =3.0.4`, `risc0-zkos-v1compat =2.2.2` |
| host | Windows 11 / Docker Desktop, server 29.1.3, linux/amd64 |

`guest/Cargo.lock` (committed) is the layer-3 resolution — the M0-relevant
configuration. Layers 1–2 re-resolve from the same manifest without the patch
config.

## The one obstacle hit, and its fix

Fresh resolution against the pinned toolchain trips **MSRV bitrot**, not any
risc0/blst/rayon incompatibility: latest `enum-ordinalize` 4.4.1 requires
rustc 1.89 and `ruint` 1.19.0 requires 1.90, vs the toolchain's 1.88.0-dev.
Fixed with `rust-version = "1.88.0"` + `resolver = "3"` in both crates
(MSRV-aware resolution) — no `--precise` pin chain, no version overrides.

## Reproduce

```
./build.sh   # from this directory; Docker required (linux/amd64)
```

Runs all three layers plus the image-ID derivation in a fresh pull of the
digest-pinned builder container.
