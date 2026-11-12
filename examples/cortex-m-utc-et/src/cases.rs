use stkcraft_time::{contract::TimeContext, Calendar, UtcDateTime};

// Independent SpiceyPy 8.3.0 / CSPICE N0067 STR2ET outputs copied from
// tests/golden/contract_utc.csv; inputs are queries, not observations.
pub const CASES: [(UtcDateTime, u64); 3] = [
    (
        UtcDateTime {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0.125,
        },
        0x405013c576f23c4a,
    ),
    (
        UtcDateTime {
            year: 2016,
            month: 12,
            day: 31,
            hour: 23,
            minute: 59,
            second: 60.125,
        },
        0x41bffa5a844f1606,
    ),
    (
        UtcDateTime {
            year: 2024,
            month: 4,
            day: 8,
            hour: 18,
            minute: 0,
            second: 0.125,
        },
        0x41c6d32252a7c365,
    ),
];

pub fn check(context: TimeContext<'_>) -> Result<(), &'static str> {
    for (utc, expected) in CASES {
        let et = context
            .utc_to_et(&utc, Calendar::Gregorian)
            .map_err(|_| "forward error")?;
        if et.seconds().to_bits() != expected {
            return Err("CSPICE bit mismatch");
        }
        let back = context
            .et_to_utc(et, Calendar::Gregorian)
            .map_err(|_| "inverse error")?;
        if (
            back.year,
            back.month,
            back.day,
            back.hour,
            back.minute,
            back.second,
        ) != (
            utc.year, utc.month, utc.day, utc.hour, utc.minute, utc.second,
        ) {
            return Err("civil roundtrip mismatch");
        }
    }
    Ok(())
}
