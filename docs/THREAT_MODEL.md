# heliograph — THREAT_MODEL

**Version: v1 — DRAFT, pre-red-team (Phase 0).**

| Version | Date | Status |
|---|---|---|
| v1 | 2026-07-14 | DRAFT. Written at Phase 0 (SPEC), before any code exists. Every mitigation names the permanent test that will pin it (HANDOFF §7); **every test in this document is *planned*, not passing** — v1 specifies the defense, it is not evidence of it. |
| v2 | after Phase 3 | Required for v0.1.0 (HANDOFF §3). Lands post-red-team with measured residual risks, the unbound-input zoo green, and every scenario below converted to a permanent CI test. |

Doctrine this document operates under (HANDOFF §0, §7, §11):

- **Fail-closed.** The verifier rejects unknown guest image IDs, unknown claim
  versions, and wrong network IDs. A proof that cannot be fully bound is a
  rejected proof.
- **The journal is the ABI of truth.** Any input that can change a verdict
  without changing the journal is a soundness bug of the highest severity.
- **A proven verdict keeps its tier.** Proofs add proven computation, never
  stronger claims; Sextant's assumptions ride verbatim in the journal.
- **Every mitigation claim is a permanent test.** A defense that exists only
  as prose in this file does not exist.

---

## 1. Scope and trust model

Heliograph produces succinct ZK proofs of Sextant verification runs and
verifies them on-chain (EVM router over the vendor's audited verifier) and
off-chain (self-contained proof artifacts). This threat model covers the
guest programs, the journal/claim schema, the prover host, the proof
artifacts, the verifier contracts, and the governance of their trust anchors.
Cardano protocol semantics are **out of scope here**: they are Sextant's, and
Sextant's SPEC + trust model is authoritative upstream (HANDOFF §0
precedence 3). Where Sextant's assumptions surface in heliograph journals,
they are cataloged as inherited (A12), not re-derived.

### 1.1 Load-bearing (trusted; an error here breaks soundness)

| Element | What it binds | Where pinned | Rotation event |
|---|---|---|---|
| **Pinned guest image ID** (SP1 program vkey `bytes32` / RISC Zero image ID — docs/notes/sp1.md §1, docs/notes/risc0.md §1) | The verification logic itself. The image ID *is* the trust anchor (ADR-004). | Allowed-image-ID registry in the router; consumer contracts; the portable-verdict CLI allowlist | Any guest change or zkVM vendor/version bump — a governed trust-anchor rotation (HANDOFF §6, §9 gate 2) |
| **Cardano trust anchors, carried as journal fields** | Mithril genesis Ed25519 vkey (per network); ancillary Ed25519 vkey (Tier-2 `AncillarySigned` basis only); anchor basis byte (`StmCertified`/`AncillarySigned`, uncoercible — Sextant `utxo.rs:121-131` via docs/notes/sextant-legs.md) | Journal fields, checked by the router/consumer against registry-pinned expected values | Re-genesis (A11); ancillary→STM basis discharge upstream |
| **Vendor proof system + wrap ceremony** | Soundness of the proof object itself | Exact-version + hash pins (HANDOFF §6); the §3 vendor-trust table below | Vendor upgrade (§9 gate 2) |
| **Verifier contract + router + its admin** | On-chain accept/reject | Vendor verifier vendored byte-identical from the audited release, never modified (HANDOFF §6); thin claim-router; allowed-image-ID registry behind a timelocked admin (§9 gate 3) | Registry/timelock operations (A8) |
| **`hg-claims` canonical codec** | Interpretation of every journal | Golden-tested canonical encoding, mirrored constant-for-constant in the Solidity router (HANDOFF §5, ADR-003) | Claim-schema change after v1 freeze (§9 gate 6) |

### 1.2 Untrusted (may lie, equivocate, or withhold at will; must never cause a false accept)

- **The prover** — the entire prover host (`hg-host`), any operator of it, and
  any prover network/marketplace it delegates to. Note: prover-network input
  privacy is undocumented for SP1 (docs/notes/sp1.md open item 6) and
  Boundless pricing is market-set (docs/notes/risc0.md §3); assume provers
  see all inputs. Irrelevant to soundness — the prover is untrusted by
  design — but disqualifying for any confidential use.
- **All fetched bytes** — aggregator certificate JSON, block CBOR, inclusion
  proofs, ancillary snapshots. Sextant's constitution carries over verbatim:
  a provider supplies *bytes, never a verdict*; hostile input can only make a
  genuine proof fail, never make an invalid one succeed (Sextant
  README.md:17-18 via docs/notes/sextant-legs.md).
- **The Mithril aggregator** — a data source whose *signatures* are verified
  and whose *service* is not trusted; it can withhold, reorder, or serve
  stale certificates (A5, A9) and can reseal unsigned envelope fields
  (Sextant reads signed protocol-message parts only, `mithril.rs:129-137`).
- **Anyone who submits proofs to the router** — updates are permissionless;
  soundness rests on the proof, liveness on anyone (A9, A13).

### 1.3 What no proof asserts (consumer obligations)

A heliograph proof is a promise made to strangers; these are the promises it
does **not** make. Consumers building on verified facts must handle each:

1. **Freshness.** No proof asserts recency — only journaled as-of slots.
   Freshness is consumer policy (A5).
2. **Canonicality of header segments** beyond the Mithril anchor — a surfaced
   Sextant assumption, inherited verbatim (A12).
3. **Unspent-ness beyond the journaled tier.** The tier ladder
   (`NotEstablished` / `WatchedWindow` / `CertifiedUnspent`) and assumption
   bits ride in the journal unchanged; a proof never upgrades a tier
   (HANDOFF §0).
4. **Adequacy of Mithril's parameters** (`k/m/phi_f`) — an external
   assumption pinned by the certificate hash but not validated by anyone's
   cryptography (A12).
5. **Prover liveness.** Proof generation can stall indefinitely (A9).

---

## 2. Adversary catalog

