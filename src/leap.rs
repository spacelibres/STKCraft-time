//! Leap-second table and the NAIF `DELTET` constants.
//!
//! The table is derived from a NAIF Leapseconds Kernel (LSK). The
//! constructor also derives the nominal UTC-second and TAI-second
//! instants of every effective date, which are required to handle
//! `23:59:60.x` correctly.

use alloc::vec::Vec;

use crate::calendar::J2000_JDN;
use crate::error::TimeError;

/// NAIF `DELTET/DELTA_T_A` nominal value: `TT - TAI` in seconds.
pub const DELTA_T_A: f64 = 32.184;
/// NAIF `DELTET/K` nominal value.
pub const K: f64 = 1.657e-3;
/// NAIF `DELTET/EB` nominal value.
pub const EB: f64 = 1.671e-2;
/// NAIF `DELTET/M` first coefficient (mean anomaly at J2000), radians.
pub const M0: f64 = 6.239996;
/// NAIF `DELTET/M` second coefficient, radians per TDB second.
pub const M1: f64 = 1.99096871e-7;

/// Julian Day Number of `1960-01-01`, the start of the SOFA historical
/// UTC model.
pub const JDN_1960: i64 = 2_436_935;

/// Julian Day Number of `1972-01-01`, where integer leap seconds begin.
pub const JDN_1972: i64 = 2_441_318;

/// One `DELTET/DELTA_AT` entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeapEntry {
    /// Julian Day Number of the UTC date on which `delta_at` takes effect
    /// at `00:00:00 UTC`.
    pub jdn: i64,
    /// `TAI - UTC` in whole seconds, effective on and after `jdn`.
    pub delta_at: i32,
}

/// Which sources are used for `TAI - UTC` before 1972.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeapSecondSource {
    /// Only the integer leap seconds announced by IERS, starting at
    /// `1972-01-01`. This matches the NAIF LSK.
    IersOnly,
    /// Additionally use the pre-1972 UTC drift model published by the
    /// IAU SOFA service (`iauDat`).
    Sofa,
}

/// What happens after the last applicable leap-second entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeapSecondPolicy {
    /// Follow the table exactly. If the table has no further entries,
    /// `TAI - UTC` stays at the last value indefinitely.
    TableDriven,
    /// Freeze `TAI - UTC` after the given Julian Day Number. Table
    /// entries after this date are ignored and the last applicable value
    /// is kept. This models a decision to stop introducing leap seconds.
    FrozenAfter { jdn: i64 },
}

impl LeapSecondPolicy {
    /// Freezes `TAI - UTC` from the given UTC date onward.
    pub fn frozen_after(year: i32, month: u8, day: u8) -> Self {
        Self::FrozenAfter {
            jdn: crate::calendar::gregorian_to_jdn(year, month, day),
        }
    }
}

/// A NAIF-aligned leap-second table.
#[derive(Clone, Debug)]
pub struct LeapSecondTable {
    /// `DELTET/DELTA_T_A`.
    pub delta_t_a: f64,
    /// `DELTET/K`.
    pub k: f64,
    /// `DELTET/EB`.
    pub eb: f64,
    /// `DELTET/M[0]`.
    pub m0: f64,
    /// `DELTET/M[1]`.
    pub m1: f64,
    entries: Vec<LeapEntry>,
    /// Nominal UTC seconds past `2000-01-01T00:00:00` at each effective
    /// date `00:00:00`.
    nominal: Vec<i64>,
    /// TAI seconds past `2000-01-01T00:00:00 TAI` at each effective date.
    tai_start: Vec<i64>,
    source: LeapSecondSource,
    policy: LeapSecondPolicy,
}

impl LeapSecondTable {
    /// Builds a table from `DELTET/DELTA_AT` entries using the nominal
    /// NAIF constants.
    pub fn new(mut entries: Vec<LeapEntry>) -> Result<Self, TimeError> {
        if entries.is_empty() {
            return Err(TimeError::LskMissing("DELTET/DELTA_AT"));
        }
        entries.sort_by_key(|e| e.jdn);

        for pair in entries.windows(2) {
            if pair[0].jdn == pair[1].jdn {
                return Err(TimeError::LskParse);
            }
        }

        let nominal: Vec<i64> = entries
            .iter()
            .map(|e| (e.jdn - J2000_JDN) * 86_400)
            .collect();
        let tai_start: Vec<i64> = entries
            .iter()
            .zip(nominal.iter())
            .map(|(e, n)| *n + e.delta_at as i64)
            .collect();

        for pair in tai_start.windows(2) {
            if pair[0] >= pair[1] {
                return Err(TimeError::LskParse);
            }
        }

        Ok(Self {
            delta_t_a: DELTA_T_A,
            k: K,
            eb: EB,
            m0: M0,
            m1: M1,
            entries,
            nominal,
            tai_start,
            source: LeapSecondSource::IersOnly,
            policy: LeapSecondPolicy::TableDriven,
        })
    }

