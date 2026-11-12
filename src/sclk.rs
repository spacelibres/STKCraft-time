//! Spacecraft clock (SCLK) kernels, type 1.
//!
//! A type-1 clock is a set of cascading integer counters. The SCLK kernel
//! stores, for one spacecraft:
//!
//! * `SCLK01_N_FIELDS_<id>` — number of fields,
//! * `SCLK01_MODULI_<id>` — modulus of each field,
//! * `SCLK01_OFFSETS_<id>` — offset added to each field value,
//! * `SCLK01_TIME_SYSTEM_<id>` — `1` for TDB/ET, `2` for TT (default 1),
//! * `SCLK01_COEFFICIENTS_<id>` — triplets `(ticks, parallel_seconds,
//!   rate)` describing the piecewise-linear map from encoded ticks to the
//!   parallel time system,
//! * `SCLK01_OUTPUT_DELIM_<id>` — delimiter used in formatted strings.
//!
//! The encoded tick count of a field vector is
//!
//! ```text
//! ticks = (((f0 - o0) * m1 + (f1 - o1)) * m2 + (f2 - o2)) * ...
//! ```
//!
//! The offsets give the value of the first count of each field. For
//! Voyager, the third field starts at 1, so its offset is 1 and the
//! encoded counter runs from 0 to 799.
//!
//! and the number of ticks in one count of the most significant field is
//! `m1 * m2 * ...`. The spacing between successive coefficient samples is
//! therefore `rate / ticks_per_msb` seconds per tick, which is the
//! behaviour verified against CSPICE `sct2e`.
//!
//! # Scope
//!
//! Encoded-tick/time conversions enforce the cumulative duration of the
//! declared partitions, rounding the accumulated raw durations as SCTE01.
//! Native-style encoded string conversion is available separately from the
//! legacy field API. Partitioned SclkClock strings use discrete SCET01 ticks;
//! continuous tick conversion rejects values outside its own domain. A map
//! without partition metadata has no declared upper bound.
//!
//! Only clock type 1 is supported. No SCLK kernel is embedded at runtime; load the
//! mission kernel you need at deployment time.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::convert::{et_to_tai, tai_to_et, tai_to_tt, tt_to_tai};
use crate::epoch::Epoch;
use crate::error::TimeError;
use crate::leap::LeapSecondTable;
use crate::math;

/// The parallel time system that SCLK coefficients map to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SclkTimeSystem {
    /// Barycentric Dynamical Time: the coefficients give ET seconds past
    /// J2000 directly.
    Tdb,
    /// Terrestrial Dynamical Time: the coefficients give TT seconds past
    /// J2000, and the LSK is needed to reach ET.
    Tdt,
}

/// One piecewise-linear segment of the tick-to-time map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SclkCoefficient {
    /// Encoded tick count at the start of the segment.
    pub ticks: f64,
    /// Parallel-time value at the start of the segment, in seconds.
    pub seconds: f64,
    /// Parallel-time rate per most-significant-field count.
    pub rate: f64,
}

/// Native short error for partition-aware SCLK string conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SclkStringError(&'static str);

impl SclkStringError {
    /// Return the corresponding CSPICE short error symbol.
    pub fn spice_short(self) -> &'static str {
        self.0
    }
}

impl core::fmt::Display for SclkStringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SclkStringError {}

/// State of NAIF's type-1 SCLK component parser across calls.
///
/// Keep one context for each independent native-style kernel-pool history.
/// A failed numeric component can affect a later blank component until a
/// numeric component is successfully parsed.
#[derive(Clone, Debug, Default)]
pub struct SclkStringContext {
    component_error: bool,
}

/// A parsed type-1 SCLK kernel for one spacecraft clock.
#[derive(Clone, Debug)]
pub struct SclkKernel {
    spacecraft_id: i32,
    data_type: i32,
    time_system: SclkTimeSystem,
    n_fields: usize,
    moduli: Vec<i64>,
    offsets: Vec<i64>,
    coefficients: Vec<SclkCoefficient>,
    delimiter: char,
    native_delimiter_code: Option<i64>,
    partition_start: Vec<f64>,
    partition_end: Vec<f64>,
    max_encoded_ticks: Option<f64>,
}

