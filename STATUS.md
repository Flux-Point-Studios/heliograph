# STATUS

> Session ritual (HANDOFF §0): read this file → `make gate` on a clean tree →
> continue from "Next actions."

- **Phase:** 0 — RECONNAISSANCE & SPEC (no phase-0 exit gate passed)
- **Milestone:** repo initialized; operating contract committed
- **Last green commit:** (initial commit — `make gate` = scaffold checks only;
  the real gate grows fmt/clippy/tests/equality-invariant as crates land)

## Next actions

1. **Sources recon** (HANDOFF Phase 0): re-read Sextant SPEC + trust model
   (authoritative — re-derive nothing); Mithril certificate formats; SP1 and
   RISC Zero docs — precompile lists (BLS12-381 pairing coverage is decisive),
   licensing, Groth16-wrap pipelines, verifier contracts. File findings in
   `docs/notes/`.
2. **`CLAIMS.md` v0**: every claim type as (inputs, journal fields, what is
   proven, what is assumed, tier). Sextant's anchor-basis distinction
   (`StmCertified` / `AncillarySigned`) carries through verbatim.
3. **`THREAT_MODEL.md` v1**: the §Phase-0 adversary catalog (under-constrained
   journals; cross-network/cross-image replay; stale-but-valid selection;
   image-ID supply chain; trusted-setup provenance; router admin compromise;
   data withholding).

Then: the bakeoff spec (blocked on thresholds below) and the Sextant
upstream-needs note (`guest` feature: `no_std + alloc`, deterministic input
encoding, no `std::time`).

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
