# Core-to-JavaScript API inventory

Every public declaration is classified below; aliases are named JS methods,
metadata fields or explicit language protocols. Checked constructors replace
Rust infallible constructors. Rust borrowing/lifetimes and Display are owned
objects/options and strings. SclkMap/Clock are SclkKernel plus TimeContext.
Format/Formatter are TimeContext.format with options and pattern.

Exact integer inputs use bigint (Epoch/BDT seconds, duration total ns, JDN,
SCLK fields); bounded civil fields use finite integer numbers; ET and numeric
clock ticks preserve binary64 bits. Metadata is detached. Test coverage is
shared tests/suite.mjs in Node/Chrome plus strict types.ts and clean installed
npm consumer. Category coverage is not a claim that every redundant Rust
spelling has an independent oracle test.

Legacy floating scale paths are explicitly named by scale/model. The
compatibility namespace delegates those same paths; UTC compositions use
TimeContext plus EopTable rather than extra floating Epoch round trips.
Constants JDN_1960/JDN_1972 are exported alongside the other time constants.
Duration ZERO/NANOSECOND/SECOND use new Duration(0n/1n/1000000000n).
Errors support semantic instanceof classification and stable .code. MCP
transport internals are deliberately unavailable in the bindings.

Civil inverse formatting rejects |TAI seconds| > 6e16 before i32 year
conversion. Numeric etcal/JD formatting rejects magnitude > 1e12. Exact
Epoch/Duration arithmetic retains the full core i64 seconds range. These
foreign-boundary limits protect core calendar narrowing; they do not change
the core. Low-level unchecked dtdb functions retain explicit extrapolation;
ScientificContext enforces its model interval.

