//! Deterministic property tests.
//!
//! These tests **run** under `cargo test`. They are not proofs: they
//! sample many inputs with a fixed seed. The formal proofs live in
//! `src/verification.rs` and run under Kani on Linux or macOS, because
//! Kani has no Windows build (see `.github/workflows/kani.yml`).

use stkcraft_time::{Duration, Epoch};

/// A small deterministic generator so failures are reproducible.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        // Numerical Recipes LCG constants.
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn range(&mut self, low: i64, high: i64) -> i64 {
        let span = (high - low) as u64;
        low + (self.next_u64() % span) as i64
    }
}

const ITERATIONS: usize = 200_000;

#[test]
fn duration_from_parts_is_canonical() {
    let mut rng = Lcg::new(0x534b_4352_4146_5421);

    for _ in 0..ITERATIONS {
        let sec = rng.range(-1_000_000_000, 1_000_000_000);
        let nanos = rng.range(-2_000_000_000, 2_000_000_000);

        let duration = Duration::from_parts(sec, nanos);

        assert!((0..1_000_000_000).contains(&duration.nanos()));
        assert_eq!(
            duration.total_nanoseconds(),
            sec as i128 * 1_000_000_000 + nanos as i128
        );
        assert_eq!(
            Duration::from_nanoseconds(duration.total_nanoseconds()),
            Some(duration)
        );
    }
}

#[test]
fn epoch_from_parts_is_canonical() {
    let mut rng = Lcg::new(0x4549_4748_5400_0001);

    for _ in 0..ITERATIONS {
        let sec = rng.range(-1_000_000_000, 1_000_000_000);
        let nanos = rng.range(-2_000_000_000, 2_000_000_000);

        let epoch = Epoch::from_parts(sec, nanos);

        assert!(epoch.nanos() < 1_000_000_000);
        assert_eq!(
            epoch.sec() as i128 * 1_000_000_000 + epoch.nanos() as i128,
            sec as i128 * 1_000_000_000 + nanos as i128
        );
    }
}

#[test]
fn duration_checked_add_is_exact() {
    let mut rng = Lcg::new(0x4144_4400_0000_0001);

    for _ in 0..ITERATIONS {
        let a = Duration::from_parts(
            rng.range(-1_000, 1_000),
            rng.range(-2_000_000_000, 2_000_000_000),
        );
        let b = Duration::from_parts(
            rng.range(-1_000, 1_000),
            rng.range(-2_000_000_000, 2_000_000_000),
        );

        if let Some(sum) = a.checked_add(b) {
            assert_eq!(
                sum.total_nanoseconds(),
                a.total_nanoseconds() + b.total_nanoseconds()
            );
            assert!((0..1_000_000_000).contains(&sum.nanos()));
        }
    }
}

fn expected_duration(total: i128) -> Option<(i64, i32)> {
    let sec = total.div_euclid(1_000_000_000);
    let nanos = total.rem_euclid(1_000_000_000);
    if sec < i64::MIN as i128 || sec > i64::MAX as i128 {
        None
    } else {
        Some((sec as i64, nanos as i32))
    }
}

#[test]
fn duration_checked_operations_match_i128_oracle_at_extremes() {
    let seconds = [
        i64::MIN,
        i64::MIN + 1,
        -1_000_000_000,
        -1,
        0,
        1,
        1_000_000_000,
        i64::MAX - 1,
        i64::MAX,
    ];
    let nanos = [
        -1_000_000_000,
        -1,
        0,
        1,
        500_000_000,
        999_999_999,
        1_000_000_000,
    ];

    let mut values = Vec::new();
    for &sec in &seconds {
        for &nano in &nanos {
            if let Some(value) = Duration::checked_from_parts(sec, nano) {
                values.push(value);
            }
        }
    }

    for &a in &values {
        let a_total = a.total_nanoseconds();
        assert_eq!(
            a.checked_neg().map(|value| (value.sec(), value.nanos())),
            expected_duration(-a_total)
        );

        for &b in &values {
            let b_total = b.total_nanoseconds();
            assert_eq!(
                a.checked_add(b).map(|value| (value.sec(), value.nanos())),
                expected_duration(a_total + b_total)
            );
            assert_eq!(
                a.checked_sub(b).map(|value| (value.sec(), value.nanos())),
                expected_duration(a_total - b_total)
            );
        }
    }
}

#[test]
fn epoch_checked_from_parts_matches_i128_oracle() {
    let cases = [
        (i64::MIN, -1_000_000_000),
        (i64::MIN, 0),
        (i64::MIN, 999_999_999),
        (i64::MAX, 0),
        (i64::MAX, 999_999_999),
        (i64::MAX, 1_000_000_000),
        (0, -1),
        (0, 1_000_000_000),
    ];

    for (sec, nanos) in cases {
        let total = sec as i128 * 1_000_000_000 + nanos as i128;
        let expected = expected_duration(total);
        let actual =
            Epoch::checked_from_parts(sec, nanos).map(|epoch| (epoch.sec(), epoch.nanos() as i32));
        assert_eq!(actual, expected, "sec={sec}, nanos={nanos}");
    }
}
