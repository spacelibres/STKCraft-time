# Python API coverage inventory

Every public declaration in core src/**/*.rs is listed with its source line.
The Rust core performs all time calculations. Checked constructors raise
TimeError/OverflowError rather than returning Option. Comparison, hashing,
operators and iteration map to Python protocols. Lifetimes map to owned,
immutable kernel/configuration snapshots and ephemeral native borrows.

Each wrapped entry is validated for existence by test_inventory.py. Golden
numerical tests cover the checked UTC, legacy UTC, SCLK, historical UTC,
ERFA geocentric/observer TDB, TCG/TCB, BDT and raw/native JDUTC categories.
Calendar/format/options, EOP parsing/interpolation/UT1, metadata, errors,
series, arithmetic and batch behavior have adapter smoke/boundary tests.
This is a capability inventory, not a claim of individual branch coverage.

The low-level compat namespace deliberately retains the legacy floating
conversion routes. Traits have protocol equivalents; MCP transport functions
remain outside the scientific package. No public scientific declaration is
intentionally omitted. Private decimal/parser internals are exercised through
SCLK/LSK/kernel-date parsers and are not exported as a second scientific API.

| Core declaration | Python surface | Status | Mapping |
| --- | --- | --- | --- |
| `bdt::TAI_MINUS_BDT_SECONDS` | `TAI_MINUS_BDT_SECONDS` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::GPS_MINUS_BDT_SECONDS` | `GPS_MINUS_BDT_SECONDS` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::ORIGIN_TAI_SECONDS` | `ORIGIN_TAI_SECONDS` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::NANOSECONDS_PER_WEEK` | `NANOSECONDS_PER_WEEK` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt` | `Bdt` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::checked_from_parts` | `Bdt` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `bdt::Bdt::checked_from_nanoseconds` | `Bdt.from_nanoseconds` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `bdt::Bdt::checked_from_seconds` | `Bdt.from_seconds` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `bdt::Bdt::sec` | `Bdt.sec` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::nanos` | `Bdt.nanos` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::seconds` | `Bdt.seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::total_nanoseconds` | `Bdt.total_nanoseconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::from_epoch` | `Bdt.from_epoch` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::to_epoch` | `Bdt.to_epoch` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::from_week_nanoseconds` | `Bdt.from_week_nanoseconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::week_nanoseconds` | `Bdt.week_nanoseconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::from_utc` | `Bdt.from_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::to_utc` | `Bdt.to_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::from_et` | `Bdt.from_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `bdt::Bdt::to_et` | `Bdt.to_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::J2000_JDN` | `J2000_JDN` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::GPS_EPOCH_JDN` | `GPS_EPOCH_JDN` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::GREGORIAN_SWITCH_JDN` | `GREGORIAN_SWITCH_JDN` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::Calendar` | `Calendar` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::is_gregorian_leap` | `is_gregorian_leap` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::is_julian_leap` | `is_julian_leap` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::days_in_month` | `days_in_month` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::gregorian_to_jdn` | `gregorian_to_jdn` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::julian_to_jdn` | `julian_to_jdn` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::jdn_to_gregorian` | `jdn_to_gregorian` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::jdn_to_julian` | `jdn_to_julian` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::jdn_to_calendar` | `jdn_to_calendar` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::calendar_to_jdn` | `calendar_to_jdn` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::calendar_to_jd` | `calendar_to_jd` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::jd_to_mjd` | `jd_to_mjd` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `calendar::mjd_to_jd` | `mjd_to_jd` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifDeltet` | `NaifDeltet` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifUtcContext` | `NaifUtcContext` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifUtcContext::from_raw_parts` | `NaifUtcContext.from_raw_parts` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifUtcContext::et_to_spice_julian_utc` | `NaifUtcContext.et_to_spice_julian_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::CalculationProfile` | `CalculationProfile` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ContractError` | `ContractError` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifEt` | `NaifEt` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifJulianUtc` | `NaifJulianUtc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifJulianUtc::days` | `NaifJulianUtc.days` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifEt::from_seconds` | `NaifEt.from_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifEt::seconds` | `NaifEt.seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifEt::profile` | `NaifEt.profile` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ScientificModel` | `ScientificModel` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::IauTdb` | `IauTdb` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::IauTdb::seconds` | `IauTdb.seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::IauTdb::model` | `IauTdb.model` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::IauTdb::profile` | `IauTdb.profile` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::NaifMetadata` | `NaifMetadata` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext` | `TimeContext` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::new` | `TimeContext` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::et_to_spice_julian_utc` | `TimeContext.et_to_spice_julian_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::profile` | `TimeContext.profile` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::metadata` | `TimeContext.metadata` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::to_et` | `TimeContext.to_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::utc_to_et` | `TimeContext.utc_to_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::to_epoch` | `TimeContext.to_epoch` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::TimeContext::et_to_utc` | `TimeContext.et_to_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ScientificContext` | `ScientificContext` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ScientificContext::new` | `ScientificContext` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ScientificContext::tt_seconds` | `ScientificContext.tt_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ScientificContext::to_tdb` | `ScientificContext.to_tdb` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `contract::ScientificContext::to_tdb_at_observer` | `ScientificContext.to_tdb_at_observer` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::utc_to_tai` | `compat.utc_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::tai_to_utc` | `compat.tai_to_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::tai_to_et` | `compat.tai_to_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::et_to_tai` | `compat.et_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::tai_to_tt` | `compat.tai_to_tt` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::tt_to_tai` | `compat.tt_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::tai_to_gps` | `compat.tai_to_gps` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::gps_to_tai` | `compat.gps_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::utc_to_et` | `compat.utc_to_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::et_to_utc` | `compat.et_to_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::ut1_minus_utc` | `compat.ut1_minus_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::tai_to_ut1` | `compat.tai_to_ut1` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::ut1_to_tai` | `compat.ut1_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::utc_to_ut1` | `compat.utc_to_ut1` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `convert::ut1_to_utc` | `compat.ut1_to_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::L_G` | `L_G` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::L_B` | `L_B` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::TDB0` | `TDB0` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::tcg_minus_tt` | `compat.tcg_minus_tt` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::tai_to_tcg` | `compat.tai_to_tcg` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::tcg_to_tai` | `compat.tcg_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::tcb_minus_tdb` | `tcb_minus_tdb` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::tai_to_tcb` | `compat.tai_to_tcb` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `coord::tcb_to_tai` | `compat.tcb_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `dtdb::dtdb_minus_tt` | `dtdb_minus_tt` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `dtdb::dtdb_minus_tt_split` | `dtdb_minus_tt_split` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `dtdb::DtdbError` | `DtdbError` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `dtdb::dtdb_minus_tt_at_observer` | `dtdb_minus_tt_at_observer` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `dtdb::tai_to_tdb_iau` | `compat.tai_to_tdb_iau` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `dtdb::tdb_iau_to_tai` | `compat.tdb_iau_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration` | `Duration` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::ZERO` | `Duration.ZERO` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::NANOSECOND` | `Duration.NANOSECOND` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::SECOND` | `Duration.SECOND` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::from_parts` | `Duration` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::checked_from_parts` | `Duration` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `duration::Duration::from_nanoseconds` | `Duration.from_nanoseconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::from_seconds_f64` | `Duration.from_seconds_f64` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::checked_from_seconds_f64` | `Duration.from_seconds_f64` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `duration::Duration::sec` | `Duration.sec` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::nanos` | `Duration.nanos` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::total_nanoseconds` | `Duration.total_nanoseconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::seconds_f64` | `Duration.seconds_f64` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::is_zero` | `Duration.is_zero` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::signum` | `Duration.signum` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::abs` | `Duration.abs` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::checked_neg` | `Duration.checked_neg` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::checked_add` | `Duration.checked_add` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::checked_sub` | `Duration.checked_sub` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::checked_mul_i64` | `Duration.checked_mul_i64` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::floor` | `Duration.floor` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::ceil` | `Duration.ceil` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `duration::Duration::round` | `Duration.round` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopSample` | `EopSample` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopSeries` | `EopSeries` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable` | `EopTable` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::new` | `EopTable` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::from_finals2000a` | `EopTable.from_finals2000a` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::from_finals2000a_series` | `EopTable.from_finals2000a_series` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::observed_only` | `EopTable.observed_only` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::ut1_minus_utc` | `EopTable.ut1_minus_utc` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::range` | `EopTable.range` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::samples` | `EopTable.samples` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::len` | `EopTable.__len__` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `eop::EopTable::is_empty` | `EopTable.is_empty` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch` | `Epoch` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::from_parts` | `Epoch` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::checked_from_parts` | `Epoch` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `epoch::Epoch::from_tai_seconds` | `Epoch.from_tai_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::checked_from_tai_seconds` | `Epoch.from_tai_seconds` | wrapped | Checked core constructor; raises instead of returning Rust Option. |
| `epoch::Epoch::tai_seconds` | `Epoch.tai_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::sec` | `Epoch.sec` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::nanos` | `Epoch.nanos` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::checked_add` | `Epoch.checked_add` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::Epoch::checked_sub` | `Epoch.checked_sub` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::UtcDateTime` | `UtcDateTime` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `epoch::UtcDateTime::new` | `UtcDateTime` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `error::TimeError` | `TimeError` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::FormatOptions` | `FormatOptions` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::FormatOptions::new` | `FormatOptions` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::FormatOptions::with_calendar` | `FormatOptions.with_calendar` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::FormatOptions::with_leap_seconds` | `FormatOptions.with_leap_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::FormatOptions::precision` | `FormatOptions.precision` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::FormatOptions::calendar` | `FormatOptions.calendar` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::iso8601` | `iso8601` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::iso8601_with` | `iso8601_with` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::spice_calendar` | `spice_calendar` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::spice_calendar_with` | `spice_calendar_with` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::etcal` | `etcal` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::jd_to_calendar` | `jd_to_calendar` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::rfc3339` | `rfc3339` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::rfc3339_with` | `rfc3339_with` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::rfc2822` | `rfc2822` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::Format` | `Format` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::Format::parse` | `Format.parse` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::Formatter` | `Formatter` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::Formatter::new` | `Formatter` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::Formatter::with_options` | `Formatter.with_options` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::strftime` | `strftime` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `format::strftime_with` | `strftime_with` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `kernel_date::KernelDate` | `KernelDate` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `kernel_date::parse_kernel_date` | `parse_kernel_date` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::DELTA_T_A` | `DELTA_T_A` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::K` | `K` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::EB` | `EB` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::M0` | `M0` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::M1` | `M1` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::JDN_1960` | `JDN_1960` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::JDN_1972` | `JDN_1972` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapEntry` | `LeapEntry` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondSource` | `LeapSecondSource` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondPolicy` | `LeapSecondPolicy` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondPolicy::frozen_after` | `LeapSecondPolicy.frozen_after` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable` | `LeapSecondTable` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::new` | `LeapSecondTable` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::with_constants` | `LeapSecondTable.with_constants` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::builtin` | `LeapSecondTable.builtin` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::from_lsk` | `LeapSecondTable.from_lsk` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::with_source` | `LeapSecondTable.with_source` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::source` | `LeapSecondTable.source` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::with_policy` | `LeapSecondTable.with_policy` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::policy` | `LeapSecondTable.policy` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::delta_at_f64` | `LeapSecondTable.delta_at_f64` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::entries` | `LeapSecondTable.entries` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::coverage` | `LeapSecondTable.coverage` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::LeapSecondTable::delta_at_for_jdn` | `LeapSecondTable.delta_at_for_jdn` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `leap::sofa_delta_at` | `sofa_delta_at` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `lsk::parse_lsk` | `LeapSecondTable.from_lsk` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `lsk::builtin` | `LeapSecondTable.builtin` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `mcp::run_stdio` | `- ` | unavailable | MCP is an optional stdio transport, not a scientific binding capability. |
| `mcp::handle_message` | `- ` | unavailable | MCP is an optional stdio transport, not a scientific binding capability. |
| `sclk::SclkTimeSystem` | `SclkTimeSystem` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkCoefficient` | `SclkCoefficient` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel` | `SclkKernel` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::parse` | `SclkKernel.parse` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::spacecraft_id` | `SclkKernel.spacecraft_id` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::data_type` | `SclkKernel.data_type` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::time_system` | `SclkKernel.time_system` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::n_fields` | `SclkKernel.n_fields` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::moduli` | `SclkKernel.moduli` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::offsets` | `SclkKernel.offsets` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::coefficients` | `SclkKernel.coefficients` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::partitions` | `SclkKernel.partitions` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::ticks_per_msb` | `SclkKernel.ticks_per_msb` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::ticks_from_fields` | `SclkKernel.ticks_from_fields` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::fields_from_ticks` | `SclkKernel.fields_from_ticks` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::parallel_seconds` | `SclkKernel.parallel_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::ticks_from_parallel` | `SclkKernel.ticks_from_parallel` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::ticks_to_et` | `SclkKernel.ticks_to_et` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::et_to_ticks` | `SclkKernel.et_to_ticks` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::ticks_to_tai` | `SclkKernel.ticks_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::tai_to_ticks` | `SclkKernel.tai_to_ticks` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::parse_string` | `SclkKernel.parse_string` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkKernel::format_string` | `SclkKernel.format_string` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkClock` | `SclkClock` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `sclk::SclkMap` | `SclkClock` | protocol | Owned SclkClock represents SclkMap; Python structural methods replace Rust trait/lifetimes. |
| `series::TimeSeries` | `TimeSeries` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `series::TimeSeries::inclusive` | `TimeSeries.inclusive` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `series::TimeSeries::exclusive` | `TimeSeries.exclusive` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `tdb::tdb_minus_tt_seconds` | `tdb_minus_tt_seconds` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `tdb::tai_to_tdb` | `compat.tai_to_tdb` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `tdb::tdb_to_tai` | `compat.tdb_to_tai` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
| `timescale::TimeScale` | `TimeScale` | wrapped | Direct core delegation; integer narrowing and finite checks occur at the boundary. |