    /// Replaces the `DELTET` constants, typically with values parsed from
    /// an LSK.
    pub fn with_constants(mut self, delta_t_a: f64, k: f64, eb: f64, m0: f64, m1: f64) -> Self {
        self.delta_t_a = delta_t_a;
        self.k = k;
        self.eb = eb;
        self.m0 = m0;
        self.m1 = m1;
        self
    }

    /// Parses the bundled `naif0012.tls`.
    ///
    /// This is a convenience snapshot, not a privileged source of truth.
    /// It carries the leap seconds known when that kernel was released
    /// (last entry `2017-01-01`). Production code that needs a specific
    /// kernel should load it explicitly with [`LeapSecondTable::from_lsk`].
    pub fn builtin() -> Result<Self, TimeError> {
        crate::lsk::builtin()
    }

    /// Builds a table from the text of a `KPL/LSK` kernel.
    ///
    /// This is the kernel-selection entry point: every conversion is a
    /// pure function of the kernel supplied here. Two kernels whose
    /// `DELTET/DELTA_AT` lists differ give different `TAI - UTC` after
    /// the first differing date, exactly as selecting a different
    /// kernel in a web tool such as the JPL WebGeocalc would.
    pub fn from_lsk(text: &str) -> Result<Self, TimeError> {
        crate::lsk::parse_lsk(text)
    }

    /// Selects the source used before 1972.
    pub fn with_source(mut self, source: LeapSecondSource) -> Self {
        self.source = source;
        self
    }

    /// Returns the selected pre-1972 source.
    pub fn source(&self) -> LeapSecondSource {
        self.source
    }

