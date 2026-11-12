use js_sys::{Array, BigInt, Reflect};
use serde_json::json;
use stkcraft_time::{self as core, *};
use wasm_bindgen::prelude::*;
fn error(code: &str, message: impl ToString) -> JsValue {
    let e = js_sys::Error::new(&message.to_string());
    e.set_name("TimeError");
    let _ = Reflect::set(&e, &"code".into(), &code.into());
    e.into()
}
fn err(e: impl std::fmt::Debug) -> JsValue {
    let s = format!("{e:?}");
    let s = s
        .strip_prefix("Time(")
        .or_else(|| s.strip_prefix("Scientific("))
        .unwrap_or(&s)
        .to_string();
    let code = s.split([' ', '(', ')', '{']).next().unwrap_or("TimeError");
    error(code, &s)
}
fn num(v: &JsValue) -> Result<f64, JsValue> {
    v.as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| error("NonFinite", "Expected finite number"))
}
fn int(v: &JsValue, min: i64, max: i64) -> Result<i64, JsValue> {
    let n = num(v)?;
    if n.fract() != 0.0 || n < min as f64 || n > max as f64 {
        return Err(error("Range", "Integer outside range"));
    }
    Ok(n as i64)
}
fn big(v: &JsValue) -> Result<i128, JsValue> {
    if !v.is_bigint() {
        return Err(error("Type", "Expected bigint"));
    }
    BigInt::from(v.clone())
        .to_string(10)
        .map_err(|_| error("Range", "Invalid bigint"))?
        .as_string()
        .unwrap()
        .parse()
        .map_err(|_| error("Overflow", "Integer outside i128"))
}
fn i64big(v: &JsValue) -> Result<i64, JsValue> {
    i64::try_from(big(v)?).map_err(|_| error("Overflow", "Integer outside i64"))
}
fn bi(v: i128) -> JsValue {
    BigInt::new(&v.to_string().into()).unwrap().into()
}
fn civil_jdn(j: i64) -> Result<i64, JsValue> {
    if j.unsigned_abs() > 700_000_000_000 {
        Err(error("Range", "JDN exceeds binding calendar range"))
    } else {
        Ok(j)
    }
}
fn finite_result(n: f64) -> Result<f64, JsValue> {
    if n.is_finite() {
        Ok(n)
    } else {
        Err(error(
            "NonFinite",
            "Calculation produced a non-finite result",
        ))
    }
}
// Conservative magnitude preconditions only; all returned values come from core.
fn inverse_bound(l: &LeapSecondTable, n: f64, periodic: bool) -> Result<(), JsValue> {
    let amplitude = if periodic { l.k.abs() } else { 0.0 };
    if n.abs() + l.delta_t_a.abs() + amplitude + 43_201.0 > 9e18 {
        return Err(error(
            "EpochOutOfRange",
            "Inverse result may exceed core Epoch range",
        ));
    }
    if periodic {
        let phase = l.m0.abs() + l.m1.abs() * n.abs();
        if !(phase + l.eb.abs()).is_finite() {
            return Err(error("NonFinite", "Kernel anomaly may overflow"));
        }
    }
    Ok(())
}
fn ut1_bound(n: f64, eop: &EopTable, lsk: &LeapSecondTable) -> Result<(), JsValue> {
    let maxdut = eop
        .samples()
        .iter()
        .map(|s| s.ut1_minus_utc.abs())
        .fold(0.0, f64::max);
    let maxleap = lsk
        .entries()
        .iter()
        .map(|e| (e.delta_at as f64).abs())
        .fold(0.0, f64::max);
    if n.abs() + maxdut + maxleap + 1.0 > 9e18 {
        return Err(error(
            "Ut1OutOfRange",
            "UT1 iteration may exceed core Epoch range",
        ));
    }
    if !n.is_finite() || n.abs() > 6e16 {
        Err(error("Ut1OutOfRange", "UT1 exceeds binding civil range"))
    } else {
        Ok(())
    }
}
fn sclk_bound(k: &SclkKernel) -> Result<(), JsValue> {
    finite_result(k.ticks_per_msb())?;
    Ok(())
}
fn eop_bound(e: &EopTable) -> Result<(), JsValue> {
    for s in e.samples() {
        if s.mjd.abs() > 7e11 {
            return Err(error(
                "EopParse",
                "EOP samples exceed supported civil/offset range",
            ));
        }
    }
    Ok(())
}
fn civil_bound(e: Epoch) -> Result<(), JsValue> {
    if e.sec().unsigned_abs() > 60_000_000_000_000_000 {
        return Err(error(
            "EpochOutOfRange",
            "Civil conversion exceeds i32 calendar range",
        ));
    }
    Ok(())
}
fn cal(s: &str) -> Result<Calendar, JsValue> {
    match s {
        "Gregorian" => Ok(Calendar::Gregorian),
        "Julian" => Ok(Calendar::Julian),
        "Mixed" => Ok(Calendar::Mixed),
        _ => Err(error("Calendar", "Unknown calendar")),
    }
}
fn utc(a: &Array) -> Result<UtcDateTime, JsValue> {
    Ok(UtcDateTime::new(
        int(&a.get(0), i32::MIN as i64, i32::MAX as i64)? as i32,
        int(&a.get(1), 1, 12)? as u8,
        int(&a.get(2), 1, 31)? as u8,
        int(&a.get(3), 0, 23)? as u8,
        int(&a.get(4), 0, 59)? as u8,
        num(&a.get(5))?,
    ))
}
fn out_utc(d: UtcDateTime) -> Array {
    [
        JsValue::from(d.year),
        d.month.into(),
        d.day.into(),
        d.hour.into(),
        d.minute.into(),
        d.second.into(),
    ]
    .into_iter()
    .collect()
}
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}
#[wasm_bindgen]
pub struct RawEpoch {
    inner: Epoch,
}
#[wasm_bindgen]
impl RawEpoch {
    #[wasm_bindgen(constructor)]
    pub fn new(seconds: JsValue, nanos: JsValue) -> Result<RawEpoch, JsValue> {
        Ok(Self {
            inner: Epoch::checked_from_parts(i64big(&seconds)?, i64big(&nanos)?)
                .ok_or_else(|| error("Overflow", "Epoch normalization overflow"))?,
        })
    }
    pub fn seconds(&self) -> JsValue {
        bi(self.inner.sec() as i128)
    }
    pub fn nanos(&self) -> u32 {
        self.inner.nanos()
    }
    pub fn add(&self, d: &RawDuration, subtract: bool) -> Result<RawEpoch, JsValue> {
        Ok(Self {
            inner: if subtract {
                self.inner.checked_sub(d.inner)
            } else {
                self.inner.checked_add(d.inner)
            }
            .ok_or_else(|| error("Overflow", "Epoch arithmetic overflow"))?,
        })
    }
    pub fn difference(&self, e: &RawEpoch) -> Result<RawDuration, JsValue> {
        let total = (self.inner.sec() as i128 - e.inner.sec() as i128) * 1_000_000_000
            + self.inner.nanos() as i128
            - e.inner.nanos() as i128;
        Duration::from_nanoseconds(total)
            .ok_or_else(|| error("Overflow", "Epoch difference outside Duration range"))?;
        Ok(RawDuration {
            inner: self.inner - e.inner,
        })
    }
    pub fn compare(&self, e: &RawEpoch) -> i32 {
        self.inner.cmp(&e.inner) as i32
    }
    pub fn approximate_seconds(&self) -> f64 {
        self.inner.tai_seconds()
    }
    #[wasm_bindgen]
    pub fn from_float(v: JsValue) -> Result<RawEpoch, JsValue> {
        Ok(Self {
            inner: Epoch::checked_from_tai_seconds(num(&v)?)
                .ok_or_else(|| error("Overflow", "Epoch outside range"))?,
        })
    }
}
#[wasm_bindgen]
pub struct RawDuration {
    inner: Duration,
}
#[wasm_bindgen]
impl RawDuration {
    #[wasm_bindgen(constructor)]
    pub fn new(total: JsValue) -> Result<RawDuration, JsValue> {
        Ok(Self {
            inner: Duration::from_nanoseconds(big(&total)?)
                .ok_or_else(|| error("Overflow", "Duration outside range"))?,
        })
    }
    pub fn total(&self) -> JsValue {
        bi(self.inner.total_nanoseconds())
    }
    pub fn seconds(&self) -> JsValue {
        bi(self.inner.sec() as i128)
    }
    pub fn nanos(&self) -> i32 {
        self.inner.nanos()
    }
    pub fn calculate(&self, op: &str, d: &RawDuration) -> Result<RawDuration, JsValue> {
        let v = match op {
            "add" => self.inner.checked_add(d.inner),
            "subtract" => self.inner.checked_sub(d.inner),
            "floor" | "ceil" | "round" => {
                if d.inner <= Duration::ZERO {
                    return Err(error("Range", "Rounding step must be positive"));
                }
                let n = self.inner.total_nanoseconds();
                let step = d.inner.total_nanoseconds();
                let q = n.div_euclid(step);
                let ceil = if n.rem_euclid(step) == 0 { q } else { q + 1 };
                let units = match op {
                    "floor" => q,
                    "ceil" => ceil,
                    _ => {
                        if n.rem_euclid(step) * 2 >= step {
                            ceil
                        } else {
                            q
                        }
                    }
                };
                if Duration::from_nanoseconds(units * step).is_none() {
                    None
                } else {
                    Some(match op {
                        "floor" => self.inner.floor(d.inner),
                        "ceil" => self.inner.ceil(d.inner),
                        _ => self.inner.round(d.inner),
                    })
                }
            }
            _ => return Err(error("Operation", "Unknown duration operation")),
        };
        Ok(Self {
            inner: v.ok_or_else(|| error("Overflow", "Duration arithmetic overflow"))?,
        })
    }
    pub fn multiply(&self, k: JsValue) -> Result<RawDuration, JsValue> {
        Ok(Self {
            inner: self
                .inner
                .checked_mul_i64(i64big(&k)?)
                .ok_or_else(|| error("Overflow", "Duration multiplication overflow"))?,
        })
    }
    pub fn negate(&self) -> Result<RawDuration, JsValue> {
        Ok(Self {
            inner: self
                .inner
                .checked_neg()
                .ok_or_else(|| error("Overflow", "Duration negation overflow"))?,
        })
    }
    pub fn approximate_seconds(&self) -> f64 {
        self.inner.seconds_f64()
    }
    #[wasm_bindgen]
    pub fn from_float(v: JsValue) -> Result<RawDuration, JsValue> {
        Ok(Self {
            inner: Duration::checked_from_seconds_f64(num(&v)?)
                .ok_or_else(|| error("Overflow", "Duration outside range"))?,
        })
    }
}
// Guard only arithmetic used by active integer-table civil conversion. Raw
// native JDUTC and the separate SOFA historical route retain their contracts.
fn check_civil_jumps(lsk: &LeapSecondTable, epoch: Epoch) -> Result<(), JsValue> {
    let first = &lsk.entries()[0];
    let first_tai =
        (first.jdn as i128 - core::calendar::J2000_JDN as i128) * 86400 + first.delta_at as i128;
    if (epoch.sec() as i128) < first_tai {
        return Ok(());
    }
    for pair in lsk.entries().windows(2) {
        if matches!(lsk.policy(), LeapSecondPolicy::FrozenAfter { jdn } if pair[1].jdn > jdn) {
            continue;
        }
        pair[1]
            .delta_at
            .checked_sub(pair[0].delta_at)
            .ok_or_else(|| err(TimeError::Overflow))?;
    }
    Ok(())
}
#[wasm_bindgen]
pub struct RawContext {
    lsk: LeapSecondTable,
}
#[wasm_bindgen]
impl RawContext {
    #[wasm_bindgen(constructor)]
    pub fn new(text: Option<String>) -> Result<RawContext, JsValue> {
        Ok(Self {
            lsk: match text {
                Some(t) => LeapSecondTable::from_lsk(&t),
                None => LeapSecondTable::builtin(),
            }
            .map_err(err)?,
        })
    }
    pub fn select(&self, source: &str, freeze: JsValue) -> Result<RawContext, JsValue> {
        let source = match source {
            "IersOnly" => LeapSecondSource::IersOnly,
            "Sofa" => LeapSecondSource::Sofa,
            _ => return Err(error("Source", "Unknown leap source")),
        };
        let policy = if freeze.is_null() || freeze.is_undefined() {
            LeapSecondPolicy::TableDriven
        } else {
            LeapSecondPolicy::FrozenAfter {
                jdn: i64big(&freeze)?,
            }
        };
        Ok(Self {
            lsk: self.lsk.clone().with_source(source).with_policy(policy),
        })
    }
    pub fn metadata(&self) -> String {
        let m = core::contract::TimeContext::new(&self.lsk).metadata();
        json!({"coverage":[m.leap_entry_dates.0.to_string(),m.leap_entry_dates.1.to_string()],"source":format!("{:?}",m.leap_source),"policy":format!("{:?}",m.leap_policy),"deltet":m.deltet,"entries":self.lsk.entries().iter().map(|e|json!([e.jdn.to_string(),e.delta_at])).collect::<Vec<_>>()}).to_string()
    }
    pub fn utc_to_epoch(&self, a: Array, c: &str) -> Result<RawEpoch, JsValue> {
        Ok(RawEpoch {
            inner: core::convert::utc_to_tai(&utc(&a)?, cal(c)?, &self.lsk).map_err(err)?,
        })
    }
    pub fn epoch_to_utc(&self, e: &RawEpoch, c: &str) -> Result<Array, JsValue> {
        civil_bound(e.inner)?;
        check_civil_jumps(&self.lsk, e.inner)?;
        Ok(out_utc(
            core::convert::tai_to_utc(e.inner, cal(c)?, &self.lsk).map_err(err)?,
        ))
    }
    pub fn utc_to_et(&self, a: Array, c: &str) -> Result<f64, JsValue> {
        Ok(core::contract::TimeContext::new(&self.lsk)
            .utc_to_et(&utc(&a)?, cal(c)?)
            .map_err(err)?
            .seconds())
    }
    pub fn et_to_utc(&self, v: JsValue, c: &str) -> Result<Array, JsValue> {
        if num(&v)?.abs() > 60_000_000_000_000_000.0 {
            return Err(error(
                "EpochOutOfRange",
                "Civil conversion exceeds i32 calendar range",
            ));
        }
        let ctx = core::contract::TimeContext::new(&self.lsk);
        let epoch = ctx
            .to_epoch(core::contract::NaifEt::from_seconds(num(&v)?).map_err(err)?)
            .map_err(err)?;
        check_civil_jumps(&self.lsk, epoch)?;
        Ok(out_utc(
            core::contract::TimeContext::new(&self.lsk)
                .et_to_utc(
                    core::contract::NaifEt::from_seconds(num(&v)?).map_err(err)?,
                    cal(c)?,
                )
                .map_err(err)?,
        ))
    }
    pub fn to_number(&self, e: &RawEpoch, scale: &str) -> Result<f64, JsValue> {
        let l = &self.lsk;
        let e = e.inner;
        finite_result(match scale {
            "ET" => core::contract::TimeContext::new(l)
                .to_et(e)
                .map_err(err)?
                .seconds(),
            "TT" => core::convert::tai_to_tt(e, l),
            "GPS" => core::convert::tai_to_gps(e),
            "TCG" => core::coord::tai_to_tcg(e, l),
            "TCG-TT" => core::coord::tcg_minus_tt(e, l),
            "TCB.IAU" => core::coord::tai_to_tcb(e, l),
            "TDB.Compact" => core::tdb::tai_to_tdb(e, l),
            "TDB.IAU" => core::contract::ScientificContext::new()
                .to_tdb(e)
                .map_err(err)?
                .seconds(),
            "TT.IAU" => core::contract::ScientificContext::new()
                .tt_seconds(e)
                .map_err(err)?,
            _ => return Err(error("Scale", "Unknown explicit scale/model")),
        })
    }
    pub fn from_number(&self, v: JsValue, scale: &str) -> Result<RawEpoch, JsValue> {
        let n = num(&v)?;
        let l = &self.lsk;
        inverse_bound(l, n, matches!(scale, "ET"))?;
        if matches!(scale, "TDB.IAU" | "TCB.IAU") && n.abs() > 1.7e9 {
            return Err(error(
                "OutsideModelRange",
                "Scientific inverse exceeds model interval",
            ));
        }
        // Legacy inverse functions are bounded before invoking their infallible constructors.
        if n.abs() > 9.0e18 {
            return Err(error(
                "EpochOutOfRange",
                "Numeric epoch outside safe conversion range",
            ));
        }
        Ok(RawEpoch {
            inner: match scale {
                "ET" => core::contract::TimeContext::new(l)
                    .to_epoch(core::contract::NaifEt::from_seconds(n).map_err(err)?)
                    .map_err(err)?,
                "TT" => core::convert::tt_to_tai(n, l),
                "GPS" => core::convert::gps_to_tai(n),
                "TCG" => core::coord::tcg_to_tai(n, l),
                "TCB.IAU" => core::coord::tcb_to_tai(n, l),
                "TDB.Compact" => core::tdb::tdb_to_tai(n, l),
                "TDB.IAU" => {
                    let e = core::dtdb::tdb_iau_to_tai(n, l);
                    core::contract::ScientificContext::new()
                        .to_tdb(e)
                        .map_err(err)?;
                    e
                }
                _ => return Err(error("Scale", "Unknown explicit scale/model")),
            },
        })
    }
    pub fn julian_utc(&self, v: JsValue) -> Result<f64, JsValue> {
        Ok(core::contract::TimeContext::new(&self.lsk)
            .et_to_spice_julian_utc(core::contract::NaifEt::from_seconds(num(&v)?).map_err(err)?)
            .map_err(err)?
            .days())
    }
    pub fn format(
        &self,
        a: Array,
        c: &str,
        style: &str,
        precision: JsValue,
        pattern: &str,
    ) -> Result<String, JsValue> {
        let d = utc(&a)?;
        core::convert::utc_to_tai(&d, cal(c)?, &self.lsk).map_err(err)?;
        if d.year == i32::MAX
            && d.month == 12
            && d.day == 31
            && d.hour == 23
            && d.minute == 59
            && d.second >= 59.0
        {
            return Err(error(
                "EpochOutOfRange",
                "Formatting carry may exceed i32 calendar year",
            ));
        }
        let opts = core::format::FormatOptions::new(int(&precision, 0, 9)? as usize)
            .with_calendar(cal(c)?)
            .with_leap_seconds(&self.lsk);
        Ok(match style {
            "iso8601" => core::format::iso8601_with(&d, &opts),
            "rfc3339" => core::format::rfc3339_with(&d, &opts),
            "spice" => core::format::spice_calendar_with(&d, &opts),
            "rfc2822" => core::format::rfc2822(&d),
            "strftime" => core::format::strftime_with(&d, pattern, &opts).map_err(err)?,
            _ => return Err(error("InvalidFormat", "Unknown format")),
        })
    }
    pub fn observer(&self, e: &RawEpoch, a: Array) -> Result<f64, JsValue> {
        Ok(core::contract::ScientificContext::new()
            .to_tdb_at_observer(
                e.inner,
                num(&a.get(0))?,
                num(&a.get(1))?,
                num(&a.get(2))?,
                num(&a.get(3))?,
            )
            .map_err(err)?
            .seconds())
    }
}
#[wasm_bindgen]
pub fn kernel_date(text: &str) -> Result<String, JsValue> {
    let d = core::parse_kernel_date(text).map_err(err)?;
    Ok(json!({"secondsPastJ2000":d.seconds_past_j2000,"year":d.year,"month":d.month,"day":d.day,"hour":d.hour,"minute":d.minute,"second":d.second}).to_string())
}
#[wasm_bindgen]
pub struct RawSeries {
    inner: TimeSeries,
}
#[wasm_bindgen]
impl RawSeries {
    #[wasm_bindgen(constructor)]
    pub fn new(a: &RawEpoch, b: &RawEpoch, d: &RawDuration, inclusive: bool) -> Self {
        Self {
            inner: if inclusive {
                TimeSeries::inclusive(a.inner, b.inner, d.inner)
            } else {
                TimeSeries::exclusive(a.inner, b.inner, d.inner)
            },
        }
    }
    pub fn next_epoch(&mut self) -> Option<RawEpoch> {
        self.inner.next().map(|inner| RawEpoch { inner })
    }
}
#[wasm_bindgen]
pub struct RawSclk {
    inner: SclkKernel,
}
#[wasm_bindgen]
impl RawSclk {
    #[wasm_bindgen(constructor)]
    pub fn new(text: &str, id: JsValue) -> Result<RawSclk, JsValue> {
        Ok(Self {
            inner: {
                let k = SclkKernel::parse(text, int(&id, i32::MIN as i64, i32::MAX as i64)? as i32)
                    .map_err(err)?;
                sclk_bound(&k)?;
                k
            },
        })
    }
    pub fn convert(&self, ctx: &RawContext, v: JsValue, inverse: bool) -> Result<f64, JsValue> {
        finite_result(
            if inverse {
                self.inner.et_to_ticks(num(&v)?, &ctx.lsk)
            } else {
                self.inner.ticks_to_et(num(&v)?, &ctx.lsk)
            }
            .map_err(err)?,
        )
    }
    pub fn to_epoch(&self, ctx: &RawContext, v: JsValue) -> Result<RawEpoch, JsValue> {
        let parallel = finite_result(self.inner.parallel_seconds(num(&v)?).map_err(err)?)?;
        inverse_bound(
            &ctx.lsk,
            parallel,
            matches!(self.inner.time_system(), SclkTimeSystem::Tdb),
        )?;
        Ok(RawEpoch {
            inner: self.inner.ticks_to_tai(num(&v)?, &ctx.lsk).map_err(err)?,
        })
    }
    pub fn from_epoch(&self, ctx: &RawContext, e: &RawEpoch) -> Result<f64, JsValue> {
        finite_result(self.inner.tai_to_ticks(e.inner, &ctx.lsk).map_err(err)?)
    }
    pub fn fields(&self, v: JsValue) -> Result<Array, JsValue> {
        let value = num(&v)?;
        if value >= 0.0 && value < i64::MAX as f64 {
            let mut remainder = value.round() as i64;
            for index in (1..self.inner.n_fields()).rev() {
                let m = self.inner.moduli()[index];
                (remainder % m)
                    .checked_add(self.inner.offsets()[index])
                    .ok_or_else(|| error("Overflow", "Decoded clock field exceeds i64"))?;
                remainder /= m;
            }
            remainder
                .checked_add(self.inner.offsets()[0])
                .ok_or_else(|| error("Overflow", "Decoded clock field exceeds i64"))?;
        }
        if num(&v)? >= i64::MAX as f64 {
            return Err(error("Range", "Ticks cannot be narrowed to i64 fields"));
        }
        Ok(self
            .inner
            .fields_from_ticks(num(&v)?)
            .map_err(err)?
            .into_iter()
            .map(|n| bi(n as i128))
            .collect())
    }
    pub fn ticks(&self, a: Array) -> Result<f64, JsValue> {
        self.inner
            .ticks_from_fields(
                &a.iter()
                    .map(|v| i64big(&v))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(err)
    }
    pub fn parse(&self, s: &str) -> Result<Array, JsValue> {
        Ok(self
            .inner
            .parse_string(s)
            .map_err(err)?
            .into_iter()
            .map(|n| bi(n as i128))
            .collect())
    }
    pub fn format(&self, a: Array) -> Result<String, JsValue> {
        self.inner
            .format_string(
                &a.iter()
                    .map(|v| i64big(&v))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(err)
    }
    pub fn parallel(&self, v: JsValue, inverse: bool) -> Result<f64, JsValue> {
        finite_result(
            if inverse {
                self.inner.ticks_from_parallel(num(&v)?)
            } else {
                self.inner.parallel_seconds(num(&v)?)
            }
            .map_err(err)?,
        )
    }
    pub fn metadata(&self) -> String {
        let k = &self.inner;
        let (p, q) = k.partitions();
        json!({"spacecraftId":k.spacecraft_id(),"dataType":k.data_type(),"timeSystem":format!("{:?}",k.time_system()),"nFields":k.n_fields(),"moduli":k.moduli().iter().map(ToString::to_string).collect::<Vec<_>>(),"offsets":k.offsets().iter().map(ToString::to_string).collect::<Vec<_>>(),"coefficients":k.coefficients().iter().map(|c|json!({"ticks":c.ticks,"seconds":c.seconds,"rate":c.rate})).collect::<Vec<_>>(),"partitionStarts":p,"partitionEnds":q,"ticksPerMsb":k.ticks_per_msb()}).to_string()
    }
}
#[wasm_bindgen]
pub struct RawEop {
    inner: EopTable,
}
#[wasm_bindgen]
impl RawEop {
    #[wasm_bindgen(constructor)]
    pub fn new(text: &str, series: &str) -> Result<RawEop, JsValue> {
        let series = match series {
            "BulletinA" => EopSeries::BulletinA,
            "BulletinB" => EopSeries::BulletinB,
            "FinalPreferred" => EopSeries::FinalPreferred,
            _ => return Err(error("Series", "Unknown EOP series")),
        };
        Ok(Self {
            inner: {
                let e = EopTable::from_finals2000a_series(text, series).map_err(err)?;
                eop_bound(&e)?;
                e
            },
        })
    }
    pub fn observed(&self) -> Result<RawEop, JsValue> {
        Ok(Self {
            inner: self.inner.observed_only().map_err(err)?,
        })
    }
    pub fn interpolate(&self, v: JsValue) -> Result<f64, JsValue> {
        finite_result(self.inner.ut1_minus_utc(num(&v)?).map_err(err)?)
    }
    pub fn to_ut1(&self, c: &RawContext, e: &RawEpoch, offset: bool) -> Result<f64, JsValue> {
        finite_result(
            if offset {
                core::convert::ut1_minus_utc(e.inner, &c.lsk, &self.inner)
            } else {
                core::convert::tai_to_ut1(e.inner, &c.lsk, &self.inner)
            }
            .map_err(err)?,
        )
    }
    pub fn from_ut1(&self, c: &RawContext, v: JsValue) -> Result<RawEpoch, JsValue> {
        ut1_bound(num(&v)?, &self.inner, &c.lsk)?;
        Ok(RawEpoch {
            inner: core::convert::ut1_to_tai(num(&v)?, &c.lsk, &self.inner).map_err(err)?,
        })
    }
    pub fn metadata(&self) -> String {
        json!({"range":self.inner.range(),"samples":self.inner.samples().iter().map(|s|json!({"mjd":s.mjd,"ut1MinusUtc":s.ut1_minus_utc,"observed":s.observed})).collect::<Vec<_>>()}).to_string()
    }
}
#[wasm_bindgen]
pub struct RawBdt {
    inner: Bdt,
}
#[wasm_bindgen]
impl RawBdt {
    #[wasm_bindgen(constructor)]
    pub fn new(sec: JsValue, ns: JsValue) -> Result<RawBdt, JsValue> {
        Ok(Self {
            inner: Bdt::checked_from_parts(i64big(&sec)?, i64big(&ns)?)
                .ok_or_else(|| error("Overflow", "BDT outside range"))?,
        })
    }
    pub fn seconds(&self) -> JsValue {
        bi(self.inner.sec() as i128)
    }
    pub fn nanos(&self) -> u32 {
        self.inner.nanos()
    }
    pub fn total(&self) -> JsValue {
        bi(self.inner.total_nanoseconds())
    }
    pub fn approximate_seconds(&self) -> f64 {
        self.inner.seconds()
    }
    pub fn to_epoch(&self) -> Result<RawEpoch, JsValue> {
        Ok(RawEpoch {
            inner: self.inner.to_epoch().map_err(err)?,
        })
    }
    pub fn from_epoch(e: &RawEpoch) -> Result<RawBdt, JsValue> {
        Ok(Self {
            inner: Bdt::from_epoch(e.inner).map_err(err)?,
        })
    }
    pub fn from_float(v: JsValue) -> Result<RawBdt, JsValue> {
        Ok(Self {
            inner: Bdt::checked_from_seconds(num(&v)?)
                .ok_or_else(|| error("Overflow", "BDT outside range"))?,
        })
    }
    pub fn from_total(v: JsValue) -> Result<RawBdt, JsValue> {
        Ok(Self {
            inner: Bdt::checked_from_nanoseconds(big(&v)?)
                .ok_or_else(|| error("Overflow", "BDT outside range"))?,
        })
    }
    pub fn week(&self) -> Array {
        let (w, n) = self.inner.week_nanoseconds();
        [bi(w as i128), bi(n as i128)].into_iter().collect()
    }
    pub fn from_week(w: JsValue, n: JsValue) -> Result<RawBdt, JsValue> {
        let n = u64::try_from(big(&n)?)
            .map_err(|_| error("Range", "Negative or oversized week nanoseconds"))?;
        Ok(Self {
            inner: Bdt::from_week_nanoseconds(i64big(&w)?, n).map_err(err)?,
        })
    }
}
#[wasm_bindgen]
impl RawContext {
    pub fn utc_batch(&self, values: Array, c: &str) -> Result<js_sys::Float64Array, JsValue> {
        let c = cal(c)?;
        let ctx = core::contract::TimeContext::new(&self.lsk);
        let mut out = Vec::new();
        for (i, v) in values.iter().enumerate() {
            let u = utc(&Array::from(&v)).map_err(|e| batch_error(e, i))?;
            out.push(
                ctx.utc_to_et(&u, c)
                    .map_err(|e| batch_error(err(e), i))?
                    .seconds(),
            );
        }
        Ok(js_sys::Float64Array::from(out.as_slice()))
    }
    pub fn et_batch(&self, values: js_sys::Float64Array, c: &str) -> Result<Array, JsValue> {
        let c = cal(c)?;
        let ctx = core::contract::TimeContext::new(&self.lsk);
        let out = Array::new();
        for (i, v) in values.to_vec().iter().enumerate() {
            if v.abs() > 6e16 {
                return Err(batch_error(
                    error(
                        "EpochOutOfRange",
                        "Civil conversion exceeds binding calendar bound",
                    ),
                    i,
                ));
            }
            let e = core::contract::NaifEt::from_seconds(*v).map_err(|e| batch_error(err(e), i))?;
            let epoch = ctx.to_epoch(e).map_err(|e| batch_error(err(e), i))?;
            check_civil_jumps(&self.lsk, epoch).map_err(|e| batch_error(e, i))?;
            out.push(&out_utc(
                ctx.et_to_utc(e, c).map_err(|e| batch_error(err(e), i))?,
            ));
        }
        Ok(out)
    }
}
fn batch_error(e: JsValue, i: usize) -> JsValue {
    let _ = Reflect::set(&e, &"index".into(), &JsValue::from(i as f64));
    e
}
#[wasm_bindgen]
impl RawSclk {
    pub fn batch(
        &self,
        c: &RawContext,
        values: js_sys::Float64Array,
        inverse: bool,
    ) -> Result<js_sys::Float64Array, JsValue> {
        let mut out = Vec::new();
        for (i, v) in values.to_vec().iter().enumerate() {
            if !v.is_finite() {
                return Err(batch_error(error("NonFinite", "Expected finite input"), i));
            }
            out.push(
                if inverse {
                    self.inner.et_to_ticks(*v, &c.lsk)
                } else {
                    self.inner.ticks_to_et(*v, &c.lsk)
                }
                .map_err(|e| batch_error(err(e), i))
                .and_then(finite_result)
                .map_err(|e| batch_error(e, i))?,
            );
        }
        Ok(js_sys::Float64Array::from(out.as_slice()))
    }
}
#[wasm_bindgen]
pub fn calendar_call(op: &str, a: Array, c: &str) -> Result<JsValue, JsValue> {
    let ca = cal(c)?;
    Ok(match op {
        "isGregorianLeap" => {
            core::calendar::is_gregorian_leap(
                int(&a.get(0), i32::MIN as i64, i32::MAX as i64)? as i32
            )
            .into()
        }
        "isJulianLeap" => {
            core::calendar::is_julian_leap(int(&a.get(0), i32::MIN as i64, i32::MAX as i64)? as i32)
                .into()
        }
        "daysInMonth" => core::calendar::days_in_month(
            int(&a.get(0), i32::MIN as i64, i32::MAX as i64)? as i32,
            int(&a.get(1), 1, 12)? as u8,
            ca,
        )
        .map(JsValue::from)
        .unwrap_or(JsValue::NULL),
        "toJdn" => bi(core::calendar::calendar_to_jdn(
            int(&a.get(0), i32::MIN as i64, i32::MAX as i64)? as i32,
            int(&a.get(1), 1, 12)? as u8,
            int(&a.get(2), 1, 31)? as u8,
            ca,
        )
        .map_err(err)? as i128),
        "fromJdn" => {
            let j = i64big(&a.get(0))?;
            if !(-700000000000..=700000000000).contains(&j) {
                return Err(error("Range", "JDN outside i32 calendar year range"));
            }
            let (y, m, d) = core::calendar::jdn_to_calendar(j, ca);
            [JsValue::from(y), m.into(), d.into()]
                .into_iter()
                .collect::<Array>()
                .into()
        }
        "toJd" => {
            let d = utc(&a)?;
            core::calendar::calendar_to_jd(d.year, d.month, d.day, d.hour, d.minute, d.second, ca)
                .map_err(err)?
                .into()
        }
        "jdToMjd" => core::calendar::jd_to_mjd(num(&a.get(0))?).into(),
        "mjdToJd" => core::calendar::mjd_to_jd(num(&a.get(0))?).into(),
        _ => return Err(error("Operation", "Unknown calendar operation")),
    })
}
#[wasm_bindgen]
pub fn model_call(op: &str, a: Array) -> Result<f64, JsValue> {
    let x = num(&a.get(0))?;
    let value = match op {
        "compactMinusTt" => core::tdb::tdb_minus_tt_seconds(x),
        "dtdbMinusTt" => core::dtdb::dtdb_minus_tt(x),
        "dtdbMinusTtSplit" => core::dtdb::dtdb_minus_tt_split(x, num(&a.get(1))?),
        "dtdbAtObserver" => core::dtdb::dtdb_minus_tt_at_observer(
            x,
            num(&a.get(1))?,
            num(&a.get(2))?,
            num(&a.get(3))?,
            num(&a.get(4))?,
            num(&a.get(5))?,
        )
        .map_err(err)?,
        "tcbMinusTdb" => core::coord::tcb_minus_tdb(x),
        "sofaDeltaAt" => core::leap::sofa_delta_at(x)
            .ok_or_else(|| error("UtcOutOfRange", "Outside SOFA coverage"))?,
        _ => return Err(error("Operation", "Unknown model operation")),
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(error("NonFinite", "Model produced a non-finite result"))
    }
}
#[wasm_bindgen]
impl RawEop {
    pub fn from_samples(rows: Array) -> Result<RawEop, JsValue> {
        let mut v = Vec::new();
        for row in rows.iter() {
            if !Array::is_array(&row) {
                return Err(error("Type", "Expected EOP sample array"));
            }
            let a = Array::from(&row);
            v.push(EopSample {
                mjd: num(&a.get(0))?,
                ut1_minus_utc: num(&a.get(1))?,
                observed: a
                    .get(2)
                    .as_bool()
                    .ok_or_else(|| error("Type", "Expected boolean observed flag"))?,
            });
        }
        Ok(Self {
            inner: {
                let e = EopTable::new(v).map_err(err)?;
                eop_bound(&e)?;
                e
            },
        })
    }
    pub fn batch(&self, c: &RawContext, values: js_sys::Float64Array) -> Result<Array, JsValue> {
        let out = Array::new();
        for (i, v) in values.to_vec().iter().enumerate() {
            let n = num(&JsValue::from(*v)).map_err(|e| batch_error(e, i))?;
            ut1_bound(n, &self.inner, &c.lsk).map_err(|e| batch_error(e, i))?;
            let e = core::convert::ut1_to_tai(n, &c.lsk, &self.inner)
                .map_err(|e| batch_error(err(e), i))?;
            out.push(&RawEpoch { inner: e }.into());
        }
        Ok(out)
    }
}
#[wasm_bindgen]
impl RawContext {
    pub fn from_entries(rows: Array, constants: Array) -> Result<RawContext, JsValue> {
        let mut entries = Vec::new();
        for row in rows.iter() {
            if !Array::is_array(&row) {
                return Err(error("Type", "Expected leap entry array"));
            }
            let a = Array::from(&row);
            entries.push(LeapEntry {
                jdn: civil_jdn(i64big(&a.get(0))?)?,
                delta_at: int(&a.get(1), i32::MIN as i64, i32::MAX as i64)? as i32,
            });
        }
        let l = LeapSecondTable::new(entries).map_err(err)?;
        let mut d = Vec::new();
        for n in constants.iter() {
            d.push(num(&n)?);
        }
        if d.len() != 5 {
            return Err(error("Range", "DELTET needs five constants"));
        }
        Ok(Self {
            lsk: l.with_constants(d[0], d[1], d[2], d[3], d[4]),
        })
    }
    pub fn delta_at(&self, j: JsValue, f: JsValue) -> Result<f64, JsValue> {
        self.lsk
            .delta_at_f64(civil_jdn(i64big(&j)?)?, num(&f)?)
            .ok_or_else(|| error("UtcOutOfRange", "Outside leap coverage"))
    }
}
#[wasm_bindgen]
pub fn raw_naif_julian(rows: Array, constants: Array, value: JsValue) -> Result<f64, JsValue> {
    let mut entries = Vec::new();
    for row in rows.iter() {
        if !Array::is_array(&row) {
            return Err(error("Type", "Expected leap entry array"));
        }
        let a = Array::from(&row);
        entries.push(LeapEntry {
            jdn: civil_jdn(i64big(&a.get(0))?)?,
            delta_at: int(&a.get(1), i32::MIN as i64, i32::MAX as i64)? as i32,
        });
    }
    if constants.length() != 5 {
        return Err(error("Range", "DELTET needs five constants"));
    }
    let d = core::contract::NaifDeltet {
        delta_t_a: num(&constants.get(0))?,
        k: num(&constants.get(1))?,
        eb: num(&constants.get(2))?,
        m0: num(&constants.get(3))?,
        m1: num(&constants.get(4))?,
    };
    Ok(core::contract::NaifUtcContext::from_raw_parts(&entries, d)
        .map_err(err)?
        .et_to_spice_julian_utc(core::contract::NaifEt::from_seconds(num(&value)?).map_err(err)?)
        .map_err(err)?
        .days())
}
#[wasm_bindgen]
impl RawDuration {
    pub fn divide(&self, k: JsValue) -> Result<RawDuration, JsValue> {
        let k = i64big(&k)?;
        if k == 0 {
            return Err(error("Range", "Division by zero"));
        }
        Duration::from_nanoseconds(self.inner.total_nanoseconds() / k as i128)
            .ok_or_else(|| error("Overflow", "Duration division overflow"))?;
        Ok(Self {
            inner: self.inner / k,
        })
    }
    pub fn from_parts(s: JsValue, n: JsValue) -> Result<RawDuration, JsValue> {
        Ok(Self {
            inner: Duration::checked_from_parts(i64big(&s)?, i64big(&n)?)
                .ok_or_else(|| error("Overflow", "Duration normalization overflow"))?,
        })
    }
}
#[wasm_bindgen]
pub fn format_numeric(style: &str, v: JsValue, p: JsValue) -> Result<JsValue, JsValue> {
    let n = num(&v)?;
    if n.abs() > 1e12 || (style == "jdToCalendar" && n.abs() > 7e11) {
        return Err(error("Range", "Numeric calendar formatting bound is 1e12"));
    }
    Ok(match style {
        "etcal" => core::format::etcal(n, int(&p, 0, 9)? as usize).into(),
        "jdToCalendar" => {
            let (y, m, d, h, mi, s) = core::format::jd_to_calendar(n);
            [
                JsValue::from(y),
                m.into(),
                d.into(),
                h.into(),
                mi.into(),
                s.into(),
            ]
            .into_iter()
            .collect::<Array>()
            .into()
        }
        _ => return Err(error("InvalidFormat", "Unknown numeric format")),
    })
}
#[wasm_bindgen]
impl RawEop {
    pub fn to_batch(&self, c: &RawContext, parts: Array) -> Result<js_sys::Float64Array, JsValue> {
        let mut out = Vec::new();
        for (i, v) in parts.iter().enumerate() {
            if !Array::is_array(&v) {
                return Err(batch_error(error("Type", "Expected epoch parts"), i));
            }
            let a = Array::from(&v);
            let e = RawEpoch::new(a.get(0), a.get(1)).map_err(|e| batch_error(e, i))?;
            out.push(
                finite_result(
                    core::convert::tai_to_ut1(e.inner, &c.lsk, &self.inner)
                        .map_err(|e| batch_error(err(e), i))?,
                )
                .map_err(|e| batch_error(e, i))?,
            );
        }
        Ok(js_sys::Float64Array::from(out.as_slice()))
    }
}
#[wasm_bindgen]
impl RawContext {
    pub fn legacy_utc_et(&self, a: Array, c: &str) -> Result<f64, JsValue> {
        finite_result(core::convert::utc_to_et(&utc(&a)?, cal(c)?, &self.lsk).map_err(err)?)
    }
    pub fn legacy_et_utc(&self, v: JsValue, c: &str) -> Result<Array, JsValue> {
        let n = num(&v)?;
        inverse_bound(&self.lsk, n, true)?;
        if n.abs() > 6e16 {
            return Err(error(
                "EpochOutOfRange",
                "Civil conversion exceeds binding calendar bound",
            ));
        }
        check_civil_jumps(&self.lsk, core::convert::et_to_tai(n, &self.lsk))?;
        Ok(out_utc(
            core::convert::et_to_utc(n, cal(c)?, &self.lsk).map_err(err)?,
        ))
    }
    pub fn legacy_to_et(&self, e: &RawEpoch) -> Result<f64, JsValue> {
        finite_result(core::convert::tai_to_et(e.inner, &self.lsk))
    }
    pub fn legacy_from_et(&self, v: JsValue) -> Result<RawEpoch, JsValue> {
        let n = num(&v)?;
        inverse_bound(&self.lsk, n, true)?;
        if n.abs() > 9e18 {
            return Err(error("EpochOutOfRange", "ET exceeds epoch range"));
        }
        Ok(RawEpoch {
            inner: core::convert::et_to_tai(n, &self.lsk),
        })
    }
}
#[wasm_bindgen]
impl RawContext {
    pub fn delta_at_integer(&self, j: JsValue) -> Result<Option<i32>, JsValue> {
        Ok(self.lsk.delta_at_for_jdn(civil_jdn(i64big(&j)?)?))
    }
}
