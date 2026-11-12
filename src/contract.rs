//! Explicit contracts for SPICE-compatible geometry and scientific time.
//!
//! [`TimeContext::new`] always selects NAIF ET. Scientific TDB requires
//! [`ScientificContext`], and its result cannot implicitly become [`NaifEt`].
//! Existing low-level conversion functions retain their behavior.
//!
//! ```compile_fail
//! use stkcraft_time::contract::{NaifEt, ScientificContext};
//! use stkcraft_time::Epoch;
//! fn query_spice_geometry(_: NaifEt) {}
//! let scientific = ScientificContext::new().to_tdb(Epoch::from_parts(0, 0)).unwrap();
//! query_spice_geometry(scientific); // Different models require an explicit decision.
//! ```

use core::fmt;

use crate::calendar::{calendar_to_jdn, Calendar, J2000_JDN};
use crate::convert;
use crate::dtdb::{dtdb_minus_tt_at_observer, DtdbError};
use crate::leap::{LeapSecondPolicy, LeapSecondSource, LeapSecondTable, JDN_1972};
use crate::{naif, Epoch, TimeError, UtcDateTime};

/// Calculation convention. Science is never selected by the default context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CalculationProfile {
    /// NAIF LSK-based ET for SPICE geometry.
    #[default]
    NaifSpice,
    /// Explicit SOFA/ERFA scientific approximation, with TT realized by TAI.
    IauScientific,
}

/// Checked-contract failures, separate from legacy low-level API errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum ContractError {
    /// Civil time or kernel coverage error.
    Time(TimeError),
    /// Scientific model domain or observer input error.
    Scientific(DtdbError),
    /// Numeric input or evaluated output is not finite.
    NonFinite,
    /// A continuous epoch cannot be represented by [`Epoch`].
    EpochOutOfRange,
    /// Historical SOFA UTC is a separate convention from this NAIF contract.
    HistoricalUtcRequiresExplicitModel,
    /// The raw N0067 Julian UTC path requires an unmodified kernel policy.
    NaifKernelModelRequired,
    /// N0067 TTRANS rejects a non-increasing paired TAI leap table.
    InvalidNaifLeapSeconds,
}

impl From<TimeError> for ContractError {
    fn from(value: TimeError) -> Self {
        Self::Time(value)
    }
}

impl From<DtdbError> for ContractError {
    fn from(value: DtdbError) -> Self {
        Self::Scientific(value)
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Time(error) => error.fmt(f),
            Self::Scientific(error) => error.fmt(f),
            Self::NonFinite => f.write_str("time contract requires finite inputs and outputs"),
            Self::EpochOutOfRange => f.write_str("time contract epoch is out of range"),
            Self::HistoricalUtcRequiresExplicitModel => {
                f.write_str("historical UTC requires the explicit SOFA conversion API")
            }
            Self::NaifKernelModelRequired => {
                f.write_str("native Julian UTC requires an IERS-only, table-driven LSK model")
            }
            Self::InvalidNaifLeapSeconds => f.write_str("SPICE(BADLEAPSECONDS)"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ContractError {}

/// Finite NAIF ET, TDB seconds past `2000-01-01T12:00:00 TDB`.
///
/// This type does not by itself guarantee any SPK's coverage or physical
/// accuracy. [`Self::seconds`] is the explicit boundary to legacy SPICE APIs.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct NaifEt(f64);

/// Formal Julian UTC returned by N0067 TTRANS's explicitly selected model.
///
/// Before the first LSK entry this uses native's constant-offset convention,
/// not physically realized historical UTC. During a positive leap second
/// the formal value stands still. It cannot implicitly become ET:
///
/// ```compile_fail
/// use stkcraft_time::contract::{NaifEt, NaifJulianUtc};
/// fn geometry_epoch(_: NaifEt) {}
/// fn wrong_scale(value: NaifJulianUtc) { geometry_epoch(value); }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct NaifJulianUtc(f64);

impl NaifJulianUtc {
    /// Unrounded native Julian days, for an explicitly chosen JDUTC consumer.
    pub fn days(self) -> f64 {
        self.0
    }
}

impl NaifEt {
    /// Accepts an explicitly declared ET input, rejecting NaN/infinity.
    pub fn from_seconds(seconds: f64) -> Result<Self, ContractError> {
        if seconds.is_finite() {
            Ok(Self(seconds))
        } else {
            Err(ContractError::NonFinite)
        }
    }
    /// TDB seconds past J2000, for a SPICE-compatible geometry interface.
    pub fn seconds(self) -> f64 {
        self.0
    }
    /// The calculation convention carried by this type.
    pub fn profile(self) -> CalculationProfile {
        CalculationProfile::NaifSpice
    }
}

/// Scientific model used to obtain an [`IauTdb`] result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScientificModel {
    /// Fairhead-Bretagnon geocentric model, as in ERFA with u=v=0.
    GeocentricFairheadBretagnon,
    /// The geocentric model plus ERFA observer-dependent diurnal terms.
    ObserverFairheadBretagnon,
}

/// Explicit scientific TDB seconds past J2000 on the TDB scale.
///
/// There is deliberately no `From<IauTdb>` implementation for [`NaifEt`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IauTdb {
    seconds: f64,
    model: ScientificModel,
}

impl IauTdb {
    /// TDB coordinate seconds; not an implicit NAIF ET conversion.
    pub fn seconds(self) -> f64 {
        self.seconds
    }
    /// Scientific model chosen explicitly by the caller.
    pub fn model(self) -> ScientificModel {
        self.model
    }
    /// Scientific convention; it never changes the default geometry context.
    pub fn profile(self) -> CalculationProfile {
        CalculationProfile::IauScientific
    }
}

/// Relevant LSK settings for reproducibility. These are not file identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NaifMetadata {
    /// First and last leap-entry dates. The last is not an expiry date.
    pub leap_entry_dates: (i64, i64),
    /// Historical source selected in the supplied table.
    pub leap_source: LeapSecondSource,
    /// Explicit future leap policy selected in the supplied table.
    pub leap_policy: LeapSecondPolicy,
    /// LSK values in order DELTA_T_A, K, EB, M0, M1.
    pub deltet: [f64; 5],
}