Format per entry: the attack → what it would achieve → current mitigations
(with the design rule or recon note that establishes them) → residual risk →
the permanent test(s) that pin the mitigation (HANDOFF §7). Test IDs are
stable names for suites to be created; locations follow the HANDOFF §5
layout (`/fixtures`, `contracts/test`, CI jobs, the unbound-input zoo with
coverage tracked in `docs/journal-coverage.md`).

Catalog letters (a)–(l) match the HANDOFF Phase-0 minimum; A13 is added from
the light-client incident survey.

### A1 (a) — Under-constrained journals

- **Attack.** A guest input byte-range reaches the journal (or influences the
  verdict) without passing through verification. The prover supplies
  arbitrary bytes for that range and obtains a *valid proof of a false
  claim*.
- **Real-world instance.** sp1-helios PR #54: the guest committed the
  prover-supplied `store.next_sync_committee` to `nextSyncCommitteeHash`
  **without verification when the `updates` list was empty**; a malicious
  prover could inject an arbitrary committee, which the contract would store
  — handing over the light client at the next period. It survived in a
  Zellic-audited flagship, and the hole lived on a *degenerate path* (empty
  update list). Fix was PR #55 + an executor-based regression test
  (docs/notes/lightclient-patterns.md §4.1).
- **What it achieves.** Total soundness failure: any Cardano "fact" the
  attacker wants, carried by a proof every honest verifier accepts.
- **Current mitigations.**
  - The journal-is-the-ABI non-negotiable (HANDOFF §0): anything
    verdict-relevant must be bound in the journal or committed via the image
    ID.
  - ADR-003: every journal field's necessity is proven by a mutant.
  - Sextant discipline inherited: every expected rejection is a *journaled
    verdict*, not a panic (HANDOFF §5) — so rejection paths are themselves
    covered by journal tests, unlike every surveyed vendor example
    (docs/notes/lightclient-patterns.md §4.6).
  - SP1-specific sub-class: **hint/unconstrained inputs**
    (`ENTER_UNCONSTRAINED`, `HINT_READ`) are not constrained by the proof;
    patched crates use them internally and the vendor puts the safety burden
    on the developer (docs/notes/sp1.md §1 "Determinism"). Hints are treated
    as a first-class unbound-input class.
- **Residual risk.** The zoo can only kill inputs it enumerates. Degenerate
  paths (empty lists, zero-length windows, absent optional fields) are where
  the real-world instance lived; vendor-crate internals (hints) are only
  auditable to the depth of the patched-crate diff.
- **Pinning tests.**
  - **T-A1-1 — the unbound-input zoo** (`make gate`-blocking, permanent): for
    *every* guest input byte-range and *every* journal field, a mutant
    demonstrating (a) mutation changes the journal/verdict, or (b) a filed,
    reviewed argument for verdict-irrelevance. Coverage ledger:
    `docs/journal-coverage.md`; gaps fail the gate (HANDOFF §7). Any input
    that silently changes a verdict = P0 stop-the-line (HANDOFF Phase 3).
  - **T-A1-2 — degenerate-path mutants** as an explicit zoo class: empty
    certificate list, zero-certificate extension, empty block window, absent
    optional fields — the sp1-helios #54 shape, per claim type.
  - **T-A1-3 — patched-vs-original-crate equality** (SP1 path): the fixture
    corpus run against guests built with and without vendor patched crates
    must produce identical journals (the vendor's own stated requirement,
    docs/notes/sp1.md §1).

### A2 (b) — Cross-network replay

- **Attack.** A valid preprod proof is presented to a mainnet-configured
  verifier (or vice versa), or a portable-verdict file is presented to an
  off-chain verifier with no chain context.
- **What it achieves.** Facts from a worthless test network accepted as
  mainnet facts.
- **Current mitigations.** Explicit `network_id` as a journal field in every
  claim, checked by the router against its configured network — *in
  addition to* continuity binding (prev-checkpoint linkage). The surveyed
  production LCs rely on implicit continuity only; Steel's
  `Commitment.configID` is the precedent for the explicit field, and the
  explicit field is what makes the portable-verdict file self-describing
  off-chain where no contract supplies context
  (docs/notes/lightclient-patterns.md §2c, §6.2; ADR-003; HANDOFF §0
  fail-closed: wrong network IDs are rejected).
- **Residual risk.** Low by construction once tested. The off-chain CLI
  verifier must enforce the same check as the router — a
  cross-layer-agreement obligation, not an extra design.
- **Pinning tests.**
  - **T-A2-1** — Foundry fork test: a valid preprod-journal proof submitted
    to a mainnet-configured router reverts with the specific
    wrong-network error (HANDOFF Phase 3 scenario, made permanent).
  - **T-A2-2** — golden journals: every claim type's committed golden
    encoding contains `network_id`; zoo mutant flipping it changes the
    journal (subsumed by T-A1-1, tracked separately for legibility).
  - **T-A2-3** — cross-layer agreement: SDK and CLI verifiers reject the same
    wrong-network artifact the router rejects (HANDOFF §7).

### A3 (c) — Cross-image replay

- **Attack.** A proof generated by a *different* guest program (an old
  heliograph version, a look-alike, or an attacker's own guest emitting a
  well-formed journal) is presented to the verifier.
- **What it achieves.** A journal that was never produced by the pinned
  verification logic gets accepted — arbitrary facts under an honest-looking
  ABI.
- **Current mitigations.** Image-ID binding in the on-chain verify itself:
  both vendors take the program identity as a verify-call parameter
  (`verifyProof(programVKey, …)` / `verify(seal, imageId, journalDigest)`,
  with RISC Zero binding the image ID *into the reconstructed claim digest*,
  not merely comparing — docs/notes/sp1.md §4, docs/notes/risc0.md §4). The
  router supplies the image ID from its timelocked allowed-image-ID registry
  (HANDOFF Phase 1); unknown image IDs are rejected (fail-closed, HANDOFF
  §0). For recursion, the inner image ID is an input and therefore MUST be
  journaled — an unjournaled inner ID is an unbound input
  (docs/notes/lightclient-patterns.md §3; docs/notes/sp1.md §6;
  docs/notes/risc0.md §6).