impl SclkKernel {
    /// Decode continuous encoded ticks to a partition-prefixed NAIF clock string.
    pub fn decode_native_sclk(&self, ticks: f64) -> Result<String, SclkStringError> {
        self.validate_native_string_metadata()?;
        let totals = self.partition_totals(false)?;
        if !ticks.is_finite() {
            return Err(SclkStringError("SPICE(VALUEOUTOFRANGE)"));
        }
        let ticks = native_nint(ticks);
        let last = *totals
            .last()
            .ok_or(SclkStringError("SPICE(KERNELVARNOTFOUND)"))?;
        let part = if ticks == last {
            totals.len() - 1
        } else {
            totals
                .iter()
                .position(|&end| ticks < end)
                .ok_or(SclkStringError("SPICE(VALUEOUTOFRANGE)"))?
        };
        if ticks < 0.0 {
            return Err(SclkStringError("SPICE(VALUEOUTOFRANGE)"));
        }
        let readout = if part == 0 {
            ticks + self.partition_start[part]
        } else {
            ticks + self.partition_start[part] - totals[part - 1]
        };
        let mut factors = alloc::vec![1.0; self.n_fields];
        for index in (0..self.n_fields - 1).rev() {
            factors[index] = factors[index + 1] * self.moduli[index + 1] as f64;
        }
        let mut remaining = native_nint(readout);
        if remaining < 0.0 || !remaining.is_finite() {
            return Err(SclkStringError("SPICE(VALUEOUTOFRANGE)"));
        }
        let mut parts = Vec::with_capacity(self.n_fields);
        for (index, &factor) in factors.iter().enumerate() {
            let component = if index + 1 == self.n_fields {
                remaining + self.offsets[index] as f64
            } else {
                let value = math::floor(remaining / factor) + self.offsets[index] as f64;
                remaining %= factor;
                value
            };
            let max_width = self.moduli[index] as f64 + self.offsets[index] as f64 - 1.0;
            if max_width + 0.5 <= 0.0 || !component.is_finite() {
                return Err(SclkStringError("SPICE(INVALIDSCLKSTRING)"));
            }
            let width = (native_log10(max_width + 0.5) as i32 + 1).max(0) as usize;
            // SCFM01 removes DPSTRF's leading sign character, including
            // for negative field values induced by a negative offset.
            let component = native_clock_component(component);
            let mut padded = String::new();
            if component.len() > 28 {
                // The 30-byte DPSTRF buffer cannot contain a sign, this
                // integer, and a decimal point. SCFM01 then observes a
                // missing decimal and pads the field width plus two zeros.
                for _ in 0..width + 2 {
                    padded.push('0');
                }
            } else {
                for _ in component.len()..width {
                    padded.push('0');
                }
                padded.push_str(&component);
            }
            parts.push(padded);
        }
        Ok(format!(
            "{}/{}",
            part + 1,
            parts.join(&self.delimiter.to_string())
        ))
    }

    /// Encode a clock string using a fresh NAIF component-parser context.
    ///
    /// Use [`Self::encode_native_sclk_with_context`] to reproduce the native
    /// parser's state across successive calls.
    pub fn encode_native_sclk(&self, text: &str) -> Result<f64, SclkStringError> {
        self.encode_native_sclk_with_context(text, &mut SclkStringContext::default())
    }

    /// Encode a clock string while preserving native parser state across calls.
    pub fn encode_native_sclk_with_context(
        &self,
        text: &str,
        context: &mut SclkStringContext,
    ) -> Result<f64, SclkStringError> {
        let text = text.split('\0').next().unwrap_or("");
        if text.is_empty() {
            return Err(SclkStringError("SPICE(EMPTYSTRING)"));
        }
        let invalid = SclkStringError("SPICE(INVALIDSCLKSTRING)");
        let bad_partition = SclkStringError("SPICE(BADPARTNUMBER)");
        self.validate_native_string_metadata()?;
        let totals = self.partition_totals(true)?;
        let (partition, clock) = match text.find('/') {
            Some(0) => return Err(bad_partition),
            Some(index) if index + 1 == text.len() => return Err(invalid),
            Some(index) => (Some(&text[..index]), &text[index + 1..]),
            None => (None, text),
        };
        if clock.trim_matches(' ').is_empty() {
            return Err(invalid);
        }
        let fields = native_clock_fields(clock);
        if fields.len() > self.n_fields {
            return Err(invalid);
        }
        let mut factors = alloc::vec![1.0; self.n_fields];
        for index in (0..self.n_fields - 1).rev() {
            factors[index] = factors[index + 1] * self.moduli[index + 1] as f64;
        }
        let mut ticks = 0.0;
        for (index, field) in fields.iter().enumerate() {
            let value = if field.is_empty() {
                if context.component_error {
                    return Err(invalid);
                }
                self.offsets[index] as f64
            } else {
                match core::str::from_utf8(field)
                    .ok()
                    .and_then(parse_native_clock_number)
                {
                    Some(value) => {
                        context.component_error = false;
                        value
                    }
                    None => {
                        context.component_error = true;
                        return Err(invalid);
                    }
                }
            } - self.offsets[index] as f64;
            if native_nint(value) < 0.0 {
                return Err(invalid);
            }
            ticks += value * factors[index];
        }
        ticks = native_nint(ticks);
        let starts: Vec<f64> = self
            .partition_start
            .iter()
            .copied()
            .map(native_nint)
            .collect();
        let ends: Vec<f64> = self
            .partition_end
            .iter()
            .copied()
            .map(native_nint)
            .collect();
        let part = if let Some(number) = partition {
            let value = parse_native_clock_number(number).ok_or(bad_partition)?;
            if value < 1.0 || value >= (starts.len() + 1) as f64 {
                return Err(bad_partition);
            }
            let index = value as usize - 1;
            if ticks < starts[index] || ticks > ends[index] {
                return Err(SclkStringError("SPICE(NOTINPART)"));
            }
            index
        } else {
            starts
                .iter()
                .zip(&ends)
                .position(|(&start, &end)| ticks >= start && ticks <= end)
                .ok_or(SclkStringError("SPICE(NOPARTITION)"))?
        };
        Ok(if part == 0 {
            ticks - starts[part]
        } else {
            ticks - starts[part] + totals[part - 1]
        })
    }

