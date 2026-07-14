//! S-0002 — SP1 arm of the M0 compile bisect (BENCH.md §6.1, ADR-001).
//! Reads Mithril certificate JSON bytes via `sp1_zkvm::io`, verifies on
//! Sextant's `mithril` path, commits a `u32` verdict. Producing this ELF via
//! `cargo prove build` is the spike's win condition; the verdict codes matter
//! only if it does.
#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let bytes = sp1_zkvm::io::read_vec();
    let verdict = verdict(&bytes);
    sp1_zkvm::io::commit(&verdict);
}

/// Layer 2: the BENCH.md §2.1 workload minus the journal codec — parse,
/// content-hash recompute, `verify_standard`.
#[cfg(feature = "mithril")]
fn verdict(bytes: &[u8]) -> u32 {
    use sextant::mithril::{Certificate, verify_standard};
    let cert = match Certificate::from_json(bytes) {
        Ok(c) => c,
        Err(_) => return 1,
    };
    if cert.compute_hash() != cert.hash {
        return 2;
    }
    match verify_standard(&cert) {
        Ok(()) => 0,
        Err(_) => 3,
    }
}

/// Layer 1: sextant `default-features = false` only — no `mithril` module
/// exists, so exercise a default-graph verify (input = vkey ‖ sig ‖ msg) to
/// keep sextant genuinely in the build.
#[cfg(not(feature = "mithril"))]
fn verdict(bytes: &[u8]) -> u32 {
    if bytes.len() < 96 {
        return 1;
    }
    let mut vkey = [0u8; 32];
    vkey.copy_from_slice(&bytes[..32]);
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&bytes[32..96]);
    u32::from(!sextant::ed25519::verify(&vkey, &bytes[96..], &sig))
}