- **Residual risk.** The registry is now the attack surface (see A8). The
  self-recursion inner-ID pattern is vendor-undocumented on the RISC Zero
  side (docs/notes/risc0.md §7.4) — M1 spike before ADR-002 freeze.
- **Pinning tests.**
  - **T-A3-1** — fork test: a valid proof from a foreign guest image carrying
    a byte-identical journal is rejected by the router (registry lookup
    fails / verifier fails).
  - **T-A3-2** — recursion chain mutant: an extension proof whose journaled
    inner image ID differs from the registry-pinned ID is rejected by router,
    SDK, and CLI; and the zoo proves the journaled inner-ID field is
    mutation-sensitive.

### A4 (d) — Claim-version downgrade

- **Attack.** A proof carrying an old or unknown claim-schema version (a
  pre-freeze v0 encoding, or a future version the deployed router does not
  understand) is presented, hoping the decoder misparses it into a
  well-formed current-version claim.
- **What it achieves.** Field confusion: bytes canonical under one version
  reinterpreted under another — the classic ABI-drift exploit.
- **Current mitigations.** Versioned canonical encoding in `hg-claims`,
  mirrored constant-for-constant in the router; **unknown versions rejected
  everywhere** — router, SDK, host, guest (ADR-003; HANDOFF §0 fail-closed).
  Steel is the precedent: 16-bit version decoded from the commitment ID,
  revert on unknown (docs/notes/lightclient-patterns.md §2c). Version and
  claim type lead the journal (sp1-vector's claim-type-first discipline,
  docs/notes/lightclient-patterns.md §3).
- **Residual risk.** Version checks are only as good as the codec's
  canonicality — one encoding per value, golden-tested (HANDOFF §5). Schema
  changes after v1 freeze are a §9-gate-6 human review, which is process,
  not cryptography.
- **Pinning tests.**
  - **T-A4-1** — unknown-version rejection, all layers: fuzzed version bytes
    against the router (Foundry fuzz), the `hg-claims` decoder (cargo fuzz),
    and the TS SDK; all reject anything ≠ the frozen version set.
  - **T-A4-2** — downgrade fixture: a committed pre-freeze (v0) encoding of
    an otherwise-valid claim is rejected by every layer, permanently.
  - **T-A4-3** — golden-encoding canonicality: byte-exact golden journals per
    claim type; any second encoding of the same value is a test failure
    (HANDOFF §7 "Golden journals").

### A5 (e) — Stale-but-valid data selection by a malicious prover

- **Attack.** The prover proves over *genuinely valid but old* data: an old
  certified checkpoint, a UTxO read as of a long-past block, a watched
  window that ended hours ago. Every byte is authentic; the proof verifies.
- **What it achieves.** A consumer that conflates "verified" with "current"
  acts on stale state — e.g. treats a since-spent UTxO as live.
- **Current mitigations.**
  - **As-of slot binding in every journal**: `as_of_height`/`as_of_slot`,
    `certified_at`, `through_block` ride from Sextant's verdict structs into
    the journal unchanged (docs/notes/sextant-legs.md Legs 4–6; CLAIMS.md
    anchor-basis + as-of fields, HANDOFF §3).
  - **Documented CONSUMER obligation: freshness is policy, not proof.** The
    survey finding is unambiguous: *no production light client proves
    freshness* — guests are clockless and evaluate any "trusting period"
    against timestamps the prover chose to include; contracts enforce
    ordering, not recency; recency is always consumer-side policy over
    journaled as-of data (docs/notes/lightclient-patterns.md §2d and §4.5
    "clockless-guest freshness theater"). Heliograph does not pretend
    otherwise (docs/notes/lightclient-patterns.md §6.6): the guest has no
    clock (Sextant is sans-io; `freshness` is caller data,
    `window.rs:211-219` via docs/notes/sextant-legs.md), so heliograph
    *cannot* prove freshness and the docs must say so in exactly those words.
  - Router monotonicity (checkpoint extensions only) prevents *regression*,
    which is A10's half of this problem.
- **Residual risk.** Permanent and structural: the consumer who ignores the
  as-of fields is unprotected, by design. This is the single largest
  documentation obligation in the product ("What a proof proves, precisely",
  HANDOFF Phase 4).
- **Pinning tests.**
  - **T-A5-1** — golden journals assert as-of fields present and
    mutation-sensitive in every claim type (zoo class).
  - **T-A5-2** — the layering demonstrated, permanently: fork test in which
    an old-but-valid proof is **accepted by the router** and **rejected by
    the example consumer's max-age policy** (`examples/evm-oracle` carries a
    recency check precisely so integrators copy the right shape). The test
    proves both halves — router does not enforce recency, consumer must.
  - **T-A5-3** — stale-data honesty scenario from HANDOFF Phase 3 kept as a
    permanent red-team test: prover deliberately selects the oldest
    acceptable inputs; assert the journal exposes the staleness verbatim.

### A6 (f) — Image-ID supply chain / non-reproducible builds

- **Attack.** The published image ID does not correspond to the published
  guest source: a compromised toolchain, a poisoned build environment, or an
  unreproducible build lets a backdoored guest hide behind a
  legitimate-looking ID. Verifiers pin the ID; nobody can independently
  recompute it.
- **What it achieves.** The trust anchor itself is forged — every downstream
  binding (A3) binds to attacker logic.