    fn validate_native_string_metadata(&self) -> Result<(), SclkStringError> {
        if self.n_fields > 10 {
            return Err(SclkStringError("SPICE(VALUEOUTOFRANGE)"));
        }
        match self.native_delimiter_code {
            None => Err(SclkStringError("SPICE(KERNELVARNOTFOUND)")),
            Some(1..=5) => Ok(()),
            Some(_) => Err(SclkStringError("SPICE(VALUEOUTOFRANGE)")),
        }
    }

    fn partition_totals(&self, round_endpoints: bool) -> Result<Vec<f64>, SclkStringError> {
        if self.partition_start.is_empty() {
            return Err(SclkStringError("SPICE(KERNELVARNOTFOUND)"));
        }
        if self.partition_start.len() > 9999 {
            return Err(SclkStringError("SPICE(TOOMANYPARTITIONS)"));
        }
        let mut totals = Vec::with_capacity(self.partition_start.len());
        let mut total = 0.0;
        for (&start, &end) in self.partition_start.iter().zip(&self.partition_end) {
            let (start, end) = if round_endpoints {
                (native_nint(start), native_nint(end))
            } else {
                (start, end)
            };
            total = native_nint(total + end - start);
            totals.push(total);
        }
        Ok(totals)
    }
    /// Parses a `KPL/SCLK` text kernel for the given clock.
    ///
    /// `spacecraft_id` may be given with either sign; kernel variables are
    /// looked up with the magnitude, matching NAIF kernels such as
    /// `MRO_SCLKSCET` (variables `_74`, clock id `-74`).
    pub fn parse(text: &str, spacecraft_id: i32) -> Result<Self, TimeError> {
        let tokens = tokenize(&extract_data(text)?);

        let n_fields_token = find_var(&tokens, "SCLK01_N_FIELDS", spacecraft_id)
            .ok_or(TimeError::SclkMissing("SCLK01_N_FIELDS"))?;
        let n_fields_values = parse_list(&tokens, n_fields_token)?;
        let n_fields = parse_i64(first(n_fields_values)?)?;
        if n_fields <= 0 || n_fields > 64 {
            return Err(TimeError::SclkParse);
        }
        let n_fields = n_fields as usize;

        let data_type = match find_var(&tokens, "SCLK_DATA_TYPE", spacecraft_id) {
            Some(index) => i32::try_from(parse_i64(first(parse_list(&tokens, index)?)?)?)
                .map_err(|_| TimeError::SclkParse)?,
            None => 1,
        };
        if data_type != 1 {
            return Err(TimeError::SclkUnsupported);
        }

        let time_system = match find_var(&tokens, "SCLK01_TIME_SYSTEM", spacecraft_id) {
            Some(index) => match parse_i64(first(parse_list(&tokens, index)?)?)? {
                1 => SclkTimeSystem::Tdb,
                2 => SclkTimeSystem::Tdt,
                _ => return Err(TimeError::SclkParse),
            },
            None => SclkTimeSystem::Tdb,
        };

        let moduli_index = find_var(&tokens, "SCLK01_MODULI", spacecraft_id)
            .ok_or(TimeError::SclkMissing("SCLK01_MODULI"))?;
        let moduli = parse_i64_list(&tokens, moduli_index)?;
        if moduli.len() != n_fields || moduli.iter().any(|&value| value <= 0) {
            return Err(TimeError::SclkParse);
        }

        let offsets = match find_var(&tokens, "SCLK01_OFFSETS", spacecraft_id) {
            Some(index) => parse_i64_list(&tokens, index)?,
            None => alloc::vec![0i64; n_fields],
        };
        if offsets.len() != n_fields {
            return Err(TimeError::SclkParse);
        }

        let coefficients_index = find_var(&tokens, "SCLK01_COEFFICIENTS", spacecraft_id)
            .ok_or(TimeError::SclkMissing("SCLK01_COEFFICIENTS"))?;
        let coefficient_values = parse_list(&tokens, coefficients_index)?;
        if coefficient_values.len() < 3 || coefficient_values.len() % 3 != 0 {
            return Err(TimeError::SclkParse);
        }
        let mut coefficients = Vec::new();
        let mut previous_ticks = f64::NEG_INFINITY;
        let mut previous_seconds = f64::NEG_INFINITY;
        for chunk in coefficient_values.chunks(3) {
            let ticks = parse_f64(chunk[0])?;
            let seconds = parse_f64(chunk[1])?;
            let rate = parse_f64(chunk[2])?;
            if ticks <= previous_ticks || seconds <= previous_seconds || rate <= 0.0 {
                return Err(TimeError::SclkParse);
            }
            previous_ticks = ticks;
            previous_seconds = seconds;
            coefficients.push(SclkCoefficient {
                ticks,
                seconds,
                rate,
            });
        }

        let native_delimiter_code = match find_var(&tokens, "SCLK01_OUTPUT_DELIM", spacecraft_id) {
            Some(index) => Some(parse_i64(first(parse_list(&tokens, index)?)?)?),
            None => None,
        };
        let delimiter = native_delimiter_code
            .map(delimiter_from_code)
            .unwrap_or('.');

        let partition_start = match find_var(&tokens, "SCLK_PARTITION_START", spacecraft_id) {
            Some(index) => parse_f64_list(&tokens, index)?,
            None => Vec::new(),
        };
        let partition_end = match find_var(&tokens, "SCLK_PARTITION_END", spacecraft_id) {
            Some(index) => parse_f64_list(&tokens, index)?,
            None => Vec::new(),
        };

        if partition_start.len() != partition_end.len() {
            return Err(TimeError::SclkParse);
        }
        // Only per-partition consistency is required. NAIF lets a clock
        // readout jump ahead or regress, which is exactly why partitions
        // exist, so a later partition may start at a lower readout than
        // the previous one ended. The encoded domain uses their durations.
        for (start, end) in partition_start.iter().zip(partition_end.iter()) {
            if !start.is_finite() || !end.is_finite() || start > end {
                return Err(TimeError::SclkParse);
            }
        }
        // SCTE01 and SCET01 round each accumulated raw partition duration.
        // This differs from SCENCD, which rounds each endpoint separately.
        let max_encoded_ticks = if partition_start.is_empty() {
            None
        } else {
            let total = partition_start
                .iter()
                .zip(&partition_end)
                .fold(0.0, |total, (&start, &end)| {
                    native_nint((end - start) + total)
                });
            if !total.is_finite() || total < 0.0 {
                return Err(TimeError::SclkParse);
            }
            Some(total)
        };

        Ok(Self {
            spacecraft_id,
            data_type,
            time_system,
            n_fields,
            moduli,
            offsets,
            coefficients,
            delimiter,
            native_delimiter_code,
            partition_start,
            partition_end,
            max_encoded_ticks,
        })
    }

