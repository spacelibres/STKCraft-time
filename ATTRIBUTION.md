# Attribution

## `src/naif0012.tls`

The bundled leap-seconds kernel is a data product of the NASA Navigation
and Ancillary Information Facility (NAIF) at the Jet Propulsion
Laboratory.

- Source: <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/>
- File: `naif0012.tls`
- Content: `KPL/LSK` leap-second table and `DELTET/*` constants

The kernel data is retained for reproducibility of the alignment tests;
comments and whitespace may differ from the upstream file. It is not
covered by the crate's MIT source license.

## `src/dtdb_table.rs`

The 787-term TDB-TT coefficient table is the Fairhead & Bretagnon (1990)
series, as implemented by the IAU Standards of Fundamental Astronomy
(`iauDtdb`) and reproduced in ERFA (`dtdb.c`).

- Series: Fairhead, L., & Bretagnon, P., *Astron. Astrophys.* 229,
  240-247 (1990)
- Reference implementation: IAU SOFA / ERFA `dtdb.c`
- Source used: <https://github.com/liberfa/erfa/blob/master/src/dtdb.c>

The coefficients are numerical data extracted verbatim by
`tools/gen_dtdb_table.py`; only the summation order is written in Rust.
ERFA is distributed under a BSD-3-Clause license derived from SOFA with
permission. The data is reproduced here for IAU/SOFA interoperability.

## Implementation provenance

The conversion equations in this crate are implemented from the public
NAIF documentation, principally:

- *SPICE Time Subsystem Required Reading* (`time.req`)
- *Leapseconds Kernel Required Reading* (`lsk.req`)

The IAU coordinate scales and the TDB model follow the IAU 2000/2006
resolutions and the SOFA/ERFA reference formulas (`tttcg`, `tcgtt`,
`tcbtdb`, `tdbtcb`, `dtdb`). CSPICE is not linked at runtime.

## `src/naif.rs`

The decimal arithmetic is adapted from the operation order of NAIF N0067
NPARSD. This preserves kernel coefficient boundaries that can differ by
one binary64 interval from a correctly rounded full-token decimal parse.
The Rust numeric grammar is intentionally stricter than NPARSD's grammar.

- Original work: California Institute of Technology under contract with NASA
- Official archive: <https://naif.jpl.nasa.gov/pub/naif/toolkit/C/PC_Windows_VisualC_64bit/packages/cspice.zip>
- Archive SHA-256: `98d60b814b412fa55294aeaaeb7dab46d849cc87a8b709ffe835d08de17625dc`
- Source entry: `cspice/src/cspice/nparsd.c`
- Source SHA-256: `2fe61837c8d3343842a4445c77beb0a38617864f10037191b0bbd6913b8feaad`
- Independent numeric fixtures: `tools/gen_naif_number_golden.py` calls
  CSPICE PRSDP and records source kernel hashes and binary64 output bits.

The original source disclaimer follows:

