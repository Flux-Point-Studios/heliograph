# The session gate (HANDOFF §7/§10): run on a clean tree every session.
# Phase 0 has no crates yet, so the gate is scaffold integrity; it GROWS —
# fmt + clippy -D warnings + tests + equality invariant + journal zoo +
# golden journals land with the workspace and are appended here, never removed.

.PHONY: gate
gate:
	@test -f HANDOFF.md || { echo "gate: HANDOFF.md missing"; exit 1; }
	@test -f STATUS.md || { echo "gate: STATUS.md missing"; exit 1; }
	@test -f LICENSE || { echo "gate: LICENSE missing"; exit 1; }
	@if [ -f Cargo.toml ]; then \
		cargo fmt --all -- --check && \
		cargo clippy --all-targets --all-features -- -D warnings && \
		cargo test --all-features; \
	else \
		echo "gate: phase-0 scaffold OK (no workspace yet — gate grows with the code)"; \
	fi
