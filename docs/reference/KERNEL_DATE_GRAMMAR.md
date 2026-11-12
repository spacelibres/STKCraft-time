# Native kernel-date grammar scope

The reference corpus is `tests/reference/kernel-dates/native.tsv`: 416 unique
original input literals, 345 native values and 71 native rejections. Every
accepted result must equal its original binary64 bits. Rejections require
`KernelDateUnsupported`; a native error is not converted to a default epoch.

Original captures are in `docs/reference/kernel-date-grammar-native/`.
The native process is the same independently compiled, unmodified N0067
reference documented in [the arithmetic evidence](KERNEL_DATE_EXACT.md).
Inputs, stdout and stderr remain unchanged. The combined TSV deduplicates
only identical input/load/result records; conflicting duplicates fail its
construction. Subsequent captures append to the original corpus without
changing any earlier expected row.

## Exercised behavior

- Calendar dates with named or numeric months, native month prefixes and
  mixed dash/slash delimiters; the actual RSSD FK literal
  `@2000-JAN-1/12:00:00` resolves to exact zero.
- TPARTV integer-token precedence: integers at least 1000 become year
  tokens. Numeric clock tokens in that category are rejected, whereas
  decimal clock tokens remain a distinct category.
- Short years expand over the default 1969--2068 interval. Ambiguous
  named-month patterns retain native year/month/day ordering.
- Out-of-range month/day/clock values use the original Muller-Wimberly
  integer arithmetic and native addition order. Calendar fields are
  normalized independently of the preserved seconds result.
- Julian dates with a leading or trailing JD label; exercised numeric
  calendar and ordinal ISO forms; explicit year/day slash, `//` and `::`
  productions. Standalone `@2000-060` is rejected by the native reference.
- A decimal is accepted only as the final numeric component. Exponential,
  leading-sign and digitless fractional time fields are rejected.

The first 254-case regression exposed the original coverage gap. A second
component grid exposed seventeen over-acceptance cases; the repair follows
native token categories instead of broadly normalizing every number. Five
fractional-day boundary cases extend the independent rounding checks. The
original seven fractional-second and eleven-literal tests remain active.

## Limits

These captures prove the exercised default N0067 profile and input forms,
not exhaustive TPARSE grammar conformance. Era/zone modifiers, alternative
time-system annotations, custom parser productions and a mutable TSETYR
context are not implemented. Input outside the supported grammar remains
explicitly unsupported. Full native error-message text is not part of this
Rust error API. UTC/LSK validation rules are separate and remain strict.