/// Default geometry time contract. The caller selects the LSK explicitly.
#[derive(Clone, Copy)]
pub struct TimeContext<'a> {
    lsk: &'a LeapSecondTable,
}

impl<'a> TimeContext<'a> {
    /// Selects NAIF ET, with no scientific-model fallback.
    pub fn new(lsk: &'a LeapSecondTable) -> Self {
        Self { lsk }
    }
    /// Native TTRANS ET-to-JDUTC, including its historical LSK convention.
    ///
    /// This explicit path bypasses integer Epoch and civil-calendar
    /// quantization. It requires IERS-only/TableDriven kernel settings;
    /// scientific SOFA and frozen future policies are separate choices.
    /// N0067 uses the first 140 entries, holds a formal JD during positive
    /// leap seconds, and assumes the first offset minus one before its
    /// first entry. This is not a scientific historical UTC realization.
    /// Existing `et_to_utc` retains its post-1972 coverage contract.
    pub fn et_to_spice_julian_utc(self, et: NaifEt) -> Result<NaifJulianUtc, ContractError> {
        if self.lsk.source() != LeapSecondSource::IersOnly
            || self.lsk.policy() != LeapSecondPolicy::TableDriven
        {
            return Err(ContractError::NaifKernelModelRequired);
        }
        naif::utc::julian_utc(et.seconds(), self.lsk).map(NaifJulianUtc)
    }
    /// The default calculation convention.
    pub fn profile(self) -> CalculationProfile {
        CalculationProfile::NaifSpice
    }
    /// Settings to record alongside the caller's source URL and file hash.
    pub fn metadata(self) -> NaifMetadata {
        NaifMetadata {
            leap_entry_dates: self.lsk.coverage(),
            leap_source: self.lsk.source(),
            leap_policy: self.lsk.policy(),
            deltet: [
                self.lsk.delta_t_a,
                self.lsk.k,
                self.lsk.eb,
                self.lsk.m0,
                self.lsk.m1,
            ],
        }
    }
    /// Converts a continuous TAI epoch to checked, model-specific ET.
    pub fn to_et(self, epoch: Epoch) -> Result<NaifEt, ContractError> {
        let tai = (epoch.sec() as i128 - 43_200) as f64 + epoch.nanos() as f64 * 1e-9;
        self.tai_seconds_to_et(tai)
    }