    /// The clock id this kernel was parsed for.
    pub fn spacecraft_id(&self) -> i32 {
        self.spacecraft_id
    }

    /// The clock type. Only type 1 is supported.
    pub fn data_type(&self) -> i32 {
        self.data_type
    }

    /// The parallel time system of the coefficients.
    pub fn time_system(&self) -> SclkTimeSystem {
        self.time_system
    }

    /// Number of fields in a formatted clock string.
    pub fn n_fields(&self) -> usize {
        self.n_fields
    }

    /// Field moduli.
    pub fn moduli(&self) -> &[i64] {
        &self.moduli
    }

    /// Field offsets.
    pub fn offsets(&self) -> &[i64] {
        &self.offsets
    }

    /// Piecewise-linear coefficients.
    pub fn coefficients(&self) -> &[SclkCoefficient] {
        &self.coefficients
    }

    /// Partition readouts used to bound the cumulative encoded domain.
    pub fn partitions(&self) -> (&[f64], &[f64]) {
        (&self.partition_start, &self.partition_end)
    }

    /// Number of ticks per count of the most significant field.
    pub fn ticks_per_msb(&self) -> f64 {
        let mut product = 1.0;
        for &modulus in self.moduli[1..].iter().rev() {
            product *= modulus as f64;
        }
        product
    }

    /// Encodes a field vector as a total tick count.
    pub fn ticks_from_fields(&self, fields: &[i64]) -> Result<f64, TimeError> {
        self.ensure_partitionless()?;
        if fields.len() != self.n_fields {
            return Err(TimeError::SclkParse);
        }

        let mut ticks: i64 = 0;
        for (index, &field) in fields.iter().enumerate() {
            let value = field
                .checked_sub(self.offsets[index])
                .ok_or(TimeError::SclkParse)?;
            if value < 0 {
                return Err(TimeError::SclkParse);
            }
            ticks = if index == 0 {
                value
            } else {
                ticks
                    .checked_mul(self.moduli[index])
                    .and_then(|partial| partial.checked_add(value))
                    .ok_or(TimeError::SclkParse)?
            };
        }

        Ok(ticks as f64)
    }

