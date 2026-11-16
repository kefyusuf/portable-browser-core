//! Totality + canonical round-trip oracle over the container session
//! format (ADR-009/014/015, ADR-021): any host-provided text parses or
//! reasons — never panics — and accepted text round-trips: the serialize →
//! deserialize hop preserves the `PartialEq` state view exactly.
#![no_main]

use browser_core::TabContainer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(container) = TabContainer::from_session_text(text) else {
        return;
    };
    let view = container.state();
    let serialized = container.to_session_text();
    let reparsed = TabContainer::from_session_text(&serialized)
        .expect("session re-parse must accept own bytes");
    assert_eq!(
        reparsed.state(),
        view,
        "session round-trip changed the state view"
    );
});
