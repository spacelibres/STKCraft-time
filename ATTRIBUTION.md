# Attribution

## `src/naif0012.tls`

The bundled leap-seconds kernel is a data product of the NASA Navigation
and Ancillary Information Facility (NAIF) at the Jet Propulsion
Laboratory.

- Source: <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/>
- File: `naif0012.tls`
- Content: `KPL/LSK` leap-second table and `DELTET/*` constants

The file is included verbatim for reproducibility of the alignment
tests. It is not covered by the crate's MIT source license.

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
`tcbtdb`, `tdbtcb`, `dtdb`). No CSPICE source code is copied, translated,
or linked.