    /// Decodes a total tick count into a field vector.
    ///
    /// Partition boundaries are not applied; the fields are those of the
    /// continuous tick encoding.
    pub fn fields_from_ticks(&self, ticks: f64) -> Result<Vec<i64>, TimeError> {
        self.ensure_partitionless()?;
        if !(ticks.is_finite() && ticks >= 0.0) {
            return Err(TimeError::SclkParse);
        }
        let rounded = math::round(ticks);
        if (ticks - rounded).abs() > 1e-6 {
            return Err(TimeError::SclkParse);
        }
        let mut remainder = rounded as i64;

        let mut fields = alloc::vec![0i64; self.n_fields];

        for index in (1..self.n_fields).rev() {
            let modulus = self.moduli[index];
            fields[index] = remainder % modulus + self.offsets[index];
            remainder /= modulus;
        }

        if remainder >= self.moduli[0] {
            return Err(TimeError::SclkParse);
        }
        fields[0] = remainder + self.offsets[0];

        Ok(fields)
    }

    /// Converts encoded ticks to the parallel time system, in seconds.
    pub fn parallel_seconds(&self, ticks: f64) -> Result<f64, TimeError> {
        if !ticks.is_finite() {
            return Err(TimeError::SclkParse);
        }
        let segment = self.segment_for_ticks(ticks)?;
        let rate = segment.rate / self.ticks_per_msb();
        Ok(segment.seconds + rate * (ticks - segment.ticks))
    }

    /// Converts a parallel-time value in seconds to encoded ticks.
    pub fn ticks_from_parallel(&self, seconds: f64) -> Result<f64, TimeError> {
        if !seconds.is_finite() {
            return Err(TimeError::SclkParse);
        }
        let segment = self.segment_for_seconds(seconds)?;
        if segment.rate == 0.0 {
            return Err(TimeError::SclkParse);
        }
        let rate = 1.0 / (segment.rate / self.ticks_per_msb());
        let ticks = segment.ticks + rate * (seconds - segment.seconds);
        self.validate_ticks(ticks)?;
        Ok(ticks)
    }

    /// Converts encoded ticks to `ET` (TDB seconds past J2000).
    pub fn ticks_to_et(&self, ticks: f64, lsk: &LeapSecondTable) -> Result<f64, TimeError> {
        let parallel = self.parallel_seconds(ticks)?;
        match self.time_system {
            SclkTimeSystem::Tdb => Ok(parallel),
            SclkTimeSystem::Tdt => Ok(crate::naif::tt_to_et(parallel, lsk)),
        }
    }

    /// Converts `ET` (TDB seconds past J2000) to encoded ticks.
    pub fn et_to_ticks(&self, et: f64, lsk: &LeapSecondTable) -> Result<f64, TimeError> {
        match self.time_system {
            SclkTimeSystem::Tdb => self.ticks_from_parallel(et),
            SclkTimeSystem::Tdt => self.ticks_from_parallel(crate::naif::et_to_tt(et, lsk)),
        }
    }

    /// Converts encoded ticks to a TAI epoch.
    pub fn ticks_to_tai(&self, ticks: f64, lsk: &LeapSecondTable) -> Result<Epoch, TimeError> {
        let parallel = self.parallel_seconds(ticks)?;
        Ok(match self.time_system {
            SclkTimeSystem::Tdb => et_to_tai(parallel, lsk),
            SclkTimeSystem::Tdt => tt_to_tai(parallel, lsk),
        })
    }

    // SCET01 rounds the interpolated ticks before checking the clock range.
    pub(crate) fn et_to_discrete_ticks(
        &self,
        et: f64,
        lsk: &LeapSecondTable,
    ) -> Result<f64, TimeError> {
        let parallel = match self.time_system {
            SclkTimeSystem::Tdb => et,
            SclkTimeSystem::Tdt => crate::naif::et_to_tt(et, lsk),
        };
        let segment = self.segment_for_seconds(parallel)?;
        if segment.rate <= 0.0 {
            return Err(TimeError::SclkParse);
        }
        let rate = 1.0 / (segment.rate / self.ticks_per_msb());
        let ticks = native_nint(segment.ticks + rate * (parallel - segment.seconds));
        if !ticks.is_finite()
            || ticks < native_nint(self.coefficients[0].ticks)
            || self.max_encoded_ticks.is_some_and(|max| ticks > max)
        {
            return Err(TimeError::SclkParse);
        }
        Ok(ticks)
    }

    /// Converts a TAI epoch to encoded ticks.
    pub fn tai_to_ticks(&self, epoch: Epoch, lsk: &LeapSecondTable) -> Result<f64, TimeError> {
        let parallel = match self.time_system {
            SclkTimeSystem::Tdb => tai_to_et(epoch, lsk),
            SclkTimeSystem::Tdt => tai_to_tt(epoch, lsk),
        };
        self.ticks_from_parallel(parallel)
    }

