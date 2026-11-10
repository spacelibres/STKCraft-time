//! Kani proof harnesses.
//!
//! These are compiled only when running `cargo kani`. Kani's sysroot
//! provides the `kani` crate; it is not a normal dependency.

use crate::{Duration, Epoch};

#[kani::proof]
fn duration_normalization_is_canonical() {
    let sec: i64 = kani::any();
    let nanos: i64 = kani::any();

    kani::assume(sec > i64::MIN + 2 && sec < i64::MAX - 2);
    kani::assume(nanos > -2_000_000_000 && nanos < 2_000_000_000);

    let duration = Duration::from_parts(sec, nanos);
    assert!(duration.nanos() >= 0);
    assert!(duration.nanos() < 1_000_000_000);
}

#[kani::proof]
fn epoch_normalization_is_canonical() {
    let sec: i64 = kani::any();
    let nanos: i64 = kani::any();

    kani::assume(sec > i64::MIN + 2 && sec < i64::MAX - 2);
    kani::assume(nanos > -2_000_000_000 && nanos < 2_000_000_000);

    let epoch = Epoch::from_parts(sec, nanos);
    assert!(epoch.nanos() < 1_000_000_000);
}

#[kani::proof]
fn duration_addition_normalizes() {
    let a_sec: i64 = kani::any();
    let a_nanos: i64 = kani::any();
    let b_sec: i64 = kani::any();
    let b_nanos: i64 = kani::any();

    kani::assume(a_sec > -1_000 && a_sec < 1_000);
    kani::assume(a_nanos > -2_000_000_000 && a_nanos < 2_000_000_000);
    kani::assume(b_sec > -1_000 && b_sec < 1_000);
    kani::assume(b_nanos > -2_000_000_000 && b_nanos < 2_000_000_000);

    let a = Duration::from_parts(a_sec, a_nanos);
    let b = Duration::from_parts(b_sec, b_nanos);

    if let Some(sum) = a.checked_add(b) {
        assert!(sum.nanos() >= 0);
        assert!(sum.nanos() < 1_000_000_000);
    }
}
