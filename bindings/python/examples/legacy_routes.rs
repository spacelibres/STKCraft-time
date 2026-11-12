//! Regenerates route-equivalence fixtures from literal public core calls.
use stkcraft_time::{contract::TimeContext, convert, Calendar, LeapSecondTable, UtcDateTime};
fn main() {
    let lsk = LeapSecondTable::builtin().unwrap();
    println!("year,month,day,hour,minute,second,legacy_et_bits,checked_et_bits,legacy_inverse_second_bits");
    for u in [
        UtcDateTime::new(2024, 1, 1, 0, 0, 0.123456789),
        UtcDateTime::new(2000, 1, 1, 12, 0, 0.123456789),
        UtcDateTime::new(2016, 12, 31, 23, 59, 60.5),
    ] {
        let et = convert::utc_to_et(&u, Calendar::Gregorian, &lsk).unwrap();
        let checked = TimeContext::new(&lsk)
            .utc_to_et(&u, Calendar::Gregorian)
            .unwrap()
            .seconds();
        let inverse = convert::et_to_utc(et, Calendar::Gregorian, &lsk).unwrap();
        println!(
            "{},{},{},{},{},{},{:016x},{:016x},{:016x}",
            u.year,
            u.month,
            u.day,
            u.hour,
            u.minute,
            u.second,
            et.to_bits(),
            checked.to_bits(),
            inverse.second.to_bits()
        );
    }
}
