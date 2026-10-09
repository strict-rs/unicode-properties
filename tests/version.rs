//! The Unicode release remains a public contract with every feature selection.

use strict_test_support::{ComparisonFailure, ensure_eq};

/// Expose the actual generated release even when both property APIs are disabled.
#[test]
fn unicode_version_is_available_independently_of_property_features()
-> Result<(), ComparisonFailure<(u64, u64, u64), (u64, u64, u64)>> {
    ensure_eq(
        unicode_properties::UNICODE_VERSION,
        (18, 0, 0),
        "the public version identifies the Unicode release used by both generated tables",
    )
    .map(drop)
}
