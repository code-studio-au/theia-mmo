use theia_core::damage::{
    Amount, MAX_VALUE, Range, Spell, Weapon,
    resolution::{Breakdown, Defences, Impact, Rounding, Scale, resolve_fixture_v1},
};
fn hit(bp: u32) -> Impact {
    Impact::Contact(Scale::new(bp).unwrap())
}
fn resolve(start: u32, impact: Impact, defences: Defences, health: u32) -> Breakdown {
    resolve_fixture_v1(
        Amount::fixed(start).unwrap(),
        impact,
        defences,
        health,
        Rounding::NearestHalfUp,
    )
    .unwrap()
}
#[test]
fn supplied_defence_example_keeps_exact_prevention_and_names_rounding() {
    let defences = Defences::physical(3000, 0, 20).unwrap();
    let result = resolve(118, hit(10000), defences, 100);
    assert_eq!(result.resolved.exact_ratio(), (313, 5));
    assert_eq!(result.armour_prevented.exact_ratio(), (177, 5));
    assert_eq!(result.resistance_prevented.floor(), 0);
    assert_eq!(result.absorbed.floor(), 20);
    assert_eq!(
        (result.rounded_damage, result.health_lost, result.overkill),
        (63, 63, 0)
    );
    let floor = resolve_fixture_v1(
        Amount::fixed(118).unwrap(),
        hit(10000),
        defences,
        100,
        Rounding::Floor,
    )
    .unwrap();
    assert_eq!(floor.rounded_damage, 62);
    assert_eq!(floor.resolved, result.resolved);
}
#[test]
fn physical_block_and_magical_resistance_remain_separate_facts() {
    let physical = resolve(
        118,
        hit(10000),
        Defences::physical(3000, 10, 20).unwrap(),
        1000,
    );
    assert_eq!(physical.blocked.floor(), 10);
    assert_eq!(physical.resolved.exact_ratio(), (263, 5));
    let magical = resolve(260, hit(10000), Defences::magical(5000, 20).unwrap(), 1000);
    assert_eq!(magical.armour_prevented.floor(), 0);
    assert_eq!(magical.resistance_prevented.floor(), 130);
    assert_eq!(magical.blocked.floor(), 0);
    assert_eq!(magical.absorbed.floor(), 20);
    assert_eq!(magical.health_lost, 110);
}
#[test]
fn supplied_outcome_scale_precedes_prevention_without_inventing_probabilities() {
    let result = resolve(
        118,
        hit(20000),
        Defences::physical(3000, 10, 20).unwrap(),
        1000,
    );
    assert_eq!(result.after_outcome.floor(), 236);
    assert_eq!(result.resolved.exact_ratio(), (676, 5));
    assert_eq!(result.health_lost, 135);
    let reduced = resolve(118, hit(7000), Defences::physical(0, 0, 0).unwrap(), 1000);
    assert_eq!(reduced.resolved.exact_ratio(), (413, 5));
    let avoided = resolve(
        118,
        Impact::Avoided,
        Defences::physical(3000, 10, 20).unwrap(),
        100,
    );
    for amount in [
        avoided.after_outcome,
        avoided.armour_prevented,
        avoided.resistance_prevented,
        avoided.blocked,
        avoided.absorbed,
        avoided.resolved,
    ] {
        assert_eq!(amount.exact_ratio(), (0, 1));
    }
    assert_eq!(avoided.health_lost, 0);
}
#[test]
fn full_prevention_stops_at_zero_and_does_not_consume_later_layers() {
    let blocked = resolve(5, hit(10000), Defences::physical(0, 10, 20).unwrap(), 100);
    assert_eq!(blocked.blocked.floor(), 5);
    assert_eq!(blocked.absorbed.floor(), 0);
    assert_eq!(blocked.health_lost, 0);
    let absorbed = resolve(5, hit(10000), Defences::physical(0, 0, 20).unwrap(), 100);
    assert_eq!(absorbed.absorbed.floor(), 5);
    assert_eq!(absorbed.health_lost, 0);
    for defences in [
        Defences::physical(10000, 10, 20).unwrap(),
        Defences::magical(10000, 20).unwrap(),
    ] {
        let result = resolve(5, hit(10000), defences, 100);
        assert_eq!(result.blocked.floor(), 0);
        assert_eq!(result.absorbed.floor(), 0);
        assert_eq!(result.rounded_damage, 0);
    }
}
#[test]
fn fractional_prevention_is_exact_and_health_clipping_is_separate() {
    let amount = Spell::new(Range::new(0, 0).unwrap(), 2500)
        .unwrap()
        .starting_damage(0, 1)
        .unwrap();
    let result = resolve_fixture_v1(
        amount,
        hit(10000),
        Defences::magical(0, 1).unwrap(),
        100,
        Rounding::Floor,
    )
    .unwrap();
    assert_eq!(result.absorbed.exact_ratio(), (1, 4));
    assert_eq!(result.resolved.floor(), 0);
    let result = resolve(
        118,
        hit(10000),
        Defences::physical(3000, 0, 20).unwrap(),
        40,
    );
    assert_eq!(
        (result.rounded_damage, result.health_lost, result.overkill),
        (63, 40, 23)
    );
    let dead = resolve(118, hit(10000), Defences::physical(0, 0, 0).unwrap(), 0);
    assert_eq!((dead.health_lost, dead.overkill), (0, 118));
}
#[test]
fn small_fractions_survive_multiple_scales_without_float_or_stage_rounding() {
    let start = Weapon::new(Range::new(0, 0).unwrap(), 100)
        .unwrap()
        .starting_damage(0, 1)
        .unwrap(); // 1/140
    let result = resolve_fixture_v1(
        start,
        hit(3333),
        Defences::physical(7777, 0, 0).unwrap(),
        100,
        Rounding::Floor,
    )
    .unwrap();
    let (n, d) = result.resolved.exact_ratio();
    assert_eq!(
        u128::from(n) * 140 * 10000 * 10000,
        u128::from(d) * 3333 * 2223
    );
    assert_eq!(result.health_lost, 0);
    assert!(n > 0);
}
#[test]
fn invalid_defences_health_and_intermediate_damage_fail_before_clipping() {
    assert!(Defences::physical(10001, 0, 0).is_none());
    assert!(Defences::magical(u16::MAX, 0).is_none());
    assert!(Defences::physical(0, MAX_VALUE + 1, 0).is_none());
    assert!(Defences::magical(0, MAX_VALUE + 1).is_none());
    assert!(Scale::new(100001).is_none());
    let defences = Defences::physical(10000, 0, 0).unwrap();
    assert!(
        resolve_fixture_v1(
            Amount::fixed(MAX_VALUE).unwrap(),
            hit(20000),
            defences,
            100,
            Rounding::Floor
        )
        .is_err()
    );
    assert!(
        resolve_fixture_v1(
            Amount::fixed(1).unwrap(),
            hit(10000),
            defences,
            MAX_VALUE + 1,
            Rounding::Floor
        )
        .is_err()
    );
    let result = resolve(0, hit(0), Defences::physical(0, 0, 0).unwrap(), 100);
    assert_eq!(result.rounded_damage, 0);
}
