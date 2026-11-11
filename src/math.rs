//! Floating-point helpers that work with and without `std`.
//!
//! With the `std` feature (the default) the standard library methods are
//! used. Without it, the same operations come from `libm`.

#[cfg(feature = "std")]
pub(crate) fn sin(x: f64) -> f64 {
    x.sin()
}
#[cfg(not(feature = "std"))]
pub(crate) fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[cfg(feature = "std")]
pub(crate) fn floor(x: f64) -> f64 {
    x.floor()
}
#[cfg(not(feature = "std"))]
pub(crate) fn floor(x: f64) -> f64 {
    libm::floor(x)
}

#[cfg(feature = "std")]
pub(crate) fn round(x: f64) -> f64 {
    x.round()
}
#[cfg(not(feature = "std"))]
pub(crate) fn round(x: f64) -> f64 {
    libm::round(x)
}

pub(crate) fn abs(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub(crate) fn to_radians(degrees: f64) -> f64 {
    degrees * core::f64::consts::PI / 180.0
}
