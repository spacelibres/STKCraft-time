//! Tests for `LeapSecondPolicy`: automatic freeze after a chosen date.

use stkcraft_time::calendar::gregorian_to_jdn;
use stkcraft_time::convert::{tai_to_utc, utc_to_tai};
use stkcraft_time::lsk::parse_lsk;
use stkcraft_time::{Calendar, LeapSecondPolicy, LeapSecondTable, TimeError, UtcDateTime};

const LSK_WITH_FUTURE: &str = r#"
KPL/LSK

\begindata

DELTET/DELTA_T_A = 32.184
DELTET/K         = 1.657D-3
DELTET/EB        = 1.671D-2
DELTET/M         = ( 6.239996D0 1.99096871D-7 )

DELTET/DELTA_AT  = ( 10, @1972-JAN-1
                     37, @2017-JAN-1
                     38, @2025-JAN-1 )

\begintext
"#;

fn table() -> LeapSecondTable {
    parse_lsk(LSK_WITH_FUTURE).expect("custom LSK must parse")
}

#[test]
fn bundled_table_freezes_after_last_entry() {
    let table = LeapSecondTable::builtin().expect("bundled LSK must parse");
    let jdn = gregorian_to_jdn(2040, 1, 1);
    assert_eq!(table.delta_at_for_jdn(jdn), Some(37));

    let dt = UtcDateTime::new(2040, 1, 1, 0, 0, 0.0);
    let tai = utc_to_tai(&dt, Calendar::Gregorian, &table).expect("valid UTC");
    let back = tai_to_utc(tai, Calendar::Gregorian, &table).expect("valid TAI");
    assert_eq!((back.year, back.month, back.day), (2040, 1, 1));
}

#[test]
fn frozen_after_constructor_matches_jdn() {
    assert_eq!(
        LeapSecondPolicy::frozen_after(2024, 1, 1),
        LeapSecondPolicy::FrozenAfter {
            jdn: gregorian_to_jdn(2024, 1, 1)
        }
    );
}

#[test]
fn table_driven_uses_future_entry() {
    let table = table();
    let jdn = gregorian_to_jdn(2025, 6, 1);
    assert_eq!(table.delta_at_for_jdn(jdn), Some(38));
}

#[test]
fn frozen_after_ignores_future_entry() {
    let table = table().with_policy(LeapSecondPolicy::FrozenAfter {
        jdn: gregorian_to_jdn(2024, 1, 1),
    });
    let jdn = gregorian_to_jdn(2025, 6, 1);
    assert_eq!(table.delta_at_for_jdn(jdn), Some(37));
}

#[test]
fn frozen_policy_round_trip_after_freeze() {
    let table = table().with_policy(LeapSecondPolicy::FrozenAfter {
        jdn: gregorian_to_jdn(2024, 1, 1),
    });

    let dt = UtcDateTime::new(2025, 6, 1, 12, 0, 0.0);
    let tai = utc_to_tai(&dt, Calendar::Gregorian, &table).expect("valid UTC");
    let back = tai_to_utc(tai, Calendar::Gregorian, &table).expect("valid TAI");

    assert_eq!((back.year, back.month, back.day), (2025, 6, 1));
    assert_eq!((back.hour, back.minute), (12, 0));
    assert!(back.second.abs() < 1e-9);
}

#[test]
fn frozen_policy_disables_future_leap_second() {
    let table = table().with_policy(LeapSecondPolicy::FrozenAfter {
        jdn: gregorian_to_jdn(2024, 1, 1),
    });

    let dt = UtcDateTime::new(2024, 12, 31, 23, 59, 60.0);
    assert_eq!(
        utc_to_tai(&dt, Calendar::Gregorian, &table),
        Err(TimeError::InvalidLeapSecond)
    );
}
