use theia_core::hit_outcomes::{BASIS_POINTS, Eligibility, Outcome, Roll, Table};
fn counts(table: Table, deflect: bool) -> [usize; 3] {
    let mut counts = [0; 3];
    for value in 0..BASIS_POINTS {
        let index = match table.resolve(Roll::new(value).unwrap(), Eligibility { deflect }) {
            Outcome::Hit => 0,
            Outcome::Miss => 1,
            Outcome::Deflect => 2,
        };
        counts[index] += 1;
    }
    counts
}
#[test]
fn exhaustive_mixed_rolls_have_exact_weights_and_ineligible_deflect_becomes_hit() {
    let table: Table =
        serde_json::from_str(r#"{"hit_bp":8500,"miss_bp":1000,"deflect_bp":500}"#).unwrap();
    assert_eq!(
        (table.hit_bp(), table.miss_bp(), table.deflect_bp()),
        (8500, 1000, 500)
    );
    assert_eq!(counts(table, true), [8500, 1000, 500]);
    assert_eq!(counts(table, false), [9000, 1000, 0]);
    for (value, expected) in [
        (0, Outcome::Miss),
        (999, Outcome::Miss),
        (1000, Outcome::Deflect),
        (1499, Outcome::Deflect),
        (1500, Outcome::Hit),
        (9999, Outcome::Hit),
    ] {
        assert_eq!(
            table.resolve(Roll::new(value).unwrap(), Eligibility { deflect: true }),
            expected
        );
    }
}
#[test]
fn certain_outcomes_and_zero_width_buckets_never_leak_an_extra_result() {
    assert_eq!(counts(Table::certain_hit(), true), [10000, 0, 0]);
    assert_eq!(
        counts(Table::new(0, 10000, 0).unwrap(), true),
        [0, 10000, 0]
    );
    assert_eq!(
        counts(Table::new(0, 0, 10000).unwrap(), true),
        [0, 0, 10000]
    );
    assert_eq!(
        counts(Table::new(0, 0, 10000).unwrap(), false),
        [10000, 0, 0]
    );
    assert_eq!(
        counts(Table::new(9000, 0, 1000).unwrap(), false),
        [10000, 0, 0]
    );
    assert_eq!(
        counts(Table::new(9000, 1000, 0).unwrap(), true),
        [9000, 1000, 0]
    );
}
#[test]
fn roll_domain_is_checked_instead_of_wrapping_or_clamping_invalid_draws() {
    assert_eq!(Roll::new(0).unwrap().value(), 0);
    assert_eq!(Roll::new(9999).unwrap().value(), 9999);
    for value in 10000..=u16::MAX {
        assert!(Roll::new(value).is_none());
    }
}
#[test]
fn malformed_ambiguous_incomplete_and_overflowing_tables_fail_closed() {
    for data in [
        r#"{"hit_bp":8500,"miss_bp":1000,"deflect_bp":501}"#,
        r#"{"hit_bp":8500,"miss_bp":1000,"deflect_bp":499}"#,
        r#"{"hit_bp":0,"miss_bp":0,"deflect_bp":0}"#,
        r#"{"hit_bp":65535,"miss_bp":65535,"deflect_bp":65535}"#,
        r#"{"hit_bp":65536,"miss_bp":0,"deflect_bp":0}"#,
        r#"{"hit_bp":10000.0,"miss_bp":0,"deflect_bp":0}"#,
        r#"{"hit_bp":-1,"miss_bp":10001,"deflect_bp":0}"#,
        r#"{"hit_bp":true,"miss_bp":0,"deflect_bp":0}"#,
        r#"{"hit_bp":null,"miss_bp":0,"deflect_bp":0}"#,
        r#"{"hit_bp":"10000","miss_bp":0,"deflect_bp":0}"#,
        r#"{"hit_bp":10000,"miss_bp":0}"#,
        r#"{"hit_bp":10000,"miss_bp":0,"deflect_bp":0,"crit_bp":100}"#,
        r#"{"hit_bp":10000,"hit_bp":0,"miss_bp":0,"deflect_bp":0}"#,
        r#"null"#,
        r#"[]"#,
        r#"{}"#,
    ] {
        assert!(
            serde_json::from_str::<Table>(data).is_err(),
            "Accepted {data}"
        );
    }
    for weights in [
        (9999, 0, 0),
        (10001, 0, 0),
        (u16::MAX, u16::MAX, u16::MAX),
        (0, 0, 0),
    ] {
        assert!(Table::new(weights.0, weights.1, weights.2).is_none());
    }
}
#[test]
fn supplied_roll_replays_identically_without_consuming_hidden_state() {
    let a = Table::new(8500, 1000, 500).unwrap();
    let b: Table =
        serde_json::from_str(r#"{ "deflect_bp":500, "miss_bp":1000, "hit_bp":8500 }"#).unwrap();
    let records = [
        (9999, true),
        (0, true),
        (1000, false),
        (1500, false),
        (42, false),
        (1499, true),
    ];
    let expected = [
        Outcome::Hit,
        Outcome::Miss,
        Outcome::Hit,
        Outcome::Hit,
        Outcome::Miss,
        Outcome::Deflect,
    ];
    for table in [a, b] {
        let replay: Vec<_> = records
            .into_iter()
            .map(|(roll, deflect)| table.resolve(Roll::new(roll).unwrap(), Eligibility { deflect }))
            .collect();
        assert_eq!(replay, expected);
        let reversed: Vec<_> = records
            .into_iter()
            .rev()
            .map(|(roll, deflect)| table.resolve(Roll::new(roll).unwrap(), Eligibility { deflect }))
            .collect();
        assert_eq!(reversed, expected.into_iter().rev().collect::<Vec<_>>());
    }
}