| Core declaration | Kind | JS spelling/protocol | Owner/category |
|---|---|---|---|
| bdt.TAI_MINUS_BDT_SECONDS | const | constants object or Duration 0n/1n/1000000000n value protocol | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.GPS_MINUS_BDT_SECONDS | const | constants object or Duration 0n/1n/1000000000n value protocol | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.ORIGIN_TAI_SECONDS | const | constants object or Duration 0n/1n/1000000000n value protocol | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.NANOSECONDS_PER_WEEK | const | constants object or Duration 0n/1n/1000000000n value protocol | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt | struct | Bdt exact class; bigint seconds/nanoseconds/week | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.checked_from_parts | fn | constructor / Duration.fromParts | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.checked_from_nanoseconds | fn | Bdt.fromNanoseconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.checked_from_seconds | fn | Bdt.fromSeconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.sec | fn | seconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.nanos | fn | nanoseconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.seconds | fn | toSeconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.total_nanoseconds | fn | totalNanoseconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.from_epoch | fn | fromEpoch | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.to_epoch | fn | toEpoch | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.from_week_nanoseconds | fn | fromWeekNanoseconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.week_nanoseconds | fn | weekNanoseconds | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.from_utc | fn | fromUtc | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.to_utc | fn | toUtc | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.from_et | fn | fromEt | Bdt exact class; bigint seconds/nanoseconds/week |
| bdt.Bdt.to_et | fn | toEt | Bdt exact class; bigint seconds/nanoseconds/week |
| calendar.J2000_JDN | const | constants object or Duration 0n/1n/1000000000n value protocol | calendar methods; bigint JDN |
| calendar.GPS_EPOCH_JDN | const | constants object or Duration 0n/1n/1000000000n value protocol | calendar methods; bigint JDN |
| calendar.GREGORIAN_SWITCH_JDN | const | constants object or Duration 0n/1n/1000000000n value protocol | calendar methods; bigint JDN |
| calendar.Calendar | enum | calendar methods; bigint JDN | calendar methods; bigint JDN |
| calendar.is_gregorian_leap | fn | isGregorianLeap | calendar methods; bigint JDN |
| calendar.is_julian_leap | fn | isJulianLeap | calendar methods; bigint JDN |
| calendar.days_in_month | fn | daysInMonth | calendar methods; bigint JDN |
| calendar.gregorian_to_jdn | fn | calendar.toJdn Gregorian | calendar methods; bigint JDN |
| calendar.julian_to_jdn | fn | calendar.toJdn Julian | calendar methods; bigint JDN |
| calendar.jdn_to_gregorian | fn | calendar.fromJdn Gregorian | calendar methods; bigint JDN |
| calendar.jdn_to_julian | fn | calendar.fromJdn Julian | calendar methods; bigint JDN |
| calendar.jdn_to_calendar | fn | calendar.fromJdn | calendar methods; bigint JDN |
| calendar.calendar_to_jdn | fn | calendar.toJdn | calendar methods; bigint JDN |
| calendar.calendar_to_jd | fn | calendar.toJd | calendar methods; bigint JDN |
| calendar.jd_to_mjd | fn | jdToMjd | calendar methods; bigint JDN |
| calendar.mjd_to_jd | fn | mjdToJd | calendar methods; bigint JDN |
| contract.CalculationProfile | enum | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ContractError | enum | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifEt | struct | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifJulianUtc | struct | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifJulianUtc.days | fn | days | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifEt.from_seconds | fn | fromSeconds | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifEt.seconds | fn | seconds | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifEt.profile | fn | profile | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ScientificModel | enum | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.IauTdb | struct | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.IauTdb.seconds | fn | seconds | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.IauTdb.model | fn | model | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.IauTdb.profile | fn | profile | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.NaifMetadata | struct | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext | struct | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.new | fn | constructor/options | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.et_to_spice_julian_utc | fn | etToSpiceJulianUtc | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.profile | fn | profile | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.metadata | fn | TimeContext.metadata | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.to_et | fn | toEt | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.utc_to_et | fn | compatibility.utcToEt (legacy); TimeContext.utcToEt (checked) | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.to_epoch | fn | toEpoch | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.TimeContext.et_to_utc | fn | compatibility.etToUtc (legacy); TimeContext.etToUtc (checked) | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ScientificContext | struct | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ScientificContext.new | fn | constructor/options | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ScientificContext.tt_seconds | fn | ttSeconds | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ScientificContext.to_tdb | fn | toTdb | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| contract.ScientificContext.to_tdb_at_observer | fn | toTdbAtObserver | TimeContext, NaifEt, NaifJulianUtc, NaifUtcContext, ScientificContext, IauTdb |
| convert.utc_to_tai | fn | TimeContext.utcToEpoch | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.tai_to_utc | fn | TimeContext.epochToUtc | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.tai_to_et | fn | compatibility.taiToEt (legacy); TimeContext.toEt (checked) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.et_to_tai | fn | compatibility.etToTai (legacy); TimeContext.toEpoch (checked) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.tai_to_tt | fn | toScale(TT) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.tt_to_tai | fn | fromScale(TT) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.tai_to_gps | fn | toScale(GPS) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.gps_to_tai | fn | fromScale(GPS) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.utc_to_et | fn | compatibility.utcToEt (legacy); TimeContext.utcToEt (checked) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.et_to_utc | fn | compatibility.etToUtc (legacy); TimeContext.etToUtc (checked) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.ut1_minus_utc | fn | EopTable.ut1MinusUtc (conversion) / interpolate (raw EOP) | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.tai_to_ut1 | fn | EopTable.toUt1 | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.ut1_to_tai | fn | EopTable.fromUt1 | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.utc_to_ut1 | fn | TimeContext.utcToUt1 | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| convert.ut1_to_utc | fn | TimeContext.ut1ToUtc | TimeContext UTC/ET and named toScale/fromScale; EopTable UT1 methods |
| coord.L_G | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.L_B | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.TDB0 | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.tcg_minus_tt | fn | TimeContext.tcgMinusTt | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.tai_to_tcg | fn | toScale(TCG) | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.tcg_to_tai | fn | fromScale(TCG) | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.tcb_minus_tdb | fn | tcbMinusTdb | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.tai_to_tcb | fn | toScale(TCB.IAU) | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| coord.tcb_to_tai | fn | fromScale(TCB.IAU) | TimeContext explicit TCG/TCB.IAU scales; models.tcbMinusTdb |
| dtdb.dtdb_minus_tt | fn | models.dtdbMinusTt | ScientificContext checked TDB; models explicit dtdb functions |
| dtdb.dtdb_minus_tt_split | fn | models.dtdbMinusTtSplit | ScientificContext checked TDB; models explicit dtdb functions |
| dtdb.DtdbError | enum | ScientificContext checked TDB; models explicit dtdb functions | ScientificContext checked TDB; models explicit dtdb functions |
| dtdb.dtdb_minus_tt_at_observer | fn | models.dtdbAtObserver | ScientificContext checked TDB; models explicit dtdb functions |
| dtdb.tai_to_tdb_iau | fn | ScientificContext.toTdb | ScientificContext checked TDB; models explicit dtdb functions |
| dtdb.tdb_iau_to_tai | fn | fromScale(TDB.IAU) | ScientificContext checked TDB; models explicit dtdb functions |
| duration.Duration | struct | Duration checked class and bigint comparison protocol | Duration checked class and bigint comparison protocol |
| duration.ZERO | const | constants object or Duration 0n/1n/1000000000n value protocol | Duration checked class and bigint comparison protocol |
| duration.NANOSECOND | const | constants object or Duration 0n/1n/1000000000n value protocol | Duration checked class and bigint comparison protocol |
| duration.SECOND | const | constants object or Duration 0n/1n/1000000000n value protocol | Duration checked class and bigint comparison protocol |
| duration.Duration.from_parts | fn | checked constructor / Duration.fromParts | Duration checked class and bigint comparison protocol |
| duration.Duration.checked_from_parts | fn | constructor / Duration.fromParts | Duration checked class and bigint comparison protocol |
| duration.Duration.from_nanoseconds | fn | Duration constructor | Duration checked class and bigint comparison protocol |
| duration.Duration.from_seconds_f64 | fn | Duration.fromSeconds | Duration checked class and bigint comparison protocol |
| duration.Duration.checked_from_seconds_f64 | fn | Duration.fromSeconds | Duration checked class and bigint comparison protocol |
| duration.Duration.sec | fn | seconds | Duration checked class and bigint comparison protocol |
| duration.Duration.nanos | fn | nanoseconds | Duration checked class and bigint comparison protocol |
| duration.Duration.total_nanoseconds | fn | totalNanoseconds | Duration checked class and bigint comparison protocol |
| duration.Duration.seconds_f64 | fn | Duration.toSeconds | Duration checked class and bigint comparison protocol |
| duration.Duration.is_zero | fn | totalNanoseconds === 0n protocol | Duration checked class and bigint comparison protocol |
| duration.Duration.signum | fn | bigint comparison protocol | Duration checked class and bigint comparison protocol |
| duration.Duration.abs | fn | abs | Duration checked class and bigint comparison protocol |
| duration.Duration.checked_neg | fn | negate | Duration checked class and bigint comparison protocol |
| duration.Duration.checked_add | fn | add | Duration checked class and bigint comparison protocol |
| duration.Duration.checked_sub | fn | subtract | Duration checked class and bigint comparison protocol |
| duration.Duration.checked_mul_i64 | fn | multiply | Duration checked class and bigint comparison protocol |
| duration.Duration.floor | fn | floor | Duration checked class and bigint comparison protocol |
| duration.Duration.ceil | fn | ceil | Duration checked class and bigint comparison protocol |
| duration.Duration.round | fn | round | Duration checked class and bigint comparison protocol |
| eop.EopSample | struct | EopTable constructor/fromSamples, metadata, observedOnly, interpolate | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopSeries | enum | EopTable constructor/fromSamples, metadata, observedOnly, interpolate | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable | struct | EopTable constructor/fromSamples, metadata, observedOnly, interpolate | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.new | fn | constructor/options | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.from_finals2000a | fn | EopTable constructor | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.from_finals2000a_series | fn | EopTable constructor series | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.observed_only | fn | observedOnly | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.ut1_minus_utc | fn | EopTable.ut1MinusUtc (conversion) / interpolate (raw EOP) | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.range | fn | metadata.range | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.samples | fn | metadata.samples | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.len | fn | metadata.samples.length | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| eop.EopTable.is_empty | fn | metadata.samples.length === 0 | EopTable constructor/fromSamples, metadata, observedOnly, interpolate |
| epoch.Epoch | struct | Epoch / UtcDateTime checked classes | Epoch / UtcDateTime checked classes |
| epoch.Epoch.from_parts | fn | checked constructor / Duration.fromParts | Epoch / UtcDateTime checked classes |
| epoch.Epoch.checked_from_parts | fn | constructor / Duration.fromParts | Epoch / UtcDateTime checked classes |
| epoch.Epoch.from_tai_seconds | fn | Epoch.fromTaiSeconds | Epoch / UtcDateTime checked classes |
| epoch.Epoch.checked_from_tai_seconds | fn | Epoch.fromTaiSeconds | Epoch / UtcDateTime checked classes |
| epoch.Epoch.tai_seconds | fn | Epoch.toTaiSeconds | Epoch / UtcDateTime checked classes |
| epoch.Epoch.sec | fn | seconds | Epoch / UtcDateTime checked classes |
| epoch.Epoch.nanos | fn | nanoseconds | Epoch / UtcDateTime checked classes |
| epoch.Epoch.checked_add | fn | add | Epoch / UtcDateTime checked classes |
| epoch.Epoch.checked_sub | fn | subtract | Epoch / UtcDateTime checked classes |
| epoch.UtcDateTime | struct | Epoch / UtcDateTime checked classes | Epoch / UtcDateTime checked classes |
| epoch.UtcDateTime.new | fn | constructor/options | Epoch / UtcDateTime checked classes |
| error.TimeError | enum | TimeError/ContractError/DtdbError code classification | TimeError/ContractError/DtdbError code classification |
| format.FormatOptions | struct | TimeContext.format options/style/pattern and formatting numeric helpers | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.FormatOptions.new | fn | constructor/options | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.FormatOptions.with_calendar | fn | FormatOptions.calendar | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.FormatOptions.with_leap_seconds | fn | TimeContext owns LSK | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.FormatOptions.precision | fn | FormatOptions.precision | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.FormatOptions.calendar | fn | FormatOptions.calendar | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.iso8601 | fn | TimeContext.format style=iso8601 | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.iso8601_with | fn | TimeContext.format style=iso8601 | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.spice_calendar | fn | TimeContext.format style=spice | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.spice_calendar_with | fn | TimeContext.format style=spice | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.etcal | fn | formatting.etcal | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.jd_to_calendar | fn | formatting.jdToCalendar | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.rfc3339 | fn | TimeContext.format style=rfc3339 | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.rfc3339_with | fn | TimeContext.format style=rfc3339 | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.rfc2822 | fn | TimeContext.format style=rfc2822 | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.Format | struct | TimeContext.format options/style/pattern and formatting numeric helpers | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.Format.parse | fn | SclkKernel constructor / parseString; formatting pattern option | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.Formatter | struct | TimeContext.format options/style/pattern and formatting numeric helpers | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.Formatter.new | fn | constructor/options | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.Formatter.with_options | fn | TimeContext.format options | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.strftime | fn | TimeContext.format style=strftime | TimeContext.format options/style/pattern and formatting numeric helpers |
| format.strftime_with | fn | TimeContext.format style=strftime | TimeContext.format options/style/pattern and formatting numeric helpers |
| kernel_date.KernelDate | struct | parseKernelDate result record | parseKernelDate result record |
| kernel_date.parse_kernel_date | fn | parseKernelDate | parseKernelDate result record |
| leap.DELTA_T_A | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.K | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.EB | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.M0 | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.M1 | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.JDN_1960 | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.JDN_1972 | const | constants object or Duration 0n/1n/1000000000n value protocol | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapEntry | struct | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondSource | enum | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondPolicy | enum | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondPolicy.frozen_after | fn | select({frozenAfterJdn: calendar.toJdn(...)}) | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable | struct | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.new | fn | constructor/options | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.with_constants | fn | TimeContext.fromEntries deltet argument | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.builtin | fn | TimeContext constructor | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.from_lsk | fn | TimeContext constructor | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.with_source | fn | TimeContext.select | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.source | fn | TimeContext.metadata.source | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.with_policy | fn | TimeContext.select | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.policy | fn | TimeContext.metadata.policy | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.delta_at_f64 | fn | TimeContext.deltaAt | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.entries | fn | TimeContext.metadata.entries | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.coverage | fn | TimeContext.metadata.coverage | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.LeapSecondTable.delta_at_for_jdn | fn | TimeContext.deltaAtForJdn (nullable integer, distinct from floating SOFA drift) | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| leap.sofa_delta_at | fn | sofaDeltaAt | TimeContext.fromEntries, select, metadata, deltaAt; models.sofaDeltaAt |
| lsk.parse_lsk | fn | TimeContext constructor | TimeContext constructor (builtin or LSK text/bytes) |
| lsk.builtin | fn | TimeContext constructor | TimeContext constructor (builtin or LSK text/bytes) |
| mcp.run_stdio | fn | Unavailable: Rust/server MCP transport; not a scientific binding API | Unavailable: Rust/server MCP transport; not a scientific binding API |
| mcp.handle_message | fn | Unavailable: Rust/server MCP transport; not a scientific binding API | Unavailable: Rust/server MCP transport; not a scientific binding API |
| mcp.handle_message_with_sclk_context | fn | Unavailable: Rust/server MCP transport; not a scientific binding API | Unavailable: Rust/server MCP transport; not a scientific binding API |
| sclk.SclkTimeSystem | enum | SclkKernel methods/metadata plus explicit TimeContext | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkCoefficient | struct | SclkKernel methods/metadata plus explicit TimeContext | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkStringError | struct | Unavailable: native partition-aware string API not yet exposed in JavaScript | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkStringError.spice_short | fn | Unavailable: native partition-aware string API not yet exposed in JavaScript | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkStringContext | struct | Unavailable: native partition-aware string API not yet exposed in JavaScript | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel | struct | SclkKernel methods/metadata plus explicit TimeContext | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.decode_native_sclk | fn | Unavailable: native partition-aware string API not yet exposed in JavaScript | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.encode_native_sclk | fn | Unavailable: native partition-aware string API not yet exposed in JavaScript | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.encode_native_sclk_with_context | fn | Unavailable: native partition-aware string API not yet exposed in JavaScript | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.parse | fn | SclkKernel constructor / parseString; formatting pattern option | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.spacecraft_id | fn | metadata.spacecraftId | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.data_type | fn | metadata.dataType | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.time_system | fn | metadata.timeSystem | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.n_fields | fn | metadata.nFields | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.moduli | fn | metadata.moduli | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.offsets | fn | metadata.offsets | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.coefficients | fn | metadata.coefficients | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.partitions | fn | metadata.partitionStarts/partitionEnds | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.ticks_per_msb | fn | metadata.ticksPerMsb | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.ticks_from_fields | fn | ticksFromFields | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.fields_from_ticks | fn | fieldsFromTicks | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.parallel_seconds | fn | parallelSeconds | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.ticks_from_parallel | fn | ticksFromParallel | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.ticks_to_et | fn | ticksToEt | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.et_to_ticks | fn | etToTicks | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.ticks_to_tai | fn | ticksToEpoch | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.tai_to_ticks | fn | epochToTicks | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.parse_string | fn | parseString | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkKernel.format_string | fn | formatString | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkClock | struct | Legacy field composition only; native partition-aware strings unavailable | SclkKernel methods/metadata plus explicit TimeContext |
| sclk.SclkMap | trait | Legacy field composition only; native partition-aware strings unavailable | SclkKernel methods/metadata plus explicit TimeContext |
| series.TimeSeries | struct | TimeSeries lazy iterable; inclusive option | TimeSeries lazy iterable; inclusive option |
| series.TimeSeries.inclusive | fn | TimeSeries inclusive:true | TimeSeries lazy iterable; inclusive option |
| series.TimeSeries.exclusive | fn | TimeSeries inclusive:false | TimeSeries lazy iterable; inclusive option |
| tdb.tdb_minus_tt_seconds | fn | models.compactMinusTt | TimeContext TDB.Compact scale; models.compactMinusTt |
| tdb.tai_to_tdb | fn | toScale(TDB.Compact) | TimeContext TDB.Compact scale; models.compactMinusTt |
| tdb.tdb_to_tai | fn | fromScale(TDB.Compact) | TimeContext TDB.Compact scale; models.compactMinusTt |
| timescale.TimeScale | enum | Scale string union plus UTC/TAI/BDT class methods | Scale string union plus UTC/TAI/BDT class methods |
| wgc_gps.WgcGpsError | enum | Unavailable: WGC GPS week/TOW formatting is not yet exposed in JavaScript | Unavailable: WGC GPS week/TOW formatting is not yet exposed in JavaScript |
| wgc_gps.gps_seconds_to_week_tow | fn | Unavailable: WGC GPS week/TOW formatting is not yet exposed in JavaScript | Unavailable: WGC GPS week/TOW formatting is not yet exposed in JavaScript |
| contract.naif_utc.NaifDeltet | struct | five-element deltet tuple passed to NaifUtcContext | Explicit raw native JDUTC context |
| contract.naif_utc.NaifUtcContext | struct | NaifUtcContext class | Explicit raw native JDUTC context |
| contract.naif_utc.from_raw_parts | fn | NaifUtcContext constructor | Explicit raw native JDUTC context |
| contract.naif_utc.et_to_spice_julian_utc | fn | NaifUtcContext.etToSpiceJulianUtc | Explicit raw native JDUTC context |
