//! IERS Earth Orientation Parameters (EOP) and UT1.
//!
//! UT1 is the Earth-rotation time scale. It cannot be derived from a
//! leap-second kernel: it requires an IERS Earth Orientation Parameters
//! product that measures the actual rotation of the Earth. The two data
//! sets are independent.
//!
//! This module stores the `UT1 - UTC` column of the IERS
//! `finals2000A.all` product and interpolates it linearly in time, which
//! is the method recommended by IERS for a daily series.
//!
//! # Column layout
//!
//! `finals2000A.all` is a **fixed-column** format. It is *not*
//! whitespace-delimited: when the month or day has two digits the year,
//! month and day run together (`"73 110"` is `1973-01-10`). Splitting on
//! whitespace therefore shifts every later column and silently reads a
//! different quantity. This parser reads the documented columns instead:
//!
//! | Field | Columns (1-based) | Notes |
//! |-------|-------------------|-------|
//! | Year  | 1-2   | two digits, `" 0"` means 2000 |
//! | Month | 3-4   | |
//! | Day   | 5-6   | |
//! | MJD   | 7-15  | |
//! | Flag  | 58    | `I` observed, `P` predicted |
//! | `UT1 - UTC` | 59-68 | seconds |
//!
//! Rows whose `UT1 - UTC` column is blank are skipped. The formatter
//! appends placeholder rows for future dates; they carry no measurement.
//!
//! # Leap seconds and interpolation
//!
//! `UT1 - UTC` jumps by one second when a positive leap second is
//! inserted, because UTC is delayed by one second while UT1 keeps
//! running. The raw daily table shows that jump between the last day of
//! June or December and the first day of the next month. Interpolating
//! the raw values directly would smear a one-second step across a whole
//! day, which is a half-second error at local noon.
//!
//! [`crate::convert::ut1_minus_utc`] therefore removes the known step
//! before interpolating, using the leap-second table. The raw
//! [`EopTable::ut1_minus_utc`] here is a plain interpolator and should be
//! used only for table inspection or where the leap-second step is
//! irrelevant.
//!
//! # Deployment
//!
//! No EOP file is bundled with the crate. Download `finals2000A.all` at
//! deployment time with `tools/fetch_eop.py` and load it with
//! [`EopTable::from_finals2000a`]. The crate never performs network I/O.

use alloc::vec::Vec;

use crate::error::TimeError;
use crate::math;

/// One IERS EOP sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EopSample {
    /// Modified Julian Date of the sample, at `00:00:00 UTC`.
    pub mjd: f64,
    /// `UT1 - UTC` in seconds.
    pub ut1_minus_utc: f64,
    /// `true` when the value is an IERS observation (`I` flag) rather
    /// than a prediction (`P` flag). Predicted values are less accurate
    /// and are revised by later releases of the file.
    pub observed: bool,
}

/// Which `UT1 - UTC` series to read from `finals2000A.all`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EopSeries {
    /// Bulletin A: the rapid-service and predicted series, columns 59-68.
    /// This is the series the `finals2000A` product is named for.
    BulletinA,
    /// Bulletin B: the final solution, in the trailing columns.
    BulletinB,
    /// Bulletin B where present, otherwise Bulletin A. This matches the
    /// combined `UT1_UTC` column used by astropy's IERS tables.
    FinalPreferred,
}

/// A table of `UT1 - UTC` samples.
#[derive(Clone, Debug)]
pub struct EopTable {
    samples: Vec<EopSample>,
}

impl EopTable {
    /// Builds a table from samples. The samples are sorted by MJD and
    /// duplicates are removed.
    pub fn new(mut samples: Vec<EopSample>) -> Result<Self, TimeError> {
        if samples.is_empty() {
            return Err(TimeError::EopMissing);
        }

        // NaN and infinity cannot be ordered, and a non-finite `UT1 - UTC`
        // would silently poison every interpolation that uses it.
        if samples
            .iter()
            .any(|sample| !sample.mjd.is_finite() || !sample.ut1_minus_utc.is_finite())
        {
            return Err(TimeError::EopParse);
        }

        samples.sort_by(|a, b| a.mjd.total_cmp(&b.mjd));
        samples.dedup_by(|a, b| math::abs(a.mjd - b.mjd) < 1e-9);

        Ok(Self { samples })
    }

    /// Parses the IERS `finals2000A.all` fixed-column format, using the
    /// Bulletin A series.
    ///
    /// Rows with a blank `UT1 - UTC` column, such as the placeholder
    /// rows for dates that have not been measured yet, are skipped.
    pub fn from_finals2000a(text: &str) -> Result<Self, TimeError> {
        Self::from_finals2000a_series(text, EopSeries::BulletinA)
    }

