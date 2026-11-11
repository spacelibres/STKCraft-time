//! A runnable tour of the public API.
//!
//! ```sh
//! cargo run --example api_tour
//! ```

use stkcraft_time::convert::{
    et_to_utc, tai_to_et, tai_to_gps, tai_to_utc, utc_to_et, utc_to_tai, utc_to_ut1,
};
use stkcraft_time::eop::{EopSample, EopTable};
use stkcraft_time::format::{iso8601, strftime};
use stkcraft_time::sclk::SclkKernel;
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

const SYNTH_SCLK: &str = r#"
KPL/SCLK

\begindata

SCLK_DATA_TYPE_999       = ( 1 )
SCLK01_TIME_SYSTEM_999   = ( 1 )
SCLK01_N_FIELDS_999      = ( 2 )
SCLK01_MODULI_999        = ( 1000 100 )
SCLK01_OFFSETS_999       = ( 0 0 )
SCLK01_OUTPUT_DELIM_999  = ( 2 )
SCLK01_COEFFICIENTS_999  = (
    0.0000000000000E+00     0.0000000000000E+00     1.0000000000000E+00
    1.0000000000000E+05     1.0000000000000E+03     1.0000000000000E+00
)

\begintext
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load a leap-second kernel and report its coverage.
    let lsk = LeapSecondTable::builtin()?;
    let (first_jdn, last_jdn) = lsk.coverage();
    println!("LSK coverage: JDN {first_jdn}..={last_jdn}");

    // 2. UTC -> TAI -> ET, using a positive leap second.
    let utc = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
    let tai = utc_to_tai(&utc, Calendar::Gregorian, &lsk)?;
    let et = tai_to_et(tai, &lsk);
    println!(
        "UTC {} -> TAI {:.9} s -> ET {et:.9} s",
        iso8601(&utc, 6),
        tai.tai_seconds()
    );

    // 3. ET -> UTC and TAI -> UTC round trips.
    let from_et = et_to_utc(et, Calendar::Gregorian, &lsk)?;
    let from_tai = tai_to_utc(tai, Calendar::Gregorian, &lsk)?;
    println!("ET -> UTC: {}", iso8601(&from_et, 6));
    println!("TAI -> UTC: {}", iso8601(&from_tai, 6));

    // 4. GPS is TAI - 19 s; this step needs no leap-second table.
    println!("GPS seconds past zero: {:.9}", tai_to_gps(tai));

    // 5. Spacecraft clock: string -> fields -> ticks -> ET.
    let sclk = SclkKernel::parse(SYNTH_SCLK, -999)?;
    let fields = sclk.parse_string("123:45")?;
    let ticks = sclk.ticks_from_fields(&fields)?;
    let sclk_et = sclk.ticks_to_et(ticks, &lsk)?;
    println!("SCLK 123:45 -> ticks {ticks:.0} -> ET {sclk_et:.9} s");

    // 6. UT1 needs an IERS EOP table, not the LSK.
    let eop = EopTable::new(vec![
        EopSample {
            mjd: 57_753.0,
            ut1_minus_utc: -0.4089,
            observed: true,
        },
        EopSample {
            mjd: 57_754.0,
            ut1_minus_utc: 0.5911,
            observed: true,
        },
        EopSample {
            mjd: 57_755.0,
            ut1_minus_utc: 0.5907,
            observed: true,
        },
    ])?;
    let ut1 = utc_to_ut1(&utc, Calendar::Gregorian, &lsk, &eop)?;
    println!("UT1 seconds past 2000-01-01T00:00:00 UT1: {ut1:.9}");

    // 7. Formatting.
    println!("strftime: {}", strftime(&from_tai, "%Y-%m-%d %H:%M:%S.%f")?);

    // 8. Convenience UTC <-> ET functions.
    let et_from_utc = utc_to_et(&utc, Calendar::Gregorian, &lsk)?;
    println!("utc_to_et: {et_from_utc:.9} s");

    // 9. Invalid leap-second labels are rejected, not guessed.
    let invalid = UtcDateTime::new(2016, 12, 30, 23, 59, 60.0);
    match utc_to_tai(&invalid, Calendar::Gregorian, &lsk) {
        Ok(_) => println!("unexpected success"),
        Err(error) => println!("rejected as expected: {error}"),
    }

    Ok(())
}
