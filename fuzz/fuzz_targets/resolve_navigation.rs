//! Totality oracle over the address-bar classification surface (ADR-004,
//! ADR-021): any host-provided text classifies into exactly one of
//! Url / Search / Rejection — a decision, never a panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = browser_core::resolve_navigation(input);
    }
});