    /// Parses `finals2000A.all` and selects the requested series.
    ///
    /// This product carries two `UT1 - UTC` series: Bulletin A at columns
    /// 59-68 (rapid service and predictions) and Bulletin B at columns
    /// 156-165 (final solution). They differ by up to several milliseconds
    /// for old data. `EopSeries::FinalPreferred` reproduces the combined
    /// choice made by astropy's IERS tables.
    pub fn from_finals2000a_series(text: &str, series: EopSeries) -> Result<Self, TimeError> {
        let mut samples = Vec::new();

        for line in text.lines() {
            // Bulletin A needs columns 59-68; Bulletin B wants more.
            if line.len() < 68 {
                continue;
            }

            let mjd_field = match line.get(6..15) {
                Some(field) => field.trim(),
                None => continue,
            };
            if mjd_field.is_empty() {
                continue;
            }

            let bulletin_a = line
                .get(58..68)
                .and_then(|field| field.trim().parse::<f64>().ok());
            let bulletin_b = line.get(155..165).and_then(|field| {
                let text = field.trim();
                if text.is_empty() {
                    None
                } else {
                    text.parse::<f64>().ok()
                }
            });

            let a_observed = line.get(57..58).map(|flag| flag != "P").unwrap_or(true);

            let (ut1_minus_utc, observed) = match series {
                EopSeries::BulletinA => match bulletin_a {
                    Some(value) => (value, a_observed),
                    None => continue,
                },
                EopSeries::BulletinB => match bulletin_b {
                    Some(value) => (value, true),
                    None => continue,
                },
                EopSeries::FinalPreferred => match bulletin_b {
                    Some(value) => (value, true),
                    None => match bulletin_a {
                        Some(value) => (value, a_observed),
                        None => continue,
                    },
                },
            };

            let mjd = match mjd_field.parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            samples.push(EopSample {
                mjd,
                ut1_minus_utc,
                observed,
            });
        }

        if samples.is_empty() {
            return Err(TimeError::EopParse);
        }

        Self::new(samples)
    }

    /// Returns a table containing only the IERS observed samples.
    ///
    /// Predicting `UT1 - UTC` is inherently uncertain; restricting a
    /// table to observations makes the supported interval explicit.
    pub fn observed_only(&self) -> Result<Self, TimeError> {
        Self::new(
            self.samples
                .iter()
                .copied()
                .filter(|sample| sample.observed)
                .collect(),
        )
    }

    /// Returns `UT1 - UTC` at the given MJD by linear interpolation.
    ///
    /// This is a raw interpolator. It does not remove the one-second step
    /// that a positive leap second introduces into `UT1 - UTC`; use
    /// [`crate::convert::ut1_minus_utc`] for an epoch on the UT1 scale.
    pub fn ut1_minus_utc(&self, mjd: f64) -> Result<f64, TimeError> {
        if !mjd.is_finite() {
            return Err(TimeError::Ut1OutOfRange);
        }

        let first = self.samples.first().ok_or(TimeError::EopMissing)?;
        let last = self.samples.last().ok_or(TimeError::EopMissing)?;

        if mjd < first.mjd - 1e-9 || mjd > last.mjd + 1e-9 {
            return Err(TimeError::Ut1OutOfRange);
        }

        let index = self.samples.partition_point(|sample| sample.mjd <= mjd);
        if index == 0 {
            return Ok(first.ut1_minus_utc);
        }
        if index >= self.samples.len() {
            return Ok(last.ut1_minus_utc);
        }

        let before = self.samples[index - 1];
        let after = self.samples[index];
        let span = after.mjd - before.mjd;

        if span <= 0.0 {
            return Ok(before.ut1_minus_utc);
        }

        let fraction = (mjd - before.mjd) / span;
        Ok(before.ut1_minus_utc + fraction * (after.ut1_minus_utc - before.ut1_minus_utc))
    }

    /// Returns the MJD range covered by the table.
    pub fn range(&self) -> (f64, f64) {
        match (self.samples.first(), self.samples.last()) {
            (Some(first), Some(last)) => (first.mjd, last.mjd),
            _ => (f64::NAN, f64::NAN),
        }
    }

    /// Returns the samples in ascending MJD order.
    pub fn samples(&self) -> &[EopSample] {
        &self.samples
    }

    /// Returns the number of samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` when the table has no samples.
    ///
    /// A constructed [`EopTable`] is never empty, so this always returns
    /// `false` in practice.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}