    /// Parses a formatted clock string into a field vector.
    pub fn parse_string(&self, sclk: &str) -> Result<Vec<i64>, TimeError> {
        self.ensure_partitionless()?;
        let mut fields = Vec::new();
        for part in sclk.split(self.delimiter) {
            fields.push(
                part.trim()
                    .parse::<i64>()
                    .map_err(|_| TimeError::SclkParse)?,
            );
        }
        if fields.len() != self.n_fields {
            return Err(TimeError::SclkParse);
        }
        Ok(fields)
    }

    /// Formats a field vector as a clock string.
    pub fn format_string(&self, fields: &[i64]) -> Result<String, TimeError> {
        self.ensure_partitionless()?;
        if fields.len() != self.n_fields {
            return Err(TimeError::SclkParse);
        }
        let delimiter = self.delimiter.to_string();
        let parts: Vec<String> = fields.iter().map(|value| value.to_string()).collect();
        Ok(parts.join(&delimiter))
    }

    /// Fields and strings are only defined without partition offsets in
    /// this version; keep encoded ticks available for partition-aware
    /// callers that apply the boundaries themselves.
    fn ensure_partitionless(&self) -> Result<(), TimeError> {
        if self.partition_start.is_empty() && self.partition_end.is_empty() {
            return Ok(());
        }
        if self.partition_start.len() == 1 && self.partition_start[0] == 0.0 {
            return Ok(());
        }
        Err(TimeError::SclkPartitionsUnsupported)
    }

    fn segment_for_ticks(&self, ticks: f64) -> Result<SclkCoefficient, TimeError> {
        self.validate_ticks(ticks)?;
        if self.coefficients.is_empty() {
            return Err(TimeError::SclkMissing("SCLK01_COEFFICIENTS"));
        }
        let index = self
            .coefficients
            .partition_point(|segment| segment.ticks <= ticks);
        let index = index.checked_sub(1).ok_or(TimeError::SclkParse)?;
        Ok(self.coefficients[index])
    }

    fn validate_ticks(&self, ticks: f64) -> Result<(), TimeError> {
        if !ticks.is_finite()
            || ticks < 0.0
            || self.max_encoded_ticks.is_some_and(|max| ticks > max)
            || self
                .coefficients
                .first()
                .is_some_and(|first| ticks < first.ticks)
        {
            return Err(TimeError::SclkParse);
        }
        Ok(())
    }

    fn segment_for_seconds(&self, seconds: f64) -> Result<SclkCoefficient, TimeError> {
        if !seconds.is_finite() {
            return Err(TimeError::SclkParse);
        }
        if self.coefficients.is_empty() {
            return Err(TimeError::SclkMissing("SCLK01_COEFFICIENTS"));
        }
        let index = self
            .coefficients
            .partition_point(|segment| segment.seconds <= seconds);
        let index = if index == 0 { 0 } else { index - 1 };
        Ok(self.coefficients[index])
    }
}

// f2c's d_lg10 uses this operation order instead of the platform log10.
fn native_log10(value: f64) -> f64 {
    core::f64::consts::LOG10_E * math::ln(value)
}

// CSPICE's f2c implementation evaluates the addition before floor.
fn native_nint(value: f64) -> f64 {
    if value >= 0.0 {
        math::floor(value + 0.5)
    } else {
        -math::floor(0.5 - value)
    }
}

// SCFM01 uses DPSTRF, whose DPSTR decimal conversion preserves at most
// fourteen significant digits. Integer components below 10^14 are exact.
fn native_clock_component(value: f64) -> String {
    let value = math::abs(value);
    if value < 1.0e14 {
        return format!("{value:.0}");
    }
    const INVERSE_POWERS: [f64; 17] = [
        1.0, 0.1, 0.01, 0.001, 1e-4, 1e-5, 1e-6, 1e-7, 1e-8, 1e-9, 1e-10, 1e-11, 1e-12, 1e-13,
        1e-14, 1e-15, 1e-16,
    ];
    let mut exponent = native_log10(value) as usize;
    let mut scaled = value;
    let mut remaining = exponent;
    while remaining > 16 {
        scaled *= 1e-16;
        remaining -= 16;
    }
    scaled *= INVERSE_POWERS[remaining];
    scaled = (native_nint(scaled * 1e13) + 0.125) * 1e-13;
    if scaled >= 10.0 {
        scaled *= 0.1;
        exponent += 1;
    }
    let mut output = String::new();
    for _ in 0..14 {
        let digit = scaled as u8;
        output.push(char::from(b'0' + digit));
        scaled = (scaled - f64::from(digit)) * 10.0;
    }
    for _ in 14..=exponent {
        output.push('0');
    }
    output
}

