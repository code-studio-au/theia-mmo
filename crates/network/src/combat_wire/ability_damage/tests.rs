use super::*;
use crate::{
    combat_wire::{Batch, MAX_BYTES},
    simulation::{
        combat::{Event, Reason, physical, white_melee as ordinary},
        creatures,
    },
};
use std::sync::Arc;
use theia_core::{abilities, coordinates::GlobalPosition, inventory, melee};
const STARTING: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-ability-starting.json");
const POLICY: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-ability-damage.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-classic-abilities.json");
struct Fixture {
    weapons: melee::Catalog,
    profile: Profile,
}
impl Fixture {
    fn new() -> Self {
        Self::with(POLICY)
    }
    fn with(policy: &[u8]) -> Self {
        let abilities = abilities::Catalog::from_json(ABILITIES).unwrap();
        let weapons = melee::Catalog::from_json(include_bytes!(
            "../../../../../content/gameplay/development-melee-weapons.json"
        ))
        .unwrap();
        let items = inventory::Catalog::from_json(include_bytes!(
            "../../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let creature = creatures::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-creature-patrol.json"
        ))
        .unwrap();
        let physical = Arc::new(
            physical::Profile::new(
                include_bytes!(
                    "../../../../../content/gameplay/development-physical-damage-definitions.json"
                ),
                include_bytes!(
                    "../../../../../content/gameplay/development-physical-damage-stats.json"
                ),
                physical::Bindings {
                    weapons: &weapons,
                    items: &items,
                    abilities: &abilities,
                    creature: &creature,
                    player_maximum_health: 100,
                    creature_maximum_health: 1000,
                },
            )
            .unwrap(),
        );
        let white = Arc::new(
            ordinary::Profile::new(
                include_bytes!("../../../../../content/gameplay/development-white-melee.json"),
                physical,
            )
            .unwrap(),
        );
        let profile = Profile::new(STARTING, policy, ABILITIES, white).unwrap();
        Self { weapons, profile }
    }
    fn fact(&self, magic: bool, landing: u16, critical: Option<u16>, health: u32) -> Fact {
        let ability = ContentId::new(if magic {
            "theia.dev.arcane-bolt"
        } else {
            "theia.dev.sword-slash"
        })
        .unwrap();
        let defender = ordinary::DefenderState {
            standing: true,
            can_react: true,
            in_front: true,
            ..Default::default()
        };
        let draws = Draws {
            base: if magic { 20 } else { 11 },
            landing: Roll::new(landing).unwrap(),
            critical: critical.map(|v| Roll::new(v).unwrap()),
        };
        let result = self
            .profile
            .evaluate(&ability, defender, draws, health)
            .unwrap();
        Fact {
            ability,
            actor: key(7, 13).unwrap(),
            actor_generation: NonZeroU64::new(2).unwrap(),
            sequence: 1,
            target: key(7, 27).unwrap(),
            position: GlobalPosition::new([8001., -2., 8002.]).unwrap(),
            defender,
            draws,
            result,
            remaining: health - result.damage.health_lost,
        }
    }
    fn batch(&self, fact: Fact) -> Batch {
        Batch {
            epoch: 7,
            sequence: 4,
            tick: 80,
            events: vec![Event::AbilityDamage(fact)],
        }
    }
    fn white(&self, incoming: bool, seq: u64) -> ordinary::Fact {
        let p = self.profile.ordinary();
        let defender = ordinary::DefenderState {
            standing: true,
            can_react: true,
            in_front: true,
            ..Default::default()
        };
        let draws = ordinary::Draws {
            base: if incoming { 5 } else { 11 },
            outcome: Roll::new(9999).unwrap(),
            glancing: None,
        };
        let item = ContentId::new("theia.dev.training-sword").unwrap();
        ordinary::Fact {
            attack: if incoming {
                p.physical().creature_attack().0.clone()
            } else {
                p.physical().weapon(&item).unwrap().0.clone()
            },
            actor: key(7, if incoming { 27 } else { 13 }).unwrap(),
            actor_generation: if incoming { None } else { NonZeroU64::new(2) },
            sequence: seq,
            target: key(7, if incoming { 13 } else { 27 }).unwrap(),
            target_generation: if incoming { NonZeroU64::new(2) } else { None },
            position: GlobalPosition::new([8001., -2., 8002.]).unwrap(),
            defender,
            draws,
            result: if incoming {
                p.evaluate_incoming(defender, draws, 100).unwrap()
            } else {
                p.evaluate_outgoing(&item, defender, draws, 1000).unwrap()
            },
            remaining: if incoming { 96 } else { 988 },
        }
    }
}
#[test]
fn physical_and_magic_hits_critical_avoidance_and_overkill_round_trip_exactly() {
    let f = Fixture::new();
    for (magic, landing, crit, hp) in [
        (false, 1000, Some(500), 1000),
        (false, 1000, Some(499), 3),
        (false, 499, None, 1000),
        (false, 500, None, 1000),
        (true, 399, None, 1000),
        (true, 400, Some(500), 1000),
        (true, 400, Some(499), 3),
    ] {
        let fact = f.fact(magic, landing, crit, hp);
        let batch = f.batch(fact);
        let encoded = batch.encode_abilities(&f.weapons, &f.profile).unwrap();
        assert_eq!(encoded.len(), HEADER_BYTES + FACT_BYTES);
        assert!(encoded.starts_with(EVENTS_TAG));
        assert_eq!(
            Batch::decode_abilities(&encoded, &f.weapons, &f.profile).unwrap(),
            batch
        );
    }
}
#[test]
fn mixed_white_and_ability_namespaces_preserve_same_sequence_and_private_player_life() {
    let f = Fixture::new();
    let mut b = f.batch(f.fact(false, 1000, Some(500), 1000));
    b.events.push(Event::WhiteMelee(f.white(false, 1)));
    b.events.push(Event::WhiteMelee(f.white(true, 1)));
    b.events.push(Event::Cancelled {
        actor: key(7, 13).unwrap(),
        sequence: 2,
        reason: Reason::Interrupted,
    });
    let bytes = b.encode_abilities(&f.weapons, &f.profile).unwrap();
    assert_eq!(
        Batch::decode_abilities(&bytes, &f.weapons, &f.profile).unwrap(),
        b
    );
    b.events.push(b.events[0].clone());
    assert!(b.encode_abilities(&f.weapons, &f.profile).is_err());
}
#[test]
fn max_batches_keep_existing_white_bound_and_new_ability_facts_under_budget() {
    let f = Fixture::new();
    for white in [false, true] {
        let mut b = f.batch(f.fact(false, 1000, Some(500), 1000));
        b.events.clear();
        for seq in 1..=12 {
            b.events.push(if white {
                Event::WhiteMelee(f.white(false, seq))
            } else {
                let mut fact = f.fact(false, 1000, Some(500), 1000);
                fact.sequence = seq;
                Event::AbilityDamage(fact)
            });
        }
        let encoded = b.encode_abilities(&f.weapons, &f.profile).unwrap();
        assert_eq!(encoded.len(), if white { 1045 } else { 961 });
        assert!(encoded.len() <= MAX_BYTES);
        assert_eq!(
            Batch::decode_abilities(&encoded, &f.weapons, &f.profile).unwrap(),
            b
        );
        b.events.push(b.events[0].clone());
        assert!(b.encode_abilities(&f.weapons, &f.profile).is_err());
    }
}
#[test]
fn legacy_codecs_and_profiles_cannot_accept_ce08_or_downgrade_its_facts() {
    let f = Fixture::new();
    let b = f.batch(f.fact(false, 1000, Some(500), 1000));
    let encoded = b.encode_abilities(&f.weapons, &f.profile).unwrap();
    let c = f.profile.catalog();
    let w = f.profile.ordinary();
    assert!(b.encode(c).is_err());
    assert!(b.encode_melee(c, &f.weapons).is_err());
    assert!(
        b.encode_physical(c, &f.weapons, f.profile.physical())
            .is_err()
    );
    assert!(b.encode_white(c, &f.weapons, w).is_err());
    assert!(Batch::decode(&encoded, c).is_err());
    assert!(Batch::decode_melee(&encoded, c, &f.weapons).is_err());
    assert!(Batch::decode_physical(&encoded, c, &f.weapons, f.profile.physical()).is_err());
    assert!(Batch::decode_white(&encoded, c, &f.weapons, w).is_err());
    let mut other = POLICY.to_vec();
    other.push(b' ');
    let other = Fixture::with(&other);
    assert!(Batch::decode_abilities(&encoded, &other.weapons, &other.profile).is_err());
    let mut wrong = encoded.clone();
    wrong[4] ^= 1;
    assert!(Batch::decode_abilities(&wrong, &f.weapons, &f.profile).is_err());
    let mut wrong = encoded;
    wrong[..4].copy_from_slice(b"CE07");
    assert!(Batch::decode_abilities(&wrong, &f.weapons, &f.profile).is_err());
}
#[test]
fn malformed_life_ordinal_roll_state_health_and_position_frames_fail_closed() {
    let f = Fixture::new();
    let encoded = f
        .batch(f.fact(false, 1000, Some(500), 1000))
        .encode_abilities(&f.weapons, &f.profile)
        .unwrap();
    let mutations: Vec<(usize, Vec<u8>)> = vec![
        (1, 0u64.to_le_bytes().to_vec()),
        (9, 0u64.to_le_bytes().to_vec()),
        (17, 13u64.to_le_bytes().to_vec()),
        (25, vec![255]),
        (26, 0u64.to_le_bytes().to_vec()),
        (34, 0u32.to_le_bytes().to_vec()),
        (38, 10000u16.to_le_bytes().to_vec()),
        (40, 10000u16.to_le_bytes().to_vec()),
        (40, u16::MAX.to_le_bytes().to_vec()),
        (42, vec![128]),
        (43, 0u32.to_le_bytes().to_vec()),
        (43, 1001u32.to_le_bytes().to_vec()),
        (47, 1000u32.to_le_bytes().to_vec()),
        (51, f64::NAN.to_le_bytes().to_vec()),
        (59, f64::INFINITY.to_le_bytes().to_vec()),
    ];
    for (offset, value) in mutations {
        let mut b = encoded.clone();
        b[HEADER_BYTES + offset..HEADER_BYTES + offset + value.len()].copy_from_slice(&value);
        assert!(
            Batch::decode_abilities(&b, &f.weapons, &f.profile).is_err(),
            "offset {offset}"
        );
    }
    let mut missed = f
        .batch(f.fact(true, 399, None, 1000))
        .encode_abilities(&f.weapons, &f.profile)
        .unwrap();
    missed[HEADER_BYTES + 40..HEADER_BYTES + 42].copy_from_slice(&0u16.to_le_bytes());
    assert!(Batch::decode_abilities(&missed, &f.weapons, &f.profile).is_err());
}
#[test]
fn every_truncation_trailing_byte_and_inconsistent_fact_is_refused() {
    let f = Fixture::new();
    let fact = f.fact(false, 1000, Some(500), 1000);
    let b = f.batch(fact.clone());
    let encoded = b.encode_abilities(&f.weapons, &f.profile).unwrap();
    for end in 0..encoded.len() {
        assert!(
            Batch::decode_abilities(&encoded[..end], &f.weapons, &f.profile).is_err(),
            "truncation {end}"
        );
    }
    let mut trailing = encoded;
    trailing.push(0);
    assert!(Batch::decode_abilities(&trailing, &f.weapons, &f.profile).is_err());
    assert!(Batch::decode_abilities(&vec![0; MAX_BYTES + 1], &f.weapons, &f.profile).is_err());
    let mut bad = fact.clone();
    bad.result.damage.resolved_damage += 1;
    assert!(
        f.batch(bad)
            .encode_abilities(&f.weapons, &f.profile)
            .is_err()
    );
    let mut bad = fact;
    bad.result.outcome = crate::simulation::combat::ability_damage::Outcome::Arcane(
        theia_core::damage::spell::Outcome::Hit,
    );
    assert!(
        f.batch(bad)
            .encode_abilities(&f.weapons, &f.profile)
            .is_err()
    );
}
#[test]
fn ce08_refuses_legacy_fixed_damage_and_generic_avoidance_events() {
    let f = Fixture::new();
    let fact = f.fact(false, 1000, Some(500), 1000);
    let mut b = f.batch(fact.clone());
    b.events = vec![Event::Damage {
        ability: fact.ability.clone(),
        actor: fact.actor,
        sequence: 1,
        target: fact.target,
        position: fact.position,
        amount: 20,
        remaining: 980,
    }];
    assert!(b.encode_abilities(&f.weapons, &f.profile).is_err());
    b.events = vec![Event::Avoided {
        ability: fact.ability,
        actor: fact.actor,
        sequence: 1,
        target: fact.target,
        position: fact.position,
        kind: crate::simulation::combat::Avoidance::Miss,
    }];
    assert!(b.encode_abilities(&f.weapons, &f.profile).is_err());
}
#[test]
fn landed_spells_and_specials_with_disabled_or_zero_critical_use_the_none_sentinel() {
    for disabled in [true, false] {
        let mut p: serde_json::Value = serde_json::from_slice(POLICY).unwrap();
        for row in p["abilities"].as_array_mut().unwrap() {
            if disabled {
                row["can_crit"] = serde_json::json!(false);
            } else {
                row["critical_bp"] = serde_json::json!(0);
            }
        }
        let f = Fixture::with(&serde_json::to_vec(&p).unwrap());
        for magic in [false, true] {
            let b = f.batch(f.fact(magic, 9999, None, 1000));
            let encoded = b.encode_abilities(&f.weapons, &f.profile).unwrap();
            assert_eq!(
                &encoded[HEADER_BYTES + 40..HEADER_BYTES + 42],
                &u16::MAX.to_le_bytes()
            );
            assert_eq!(
                Batch::decode_abilities(&encoded, &f.weapons, &f.profile).unwrap(),
                b
            );
        }
    }
}