- **Current mitigations.**
  - **Docker-pinned toolchains per vendor.** SP1: `cargo prove build
    --docker --tag vX.Y.Z` (image `ghcr.io/succinctlabs/sp1`) is the vendor's
    documented reproducible path; non-docker builds are explicitly warned
    non-reproducible (docs/notes/sp1.md §1). RISC Zero: the docker toolchain
    is the *only* canonical path — vendor code comments state non-docker
    builds are not identical across architectures (docs/notes/risc0.md §1).
    Non-container builds are treated as non-canonical, full stop.
  - **Two-independent-builds requirement** (HANDOFF §3, ship-blocking):
    two independent builds → identical image ID, with a documented,
    publicly recomputable procedure; release artifacts carry the build
    recipe (ADR-004).
  - Guest changes must note their image-ID impact in the commit message
    (HANDOFF §10); image-ID rotation is a governed event (§9 gate 2).
- **Residual risk.** Reproducibility pins source→ID; it does not audit the
  vendor's toolchain container itself (that trust is documented in §3
  below). A compromise of the vendor's published Docker image is inside the
  vendor-trust surface, not fixable here.
- **Pinning tests.**
  - **T-A6-1** — CI job `reproducible-image-id`: two builds on independent
    runners (different host OS/arch) via the pinned container produce
    byte-identical ELF (SHA-512) and identical image ID/vkey; CI-blocking
    from M1 onward.
  - **T-A6-2** — Phase-3 clean-room rebuild audit: a from-scratch machine,
    following only the published recipe, reproduces the shipped image ID
    (HANDOFF Phase 3 "reproducibility audit"); repeated at every release.

### A7 (g) — Trusted-setup provenance of the wrap circuit

- **Attack.** The Groth16/PLONK wrap circuit's trusted setup was compromised
  (toxic waste retained): whoever holds it can forge wrap-layer proofs that
  the on-chain verifier accepts, without ever touching the guest.
- **What it achieves.** On-chain soundness failure below heliograph's entire
  stack — indistinguishable from a valid proof at the contract.
- **Current mitigations — document, not fix (HANDOFF Phase 3).** This is
  vendor trust; heliograph's job is to state it precisely and pin the
  artifacts:
  - **SP1 (Groth16): circuit-specific ceremony** run by Succinct — 18 named
    participants (incl. Etherealize, Polygon, OP Labs, Offchain Labs,
    Coinbase representatives) on Semaphore's ceremony tooling; the vendor's
    own docs say users uncomfortable with this should use PLONK, whose SRS
    is the **Aztec Ignition universal ceremony** (still an SRS assumption,
    not "no trusted setup" — docs/notes/sp1.md §3). Wrap artifacts are
    versioned (`SP1_CIRCUIT_VERSION`) and downloaded as binaries.
  - **RISC Zero (Groth16): PSE-coordinated, re-verifiable ceremony** — phase
    1 is the Hermez `powersOfTau28_hez_final_23.ptau` (2^23); phase 2 run on
    PSE's p0tion/DefinitelySetup with per-contributor GitHub Gist
    attestations; `stark_verify.r1cs` SHA-256 pinned
    (`84d3c34b7c0eb55ad1b16b24f75e0b9de307f7b74089ea4a20a998390ee24178`);
    a full third-party re-verification procedure (`snarkjs zkey verify`) is
    documented (docs/notes/risc0.md §3). This is the stronger provenance
    story of the two: independently re-executable, not just attested.
  - 1-of-N honesty is the assumption in both ceremonies; security reduces to
    at least one honest participant having destroyed their contribution.
- **Residual risk.** Irreducible without re-running a ceremony (out of
  scope: no custom circuits, §9 gate 5). If either vendor's ceremony were
  compromised, heliograph's on-chain layer is unsound while local
  STARK-level verification (pre-wrap) remains sound — worth stating in the
  integrator docs as the difference between on-chain and native
  verification paths.
- **Pinning tests.**
  - **T-A7-1** — CI pin check: wrap-circuit artifact digests
    (`SP1_CIRCUIT_VERSION` artifacts; RISC Zero `stark_verify.r1cs` hash and
    zkey identifiers) asserted against the values recorded in this document
    and in the dependency lockfiles; any drift fails the gate (HANDOFF §6:
    exact version + hash pins).
  - **T-A7-2** — (RISC Zero path, once, then per vendor bump) execute the
    vendor's documented `snarkjs zkey verify` re-verification and file the
    transcript in `docs/notes/`; a §9-gate-2 checklist item for every
    vendor upgrade.

### A8 (h) — Verifier-contract admin compromise

- **Attack.** Whoever controls the router's admin key (or the vendor
  gateway's owner key) rotates in a malicious image ID, points the router at
  a fake verifier, or rewrites stored state.
- **What it achieves.** Everything A3 prevents, restored by governance: the
  registry legitimizes attacker logic; consumers keep reading "verified"
  facts.
- **Current mitigations.** Heliograph deliberately picks the **narrow end of
  the admin ladder** surveyed in
  docs/notes/lightclient-patterns.md §2b/§4.4:
  - The anti-pattern to name: Blobstream0 — UUPS-upgradeable, owner can
    `adminSetImageId`, `adminSetVerifier`, and `adminSetTrustedState` (*can
    rewrite the trusted head outright*): full takeover concentrated in one
    key. The other end: SP1Tendermint (no admin at all) and SP1Helios
    (guardian limited to vkey rotation, renounceable to the zero address).
  - Heliograph's design (HANDOFF Phase 1 + docs/notes/lightclient-patterns.md
    §6.3): vendor's audited verifier **untouched and unowned by us**; a thin
    router whose *only* admin surface is the allowed-image-ID / trust-anchor
    registry, behind a **timelocked admin**; **no admin action may affect an
    already-proven checkpoint** — no `adminSetTrustedState` equivalent
    exists in the ABI, by construction. Key custody and timelock parameters
    are a §9-gate-3 human decision.
  - Prior art for the governance shape: RISC Zero's own router is owned by a
    `TimelockController` with a multisig proposer, tombstones removed
    selectors so they can never be reused, and pairs verifiers with an
    emergency-stop that includes a *permissionless* circuit breaker (anyone
    presenting a proof-of-unsoundness halts the verifier, permanently)
    (docs/notes/risc0.md §4). Adopting the emergency-stop pattern for the
    heliograph router is an open design question for ADR review.