// LPARSM treats repeated non-space delimiters as blank components, while
// whitespace adjacent to another delimiter does not add a component.
fn native_clock_fields(clock: &str) -> Vec<&[u8]> {
    let bytes = clock.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    let mut begin = 0;
    let mut fields = Vec::new();
    while begin < end && fields.len() < 10 {
        while begin < end && bytes[begin] == b' ' {
            begin += 1;
        }
        let mut next = begin;
        while next < end && !b".:-, ".contains(&bytes[next]) {
            next += 1;
        }
        if next < end && bytes[next] == b' ' {
            while next < end && bytes[next] == b' ' {
                next += 1;
            }
            if next < end && !b".:-, ".contains(&bytes[next]) {
                next -= 1;
            }
        }
        let stop = next.min(begin.saturating_add(30));
        let mut field = &bytes[begin..stop];
        while field.first() == Some(&b' ') {
            field = &field[1..];
        }
        while field.last() == Some(&b' ') {
            field = &field[..field.len() - 1];
        }
        fields.push(field);
        begin = next + 1;
    }
    if end > 0 && b".:-,".contains(&bytes[end - 1]) && fields.len() < 10 {
        fields.push(&[]);
    }
    fields
}

// NPARSD accepts PI, an implicit exponent mantissa, and exponent strings
// whose digit accumulation stops on underflow. Preserve its operation order
// through the existing NAIF numeric arithmetic after lexical normalization.
fn parse_native_clock_number(token: &str) -> Option<f64> {
    let text: String = token
        .chars()
        .filter(|c| *c != ' ' && *c != ',')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let (sign, unsigned) = match text.as_bytes().first() {
        Some(b'-') => ("-", &text[1..]),
        Some(b'+') => ("+", &text[1..]),
        _ => ("", text.as_str()),
    };
    if unsigned == "PI" {
        return Some(if sign == "-" {
            -core::f64::consts::PI
        } else {
            core::f64::consts::PI
        });
    }
    let position = unsigned.find(['E', 'D']);
    let Some(position) = position else {
        return crate::naif::parse_f64(&text);
    };
    let mantissa = &unsigned[..position];
    let mut integer = 0.0;
    let mut fraction = 0.0;
    let mut decimal = false;
    let mut zero_integer = true;
    let mut shift = 0_i64;
    let mut digits_found = false;
    for byte in mantissa.bytes() {
        if byte == b'.' && !decimal {
            decimal = true;
            zero_integer = integer == 0.0;
            continue;
        }
        if !byte.is_ascii_digit() {
            return None;
        }
        digits_found = true;
        let digit = f64::from(byte - b'0');
        if !decimal {
            if integer < 1e15 {
                integer = integer * 10.0 + digit;
            } else {
                shift += 1;
            }
        } else if zero_integer && fraction < 1e15 {
            fraction = fraction * 10.0 + digit;
            shift -= 1;
        }
    }
    let raw_exponent = &unsigned[position + 1..];
    let (exp_sign, exponent) = if let Some(value) = raw_exponent.strip_prefix('-') {
        (-1_i64, value)
    } else {
        (
            1_i64,
            raw_exponent.strip_prefix('+').unwrap_or(raw_exponent),
        )
    };
    let mut power = 0_i64;
    for digit in exponent.bytes() {
        if !digit.is_ascii_digit() || power + shift > 308 {
            return None;
        }
        if power + shift >= -309 {
            power = power * 10 + exp_sign * i64::from(digit - b'0');
        }
    }
    let mantissa = if digits_found { mantissa } else { "1" };
    crate::naif::parse_f64(&format!("{sign}{mantissa}E{power}"))
}

/// A spacecraft clock bound to a leap-second kernel.
///
/// The [`SclkMap`] trait needs the LSK for the `TT -> ET` step of clocks
/// whose coefficients are in TT.
pub struct SclkClock<'a> {
    /// Parsed SCLK kernel.
    pub kernel: &'a SclkKernel,
    /// Leap-second kernel used to relate TT and ET.
    pub lsk: &'a LeapSecondTable,
}

impl SclkMap for SclkClock<'_> {
    fn sclk_to_et(&self, sclk: &str) -> Result<f64, TimeError> {
        if !self.kernel.partition_start.is_empty()
            && (sclk.contains('/') || self.kernel.ensure_partitionless().is_err())
        {
            let ticks = self
                .kernel
                .encode_native_sclk(sclk)
                .map_err(|_| TimeError::SclkParse)?;
            return self.kernel.ticks_to_et(ticks, self.lsk);
        }
        let fields = self.kernel.parse_string(sclk)?;
        let ticks = self.kernel.ticks_from_fields(&fields)?;
        self.kernel.ticks_to_et(ticks, self.lsk)
    }

    fn et_to_sclk(&self, et: f64) -> Result<String, TimeError> {
        if self.kernel.ensure_partitionless().is_err() {
            let ticks = self.kernel.et_to_discrete_ticks(et, self.lsk)?;
            return self
                .kernel
                .decode_native_sclk(ticks)
                .map_err(|_| TimeError::SclkParse);
        }
        let ticks = self.kernel.et_to_ticks(et, self.lsk)?;
        let fields = self.kernel.fields_from_ticks(ticks)?;
        self.kernel.format_string(&fields)
    }
}