> THIS SOFTWARE AND ANY RELATED MATERIALS WERE CREATED BY THE CALIFORNIA
> INSTITUTE OF TECHNOLOGY (CALTECH) UNDER A U.S. GOVERNMENT CONTRACT WITH
> THE NATIONAL AERONAUTICS AND SPACE ADMINISTRATION (NASA). THE SOFTWARE
> IS TECHNOLOGY AND SOFTWARE PUBLICLY AVAILABLE UNDER U.S. EXPORT LAWS
> AND IS PROVIDED "AS-IS" TO THE RECIPIENT WITHOUT WARRANTY OF ANY KIND,
> INCLUDING ANY WARRANTIES OF PERFORMANCE OR MERCHANTABILITY OR FITNESS
> FOR A PARTICULAR USE OR PURPOSE (AS SET FORTH IN UNITED STATES UCC
> SECTIONS 2312-2313) OR FOR ANY PURPOSE WHATSOEVER, FOR THE SOFTWARE AND
> RELATED MATERIALS, HOWEVER USED.
>
> IN NO EVENT SHALL CALTECH, ITS JET PROPULSION LABORATORY, OR NASA BE
> LIABLE FOR ANY DAMAGES AND/OR COSTS, INCLUDING, BUT NOT LIMITED TO,
> INCIDENTAL OR CONSEQUENTIAL DAMAGES OF ANY KIND, INCLUDING ECONOMIC
> DAMAGE OR INJURY TO PROPERTY AND LOST PROFITS, REGARDLESS OF WHETHER
> CALTECH, JPL, OR NASA BE ADVISED, HAVE REASON TO KNOW, OR, IN FACT,
> SHALL KNOW OF THE POSSIBILITY.
>
> RECIPIENT BEARS ALL RISK RELATING TO QUALITY AND PERFORMANCE OF THE
> SOFTWARE AND ANY RELATED MATERIALS, AND AGREES TO INDEMNIFY CALTECH AND
> NASA FOR ALL THIRD-PARTY CLAIMS RESULTING FROM THE ACTIONS OF RECIPIENT
> IN THE USE OF THE SOFTWARE.


## `src/contract.rs` NAIF operation order

The checked contract follows N0067 TTRANS UTC day-before/day-of anchors
and UNITIM's numerical TAI/TT/TDB operation order. These choices preserve
observable input-representation rounding, rather than fitting answers.
The archive and original NAIF disclaimer above also apply to this reference.

- `ttrans.c` SHA-256: `9644a008c0dd12349b873eefb46821d444b224c24f9e26cb6e617bb95ec1d57d`
- `unitim.c` SHA-256: `cdc57024ff54190648b8e858d7489d70bdb856c39832628c8ba8159c2c0325fd`

## `src/dtdb.rs` observer model

The observer-dependent terms and split-date model follow the SOFA-derived
ERFA implementation at commit `1a8044cde5b7763295d472a6443387239127c6c8`.
Source SHA-256: `228bd7c7c1c88dda44c29a076f289c968f64d77a2dc389138b03ab5d1d442105`.
The original ERFA notice is retained below and distributed with binaries.

> Copyright (C) 2013-2023, NumFOCUS Foundation.
> All rights reserved.
>
> This library is derived, with permission, from the International
> Astronomical Union's "Standards of Fundamental Astronomy" library,
> available from http://www.iausofa.org.
>
> The ERFA version is intended to retain identical functionality to
> the SOFA library, but made distinct through different function and
> file names, as set out in the SOFA license conditions.  The SOFA
> original has a role as a reference standard for the IAU and IERS,
> and consequently redistribution is permitted only in its unaltered
> state.  The ERFA version is not subject to this restriction and
> therefore can be included in distributions which do not support the
> concept of "read only" software.
>
> Although the intent is to replicate the SOFA API (other than
> replacement of prefix names) and results (with the exception of
> bugs;  any that are discovered will be fixed), SOFA is not
> responsible for any errors found in this version of the library.
>
> If you wish to acknowledge the SOFA heritage, please acknowledge
> that you are using a library derived from SOFA, rather than SOFA
> itself.
>
>
> TERMS AND CONDITIONS
>
> Redistribution and use in source and binary forms, with or without
> modification, are permitted provided that the following conditions
> are met:
>
> 1 Redistributions of source code must retain the above copyright
>   notice, this list of conditions and the following disclaimer.
>
> 2 Redistributions in binary form must reproduce the above copyright
>   notice, this list of conditions and the following disclaimer in
>   the documentation and/or other materials provided with the
>   distribution.
>
> 3 Neither the name of the Standards Of Fundamental Astronomy Board,
>   the International Astronomical Union nor the names of its
>   contributors may be used to endorse or promote products derived
>   from this software without specific prior written permission.
>
> THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
> "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
> LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
> FOR A PARTICULAR PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE
> COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
> INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
> BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
> LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
> CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
> LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN
> ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
> POSSIBILITY OF SUCH DAMAGE.
>
