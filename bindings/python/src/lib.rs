use c::contract::{NaifEt, ScientificContext, TimeContext};
use c::{Calendar, Duration, EopTable, Epoch, LeapSecondTable, SclkKernel, UtcDateTime};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use stkcraft_time as c;
type UtcFields = (i32, u8, u8, u8, u8, f64);
fn err(e: impl std::fmt::Debug) -> PyErr {
    PyValueError::new_err(format!("{e:?}"))
}

fn native_sclk_error(error: c::sclk::NativeSclkError) -> PyErr {
    match error {
        c::sclk::NativeSclkError::String(error) | c::sclk::NativeSclkError::Native(error) => {
            PyValueError::new_err(error.spice_short())
        }
        c::sclk::NativeSclkError::Time(error) => err(error),
    }
}
fn indexed(e: PyErr, index: usize) -> PyErr {
    Python::attach(|py| {
        let _ = e.value(py).setattr("index", index);
    });
    e
}
fn overflow() -> PyErr {
    err("Overflow")
}
// Civil integer UTC uses checked i32 adjacent offsets. Preserve inactive
// frozen entries and the pre-first-entry SOFA route, just as the BDT facade.
// This is an arithmetic boundary check, not physical leap-table validation.
fn check_active_kernel_jumps(l: &LeapSecondTable) -> PyResult<()> {
    for pair in l.entries().windows(2) {
        let active = match l.policy() {
            c::LeapSecondPolicy::TableDriven => true,
            c::LeapSecondPolicy::FrozenAfter { jdn } => pair[1].jdn <= jdn,
        };
        if active {
            pair[1]
                .delta_at
                .checked_sub(pair[0].delta_at)
                .ok_or_else(overflow)?;
        }
    }
    Ok(())
}
fn check_utc_kernel(u: &UtcDateTime, calendar: Calendar, l: &LeapSecondTable) -> PyResult<()> {
    let day = c::calendar::calendar_to_jdn(u.year, u.month, u.day, calendar).map_err(err)?;
    if day >= c::leap::JDN_1972 {
        check_active_kernel_jumps(l)?;
    }
    Ok(())
}
fn check_epoch_kernel(e: Epoch, l: &LeapSecondTable) -> PyResult<()> {
    if let Some(first) = l.entries().first() {
        let start =
            (first.jdn as i128 - c::calendar::J2000_JDN as i128) * 86400 + first.delta_at as i128;
        if e.sec() as i128 >= start {
            check_active_kernel_jumps(l)?;
        }
    }
    Ok(())
}
// Guard only the representability boundary; scientific calculations stay in core.
fn civil_pre(e: Epoch, calendar: Calendar) -> PyResult<()> {
    let j = e.sec().div_euclid(86400) + c::calendar::J2000_JDN;
    let lo = c::calendar::calendar_to_jdn(i32::MIN, 1, 1, calendar).map_err(err)?;
    let hi = c::calendar::calendar_to_jdn(i32::MAX, 12, 31, calendar).map_err(err)?;
    if j < lo - 25000 || j > hi + 25000 {
        return Err(overflow());
    }
    Ok(())
}
fn civil_post(e: Epoch, u: UtcDateTime, calendar: Calendar) -> PyResult<UtcDateTime> {
    let j = c::calendar::calendar_to_jdn(u.year, u.month, u.day, calendar).map_err(err)?;
    let tai_j = e.sec().div_euclid(86400) + c::calendar::J2000_JDN;
    // Whole UTC offsets are i32. A wrapped i32 year differs by billions of years.
    if (j as i128 - tai_j as i128).abs() > 25000 {
        return Err(overflow());
    }
    Ok(u)
}
fn civil(e: Epoch, calendar: Calendar, l: &LeapSecondTable) -> PyResult<UtcDateTime> {
    civil_pre(e, calendar)?;
    check_epoch_kernel(e, l)?;
    let u = c::convert::tai_to_utc(e, calendar, l).map_err(err)?;
    civil_post(e, u, calendar)
}

