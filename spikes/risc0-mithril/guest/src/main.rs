//! S-0001 spike guest: one Mithril standard-certificate verification on
//! Sextant's `mithril` path, inside the RISC Zero zkVM. Journals a u32 verdict
//! code — accepted or rejected, never a bare panic (HANDOFF §5).
#![no_main]
risc0_zkvm::guest::entry!(main);

use risc0_zkvm::guest::env;

fn main() {
    let cert_json: Vec<u8> = env::read();
    env::commit(&verdict(&cert_json));
}

/// Verdict codes: 0 = accepted; 1 = JSON parse failure; 2..=8 = the
/// `StandardError` variants in declaration order (mithril.rs:392).
#[cfg(feature = "mithril")]
fn verdict(cert_json: &[u8]) -> u32 {
    use sextant::mithril::{verify_standard, Certificate, StandardError};
    let cert = match Certificate::from_json(cert_json) {
        Ok(cert) => cert,
        Err(_) => return 1,
    };
    match verify_standard(&cert) {
        Ok(()) => 0,
        Err(StandardError::NotStandard) => 2,
        Err(StandardError::MessageMismatch) => 3,
        Err(StandardError::WeakParameters) => 4,
        Err(StandardError::ImplausibleAvk) => 5,
        Err(StandardError::MalformedAvk) => 6,
        Err(StandardError::MalformedSignature) => 7,
        Err(StandardError::InvalidMultiSignature) => 8,
    }
}

/// Bisect layer 1 (no `mithril` feature): prove the default no_std-canary-gated
/// sextant graph compiles and links for the zkVM target. Touches a real
/// verify-path entry point so the linker cannot drop the rlib wholesale.
#[cfg(not(feature = "mithril"))]
fn verdict(_cert_json: &[u8]) -> u32 {
    let n = sextant::nonce::combine(&[0u8; 32], &[0u8; 32]);
    u32::MAX - u32::from(n[0])
}
