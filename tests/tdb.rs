//! Tests for the distinction between NAIF `ET` and ESA `TDB`.

use stkcraft_time::convert::{tai_to_et, tt_to_tai};
use stkcraft_time::tdb::{tai_to_tdb, tdb_to_tai};
use stkcraft_time::LeapSecondTable;

fn lsk() -> LeapSecondTable {
    LeapSecondTable::builtin().expect("bundled naif0012.tls must parse")
}

#[test]
fn esa_tdb_and_naif_et_agree_within_model_error() {
    let table = lsk();

    for days in [-400.0, -100.0, 0.0, 100.0, 400.0] {
        let tt = days * 86_400.0;
        let tai = tt_to_tai(tt, &table);
        let et = tai_to_et(tai, &table);
        let tdb = tai_to_tdb(tai, &table);

        assert!(
            (tdb - et).abs() < 0.002,
            "day {days}: ET = {et}, TDB = {tdb}"
        );
    }
}

#[test]
fn tdb_round_trip() {
    let table = lsk();
    let tdb = 12_345.678;
    let tai = tdb_to_tai(tdb, &table);
    let back = tai_to_tdb(tai, &table);

    assert!((back - tdb).abs() < 1e-6, "TDB round trip {back}");
}