/// Converts between a spacecraft clock string and `ET` (TDB seconds past
/// J2000).
pub trait SclkMap {
    /// Converts an SCLK string to `ET`.
    fn sclk_to_et(&self, sclk: &str) -> Result<f64, TimeError>;

    /// Converts `ET` to an SCLK string.
    fn et_to_sclk(&self, et: f64) -> Result<String, TimeError>;
}

/// Extracts every `\begindata` ... `\begintext` block.
fn extract_data(text: &str) -> Result<String, TimeError> {
    const BEGIN: &str = "\\begindata";
    const END: &str = "\\begintext";

    let mut data = String::new();
    let mut rest = text;
    let mut found = false;

    while let Some(start) = rest.find(BEGIN) {
        found = true;
        let after = &rest[start + BEGIN.len()..];
        let end = after.find(END).unwrap_or(after.len());
        data.push_str(&after[..end]);
        data.push('\n');
        if end == after.len() {
            break;
        }
        rest = &after[end + END.len()..];
    }

    if found {
        Ok(data)
    } else {
        Err(TimeError::SclkParse)
    }
}

/// Splits on whitespace and the assignment punctuation.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '=' | '(' | ')' | ',' => {
                if !current.is_empty() {
                    tokens.push(core::mem::take(&mut current));
                }
                tokens.push(ch.to_string());
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(core::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Finds the index of a kernel variable token, trying both id signs.
fn find_var(tokens: &[String], base: &str, spacecraft_id: i32) -> Option<usize> {
    let magnitude = i64::from(spacecraft_id).abs();
    let candidates = [
        format!("{base}_{spacecraft_id}"),
        format!("{base}_{magnitude}"),
        format!("{base}_-{magnitude}"),
    ];
    tokens
        .iter()
        .rposition(|token| candidates.iter().any(|candidate| candidate == token))
}

/// Reads one scalar or the parenthesized list following `name =`.
fn parse_list(tokens: &[String], name_index: usize) -> Result<Vec<&str>, TimeError> {
    let mut index = name_index + 1;
    expect(tokens, index, "=")?;
    index += 1;
    let value = tokens.get(index).ok_or(TimeError::SclkParse)?;
    if value != "(" {
        if matches!(value.as_str(), "=" | ")" | ",")
            || tokens.get(index + 1).is_some_and(|token| token == "=")
        {
            return Err(TimeError::SclkParse);
        }
        return Ok(alloc::vec![value.as_str()]);
    }
    index += 1;

    let mut values = Vec::new();
    while index < tokens.len() && tokens[index] != ")" {
        if matches!(tokens[index].as_str(), "(" | "=") {
            return Err(TimeError::SclkParse);
        }
        if tokens[index] != "," {
            values.push(tokens[index].as_str());
        }
        index += 1;
    }
    if index >= tokens.len() {
        return Err(TimeError::SclkParse);
    }
    Ok(values)
}

fn expect(tokens: &[String], index: usize, expected: &str) -> Result<(), TimeError> {
    if tokens.get(index).map(String::as_str) == Some(expected) {
        Ok(())
    } else {
        Err(TimeError::SclkParse)
    }
}

fn first(values: Vec<&str>) -> Result<&str, TimeError> {
    values.into_iter().next().ok_or(TimeError::SclkParse)
}

fn parse_i64(token: &str) -> Result<i64, TimeError> {
    if let Ok(value) = token.parse::<i64>() {
        return Ok(value);
    }
    // Kernel-pool integer retrieval accepts numeric exponents and rounds
    // to the nearest integer (ties away from zero), as NAIF GIPOOL does.
    let value = math::round(parse_f64(token)?);
    if value < i64::MIN as f64 || value >= -(i64::MIN as f64) {
        return Err(TimeError::SclkParse);
    }
    Ok(value as i64)
}

fn parse_f64(token: &str) -> Result<f64, TimeError> {
    crate::naif::parse_f64(token).ok_or(TimeError::SclkParse)
}

fn parse_i64_list(tokens: &[String], name_index: usize) -> Result<Vec<i64>, TimeError> {
    parse_list(tokens, name_index)?
        .into_iter()
        .map(parse_i64)
        .collect()
}

fn parse_f64_list(tokens: &[String], name_index: usize) -> Result<Vec<f64>, TimeError> {
    parse_list(tokens, name_index)?
        .into_iter()
        .map(parse_f64)
        .collect()
}

fn delimiter_from_code(code: i64) -> char {
    match code {
        2 => ':',
        3 => '-',
        4 => ',',
        5 => ' ',
        _ => '.',
    }
}