- **Residual risk.** A timelock converts key compromise from instant
  takeover into a *detectable, delayed* takeover; it does not remove it.
  Consumers get the timelock window to exit. The SP1 gateway's own
  ownership is undocumented on the fetched pages (docs/notes/sp1.md open
  item 5) — must be resolved before M4; until then the SP1 on-chain path
  carries an unquantified admin surface upstream of ours.
- **Pinning tests.**
  - **T-A8-1** — ABI-surface test (permanent): the router exposes **no**
    function that mutates a stored checkpoint or verified fact;
    enumerated-selector test fails if any state-rewriting admin function is
    ever added.
  - **T-A8-2** — timelock fork tests: registry add/remove of an image ID or
    anchor takes effect only after the configured delay; the pending change
    emits an event at proposal time (consumer early-warning); an
    admin-abuse-under-timelock scenario from HANDOFF Phase 3 kept permanent.
  - **T-A8-3** — fuzz the registry state machine (add/remove/propose/cancel
    orderings) for paths that bypass the delay.

### A9 (i) — Data withholding / liveness

- **Attack.** The prover (or every data source it depends on: aggregator,
  RPC providers) stalls — no new proofs are produced. Distinct flavor:
  sources serve enough data to verify but withhold the specific blocks or
  certificates needed to *advance*.