    // NAIF N0067 UNITIM evaluates the forward periodic term at TDT.
    // Preserve its operation order; the low-level legacy fixed point is
    // retained independently for existing callers. See ATTRIBUTION.md.
    fn tai_seconds_to_et(self, tai: f64) -> Result<NaifEt, ContractError> {
        let tt = tai + self.lsk.delta_t_a;
        NaifEt::from_seconds(naif::tt_to_et(tt, self.lsk))
    }
    /// Converts validated civil UTC to NAIF ET. Historical SOFA UTC stays
    /// in the explicit low-level SOFA API rather than changing this contract.
    pub fn utc_to_et(self, utc: &UtcDateTime, calendar: Calendar) -> Result<NaifEt, ContractError> {
        let jdn = calendar_to_jdn(utc.year, utc.month, utc.day, calendar)?;
        if jdn < JDN_1972 {
            return Err(ContractError::HistoricalUtcRequiresExplicitModel);
        }
        // Validate the civil label and LSK support, including removed seconds.
        convert::utc_to_tai(utc, calendar, self.lsk)?;
        // TTRANS keeps a pair of anchors (day-before/day-of) per LSK entry.
        // Its intermediate rounding is observable for fractional UTC inputs.
        // Do not replace this with a large absolute Epoch converted to f64.
        let mut previous_delta = self.lsk.entries()[0].delta_at as i64 - 1;
        let mut anchor = None;
        for (index, entry) in self.lsk.entries().iter().enumerate() {
            if !self.lsk.is_active_index(index) {
                break;
            }
            let formal = (entry.jdn as i128 - J2000_JDN as i128) * 86400 - 43200;
            if entry.jdn - 1 <= jdn {
                let tai = i64::try_from(formal - 86400 + previous_delta as i128)
                    .map_err(|_| ContractError::EpochOutOfRange)?;
                anchor = Some((entry.jdn - 1, tai));
            }
            if entry.jdn <= jdn {
                let tai = i64::try_from(formal + entry.delta_at as i128)
                    .map_err(|_| ContractError::EpochOutOfRange)?;
                anchor = Some((entry.jdn, tai));
            }
            previous_delta = entry.delta_at as i64;
        }
        let (day, tai_anchor) = anchor.ok_or(TimeError::UtcOutOfRange)?;
        let mut seconds = (utc.hour as i64 * 3600 + utc.minute as i64 * 60) as f64 + utc.second;
        seconds += (jdn - day) as f64 * 86400.0;
        self.tai_seconds_to_et(tai_anchor as f64 + seconds)
    }
    /// Checked inverse; finite ET may still exceed the integer epoch range.
    pub fn to_epoch(self, et: NaifEt) -> Result<Epoch, ContractError> {
        let tt = naif::et_to_tt(et.0, self.lsk);
        let tai = (tt - self.lsk.delta_t_a) + 43_200.0;
        if !tai.is_finite() {
            return Err(ContractError::NonFinite);
        }
        Epoch::checked_from_tai_seconds(tai).ok_or(ContractError::EpochOutOfRange)
    }
    /// Converts NAIF ET to UTC under the same explicit LSK contract.
    pub fn et_to_utc(self, et: NaifEt, calendar: Calendar) -> Result<UtcDateTime, ContractError> {
        let epoch = self.to_epoch(et)?;
        let jdn = convert::utc_jdn(epoch, self.lsk).ok_or(TimeError::UtcOutOfRange)?;
        if jdn < JDN_1972 {
            return Err(ContractError::HistoricalUtcRequiresExplicitModel);
        }
        if jdn > calendar_to_jdn(i32::MAX, 12, 31, calendar)? {
            return Err(ContractError::EpochOutOfRange);
        }
        let utc = convert::tai_to_utc(epoch, calendar, self.lsk)?;
        Ok(utc)
    }
}

/// Explicit scientific time realized through TT(TAI), not TT(BIPM).
///
/// No LSK constants, observer location or UT1 measurements are inferred.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScientificContext;

impl ScientificContext {
    /// Selects the scientific context explicitly.
    pub fn new() -> Self {
        Self
    }
    /// TT(TAI) seconds past J2000 TT, independent of LSK DELTET constants.
    pub fn tt_seconds(self, epoch: Epoch) -> Result<f64, ContractError> {
        let tt = Self::tt_epoch(epoch)?;
        Ok((tt.sec() as i128 - 43_200) as f64 + tt.nanos() as f64 * 1e-9)
    }

    fn tt_epoch(epoch: Epoch) -> Result<Epoch, ContractError> {
        epoch
            .checked_add(crate::Duration::from_parts(32, 184_000_000))
            .ok_or(ContractError::EpochOutOfRange)
    }

    fn tt_date(epoch: Epoch) -> Result<(f64, f64), ContractError> {
        let tt = Self::tt_epoch(epoch)?;
        let first = Epoch::from_parts((2_433_283 - 2_451_545) * 86400, 0);
        let last = Epoch::from_parts((2_469_808 - 2_451_545) * 86400, 0);
        if tt < first || tt > last {
            return Err(ContractError::Scientific(DtdbError::OutsideModelRange));
        }
        let days = tt.sec().div_euclid(86400);
        let seconds = tt.sec().rem_euclid(86400) as f64 + tt.nanos() as f64 * 1e-9;
        Ok((2_451_544.5 + days as f64, seconds / 86400.0))
    }
    /// Geocentric scientific TDB, checked against the model's stated date domain.
    pub fn to_tdb(self, epoch: Epoch) -> Result<IauTdb, ContractError> {
        let tt = self.tt_seconds(epoch)?;
        let (jd1, jd2) = Self::tt_date(epoch)?;
        let delta = dtdb_minus_tt_at_observer(jd1, jd2, 0.0, 0.0, 0.0, 0.0)?;
        Ok(IauTdb {
            seconds: tt + delta,
            model: ScientificModel::GeocentricFairheadBretagnon,
        })
    }
    /// Observer-dependent model. UT1 fraction is in [0,1), longitude is
    /// east-positive radians, and u/v are cylindrical distances in km.
    /// This is not a spacecraft proper-time conversion or Earth-frame solver.
    pub fn to_tdb_at_observer(
        self,
        epoch: Epoch,
        ut1_fraction: f64,
        longitude: f64,
        u_km: f64,
        v_km: f64,
    ) -> Result<IauTdb, ContractError> {
        let tt = self.tt_seconds(epoch)?;
        let (jd1, jd2) = Self::tt_date(epoch)?;
        let delta = dtdb_minus_tt_at_observer(jd1, jd2, ut1_fraction, longitude, u_km, v_km)?;
        Ok(IauTdb {
            seconds: tt + delta,
            model: ScientificModel::ObserverFairheadBretagnon,
        })
    }
}
