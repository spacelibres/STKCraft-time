# Native raw JDUTC error boundary

Issue #4 concerns data that a native kernel pool can retain but the scientific
`LeapSecondTable` constructor rejects. Issue #3's valid historical conversion
and separate GeoCalc SPK10 regression must remain unchanged.

Add a dedicated `contract::NaifUtcContext` and explicit `NaifDeltet` constants.
`from_raw_parts` borrows a nonempty structured `LeapEntry` slice without sorting
or validating its TAI order. It requires finite constants and leaves native
paired-table validation to `et_to_spice_julian_utc`. This preserves the native
calculation-stage `SPICE(BADLEAPSECONDS)` result. The entry representation is
still whole offsets at integral Julian dates; arbitrary raw kernel variables,
fractional entries and missing-variable symbols are outside this update.

Keep `LeapSecondTable::new`, its scientific/civil validation and existing
`TimeContext` signatures. Both native JDUTC contexts share one numeric path and
the same three UNITIM inverse iterations. Preserve original order, native's
first-140-entry selection and every unrounded output. Do not create a temporary
scientific table or substitute nominal constants.

The existing 18,863-query independent corpus is immutable. The new path must
match all 18,493 numeric outputs and all 370 native error symbols, including the
185 constructor-blocked records. Retain the old typed-constructor distinction
as a separate regression. Add independent unsorted/duplicate and capacity-tail
captures before implementation. Require a red regression, full repository
verification, independent review and verified publication of patch 0.1.31.
