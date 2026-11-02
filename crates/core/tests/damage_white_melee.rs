use theia_core::{
    damage::white_melee::{Eligibility, Outcome, Rates},
    hit_outcomes::Roll,
};
fn all() -> Eligibility {
    Eligibility {
        dodge: true,
        parry: true,
        block: true,
        glancing: true,
        critical: true,
        crushing: true,
    }
}
fn roll(value: u16) -> Roll {
    Roll::new(value).unwrap()
}
#[test]
fn every_roll_has_one_distinct_outcome_and_exact_declared_frequency() {
    let weights = [500, 600, 700, 800, 900, 1000, 1100];
    let table = Rates::new(weights).unwrap().distribution(all());
    assert_eq!(table.weights(), [500, 600, 700, 800, 900, 1000, 1100, 4400]);
    let outcomes = [
        Outcome::Miss,
        Outcome::Dodge,
        Outcome::Parry,
        Outcome::Block,
        Outcome::Glancing,
        Outcome::Critical,
        Outcome::Crushing,
        Outcome::Hit,
    ];
    let mut counts = [0u16; 8];
    for value in 0..10_000 {
        let outcome = table.resolve(roll(value));
        counts[outcomes.iter().position(|o| *o == outcome).unwrap()] += 1;
    }
    assert_eq!(counts, table.weights());
}
#[test]
fn each_half_open_boundary_selects_the_next_nonempty_interval() {
    let table = Rates::new([1, 1, 1, 1, 1, 1, 1])
        .unwrap()
        .distribution(all());
    assert_eq!(table.resolve(roll(0)), Outcome::Miss);
    assert_eq!(table.resolve(roll(1)), Outcome::Dodge);
    assert_eq!(table.resolve(roll(2)), Outcome::Parry);
    assert_eq!(table.resolve(roll(3)), Outcome::Block);
    assert_eq!(table.resolve(roll(4)), Outcome::Glancing);
    assert_eq!(table.resolve(roll(5)), Outcome::Critical);
    assert_eq!(table.resolve(roll(6)), Outcome::Crushing);
    assert_eq!(table.resolve(roll(7)), Outcome::Hit);
    assert_eq!(table.resolve(roll(9999)), Outcome::Hit);
}
#[test]
fn ineligible_outcomes_consume_no_domain_and_do_not_change_miss() {
    let rates = Rates::new([500; 7]).unwrap();
    let table = rates.distribution(Eligibility::default());
    assert_eq!(table.weights(), [500, 0, 0, 0, 0, 0, 0, 9500]);
    assert_eq!(table.resolve(roll(499)), Outcome::Miss);
    assert_eq!(table.resolve(roll(500)), Outcome::Hit);
    let table = rates.distribution(Eligibility {
        critical: true,
        ..Default::default()
    });
    assert_eq!(table.resolve(roll(500)), Outcome::Critical);
    assert_eq!(table.resolve(roll(1000)), Outcome::Hit);
}
#[test]
fn earlier_buckets_push_later_crit_crush_and_hit_off_the_table() {
    let rates = Rates::new([500, 500, 500, 500, 4000, 6000, 1500]).unwrap();
    assert_eq!(
        rates.distribution(all()).weights(),
        [500, 500, 500, 500, 4000, 4000, 0, 0]
    );
    let behind_mask = Eligibility {
        parry: false,
        block: false,
        ..all()
    };
    assert_eq!(
        rates.distribution(behind_mask).weights(),
        [500, 500, 0, 0, 4000, 5000, 0, 0]
    );
    assert_eq!(rates.nominal(), [500, 500, 500, 500, 4000, 6000, 1500]);
}
#[test]
fn independently_maximal_rates_cannot_wrap_or_create_extra_probability() {
    let table = Rates::new([10_000; 7]).unwrap().distribution(all());
    assert_eq!(table.weights(), [10_000, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(table.resolve(roll(9999)), Outcome::Miss);
    let rates = Rates::new([0, 10_000, 10_000, 10_000, 10_000, 10_000, 10_000]).unwrap();
    assert_eq!(
        rates.distribution(all()).weights(),
        [0, 10_000, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        rates.distribution(Eligibility::default()).weights(),
        [0, 0, 0, 0, 0, 0, 0, 10_000]
    );
}
#[test]
fn malformed_individual_rates_and_rolls_fail_before_resolution() {
    for index in 0..7 {
        for bad in [10_001, u16::MAX] {
            let mut weights = [0; 7];
            weights[index] = bad;
            assert!(Rates::new(weights).is_none());
        }
    }
    assert!(Roll::new(10_000).is_none());
    assert!(Roll::new(u16::MAX).is_none());
}
