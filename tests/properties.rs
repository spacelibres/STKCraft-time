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