    /// Sets the future leap-second policy.
    pub fn with_policy(mut self, policy: LeapSecondPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Returns the future leap-second policy.
    pub fn policy(&self) -> LeapSecondPolicy {
        self.policy
    }

    /// Returns `TAI - UTC` at a UTC date, using the selected source.
    ///
    /// `day_fraction` is the fraction of the UTC day in `0.0..1.0`.
    pub fn delta_at_f64(&self, jdn: i64, day_fraction: f64) -> Option<f64> {
        if jdn >= JDN_1972 {
            self.delta_at_for_jdn(jdn).map(|value| value as f64)
        } else if self.source == LeapSecondSource::Sofa && jdn >= JDN_1960 {
            let mjd = (jdn - 2_400_001) as f64 + day_fraction;
            sofa_delta_at(mjd)
        } else {
            None
        }
    }

    /// TAI start of the first integer-leap-second interval.
    pub(crate) fn first_tai_start(&self) -> i64 {
        self.tai_start[0]
    }

    /// Returns the `DELTET/DELTA_AT` entries in ascending date order.
    pub fn entries(&self) -> &[LeapEntry] {
        &self.entries
    }

    /// Returns the `(first_jdn, last_jdn)` range of effective
    /// leap-second dates present in this table, as Julian Day Numbers.
    ///
    /// The first value is where the kernel's leap-second data starts;
    /// Civil UTC conversions reject dates before it. The explicitly named
    /// native JDUTC path selects TTRANS's separate historical convention.
    /// The second
    /// value is the most recent leap-second date the kernel knows about.
    ///
    /// After `last_jdn` the last `TAI - UTC` value is held constant.
    /// That is NAIF's documented behaviour (`TableDriven`), not data
    /// invented by this crate: a kernel cannot contain a leap second
    /// that was announced after it was released. When you need a hard
    /// guarantee past the kernel, compare `last_jdn` with the kernel's
    /// release date and update the kernel.
    pub fn coverage(&self) -> (i64, i64) {
        let first = self.entries.first().map(|entry| entry.jdn).unwrap_or(0);
        let last = self.entries.last().map(|entry| entry.jdn).unwrap_or(0);
        (first, last)
    }

    /// Nominal UTC seconds past `2000-01-01T00:00:00` at the effective
    /// date `index`.
    pub(crate) fn nominal_sec(&self, index: usize) -> i64 {
        self.nominal[index]
    }

    /// Returns the index of the last table entry whose TAI start is less
    /// than or equal to `tai_sec`, capped by the active policy.
    pub(crate) fn interval_index(&self, tai_sec: i64) -> Option<usize> {
        let max = self.max_index()?;
        let idx = self.tai_start.partition_point(|&t| t <= tai_sec);
        if idx == 0 {
            None
        } else {
            Some((idx - 1).min(max))
        }
    }

    /// Returns whether a table entry is active under the current policy.
    pub(crate) fn is_active_index(&self, index: usize) -> bool {
        match self.max_index() {
            Some(max) => index <= max,
            None => false,
        }
    }

    /// Highest table index allowed by the policy.
    fn max_index(&self) -> Option<usize> {
        match self.policy {
            LeapSecondPolicy::TableDriven => Some(self.entries.len() - 1),
            LeapSecondPolicy::FrozenAfter { jdn } => {
                let count = self.entries.partition_point(|e| e.jdn <= jdn);
                count.checked_sub(1)
            }
        }
    }

    /// Returns `TAI - UTC` for a UTC date, or `None` before the first
    /// active table entry.
    pub fn delta_at_for_jdn(&self, jdn: i64) -> Option<i32> {
        let lookup = match self.policy {
            LeapSecondPolicy::TableDriven => jdn,
            LeapSecondPolicy::FrozenAfter { jdn: freeze } => jdn.min(freeze),
        };
        let idx = self.entries.partition_point(|e| e.jdn <= lookup);
        if idx == 0 {
            None
        } else {
            Some(self.entries[idx - 1].delta_at)
        }
    }
}

/// Pre-1972 UTC drift table from IAU SOFA `iauDat`.
///
/// Each row is `(start_jdn, base, reference_mjd, drift_per_day)`.
/// Between rows, `TAI - UTC = base + (MJD - reference_mjd) * drift`.
const SOFA_ROWS: &[(i64, f64, f64, f64)] = &[
    (2_436_935, 1.417_818_0, 37_300.0, 0.001_296_0), // 1960-01-01
    (2_437_301, 1.422_818_0, 37_300.0, 0.001_296_0), // 1961-01-01
    (2_437_513, 1.372_818_0, 37_300.0, 0.001_296_0), // 1961-08-01
    (2_437_666, 1.845_858_0, 37_665.0, 0.001_123_2), // 1962-01-01
    (2_438_335, 1.945_858_0, 37_665.0, 0.001_123_2), // 1963-11-01
    (2_438_396, 3.240_130_0, 38_761.0, 0.001_296_0), // 1964-01-01
    (2_438_487, 3.340_130_0, 38_761.0, 0.001_296_0), // 1964-04-01
    (2_438_640, 3.440_130_0, 38_761.0, 0.001_296_0), // 1964-09-01
    (2_438_762, 3.540_130_0, 38_761.0, 0.001_296_0), // 1965-01-01
    (2_438_821, 3.640_130_0, 38_761.0, 0.001_296_0), // 1965-03-01
    (2_438_943, 3.740_130_0, 38_761.0, 0.001_296_0), // 1965-07-01
    (2_439_005, 3.840_130_0, 38_761.0, 0.001_296_0), // 1965-09-01
    (2_439_127, 4.313_170_0, 39_126.0, 0.002_592_0), // 1966-01-01
    (2_439_888, 4.213_170_0, 39_126.0, 0.002_592_0), // 1968-02-01
];

/// Returns `TAI - UTC` in seconds for a Modified Julian Date in the
/// pre-1972 UTC drift era, or `None` before 1960-01-01.
///
/// This is the IAU SOFA `iauDat` model. It returns fractional seconds
/// because UTC seconds were not SI seconds before 1972.
pub fn sofa_delta_at(mjd: f64) -> Option<f64> {
    let mut value = None;

    for &(start_jdn, base, reference_mjd, drift) in SOFA_ROWS {
        let start_mjd = (start_jdn - 2_400_001) as f64;
        if mjd < start_mjd {
            break;
        }
        value = Some(base + (mjd - reference_mjd) * drift);
    }

    value
}
