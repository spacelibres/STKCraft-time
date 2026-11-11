use stkcraft_time::dtdb::{
    dtdb_minus_tt, dtdb_minus_tt_at_observer, dtdb_minus_tt_split, DtdbError,
};

#[test]
fn erfa_reference_observer_case() {
    let actual =
        dtdb_minus_tt_at_observer(2_448_939.5, 0.123, 0.76543, 5.0123, 5525.242, 3190.0).unwrap();
    assert!((actual - (-1.280_368_005_936_999e-3)).abs() < 1e-15);
}

#[test]
fn rejects_invalid_ut1_and_retains_split_precision() {
    assert_eq!(
        dtdb_minus_tt_at_observer(2_451_545.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        Err(DtdbError::InvalidUt1Fraction)
    );
    assert_ne!(
        dtdb_minus_tt_split(2_451_545.0, 0.0),
        dtdb_minus_tt_split(2_451_545.0, 1e-10)
    );
}

#[test]
fn split_model_guard_respects_sub_jd_ulp_boundary_offsets() {
    for bound in [2_433_282.5, 2_469_807.5] {
        assert!(dtdb_minus_tt_at_observer(bound, 0.0, 0.0, 0.0, 0.0, 0.0).is_ok());
    }
    assert_eq!(
        dtdb_minus_tt_at_observer(2_433_282.5, -1e-12, 0.0, 0.0, 0.0, 0.0),
        Err(DtdbError::OutsideModelRange)
    );
    assert_eq!(
        dtdb_minus_tt_at_observer(2_469_807.5, 1e-12, 0.0, 0.0, 0.0, 0.0),
        Err(DtdbError::OutsideModelRange)
    );
}

#[test]
fn agrees_with_independent_pyerfa_observer_oracle() {
    let mut count = 0;
    for line in include_str!("golden/dtdb_observer.csv").lines() {
        if line.starts_with('#') || line.starts_with("jd1,") || line.is_empty() {
            continue;
        }
        let row: Vec<f64> = line
            .split(',')
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(row.len(), 7);
        let actual =
            dtdb_minus_tt_at_observer(row[0], row[1], row[2], row[3], row[4], row[5]).unwrap();
        // Port agreement only, not an absolute model-accuracy assertion.
        assert!((actual - row[6]).abs() < 2e-17, "{line}: got {actual}");
        count += 1;
    }
    assert_eq!(count, 724);
}

#[test]
fn rejects_each_nonfinite_input_and_invalid_coordinate() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..6 {
            let mut args = [2_451_545.0, 0.0, 0.5, 1.0, 100.0, -100.0];
            args[index] = value;
            assert_eq!(
                dtdb_minus_tt_at_observer(args[0], args[1], args[2], args[3], args[4], args[5]),
                Err(DtdbError::NonFiniteInput)
            );
        }
    }
    for ut1 in [-1.0, -f64::EPSILON, 1.0, 2.0] {
        assert_eq!(
            dtdb_minus_tt_at_observer(2_451_545.0, 0.0, ut1, 0.0, 0.0, 0.0),
            Err(DtdbError::InvalidUt1Fraction)
        );
    }
    assert_eq!(
        dtdb_minus_tt_at_observer(2_451_545.0, 0.0, 0.0, 0.0, -0.001, 0.0),
        Err(DtdbError::InvalidObserverDistance)
    );
    // Signed v and large finite distances are accepted without invented bounds.
    assert!(
        dtdb_minus_tt_at_observer(2_451_545.0, 0.0, 0.0, 0.0, f64::MAX, -f64::MAX)
            .unwrap()
            .is_finite()
    );
    assert_eq!(
        dtdb_minus_tt_at_observer(f64::MAX, f64::MAX, 0.0, 0.0, 0.0, 0.0),
        Err(DtdbError::OutsideModelRange)
    );
}

#[test]
fn geocentre_is_independent_of_ut1_and_longitude() {
    for (jd1, jd2) in [(2_451_545.0, 123.456), (2_400_000.5, 50_000.25)] {
        let geocentric = dtdb_minus_tt_split(jd1, jd2);
        for (ut1, longitude) in [(0.0, 0.0), (0.123, -3.0), (0.999999, 5.0)] {
            assert_eq!(
                dtdb_minus_tt_at_observer(jd1, jd2, ut1, longitude, 0.0, 0.0).unwrap(),
                geocentric
            );
        }
        assert_eq!(dtdb_minus_tt(jd1), dtdb_minus_tt_split(jd1, 0.0));
    }
}

#[test]
fn equivalent_splits_and_longitude_turns_agree() {
    let reference =
        dtdb_minus_tt_at_observer(2_451_545.0, 123.25, 0.125, -1.0, 4500.0, -4500.0).unwrap();
    for (jd1, jd2) in [
        (2_451_668.25, 0.0),
        (2_400_000.5, 51_667.75),
        (2_451_668.0, 0.25),
    ] {
        let actual = dtdb_minus_tt_at_observer(
            jd1,
            jd2,
            0.125,
            -1.0 + core::f64::consts::TAU,
            4500.0,
            -4500.0,
        )
        .unwrap();
        assert!((actual - reference).abs() < 2e-17);
    }
}
