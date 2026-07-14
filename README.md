# heliograph

**Portable ZK verdicts for Cardano** — succinct zero-knowledge proofs of
[Sextant](https://github.com/Flux-Point-Studios/sextant) verification runs,
checkable by any chain, contract, or auditor.

Sextant made Cardano verification embeddable: Mithril certificate chains,
Praos header segments, and tx/UTxO reads verified on an independent, fail-closed
path. Heliograph makes those verdicts **portable**: the same verification legs
run inside a zkVM guest and commit a structured **journal** — the claim — so a
consumer verifies a proof for cents instead of re-running the verification.

**North star:** any contract, chain, or auditor can verify a fact about Cardano
without trusting the prover. The prover is untrusted infrastructure; only the
pinned guest image ID and the Cardano trust anchors are load-bearing.

## What this is (and is not)

- A **state oracle layer**: proofs of Cardano facts, consumable on EVM first.
- **Not** a bridge: no custody, no wrapped assets, no token. Bridges are
  consumers of heliograph, not heliograph.
- **No stronger claims than Sextant**: a proof of a Tier-1 windowed verdict is
  still Tier-1, its assumptions carried verbatim into the journal. If Sextant
  can't verify it, heliograph can't prove it.
- **zkVM-only** (v0.1): guest programs in Rust; no custom circuits.

## Status

**Pre-alpha — Phase 0 (RECONNAISSANCE & SPEC).** Nothing here is usable yet.
See [`STATUS.md`](STATUS.md) for the live phase state and
[`HANDOFF.md`](HANDOFF.md) for the operating contract this project is built
under (phases, gates, verification doctrine, threat-model requirements).

## Layout (target)

```
/crates
  hg-claims        claim schema: types + canonical codec (no_std; shared)
  hg-guest         zkVM guest: sextant legs behind claim dispatch
  hg-host          prover CLI/daemon; fetch adapters; proof artifact I/O
  hg-sdk-ts        TypeScript verification + contract bindings
/contracts         Foundry: vendor verifier (vendored, unmodified) + router
/fixtures          proofs, journals, golden encodings; reuses Sextant vectors
/examples          evm-oracle/  portable-verdict/  checkpoint-update/
/docs              CLAIMS.md THREAT_MODEL.md BENCH.md adr/ notes/
```

## License

Apache-2.0.
