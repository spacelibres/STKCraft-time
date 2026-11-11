//! Time-scale identifiers.
//!
//! The variants are intentionally explicit. They are not interchangeable:
//! UTC has leap seconds, TAI is continuous, TT is offset from TAI by a
//! constant, ET is NAIF's TDB-based ephemeris time, and GPS has its own
//! epoch and constant offset from TAI.

/// A supported time scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeScale {
    /// Coordinated Universal Time, a civil representation with leap
    /// seconds.
    Utc,
    /// International Atomic Time, continuous.
    Tai,
    /// Terrestrial Time: `TAI + 32.184 s`.
    Tt,
    /// NAIF ephemeris time: TDB seconds past the J2000 epoch.
    Et,
    /// GPS time: `TAI - 19 s`.
    Gps,
    /// Universal Time UT1, the Earth-rotation scale.
    ///
    /// UT1 is *not* derived from the leap-second kernel. Converting to
    /// or from UT1 requires an IERS Earth Orientation Parameters table
    /// (see [`crate::eop::EopTable`]) that measures the actual rotation
    /// of the Earth. UT1 is therefore kept separate from the dynamical
    /// scales TT and ET: it is an Earth-rotation time scale, not a
    /// uniform atomic time scale.
    Ut1,
    /// Geocentric Coordinate Time; see the exact affine relation in [`crate::coord`].
    ///
    /// See [`crate::coord`]. This is a coordinate scale, not a clock.
    Tcg,
    /// Barycentric Coordinate Time; see the exact affine relation in [`crate::coord`].
    ///
    /// See [`crate::coord`]. This is a coordinate scale, not a clock.
    Tcb,
}