fn cal(s: &str) -> PyResult<Calendar> {
    match s {
        "Gregorian" => Ok(Calendar::Gregorian),
        "Julian" => Ok(Calendar::Julian),
        "Mixed" => Ok(Calendar::Mixed),
        _ => Err(err("InvalidCalendar")),
    }
}
fn finite(x: f64) -> PyResult<f64> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(err("NonFinite"))
    }
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeEpoch(Epoch);
#[pymethods]
impl NativeEpoch {
    #[new]
    fn new(sec: i64, nanos: i64) -> PyResult<Self> {
        Epoch::checked_from_parts(sec, nanos)
            .map(Self)
            .ok_or_else(overflow)
    }
    #[staticmethod]
    fn from_seconds(x: f64) -> PyResult<Self> {
        Epoch::checked_from_tai_seconds(x)
            .map(Self)
            .ok_or_else(overflow)
    }
    fn parts(&self) -> (i64, u32) {
        (self.0.sec(), self.0.nanos())
    }
    fn seconds(&self) -> f64 {
        self.0.tai_seconds()
    }
    fn add(&self, d: &NativeDuration) -> PyResult<Self> {
        self.0.checked_add(d.0).map(Self).ok_or_else(overflow)
    }
    fn sub(&self, d: &NativeDuration) -> PyResult<Self> {
        self.0.checked_sub(d.0).map(Self).ok_or_else(overflow)
    }
    fn difference(&self, e: &Self) -> PyResult<NativeDuration> {
        let delta = (self.0.sec() as i128 - e.0.sec() as i128) * 1_000_000_000
            + self.0.nanos() as i128
            - e.0.nanos() as i128;
        Duration::from_nanoseconds(delta).ok_or_else(overflow)?;
        Ok(NativeDuration(self.0 - e.0))
    }
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeDuration(Duration);
#[pymethods]
impl NativeDuration {
    #[new]
    fn new(sec: i64, nanos: i64) -> PyResult<Self> {
        Duration::checked_from_parts(sec, nanos)
            .map(Self)
            .ok_or_else(overflow)
    }
    #[staticmethod]
    fn from_ns(ns: i128) -> PyResult<Self> {
        Duration::from_nanoseconds(ns)
            .map(Self)
            .ok_or_else(overflow)
    }
    #[staticmethod]
    fn from_seconds(x: f64) -> PyResult<Self> {
        Duration::checked_from_seconds_f64(x)
            .map(Self)
            .ok_or_else(overflow)
    }
    fn parts(&self) -> (i64, i32) {
        (self.0.sec(), self.0.nanos())
    }
    fn total_ns(&self) -> i128 {
        self.0.total_nanoseconds()
    }
    fn seconds(&self) -> f64 {
        self.0.seconds_f64()
    }
    fn signum(&self) -> i8 {
        self.0.signum()
    }
    fn neg(&self) -> PyResult<Self> {
        self.0.checked_neg().map(Self).ok_or_else(overflow)
    }
    fn add(&self, d: &Self) -> PyResult<Self> {
        self.0.checked_add(d.0).map(Self).ok_or_else(overflow)
    }
    fn sub(&self, d: &Self) -> PyResult<Self> {
        self.0.checked_sub(d.0).map(Self).ok_or_else(overflow)
    }
    fn mul(&self, n: i64) -> PyResult<Self> {
        self.0.checked_mul_i64(n).map(Self).ok_or_else(overflow)
    }
    fn div(&self, n: i64) -> PyResult<Self> {
        if n == 0 {
            return Err(err("DivisionByZero"));
        }
        Duration::from_nanoseconds(self.0.total_nanoseconds() / n as i128)
            .map(Self)
            .ok_or_else(overflow)
    }
    fn quantize(&self, step: &Self, mode: &str) -> PyResult<Self> {
        if step.0 <= Duration::ZERO {
            return Err(err("InvalidStep"));
        }
        let total = self.0.total_nanoseconds();
        let unit = step.0.total_nanoseconds();
        let q = total.div_euclid(unit);
        let r = total.rem_euclid(unit);
        let candidate = match mode {
            "floor" => q * unit,
            "ceil" => (q + i128::from(r != 0)) * unit,
            _ => (q + i128::from(r * 2 >= unit)) * unit,
        };
        Duration::from_nanoseconds(candidate).ok_or_else(overflow)?;
        Ok(Self(match mode {
            "floor" => self.0.floor(step.0),
            "ceil" => self.0.ceil(step.0),
            _ => self.0.round(step.0),
        }))
    }
    fn display(&self) -> String {
        self.0.to_string()
    }
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeUtc(UtcDateTime);
#[pymethods]
impl NativeUtc {
    #[new]
    fn new(y: i32, m: u8, d: u8, h: u8, min: u8, s: f64) -> PyResult<Self> {
        finite(s)?;
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return Err(err(c::TimeError::InvalidDate {
                year: y,
                month: m,
                day: d,
            }));
        }
        if h > 23 || min > 59 {
            return Err(err(c::TimeError::InvalidTime {
                hour: h,
                minute: min,
            }));
        }
        if !(0.0..61.0).contains(&s) {
            return Err(err(c::TimeError::InvalidSecond));
        }
        Ok(Self(UtcDateTime::new(y, m, d, h, min, s)))
    }
    fn fields(&self) -> (i32, u8, u8, u8, u8, f64) {
        let d = &self.0;
        (d.year, d.month, d.day, d.hour, d.minute, d.second)
    }
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeLsk(LeapSecondTable);
#[pymethods]
impl NativeLsk {
    #[new]
    fn new(text: Option<&str>) -> PyResult<Self> {
        match text {
            Some(t) => LeapSecondTable::from_lsk(t),
            None => LeapSecondTable::builtin(),
        }
        .map(Self)
        .map_err(err)
    }
    #[staticmethod]
    fn from_entries(entries: Vec<(i64, i32)>) -> PyResult<Self> {
        for &(j, d) in &entries {
            let v = (j as i128 - c::calendar::J2000_JDN as i128) * 86400 + d as i128;
            if v < i64::MIN as i128 || v > i64::MAX as i128 {
                return Err(overflow());
            }
        }
        LeapSecondTable::new(
            entries
                .into_iter()
                .map(|(jdn, delta_at)| c::LeapEntry { jdn, delta_at })
                .collect(),
        )
        .map(Self)
        .map_err(err)
    }
    fn entries(&self) -> Vec<(i64, i32)> {
        self.0
            .entries()
            .iter()
            .map(|e| (e.jdn, e.delta_at))
            .collect()
    }
    fn coverage(&self) -> (i64, i64) {
        self.0.coverage()
    }
    fn constants(&self) -> (f64, f64, f64, f64, f64) {
        let l = &self.0;
        (l.delta_t_a, l.k, l.eb, l.m0, l.m1)
    }
    fn with_constants(&self, a: f64, k: f64, eb: f64, m0: f64, m1: f64) -> PyResult<Self> {
        for n in [a, k, eb, m0, m1] {
            finite(n)?;
        }
        Ok(Self(self.0.clone().with_constants(a, k, eb, m0, m1)))
    }
    fn source(&self) -> String {
        format!("{:?}", self.0.source())
    }
    fn policy(&self) -> Option<i64> {
        match self.0.policy() {
            c::LeapSecondPolicy::TableDriven => None,
            c::LeapSecondPolicy::FrozenAfter { jdn } => Some(jdn),
        }
    }
    fn with_source(&self, s: &str) -> PyResult<Self> {
        let source = match s {
            "IersOnly" => c::LeapSecondSource::IersOnly,
            "Sofa" => c::LeapSecondSource::Sofa,
            _ => return Err(err("InvalidSource")),
        };
        Ok(Self(self.0.clone().with_source(source)))
    }
    fn with_policy(&self, jdn: Option<i64>) -> Self {
        Self(self.0.clone().with_policy(match jdn {
            Some(jdn) => c::LeapSecondPolicy::FrozenAfter { jdn },
            None => c::LeapSecondPolicy::TableDriven,
        }))
    }
    fn delta_at(&self, jdn: i64, f: f64) -> PyResult<Option<f64>> {
        finite(f)?;
        Ok(self.0.delta_at_f64(jdn, f))
    }
    fn delta_at_for_jdn(&self, jdn: i64) -> Option<i32> {
        self.0.delta_at_for_jdn(jdn)
    }
    fn to_et(&self, e: &NativeEpoch) -> PyResult<f64> {
        TimeContext::new(&self.0)
            .to_et(e.0)
            .map(|v| v.seconds())
            .map_err(err)
    }
    fn utc_to_et(&self, u: &NativeUtc, calendar: &str) -> PyResult<f64> {
        check_utc_kernel(&u.0, cal(calendar)?, &self.0)?;
        TimeContext::new(&self.0)
            .utc_to_et(&u.0, cal(calendar)?)
            .map(|v| v.seconds())
            .map_err(err)
    }
    fn to_epoch(&self, et: f64) -> PyResult<NativeEpoch> {
        TimeContext::new(&self.0)
            .to_epoch(NaifEt::from_seconds(et).map_err(err)?)
            .map(NativeEpoch)
            .map_err(err)
    }
    fn et_to_utc(&self, et: f64, calendar: &str) -> PyResult<NativeUtc> {
        {
            let ctx = TimeContext::new(&self.0);
            let et = NaifEt::from_seconds(et).map_err(err)?;
            let e = ctx.to_epoch(et).map_err(err)?;
            let calendar = cal(calendar)?;
            civil_pre(e, calendar)?;
            check_epoch_kernel(e, &self.0)?;
            let u = ctx.et_to_utc(et, calendar).map_err(err)?;
            civil_post(e, u, calendar).map(NativeUtc)
        }
    }
    fn julian(&self, et: f64) -> PyResult<f64> {
        TimeContext::new(&self.0)
            .et_to_spice_julian_utc(NaifEt::from_seconds(et).map_err(err)?)
            .map(|v| v.days())
            .map_err(err)
    }
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeEop(EopTable);
#[pymethods]
impl NativeEop {
    #[new]
    fn new(samples: Vec<(f64, f64, bool)>) -> PyResult<Self> {
        for &(mjd, _, _) in &samples {
            if !mjd.is_finite() || mjd.abs() > 700_000_000_000.0 {
                return Err(overflow());
            }
        }
        EopTable::new(
            samples
                .into_iter()
                .map(|(mjd, ut1_minus_utc, observed)| c::EopSample {
                    mjd,
                    ut1_minus_utc,
                    observed,
                })
                .collect(),
        )
        .map(Self)
        .map_err(err)
    }
    #[staticmethod]
    fn parse(text: &str, series: &str) -> PyResult<Self> {
        let s = match series {
            "BulletinA" => c::EopSeries::BulletinA,
            "BulletinB" => c::EopSeries::BulletinB,
            "FinalPreferred" => c::EopSeries::FinalPreferred,
            _ => return Err(err("InvalidSeries")),
        };
        let table = EopTable::from_finals2000a_series(text, s).map_err(err)?;
        for sample in table.samples() {
            if sample.mjd.abs() > 700_000_000_000.0 {
                return Err(overflow());
            }
        }
        Ok(Self(table))
    }
    fn observed_only(&self) -> PyResult<Self> {
        self.0.observed_only().map(Self).map_err(err)
    }
    fn range(&self) -> (f64, f64) {
        self.0.range()
    }
    fn samples(&self) -> Vec<(f64, f64, bool)> {
        self.0
            .samples()
            .iter()
            .map(|s| (s.mjd, s.ut1_minus_utc, s.observed))
            .collect()
    }
    fn dut1(&self, mjd: f64) -> PyResult<f64> {
        finite(mjd)?;
        self.0.ut1_minus_utc(mjd).map_err(err).and_then(finite)
    }
    fn convert(&self, l: &NativeLsk, e: &NativeEpoch, mode: &str) -> PyResult<f64> {
        check_epoch_kernel(e.0, &l.0)?;
        match mode {
            "offset" => c::convert::ut1_minus_utc(e.0, &l.0, &self.0),
            _ => c::convert::tai_to_ut1(e.0, &l.0, &self.0),
        }
        .map_err(err)
        .and_then(finite)
    }
    fn to_tai(&self, l: &NativeLsk, x: f64) -> PyResult<NativeEpoch> {
        finite(x)?;
        Epoch::checked_from_tai_seconds(x).ok_or_else(overflow)?;
        if self.0.samples().iter().any(|sample| {
            let day = c::calendar::J2000_JDN + (sample.mjd - 51544.0).round() as i64;
            l.0.delta_at_for_jdn(day).is_some()
        }) {
            check_active_kernel_jumps(&l.0)?;
        }
        for sample in self.0.samples() {
            for delta in [-2147483648.0, 2147483648.0] {
                Epoch::checked_from_tai_seconds(x - sample.ut1_minus_utc + delta)
                    .ok_or_else(overflow)?;
            }
        }
        c::convert::ut1_to_tai(x, &l.0, &self.0)
            .map(NativeEpoch)
            .map_err(err)
    }
}
#[pyclass]
#[derive(Default)]
struct NativeSclkStringContext(c::sclk::SclkStringContext);
#[pymethods]
impl NativeSclkStringContext {
    #[new]
    fn new() -> Self {
        Self::default()
    }
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeSclk(SclkKernel);
#[pymethods]
impl NativeSclk {
    #[new]
    fn new(text: &str, id: i32) -> PyResult<Self> {
        SclkKernel::parse(text, id).map(Self).map_err(err)
    }
    fn info(&self) -> (i32, i32, String, usize, Vec<i64>, Vec<i64>, f64) {
        (
            self.0.spacecraft_id(),
            self.0.data_type(),
            format!("{:?}", self.0.time_system()),
            self.0.n_fields(),
            self.0.moduli().to_vec(),
            self.0.offsets().to_vec(),
            self.0.ticks_per_msb(),
        )
    }
    fn coefficients(&self) -> Vec<(f64, f64, f64)> {
        self.0
            .coefficients()
            .iter()
            .map(|x| (x.ticks, x.seconds, x.rate))
            .collect()
    }
    fn partitions(&self) -> (Vec<f64>, Vec<f64>) {
        let (a, b) = self.0.partitions();
        (a.to_vec(), b.to_vec())
    }
    fn ticks_from_fields(&self, f: Vec<i64>) -> PyResult<f64> {
        self.0.ticks_from_fields(&f).map_err(err).and_then(finite)
    }
    fn fields_from_ticks(&self, x: f64) -> PyResult<Vec<i64>> {
        finite(x)?;
        let (starts, ends) = self.0.partitions();
        if !(starts.is_empty() && ends.is_empty() || starts.len() == 1 && starts[0] == 0.0) {
            return self.0.fields_from_ticks(x).map_err(err);
        }
        let rounded = x.round();
        if x < 0.0 || (x - rounded).abs() > 1e-6 {
            return self.0.fields_from_ticks(x).map_err(err);
        }
        if rounded >= i64::MAX as f64 {
            return Err(overflow());
        }
        // Validation-only mirror of the core's reverse modulus decomposition.
        let mut remainder = rounded as i64;
        for index in (1..self.0.n_fields()).rev() {
            let modulus = self.0.moduli()[index];
            (remainder % modulus)
                .checked_add(self.0.offsets()[index])
                .ok_or_else(overflow)?;
            remainder /= modulus;
        }
        if remainder < self.0.moduli()[0] {
            remainder
                .checked_add(self.0.offsets()[0])
                .ok_or_else(overflow)?;
        }
        self.0.fields_from_ticks(x).map_err(err)
    }
    fn parse_string(&self, s: &str) -> PyResult<Vec<i64>> {
        self.0.parse_string(s).map_err(err)
    }
    fn format_string(&self, f: Vec<i64>) -> PyResult<String> {
        self.0.format_string(&f).map_err(err)
    }
    fn decode_native_sclk(&self, ticks: f64) -> PyResult<String> {
        finite(ticks)?;
        self.0
            .decode_native_sclk(ticks)
            .map_err(|e| PyValueError::new_err(e.spice_short()))
    }
    fn encode_native_sclk(&self, clock: &str) -> PyResult<f64> {
        self.0
            .encode_native_sclk(clock)
            .map_err(|e| PyValueError::new_err(e.spice_short()))
    }
    fn encode_native_sclk_with_context(
        &self,
        clock: &str,
        context: &mut NativeSclkStringContext,
    ) -> PyResult<f64> {
        self.0
            .encode_native_sclk_with_context(clock, &mut context.0)
            .map_err(|e| PyValueError::new_err(e.spice_short()))
    }
    fn clock_to_et(&self, clock: &str, lsk: &NativeLsk) -> PyResult<f64> {
        use c::sclk::SclkMap;
        c::sclk::SclkClock {
            kernel: &self.0,
            lsk: &lsk.0,
        }
        .sclk_to_et(clock)
        .map_err(err)
        .and_then(finite)
    }
    fn native_clock_to_et(&self, clock: &str, lsk: &NativeLsk) -> PyResult<f64> {
        c::sclk::SclkClock {
            kernel: &self.0,
            lsk: &lsk.0,
        }
        .native_sclk_to_et(clock)
        .map_err(native_sclk_error)
        .and_then(finite)
    }
    fn native_clock_to_et_with_context(
        &self,
        clock: &str,
        lsk: &NativeLsk,
        context: &mut NativeSclkStringContext,
    ) -> PyResult<f64> {
        c::sclk::SclkClock {
            kernel: &self.0,
            lsk: &lsk.0,
        }
        .native_sclk_to_et_with_context(clock, &mut context.0)
        .map_err(native_sclk_error)
        .and_then(finite)
    }
    fn native_clock_from_et(&self, et: f64, lsk: &NativeLsk) -> PyResult<String> {
        finite(et)?;
        c::sclk::SclkClock {
            kernel: &self.0,
            lsk: &lsk.0,
        }
        .native_et_to_sclk(et)
        .map_err(native_sclk_error)
    }
    fn clock_from_et(&self, et: f64, lsk: &NativeLsk) -> PyResult<String> {
        use c::sclk::SclkMap;
        finite(et)?;
        c::sclk::SclkClock {
            kernel: &self.0,
            lsk: &lsk.0,
        }
        .et_to_sclk(et)
        .map_err(err)
    }
    fn parallel(&self, x: f64, inverse: bool) -> PyResult<f64> {
        finite(x)?;
        if inverse {
            self.0.ticks_from_parallel(x)
        } else {
            self.0.parallel_seconds(x)
        }
        .map_err(err)
        .and_then(finite)
    }
    fn convert(&self, x: f64, l: &NativeLsk, inverse: bool) -> PyResult<f64> {
        finite(x)?;
        if inverse {
            self.0.et_to_ticks(x, &l.0)
        } else {
            self.0.ticks_to_et(x, &l.0)
        }
        .map_err(err)
        .and_then(finite)
    }
    fn to_tai(&self, x: f64, l: &NativeLsk) -> PyResult<NativeEpoch> {
        finite(x)?;
        let parallel = self.0.parallel_seconds(x).map_err(err)?;
        let scale = match self.0.time_system() {
            c::SclkTimeSystem::Tdb => "et",
            c::SclkTimeSystem::Tdt => "tt",
        };
        legacy_from(parallel, l, scale)?;
        self.0.ticks_to_tai(x, &l.0).map(NativeEpoch).map_err(err)
    }
    fn tai_to_ticks(&self, e: &NativeEpoch, l: &NativeLsk) -> PyResult<f64> {
        self.0.tai_to_ticks(e.0, &l.0).map_err(err)
    }
}
#[pyclass]
struct NativeSeries(c::TimeSeries);
#[pymethods]
impl NativeSeries {
    #[new]
    fn new(a: &NativeEpoch, b: &NativeEpoch, d: &NativeDuration, inclusive: bool) -> Self {
        Self(if inclusive {
            c::TimeSeries::inclusive(a.0, b.0, d.0)
        } else {
            c::TimeSeries::exclusive(a.0, b.0, d.0)
        })
    }
    fn next(&mut self) -> Option<NativeEpoch> {
        self.0.next().map(NativeEpoch)
    }
}
#[pyfunction]
fn scientific(e: &NativeEpoch, observer: Option<(f64, f64, f64, f64)>) -> PyResult<(f64, String)> {
    let ctx = ScientificContext::new();
    let t = if let Some((f, l, u, v)) = observer {
        ctx.to_tdb_at_observer(e.0, f, l, u, v)
    } else {
        ctx.to_tdb(e.0)
    }
    .map_err(err)?;
    Ok((t.seconds(), format!("{:?}", t.model())))
}
#[pyfunction]
fn scientific_tt(e: &NativeEpoch) -> PyResult<f64> {
    ScientificContext::new().tt_seconds(e.0).map_err(err)
}
#[pyfunction]
fn utc_tai(u: &NativeUtc, l: &NativeLsk, calendar: &str) -> PyResult<NativeEpoch> {
    check_utc_kernel(&u.0, cal(calendar)?, &l.0)?;
    c::convert::utc_to_tai(&u.0, cal(calendar)?, &l.0)
        .map(NativeEpoch)
        .map_err(err)
}
#[pyfunction]
fn tai_utc(e: &NativeEpoch, l: &NativeLsk, calendar: &str) -> PyResult<NativeUtc> {
    civil(e.0, cal(calendar)?, &l.0).map(NativeUtc)
}
#[pyfunction]
fn kernel_date(text: &str) -> PyResult<(f64, UtcFields)> {
    let d = c::parse_kernel_date(text).map_err(err)?;
    Ok((
        d.seconds_past_j2000,
        (d.year, d.month, d.day, d.hour, d.minute, d.second),
    ))
}
#[pyfunction]
fn format_utc(
    u: &NativeUtc,
    kind: &str,
    precision: usize,
    calendar: &str,
    l: Option<&NativeLsk>,
    pattern: &str,
) -> PyResult<String> {
    if precision > 18 {
        return Err(err("InvalidPrecision"));
    }
    if u.0.year == i32::MAX
        && u.0.month == 12
        && u.0.day == 31
        && u.0.hour == 23
        && u.0.minute == 59
        && u.0.second >= 59.0
    {
        return Err(overflow());
    }
    let mut o = c::format::FormatOptions::new(precision).with_calendar(cal(calendar)?);
    if let Some(l) = l {
        check_utc_kernel(&u.0, cal(calendar)?, &l.0)?;
        o = o.with_leap_seconds(&l.0);
    }
    Ok(match kind {
        "iso8601" => c::format::iso8601_with(&u.0, &o),
        "rfc3339" => c::format::rfc3339_with(&u.0, &o),
        "spice_calendar" => c::format::spice_calendar_with(&u.0, &o),
        "rfc2822" => c::format::rfc2822(&u.0),
        "strftime" => c::format::strftime_with(&u.0, pattern, &o).map_err(err)?,
        _ => return Err(err("InvalidFormat")),
    })
}
#[pyfunction]
fn legacy_to(e: &NativeEpoch, l: &NativeLsk, scale: &str) -> PyResult<f64> {
    finite(match scale {
        "et" => c::convert::tai_to_et(e.0, &l.0),
        "tt" => c::convert::tai_to_tt(e.0, &l.0),
        "gps" => c::convert::tai_to_gps(e.0),
        "tcg" => c::coord::tai_to_tcg(e.0, &l.0),
        "tcb" => c::coord::tai_to_tcb(e.0, &l.0),
        "tdb" => c::tdb::tai_to_tdb(e.0, &l.0),
        "tdb_iau" => c::dtdb::tai_to_tdb_iau(e.0, &l.0),
        "tcg_minus_tt" => c::coord::tcg_minus_tt(e.0, &l.0),
        _ => return Err(err("InvalidScale")),
    })
}
// VALIDATION ONLY, tied to the exact =0.2.0 std core dependency. These
// discarded f64 intermediates mirror convert/tdb/dtdb/coord inverse order to
// check the argument before their infallible Epoch constructor. Every returned
// scientific result still comes from the original public core function below.
fn preflight_tdb_tt(tdb: f64, iau: bool) -> f64 {
    let mut tt = tdb;
    for _ in 0..4 {
        let jd_tt = 2451545.0 + tt / 86400.0;
        let delta = if iau {
            c::dtdb::dtdb_minus_tt(jd_tt)
        } else {
            c::tdb::tdb_minus_tt_seconds(jd_tt)
        };
        let next = tdb - delta;
        if (next - tt).abs() < 1e-12 {
            tt = next;
            break;
        }
        tt = next;
    }
    tt
}
fn preflight_legacy_inverse(x: f64, l: &LeapSecondTable, scale: &str) -> PyResult<()> {
    let tai = match scale {
        "et" => {
            let mean_anomaly = l.m0 + l.m1 * x;
            let eccentric_anomaly = mean_anomaly + l.eb * mean_anomaly.sin();
            x - l.delta_t_a + 43200.0 - l.k * eccentric_anomaly.sin()
        }
        "tt" => x - l.delta_t_a + 43200.0,
        "gps" => {
            let origin = (c::calendar::GPS_EPOCH_JDN - c::calendar::J2000_JDN) * 86400 + 19;
            x + origin as f64
        }
        "tdb" => preflight_tdb_tt(x, false) - l.delta_t_a + 43200.0,
        "tdb_iau" => preflight_tdb_tt(x, true) - l.delta_t_a + 43200.0,
        "tcg" => {
            let rate = c::coord::L_G / (1.0 - c::coord::L_G);
            let offset = (2451545.0 - 2443144.5003725) * rate * 86400.0;
            let tt = (x - offset) / (1.0 + rate);
            tt - l.delta_t_a + 43200.0
        }
        "tcb" => {
            let rate = c::coord::L_B / (1.0 - c::coord::L_B);
            let offset =
                (2451545.0 - 2443144.5003725) * rate * 86400.0 - c::coord::TDB0 * (1.0 + rate);
            let tdb = (x - offset) / (1.0 + rate);
            preflight_tdb_tt(tdb, true) - l.delta_t_a + 43200.0
        }
        _ => return Err(err("InvalidScale")),
    };
    Epoch::checked_from_tai_seconds(tai).ok_or_else(overflow)?;
    Ok(())
}
#[pyfunction]
fn legacy_from(x: f64, l: &NativeLsk, scale: &str) -> PyResult<NativeEpoch> {
    finite(x)?;
    preflight_legacy_inverse(x, &l.0, scale)?;
    let e = match scale {
        "et" => c::convert::et_to_tai(x, &l.0),
        "tt" => c::convert::tt_to_tai(x, &l.0),
        "gps" => c::convert::gps_to_tai(x),
        "tcg" => c::coord::tcg_to_tai(x, &l.0),
        "tcb" => c::coord::tcb_to_tai(x, &l.0),
        "tdb" => c::tdb::tdb_to_tai(x, &l.0),
        "tdb_iau" => c::dtdb::tdb_iau_to_tai(x, &l.0),
        _ => return Err(err("InvalidScale")),
    };
    Ok(NativeEpoch(e))
}

#[pyfunction]
fn model(
    name: &str,
    x: f64,
    y: f64,
    observer: Option<(f64, f64, f64, f64)>,
) -> PyResult<Option<f64>> {
    finite(x)?;
    finite(y)?;
    let value = match name {
        "jd_to_mjd" => c::calendar::jd_to_mjd(x),
        "mjd_to_jd" => c::calendar::mjd_to_jd(x),
        "sofa_delta_at" => return Ok(c::leap::sofa_delta_at(x)),
        "tcb_minus_tdb" => c::coord::tcb_minus_tdb(x),
        "tdb_minus_tt_seconds" => c::tdb::tdb_minus_tt_seconds(x),
        "dtdb_minus_tt" => c::dtdb::dtdb_minus_tt(x),
        "dtdb_minus_tt_split" => c::dtdb::dtdb_minus_tt_split(x, y),
        "dtdb_minus_tt_at_observer" => {
            let (f, l, u, v) = observer.ok_or_else(|| err("MissingObserver"))?;
            c::dtdb::dtdb_minus_tt_at_observer(x, y, f, l, u, v).map_err(err)?
        }
        _ => return Err(err("InvalidModel")),
    };
    Ok(Some(finite(value)?))
}
#[pyfunction]
fn calendar_to_jdn(y: i32, m: u8, d: u8, calendar: &str) -> PyResult<i64> {
    c::calendar::calendar_to_jdn(y, m, d, cal(calendar)?).map_err(err)
}
#[pyfunction]
fn calendar_to_jd(u: &NativeUtc, calendar: &str) -> PyResult<f64> {
    let d = &u.0;
    c::calendar::calendar_to_jd(
        d.year,
        d.month,
        d.day,
        d.hour,
        d.minute,
        d.second,
        cal(calendar)?,
    )
    .map_err(err)
}
#[pyfunction]
fn jdn_to_calendar(j: i64, calendar: &str) -> PyResult<(i32, u8, u8)> {
    if !(-700_000_000_000..=700_000_000_000).contains(&j) {
        return Err(overflow());
    }
    Ok(c::calendar::jdn_to_calendar(j, cal(calendar)?))
}
#[pyfunction]
fn jd_to_calendar(j: f64) -> PyResult<(i32, u8, u8, u8, u8, f64)> {
    finite(j)?;
    if j.abs() > 700_000_000_000.0 {
        return Err(overflow());
    }
    Ok(c::format::jd_to_calendar(j))
}
#[pyfunction]
fn days_in_month(y: i32, m: u8, calendar: &str) -> PyResult<Option<u8>> {
    Ok(c::calendar::days_in_month(y, m, cal(calendar)?))
}
#[pyfunction]
fn leap_year(y: i32, julian: bool) -> bool {
    if julian {
        c::calendar::is_julian_leap(y)
    } else {
        c::calendar::is_gregorian_leap(y)
    }
}
#[pyfunction]
fn etcal(x: f64, p: usize) -> PyResult<String> {
    finite(x)?;
    if x.abs() > 1e16 || p > 18 {
        return Err(overflow());
    }
    Ok(c::format::etcal(x, p))
}
#[pyfunction]
fn validate_format(pattern: &str) -> PyResult<()> {
    c::format::Format::parse(pattern).map(|_| ()).map_err(err)
}
#[pyfunction]
fn raw_julian(
    entries: Vec<(i64, i32)>,
    constants: (f64, f64, f64, f64, f64),
    et: f64,
) -> PyResult<f64> {
    let entries: Vec<_> = entries
        .into_iter()
        .map(|(jdn, delta_at)| c::LeapEntry { jdn, delta_at })
        .collect();
    let (a, k, eb, m0, m1) = constants;
    let ctx = c::contract::NaifUtcContext::from_raw_parts(
        &entries,
        c::contract::NaifDeltet {
            delta_t_a: a,
            k,
            eb,
            m0,
            m1,
        },
    )
    .map_err(err)?;
    ctx.et_to_spice_julian_utc(NaifEt::from_seconds(et).map_err(err)?)
        .map(|v| v.days())
        .map_err(err)
}
#[pyfunction]
fn wgc_gps_week_tow(seconds: f64, lsk: &NativeLsk) -> PyResult<String> {
    c::wgc_gps::gps_seconds_to_week_tow(seconds, &lsk.0)
        .map_err(|error| PyValueError::new_err(format!("WgcGps({error:?})")))
}

#[pyfunction]
fn wgc_parse_week_tow(value: &str) -> PyResult<f64> {
    c::wgc_gps::parse_week_tow(value)
        .map_err(|error| PyValueError::new_err(format!("WgcGps({error:?})")))
}

#[pyfunction]
fn wgc_week_to_et(value: &str, lsk: &NativeLsk) -> PyResult<f64> {
    c::wgc_gps::week_tow_to_et(value, &lsk.0)
        .map(|et| et.seconds())
        .map_err(|error| PyValueError::new_err(format!("WgcGps({error:?})")))
}

#[pyfunction]
fn wgc_gps_seconds_to_et(value: f64, lsk: &NativeLsk) -> PyResult<f64> {
    c::wgc_gps::gps_seconds_to_et(value, &lsk.0)
        .map(|et| et.seconds())
        .map_err(|error| PyValueError::new_err(format!("WgcGps({error:?})")))
}

#[pyfunction]
fn wgc_et_to_gps_display_seconds(et: f64, lsk: &NativeLsk) -> PyResult<f64> {
    let et = NaifEt::from_seconds(et).map_err(err)?;
    c::wgc_gps::et_to_gps_display_seconds(et, &lsk.0)
        .map_err(|error| PyValueError::new_err(format!("WgcGps({error:?})")))
}

#[pyfunction]
fn wgc_calendar_doy(et: f64, system: &str, lsk: &NativeLsk) -> PyResult<String> {
    wgc_calendar_format(et, system, lsk, false)
}

#[pyfunction]
fn wgc_calendar_ymd(et: f64, system: &str, lsk: &NativeLsk) -> PyResult<String> {
    wgc_calendar_format(et, system, lsk, true)
}

fn wgc_calendar_format(et: f64, system: &str, lsk: &NativeLsk, ymd: bool) -> PyResult<String> {
    use c::wgc_gps::WgcCalendarSystem;
    let system = match system {
        "UTC" => WgcCalendarSystem::Utc,
        "TDB" => WgcCalendarSystem::Tdb,
        "TT" => WgcCalendarSystem::Tt,
        "TDT" => WgcCalendarSystem::Tdt,
        "GPS" => WgcCalendarSystem::Gps,
        _ => return Err(PyValueError::new_err("InvalidTimeSystem")),
    };
    let et = NaifEt::from_seconds(et).map_err(err)?;
    if ymd {
        c::wgc_gps::et_to_calendar_ymd(et, system, &lsk.0)
    } else {
        c::wgc_gps::et_to_calendar_doy(et, system, &lsk.0)
    }
    .map_err(err)
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NativeEpoch>()?;
    m.add_class::<NativeDuration>()?;
    m.add_class::<NativeUtc>()?;
    m.add_class::<NativeLsk>()?;
    m.add_class::<NativeEop>()?;
    m.add_class::<NativeSclk>()?;
    m.add_class::<NativeSclkStringContext>()?;
    m.add_class::<NativeSeries>()?;
    m.add_class::<NativeBdt>()?;
    macro_rules! f {($($n:ident),*)=>{$(m.add_function(wrap_pyfunction!($n,m)?)?;)*}}
    f!(
        legacy_utc_et,
        legacy_et_utc,
        raw_validate,
        ut1_inverse_batch,
        utc_et_batch,
        et_utc_batch,
        sclk_batch,
        ut1_batch,
        scientific,
        scientific_tt,
        utc_tai,
        tai_utc,
        kernel_date,
        format_utc,
        legacy_to,
        legacy_from,
        model,
        calendar_to_jdn,
        calendar_to_jd,
        jdn_to_calendar,
        jd_to_calendar,
        days_in_month,
        leap_year,
        etcal,
        validate_format,
        raw_julian,
        wgc_gps_week_tow,
        wgc_parse_week_tow,
        wgc_week_to_et,
        wgc_gps_seconds_to_et,
        wgc_et_to_gps_display_seconds,
        wgc_calendar_doy,
        wgc_calendar_ymd
    );
    m.add("VERSION", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct NativeBdt(c::bdt::Bdt);
#[pymethods]
impl NativeBdt {
    #[new]
    fn new(sec: i64, nanos: i64) -> PyResult<Self> {
        c::bdt::Bdt::checked_from_parts(sec, nanos)
            .map(Self)
            .ok_or_else(overflow)
    }
    #[staticmethod]
    fn from_ns(ns: i128) -> PyResult<Self> {
        c::bdt::Bdt::checked_from_nanoseconds(ns)
            .map(Self)
            .ok_or_else(overflow)
    }
    #[staticmethod]
    fn from_seconds(x: f64) -> PyResult<Self> {
        c::bdt::Bdt::checked_from_seconds(x)
            .map(Self)
            .ok_or_else(overflow)
    }
    #[staticmethod]
    fn from_epoch(e: &NativeEpoch) -> PyResult<Self> {
        c::bdt::Bdt::from_epoch(e.0).map(Self).map_err(err)
    }
    #[staticmethod]
    fn from_week(w: i64, n: u64) -> PyResult<Self> {
        c::bdt::Bdt::from_week_nanoseconds(w, n)
            .map(Self)
            .map_err(err)
    }
    #[staticmethod]
    fn from_utc(u: &NativeUtc, calendar: &str, l: &NativeLsk) -> PyResult<Self> {
        c::bdt::Bdt::from_utc(&u.0, cal(calendar)?, &l.0)
            .map(Self)
            .map_err(err)
    }
    #[staticmethod]
    fn from_et(et: f64, l: &NativeLsk) -> PyResult<Self> {
        c::bdt::Bdt::from_et(
            NaifEt::from_seconds(et).map_err(err)?,
            TimeContext::new(&l.0),
        )
        .map(Self)
        .map_err(err)
    }
    fn parts(&self) -> (i64, u32) {
        (self.0.sec(), self.0.nanos())
    }
    fn total_ns(&self) -> i128 {
        self.0.total_nanoseconds()
    }
    fn seconds(&self) -> f64 {
        self.0.seconds()
    }
    fn to_epoch(&self) -> PyResult<NativeEpoch> {
        self.0.to_epoch().map(NativeEpoch).map_err(err)
    }
    fn week(&self) -> (i64, u64) {
        self.0.week_nanoseconds()
    }
    fn to_utc(&self, calendar: &str, l: &NativeLsk) -> PyResult<NativeUtc> {
        self.0
            .to_utc(cal(calendar)?, &l.0)
            .map(NativeUtc)
            .map_err(err)
    }
    fn to_et(&self, l: &NativeLsk) -> PyResult<f64> {
        self.0
            .to_et(TimeContext::new(&l.0))
            .map(|e| e.seconds())
            .map_err(err)
    }
}
#[pyfunction]
fn utc_et_batch(l: &NativeLsk, values: Vec<NativeUtc>, calendar: &str) -> PyResult<Vec<f64>> {
    cal(calendar)?;
    values
        .iter()
        .enumerate()
        .map(|(i, v)| l.utc_to_et(v, calendar).map_err(|e| indexed(e, i)))
        .collect()
}
#[pyfunction]
fn et_utc_batch(l: &NativeLsk, values: Vec<f64>, calendar: &str) -> PyResult<Vec<NativeUtc>> {
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| l.et_to_utc(v, calendar).map_err(|e| indexed(e, i)))
        .collect()
}
#[pyfunction]
fn sclk_batch(
    k: &NativeSclk,
    l: &NativeLsk,
    values: Vec<f64>,
    inverse: bool,
) -> PyResult<Vec<f64>> {
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| k.convert(v, l, inverse).map_err(|e| indexed(e, i)))
        .collect()
}
#[pyfunction]
fn ut1_batch(eop: &NativeEop, l: &NativeLsk, values: Vec<NativeEpoch>) -> PyResult<Vec<f64>> {
    values
        .iter()
        .enumerate()
        .map(|(i, v)| eop.convert(l, v, "ut1").map_err(|e| indexed(e, i)))
        .collect()
}
#[pyfunction]
fn legacy_utc_et(u: &NativeUtc, l: &NativeLsk, calendar: &str) -> PyResult<f64> {
    check_utc_kernel(&u.0, cal(calendar)?, &l.0)?;
    c::convert::utc_to_et(&u.0, cal(calendar)?, &l.0)
        .map_err(err)
        .and_then(finite)
}
#[pyfunction]
fn legacy_et_utc(x: f64, l: &NativeLsk, calendar: &str) -> PyResult<NativeUtc> {
    let e = legacy_from(x, l, "et")?;
    let calendar = cal(calendar)?;
    civil_pre(e.0, calendar)?;
    check_epoch_kernel(e.0, &l.0)?;
    let u = c::convert::et_to_utc(x, calendar, &l.0).map_err(err)?;
    civil_post(e.0, u, calendar).map(NativeUtc)
}
#[pyfunction]
fn raw_validate(entries: Vec<(i64, i32)>, constants: (f64, f64, f64, f64, f64)) -> PyResult<()> {
    let entries: Vec<_> = entries
        .into_iter()
        .map(|(jdn, delta_at)| c::LeapEntry { jdn, delta_at })
        .collect();
    let (a, k, eb, m0, m1) = constants;
    c::contract::NaifUtcContext::from_raw_parts(
        &entries,
        c::contract::NaifDeltet {
            delta_t_a: a,
            k,
            eb,
            m0,
            m1,
        },
    )
    .map(|_| ())
    .map_err(err)
}
#[pyfunction]
fn ut1_inverse_batch(
    eop: &NativeEop,
    l: &NativeLsk,
    values: Vec<f64>,
) -> PyResult<Vec<NativeEpoch>> {
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| eop.to_tai(l, v).map_err(|e| indexed(e, i)))
        .collect()
}