- **What it achieves.** Nothing against soundness — **the prover cannot
  forge, it can only stall** (Sextant constitution: hostile input can only
  make a genuine proof fail; heliograph inherits it whole). The damage is
  staleness: consumers act on old-but-valid facts (A5's territory) or halt.
- **Current mitigations.**
  - Proofs are permissionlessly verifiable and the router's update path is
    permissionless — anyone can run a prover and submit; **N independent
    provers are possible by design, one is operated** in v0.1 (HANDOFF §2).
    Liveness rests on anyone, soundness on no one — the surveyed pattern
    (docs/notes/lightclient-patterns.md §1.4).
  - Sextant's verdict shapes make withholding legible rather than dangerous:
    a withheld block collapses to a `Stalled` verdict, never a false
    `Unspent` (`window.rs:226-262` via docs/notes/sextant-legs.md Leg 5).
  - The portable-verdict artifact is self-contained: already-issued proofs
    remain verifiable forever with no prover, no chain, no vendor service
    (M3 gate, HANDOFF Phase 2).
- **Residual risk.** v0.1's single operated prover is a real liveness SPOF
  and is documented as such — no realtime promises, proof latency is
  minutes (HANDOFF §2). Consumer recency policy (A5) is what converts
  "stale" from a hazard into a rejected input. Ops cadence (how stale the
  public checkpoint may get) is an operational SLO, not a proof property —
  the surveyed LCs treat it the same way
  (docs/notes/lightclient-patterns.md §2d "liveness cadence").
- **Pinning tests.**
  - **T-A9-1** — offline verification: `portable-verdict` demo verifies a
    committed proof artifact end-to-end with network access disabled
    (M3 gate, kept as a permanent CI test).
  - **T-A9-2** — permissionless-update fork test: an arbitrary unprivileged
    address submits a valid proof and the router accepts (no caller gate
    regression).
  - **T-A9-3** — withholding fixtures: truncated/gapped input sets produce
    journaled `Stalled`-class verdicts (never `Unspent`), in-guest —
    rejection-parity corpus (equality invariant covers this; tracked
    separately for the liveness story).

### A10 (j) — Malicious-prover input SELECTION games

- **Attack.** All inputs are authentic and every proof is valid — but the
  prover *chooses* which inputs to prove: an old checkpoint instead of the
  tip (regression), a valid-header minority fork that was never certified,
  the oldest window that still satisfies the letter of the claim.
- **What it achieves.** Consumers who assume "the proven state = the
  canonical current state" are steered onto authentic-but-unrepresentative
  facts. This is A5's generalization from time to *branch* selection.
- **Current mitigations.** **Anchor binding makes selection
  journal-visible.**
  - Every claim journals its anchor basis and anchor identity: the
    genesis-rooted certificate chain (root hash, tip hash), the certified
    root, `eta0`, `certified_at`/`anchor_height` (CLAIMS.md fields per
    HANDOFF §3; Sextant journal-field inventories,
    docs/notes/sextant-legs.md §1). A minority fork pre-anchor produces a
    journal whose anchor fields do not match the certified chain any
    consumer pins — visible, checkable, rejectable.
  - Header segments are *not* proven canonical, and the journal says so:
    canonicality rests on the Mithril anchor, a surfaced assumption
    (Sextant README.md:143 via docs/notes/sextant-legs.md Leg 1). Heliograph
    proves exactly what Sextant verifies, no more (HANDOFF §2).
  - On-chain, the router accepts extensions only: `journal.prev_checkpoint
    == stored checkpoint` plus monotonic epoch/slot — old-checkpoint
    regression is structurally rejected, the same rule every surveyed
    production LC enforces (docs/notes/lightclient-patterns.md §1.4, §2b).
  - Sextant's own anti-selection anchors ride through: `require_through`
    closes the truncate-before-the-spend evasion (`window.rs:27-31,456-458`
    via docs/notes/sextant-legs.md Leg 5).
- **Residual risk.** Off-chain consumers of portable verdicts have no stored
  checkpoint; for them the journal's anchor fields are the *only* defense,
  and the consumer must actually check them against a trusted anchor of
  their own choosing. Documentation obligation, same class as A5's.
- **Pinning tests.**
  - **T-A10-1** — old-checkpoint fork test: a valid proof extending
    checkpoint N−k is rejected by a router whose stored checkpoint is N.
  - **T-A10-2** — minority-fork fixture: a valid-header, non-certified fork
    segment (Sextant mutant corpus class) proven in-guest yields a journal
    whose anchor fields mismatch the certified fixtures; assert the
    mismatch is machine-visible in journal bytes (zoo class + golden
    journal).
  - **T-A10-3** — HANDOFF Phase-3 malicious-prover games kept as permanent
    tests: adversarially selected-but-valid input sets per claim type, each
    asserting the journal exposes the selection.

### A11 (k) — Re-genesis events (anchor rotation)

- **Attack.** Not an attacker — an upstream *event* that behaves like one if
  unhandled. Mithril re-genesis resets the certificate chain and rotates the
  genesis anchor: proofs rooted at the old genesis vkey remain
  cryptographically valid for a chain that is no longer the network's
  operative one; verifiers pinned to the old anchor accept them
  indefinitely.
- **What it would achieve (if unhandled).** A frozen or forked view of
  Cardano presented as current — stale-anchor replay at the trust-root
  level.
- **This is real and recurring.** Both networks were re-genesised in
  February 2025 (preprod genesis epoch 196 sealed 2025-02-09; mainnet epoch
  539 sealed 2025-02-13, coinciding with the Pythagoras era switch and
  client security advisory GHSA-724h-fpm5-4qvr); both chains are ~105 hops
  deep *because* of it (docs/notes/mithril-recursion-watch.md §3). The next
  likely trigger is already visible: recursive-certificate activation may
  imply an era switch and re-genesis (docs/notes/mithril-recursion-watch.md
  §4 ask 4).
- **Current mitigations.**
  - **The genesis vkey is a journal field, not a constant baked into the
    image** (docs/notes/mithril-recursion-watch.md §3: "the pinned genesis
    vkey/cert must be a versioned input of the checkpoint claim").
    Every checkpoint journal states which root it is rooted at; a
    stale-anchor proof is *legible* as such.
  - **Registry governance for anchor updates:** the router's expected
    genesis anchor lives in the same timelocked registry as image IDs
    (HANDOFF Phase 1); a re-genesis is handled as a governed trust-anchor
    rotation — proposal, timelock, event, cutover — exactly like a vendor
    bump (§6, §9 gates 2–3). No silent switch.
  - Anchor-placement note: the examples pass anchors as committed inputs
    and pin them in the contract; a guest-const anchor would instead rotate
    the *image ID* per network/re-genesis. Either way the rotation is
    governed; the const-vs-input decision is owned by ADR-002/ADR-004 and
    must record the rejected option
    (docs/notes/lightclient-patterns.md §2a, §6.1). v1 of this document
    assumes the journaled-field + registry-pin design per HANDOFF §1.
- **Residual risk.** Between the re-genesis and the registry cutover, no
  new proofs can land (liveness gap, acceptable); consumers holding
  portable verdicts across a re-genesis must re-check the journaled anchor
  against the *new* root — a documented consumer obligation. Detection of a
  re-genesis is operational (watch the aggregator + advisories), not
  cryptographic.
- **Pinning tests.**
  - **T-A11-1** — re-genesis fixture pair: chains rooted at the pre- and
    post-Feb-2025 genesis keys; a proof rooted at the old key against a
    router pinned to the new anchor is rejected; the journal's genesis-vkey
    field is asserted present and mutation-sensitive (zoo class).
  - **T-A11-2** — anchor-rotation drill: fork test walking the full
    governed rotation (propose new anchor → timelock elapses → old-anchor
    proofs rejected, new-anchor proofs accepted); kept permanent so the
    playbook can never rot.

### A12 (l) — Sextant-inherited assumption failures

- **Attack.** One of Sextant's *surfaced, assumed-true* conditions fails in
  the world: `mithril_quorum` (a colluding registered block producer serves
  a valid-header chain that omits a spend — Sextant verifies headers, links,
  and the height bound but does not bind each served block to the certified
  tx root, `window.rs:174-183`); honest-majority header authorship (a valid
  Praos segment is not proven canonical — README.md:143); the adequacy of
  Mithril's `k/m/phi_f` against the real adversarial stake fraction (Leg 2);
  the single-IOG-key `AncillarySigned` basis for the Tier-2 bootstrap
  (`ancillary.rs:32-41`); the caller-supplied `eta0` epoch nonce. All via
  docs/notes/sextant-legs.md.
- **What it achieves.** A true verdict under the stated assumptions that is
  false in the world. **These are not heliograph bugs** — heliograph proves
  that Sextant's verification ran; if Sextant's assumptions fail, the
  proven verdict inherits the failure.
- **Current mitigations — legibility, verbatim.**
  - **Inherited verbatim, surfaced in journals:** the assumption bits
    (`mithril_quorum`, `data_complete`), tier byte, anchor-basis byte, and
    every scoping field cross from Sextant's ABI structs into the journal
    unchanged (docs/notes/sextant-legs.md §1 journal-field inventories;
    `SextantWatchVerdict` ffi.rs:884-919 adopted "nearly verbatim").
  - **A proven verdict keeps its tier** (HANDOFF §0 non-negotiable):
    a proof of a Tier-1 windowed verdict is still Tier-1; laundering a tier
    through a proof is forbidden. The claim schema makes tier laundering a
    schema violation, not a judgment call.
  - Heliograph adds no claim semantics beyond Sextant's (HANDOFF §2) — so
    the assumption surface here is exactly Sextant's documented one, no
    larger.
- **Residual risk.** The assumptions themselves — that is their nature.
  What heliograph owes strangers is that no journal ever *hides* one. Note
  also the fixture gulf: preprod under-exercises mainnet parameters by
  orders of magnitude (k=5 vs k=1,944; 11 vs ~2,434 lottery indices per
  cert — docs/notes/mithril-recursion-watch.md §3), so an assumption- or
  bounds-related path tested only on preprod is under-tested; the corpus
  must include mainnet-read-only fixtures (HANDOFF §3, §5).
- **Pinning tests.**
  - **T-A12-1** — the equality invariant (CI-blocking, HANDOFF §7): for
    every fixture in the Sextant corpus — golden AND mutant — guest verdict
    == native Sextant verdict, **and journal tier/assumptions match the
    native metadata**. Rejection parity included.
  - **T-A12-2** — tier-laundering negative test: the `hg-claims` codec
    rejects any encoding of a Tier-1 verdict lacking its assumption bits or
    carrying an upgraded tier byte; golden journals assert
    assumption/tier/basis fields present per claim type.
  - **T-A12-3** — corpus composition gate: the fixture set includes
    mainnet-read-only certificates and mutants at mainnet parameters; CI
    fails if a claim type's corpus is preprod-only.

### A13 (+) — Griefing with valid proofs

- **Attack.** Economic, not soundness: spam the router with *valid* minimal
  updates (tiny extensions that advance state uselessly), or feed the
  prover pathological-but-capped inputs to burn proving budget.
- **What it achieves.** Gas and proving-cost exhaustion; checkpoint churn
  that degrades indexers.
- **Current mitigations.** Blobstream0 shipped `minBatchSize` only after the
  fact (PR #49, "prevent DOS attacks") — the catalog entry exists so
  heliograph decides *before* deployment
  (docs/notes/lightclient-patterns.md §4.3, §6.5): permissionless update +
  an explicit minimum-progress rule (or a recorded decision not to have
  one) is the design point. Guest-side, Sextant's compiled-in DoS caps
  bound cycles per input (`MAX_STM_BLOB_HEX` 4 MiB, `MAX_AVK_LEAVES` 2^24,
  `MAX_SINGLE_SIGS` 2^16, `MAX_LOTTERY_INDICES` 2^18 — `guard_stm_bounds`,
  mithril.rs:564-610 via docs/notes/sextant-legs.md).
- **Residual risk.** Whatever rule is chosen trades liveness granularity
  against spam; the decision belongs in the router ADR with the parameter
  recorded.
- **Pinning tests.**
  - **T-A13-1** — fork test: a valid update below the minimum-progress rule
    is rejected (or, if the recorded decision is "no rule", a test that
    documents and exercises the accepted-spam behavior so the decision
    stays visible).
  - **T-A13-2** — guest-bounds fixtures: inputs at and just over each DoS
    cap; over-cap inputs produce journaled rejections at bounded cycle
    counts (rejection parity + cost bound).

---

## 3. Vendor-trust surface (document, not fix)

Heliograph consumes both vendors' proof systems as unmodifiable, pinned
dependencies (HANDOFF §2, §6). This table is the trust we take rather than
verify; each row is re-pinned at every vendor bump (§9 gate 2). Sources:
docs/notes/sp1.md, docs/notes/risc0.md (recon 2026-07-14; re-pin exact
versions at M0).

| Surface | SP1 (Succinct) — v6 "Hypercube" line | RISC Zero — v3.0.5 / risc0-ethereum v3.0.1 |
|---|---|---|
| **Proving-system soundness assumptions** | STARKs over KoalaBear field, Poseidon2 in the random-oracle model; FRI in the unique-decoding regime (Hypercube); vendor states recursion incurs no security loss over steps (sp1.md §3) | rv32im STARK circuit + recursion STARK + STARK-to-SNARK R1CS (circom); assumption stack documented across vendor audit set (risc0.md §3–4) |
| **Wrap ceremony (trusted setup)** | Groth16: **circuit-specific ceremony** by Succinct, 18 named participants, Semaphore tooling. PLONK alternative: Aztec Ignition **universal** SRS (an SRS assumption remains). Artifacts versioned via `SP1_CIRCUIT_VERSION`, downloaded as binaries (sp1.md §3) | Groth16: phase 1 = Hermez `powersOfTau28_hez_final_23.ptau` (2^23); phase 2 = **PSE p0tion/DefinitelySetup**, per-contributor Gist attestations, pinned `stark_verify.r1cs` SHA-256, **independently re-verifiable** via documented `snarkjs zkey verify` procedure (risc0.md §3) |
| **Verifier contract + audit provenance** | `sp1-contracts` v6.1.1 (Groth16/PLONK verifiers + gateway; verifiers freezable); README states a Veridise audit; zkVM audit set: Cantina, Code4rena, Zellic (Hypercube), Kalos, rkm0959, Veridise (sp1.md §4). ⚠ Gateway ownership undocumented — resolve before M4 (sp1.md open item 5). ⚠ Repo lacks a root LICENSE (SPDX-per-file MIT) — confirm before vendoring (sp1.md §5) | `risc0-ethereum` v3.0.1, Apache-2.0. Audits (rz-security registry): Hexens zkVM 2023-10, Hexens STARK-to-SNARK 2024-05, **Hexens SNARK Verifier Contract 2024-06**, Veridise zkVM 2025-02, Veridise bigint2 2024-12, Veridise keccak 2025-02 (risc0.md §4) |
| **Verifier admin surface (upstream of ours)** | Gateway routes by proof-prefix to verifier builds; verifiers can be frozen permanently; owner identity unresolved (sp1.md §4) | Router owned by `TimelockController`, multisig proposer; removed selectors tombstoned forever; per-verifier emergency stop incl. **permissionless proof-of-unsoundness circuit breaker**, unrestartable (risc0.md §4) |
| **Reproducible-build path (image ID provenance)** | `cargo prove build --docker --tag vX.Y.Z` (`ghcr.io/succinctlabs/sp1`); non-docker explicitly non-reproducible (sp1.md §1) | Docker toolchain (`cargo risczero build`); vendor code comments state non-docker builds differ across architectures — docker is the only canonical path (risc0.md §1) |
| **Known caveats carried into this model** | Hints/unconstrained inputs are developer-audited (A1); prover-network input privacy undocumented (§1.2); determinism structural, not stated — M0 asserts identical-inputs ⇒ identical-journal across hosts (sp1.md open item 2) | Precompiles not constant-time (vendor-stated) — irrelevant here, all heliograph inputs are public (risc0.md §2); Groth16 wrap x86-only; self-recursion inner-ID pattern vendor-undocumented, M1 spike (risc0.md §6–7) |

**Watch row (not yet a dependency):** if ADR-005 ever swaps the STM leg for
native recursive Mithril certificates, the trust surface moves rather than
vanishes: image ID → Midnight Halo2/KZG circuit VK + SRS provenance. Today
that setup is regenerated in-function ("unsafe setup", upstream #3300) and
the Midnight ZK library audit is in progress — the swap cannot clear the
fail-closed bar until both are fixed and published
(docs/notes/mithril-recursion-watch.md §2, §4 asks 2 and 6).

---

## 4. Out of scope for v0.1

Per HANDOFF §2 — hard boundaries, each with the reason it is a boundary:

| Excluded | One-line reason |
|---|---|
| **Asset-transfer bridges / custody / wrapped assets / tokens** | Heliograph is the state-oracle layer; bridges are *consumers* of it — custody risk is theirs by design, and importing it would multiply this document by the entire bridge-exploit corpus. |
| **Realtime consumption** | Proof latency is minutes; consumers needing seconds are out of scope and the docs say so — no engineering here can promise otherwise honestly. |
| **Prover decentralization protocol** | v0.1 designs so N independent provers *can* exist (permissionless verification and submission) and operates one; a decentralization protocol is its own product with its own threat model. |
| **Inbound leg** (Plutus verifying foreign proofs, CIP-381/Halo2-Plutus) | Outbound only in v0.1; the two-way vision is a one-page note, not a build. |
| **New claim semantics beyond Sextant** | If Sextant can't verify it, heliograph can't prove it — the assumption surface stays exactly Sextant's (A12). |
| **Custom circuits / vendor proof-system modifications** | §9 human gate, default answer no; every threat above assumes the vendor stack is consumed byte-identical. |

---

## 5. Test ledger

Cross-reference of every pinning test named above, for wiring into
`make gate` and `docs/journal-coverage.md`. All are **planned** at v1.

| Test | Adversary | Suite / location (per HANDOFF §5 layout) | Gate |
|---|---|---|---|
| T-A1-1 unbound-input zoo | A1 (all) | guest test crates + `docs/journal-coverage.md` | `make gate`, CI-blocking |
| T-A1-2 degenerate-path mutants | A1 | zoo class | `make gate` |
| T-A1-3 patched-vs-original crates | A1 | CI matrix job (SP1 path) | CI |
| T-A2-1 wrong-network revert | A2 | `contracts/test` fork tests | CI |
| T-A2-2 network_id golden + mutant | A2 | golden journals + zoo | `make gate` |
| T-A2-3 cross-layer network agreement | A2 | SDK + CLI + fork tests | CI |
| T-A3-1 foreign-image rejection | A3 | `contracts/test` fork tests | CI |
| T-A3-2 recursion inner-ID chain | A3 | zoo + fork tests | `make gate` + CI |
| T-A4-1 unknown-version rejection (fuzz) | A4 | Foundry fuzz + cargo fuzz + SDK tests | CI |
| T-A4-2 downgrade fixture | A4 | `/fixtures` + all verifier layers | CI |
| T-A4-3 golden-encoding canonicality | A4 | `hg-claims` golden tests | `make gate` |
| T-A5-1 as-of fields golden + mutant | A5 | golden journals + zoo | `make gate` |
| T-A5-2 router-accepts / consumer-rejects | A5 | `examples/evm-oracle` fork test | CI |
| T-A5-3 stale-selection honesty | A5 | Phase-3 scenario, permanent | CI |
| T-A6-1 two-independent-builds | A6 | CI job `reproducible-image-id` | CI-blocking from M1 |
| T-A6-2 clean-room rebuild audit | A6 | Phase-3 + per-release checklist | release gate |
| T-A7-1 wrap-artifact digest pins | A7 | CI pin check | CI |
| T-A7-2 ceremony re-verification | A7 | per vendor bump (§9 gate 2 checklist) | gate 2 |
| T-A8-1 no-state-rewrite ABI surface | A8 | `contracts/test` | CI |
| T-A8-2 timelock delay + event | A8 | `contracts/test` fork tests | CI |
| T-A8-3 registry state-machine fuzz | A8 | Foundry fuzz | CI |
| T-A9-1 offline portable verdict | A9 | `examples/portable-verdict`, network-disabled | M3 gate → CI |
| T-A9-2 permissionless update | A9 | `contracts/test` | CI |
| T-A9-3 withholding → Stalled parity | A9 | equality-invariant corpus | `make gate` |
| T-A10-1 old-checkpoint regression | A10 | `contracts/test` fork tests | CI |
| T-A10-2 minority-fork journal visibility | A10 | `/fixtures` mutants + zoo | `make gate` |
| T-A10-3 selection-games suite | A10 | Phase-3 scenarios, permanent | CI |
| T-A11-1 re-genesis fixture pair | A11 | `/fixtures` + `contracts/test` | CI |
| T-A11-2 anchor-rotation drill | A11 | `contracts/test` fork tests | CI |
| T-A12-1 equality invariant (tier/assumptions) | A12 | full Sextant corpus, CI-blocking | `make gate` |
| T-A12-2 tier-laundering negative | A12 | `hg-claims` schema tests | `make gate` |
| T-A12-3 mainnet-fixture corpus gate | A12 | corpus composition check | CI |
| T-A13-1 minimum-progress rule | A13 | `contracts/test` | CI |
| T-A13-2 DoS-cap bounds fixtures | A13 | guest tests + `/fixtures` | `make gate` |

---

*v1 freeze checklist (before Phase-0 exit gate): read the Zellic sp1-helios
and Veridise blobstream0 audit PDFs and harvest any additional adversaries
(docs/notes/lightclient-patterns.md, open follow-ups); resolve the SP1
gateway ownership question or record it as an M4 blocker. v2 lands after
Phase 3 with the zoo green and residual risks measured, and is a v0.1.0
ship requirement (HANDOFF §3).*
