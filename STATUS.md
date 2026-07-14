# STATUS

> Session ritual (HANDOFF §0): read this file → `make gate` on a clean tree →
> continue from "Next actions."

- **Phase:** 1 — DESIGN (ADRs). **Phase-0 exit gate PASSED 2026-07-14**,
  tagged `phase-0`.
- **Milestone:** Phase 0 complete — recon (5 notes), `CLAIMS.md` v0,
  `THREAT_MODEL.md` v1 (14 adversaries, both audits harvested), `BENCH.md`
  (thresholds signed), upstream note (paste-ready). Exit gate: claims+sources
  ✓ / threat model reviewed ✓ / bakeoff approved (signed) ✓ / go-no-go
  explicit ✓.
- **Last green commit:** `make gate` green (scaffold checks; the real gate
  grows fmt/clippy/tests/equality-invariant as crates land)

## Done this phase

- **Recon** (`docs/notes/`): sextant-legs (the claims inventory + guest-safety:
  default graph ≈ no_std+alloc already; the ONE blocker is `mithril` →
  mithril-stm 0.10.5 → blst 0.3.16 C + rayon), sp1 (NO BLS12-381 pairing
  precompile; ~M cycles/pairing in software), risc0 (no pairing precompile
  either, BUT vendor ships an accelerated blst fork at **v0.3.16-risczero.0 —
  mithril-stm's exact pin**; plausibly a one-line `[patch.crates-io]`),
  mithril-recursion-watch (mainnet chain = 106 certs post Feb-2025 re-genesis;
  extend-by-one = 1 STM verify; upstream halo2_ivc is real and far along;
  **re-genesis recurs → genesis anchor is a versioned claim input, never an
  image constant**), lightclient-patterns (production LCs chain contract-side;
  the sp1-helios PR #54 under-constrained-journal incident; freshness is always
  consumer policy).
- **`docs/CLAIMS.md` v0** — claim types 0x0001–0x0005 (checkpoint,
  checkpoint-extension, header-segment, tx-inclusion, utxo-read) + deferred
  Tier-1/Tier-2 types; common journal header (claim_version, claim_type,
  network_id, genesis_vkey, verdict); journaled rejections; every field seeded
  for the mutant zoo. Byte layout deferred to ADR-003.
- **`docs/THREAT_MODEL.md` v1** — 14-adversary catalog (A1 under-constrained
  journals … A14 deployment-init), vendor-trust table, out-of-scope. Both
  audit PDFs read in full and harvested: Zellic sp1-helios (findings 3.1–3.5;
  the meta-finding — guest in scope, zero guest findings, missed the PR #54
  hole — folded into A1) and Veridise blobstream0 (V-BLOB-VUL-001..004). New:
  A14 (deploy-init, from the constructor key-domain bug), the load-bearing-
  checks ledger (A10), host-bindings-from-ABI + no-silent-canonicalization
  (A4), error-selector coverage (A8), mandatory minimum-progress + front-run
  race (A13). Pre-red-team (v2 lands after Phase 3).

All five Phase-0 deliverables are now DRAFTED (see also `docs/BENCH.md` — the
M0 bakeoff spec with PROVISIONAL thresholds + the append-only ledger skeleton,
including the mandatory mainnet-read-only fixture and tamper/determinism
controls — and `docs/notes/sextant-upstream-needs.md`, the paste-ready guest
feature ask: the RISC Zero path needs ZERO Sextant changes via the
v0.3.16-risczero.0 blst fork; the real asks are the no_std default graph and
the SP1-path pairing story).

## Next actions

**Phase 1 — DESIGN (ADRs).** Non-gated design work proceeds autonomously; the
two human touchpoints are flagged as such.

1. **ADR-000** — ecosystem/naming availability sweep (crates.io `hg-*` / npm
   `@heliograph` / GH org / ENS-if-relevant); pure research, unblocks crate
   naming for the workspace.
2. **ADR-002 / ADR-003 / ADR-004 / ADR-005** — recursion architecture,
   canonical journal codec, reproducible-build path, checkpoint-source trait.
   Draftable now from the recon; each cites its notes.
3. **ADR-001 (zkVM vendor selection)** — SKELETON only. The DECISION section is a
   **§9 human gate** and awaits M0 numbers; Phase 1 writes the criteria +
   bakeoff-wiring, not the choice.

### Human touchpoints (not blocking the above)

- **Upstream note filing is a §9 gate.** `docs/notes/sextant-upstream-needs.md`
  is paste-ready, but filing it on the Sextant repo is "public beyond the
  created repo" — needs a human go. Held.
- **`{{PROVER_BUDGET}}`** — required only before any PAID proving run
  (§9 gate 4). Local/CI runs proceed without it.

## HANDOFF §12 items — status after the 2026-07-14 human greenlight

| Item | Value | Status |
|---|---|---|
| `MAX_PROOF_TIME` | ≤ 10 min / checkpoint-update | **SIGNED** (BENCH.md §5 rev 1) |
| `MAX_PROOF_COST` | ≤ $1 / update | **SIGNED** |
| `BENCH_HARDWARE` | AWS `g6e.xlarge` (1× L40S) | **SIGNED** |
| `REGRESSION_PCT` | > 10 % CI flag | **SIGNED** |
| `EVM_TESTNET` | Base Sepolia | **SIGNED** (M4 target) |
| v0.1 claim scope | inclusion + UTxO-read (both; types 0x0001–0x0005) | **SIGNED** |
| License | Apache-2.0 | **CONFIRMED** |
| `PROJECT_NAME` / `GITHUB_ORG` | heliograph / Flux-Point-Studios | resolved 2026-07-14 |
| `{{PROVER_BUDGET}}` | — | **STILL OPEN** — required before any PAID proving
  run (§9 gate 4); local/CPU/owned-GPU runs are unblocked without it |
| Security disclosure contact | proposed: GitHub Private Vulnerability
  Reporting on this repo | **PROPOSED** — needed by Phase-3 SECURITY.md;
  adopt-or-amend before then |

ADR-000 still owes the crates.io / npm / ecosystem availability sweep
(Phase 1).

## Pending §9 gates

- None in flight. (Repo publication was §9-gate-1, satisfied by direct human
  instruction 2026-07-14.)
