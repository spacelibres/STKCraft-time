# WebGeocalc GPS week/TOW output design

The first increment targets numeric GPS seconds past the GPS epoch as input
and WebGeocalc's six-decimal week/TOW string as output. It is an explicit
WebGeocalc compatibility path, not a replacement for exact integer time
representations or a claim that all Time Conversion formats are implemented.

The interface accepts binary64 GPS epoch seconds and an explicit
LSK. It computes a candidate WGC-compatible path through the existing N0067
TAI-to-TDB and TDB-to-TAI UNITIM arithmetic, then splits the resulting GPS
seconds by 604800. The observed week field first saturates GPS seconds to
i32 and divides toward zero; the remainder uses the full binary64 value.
Format the shortest decimal remainder to six places with half-away rounding.
Preserve negative zero and do not carry a rounded output remainder into the
displayed week. Tested non-finite inputs return WGC's literal `0, NaN`;
non-finite arithmetic from a finite input returns an explicit error. The fixed GPS origin in TAI is 1980-01-06 00:00:19 TAI,
expressed relative to J2000 TAI noon.

Acceptance uses all 354 independently captured live WGC2 input/output rows,
including negative weeks, submicrosecond boundaries and seeded values. These
are output strings and do not establish raw intermediate binary64 equality.
The reference source records the server-managed kernel-set limitation and
local/official LSK hashes. Test against native UNITIM independently, preserve
all old GPS conversion behavior, and provide Python integration before release.
Input week-string parsing, UTC/day-of-year and custom output remain separate
open requirements. Do not extrapolate this finite corpus into a full-parity
claim.
