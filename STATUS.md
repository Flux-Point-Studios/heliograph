# STATUS

> Session ritual (HANDOFF §0): read this file → `make gate` on a clean tree →
> continue from "Next actions."

- **Phase:** 0 — RECONNAISSANCE & SPEC (exit gate NOT yet passed)
- **Milestone:** recon filed (5 notes, `docs/notes/`); `CLAIMS.md` v0 +
  `THREAT_MODEL.md` v1 drafted
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
- **`docs/THREAT_MODEL.md` v1** — 12-adversary catalog (A1 under-constrained
  journals … A13 griefing), vendor-trust table, out-of-scope. Pre-red-team
  (v2 lands after Phase 3).

## Next actions

1. **Bakeoff spec** (`docs/BENCH-SPEC.md` or BENCH.md preamble): identical
   guest workload = one real preprod STM certificate verify via Sextant's
   `mithril` path; measure cycles/wall/cost/wrap/gas on both zkVMs. Draft with
   PROVISIONAL thresholds (marked) — approval needs the §12 numbers below.
   MUST bench a mainnet-read-only fixture too (preprod k=5 vs mainnet k=1944).
2. **Sextant upstream-needs note**: the `guest` feature (no_std+alloc
   entrypoints, deterministic input encoding, no `std::time`) + the blst
   question (RISC Zero's fork matches the pin; SP1 path needs a pure-Rust
   pairing story or an mithril-stm backend seam).
3. **Phase-0 exit review**: claims + threat + bakeoff spec against the HANDOFF
   exit gate; read the two unread audit PDFs (Zellic/sp1-helios,
   Veridise/blobstream0) before freezing THREAT_MODEL v1.

## Blockers — HANDOFF §12 open items needing human input

| Item | Suggestion on the table | Needed for |
|---|---|---|
| `{{MAX_PROOF_TIME}}` | ≤ 10 min / checkpoint-update on bench HW | bakeoff spec + M0 go/no-go |
| `{{MAX_PROOF_COST}}` | ≤ $1 / update | bakeoff spec + M0 go/no-go |
| `{{BENCH_HARDWARE}}` | one pinned GPU cloud SKU | bakeoff spec |
| `{{EVM_TESTNET}}` | Base Sepolia | M4 deployment target |
| `{{PROVER_BUDGET}}` | — | Phase 2–3 proving spend cap (§9 gate 4) |
| v0.1 claim scope | inclusion + UTxO-read (both) | CLAIMS.md v1 freeze |
| Security disclosure contact | — | SECURITY.md |
| License | Apache-2.0 (default; committed) | confirm or amend |

**Recommendation:** accept the suggested defaults for the bakeoff thresholds
and Base Sepolia; the only items with no sensible default are
`{{PROVER_BUDGET}}` and the disclosure contact. Phase-0 recon and CLAIMS/THREAT
drafting are NOT blocked on any of these — only the bakeoff spec's exit gate is.

Resolved: `{{PROJECT_NAME}}` = **heliograph**, `{{GITHUB_ORG}}` =
**Flux-Point-Studios** (repo created by direct instruction, 2026-07-14).
ADR-000 still owes the crates.io / npm / ecosystem availability sweep.

## Pending §9 gates

- None in flight. (Repo publication was §9-gate-1, satisfied by direct human
  instruction 2026-07-14.)
