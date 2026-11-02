use super::*;
use crate::{
    combat_wire::{Batch, MAX_BYTES},
    simulation::{
        combat::{Event, Reason, physical},
        creatures,
    },
};
use std::sync::Arc;
use theia_core::{
    abilities, coordinates::GlobalPosition, damage::white_melee::Outcome, inventory, melee,
};
const STATS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-physical-damage-stats.json");
const DATA: &[u8] = include_bytes!("../../../../../content/gameplay/development-white-melee.json");
struct Fixture {
    abilities: abilities::Catalog,
    weapons: melee::Catalog,
    profile: Profile,
}
impl Fixture {
    fn new() -> Self {
        Self::with(STATS, DATA)
    }
    fn with(stats: &[u8], data: &[u8]) -> Self {
        let abilities = abilities::Catalog::from_json(include_bytes!(
            "../../../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap();
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
        let physical = physical::Profile::new(
            include_bytes!(
                "../../../../../content/gameplay/development-physical-damage-definitions.json"
            ),
            stats,
            physical::Bindings {
                weapons: &weapons,
                items: &items,
                abilities: &abilities,
                creature: &creature,
                player_maximum_health: 100,
                creature_maximum_health: 1000,
            },
        )
        .unwrap();
        Self {
            abilities,
            weapons,
            profile: Profile::new(data, Arc::new(physical)).unwrap(),
        }
    }
    fn fact(&self, weapon: bool, health: u32, outcome: u16, glancing: Option<u8>) -> Fact {
        let defender = DefenderState {
            standing: true,
            can_react: true,
            in_front: true,
            usable_melee_weapon: true,
            usable_shield: true,
            ..Default::default()
        };
        let draws = Draws {
            base: if weapon { 11 } else { 5 },
            outcome: Roll::new(outcome).unwrap(),
            glancing,
        };
        let (attack, actor, actor_generation, target, target_generation, result) = if weapon {
            let item = ContentId::new("theia.dev.training-sword").unwrap();
            (
                self.profile.physical().weapon(&item).unwrap().0.clone(),
                key(7, 13).unwrap(),
                NonZeroU64::new(1),
                key(7, 27).unwrap(),
                None,
                self.profile
                    .evaluate_outgoing(&item, defender, draws, health)
                    .unwrap(),
            )
        } else {
            (
                self.profile.physical().creature_attack().0.clone(),
                key(7, 27).unwrap(),
                None,
                key(7, 13).unwrap(),
                NonZeroU64::new(1),
                self.profile
                    .evaluate_incoming(defender, draws, health)
                    .unwrap(),
            )
        };
        Fact {
            attack,
            actor,
            actor_generation,
            sequence: 1,
            target,
            target_generation,
            position: GlobalPosition::new([8008., -11.9, 8000.]).unwrap(),
            defender,
            draws,
            result,
            remaining: health - result.health_lost,
        }
    }
    fn batch(&self, fact: Fact) -> Batch {
        Batch {
            epoch: 7,
            sequence: 1,
            tick: 20,
            events: vec![Event::WhiteMelee(fact)],
        }
    }
    fn encode(&self, batch: &Batch) -> Result<Vec<u8>> {
        batch.encode_white(&self.abilities, &self.weapons, &self.profile)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Batch> {
        Batch::decode_white(bytes, &self.abilities, &self.weapons, &self.profile)
    }
}
#[test]
fn explicit_context_round_trips_both_directions_and_avoidance_crit_and_overkill() {
    let f = Fixture::new();
    for (weapon, roll) in [
        (true, 0),
        (true, 50),
        (true, 550),
        (true, 9999),
        (false, 0),
        (false, 500),
        (false, 1000),
        (false, 9999),
    ] {
        for health in [1, 100] {
            let batch = f.batch(f.fact(weapon, health, roll, None));
            let bytes = f.encode(&batch).unwrap();
            assert!(bytes.starts_with(EVENTS_TAG));
            assert_eq!(&bytes[4..36], &f.profile.context_hash());
            assert_eq!(bytes.len(), HEADER_BYTES + FACT_BYTES);
            assert_eq!(f.decode(&bytes).unwrap(), batch);
        }
    }
}
#[test]
fn encoding_rejects_wrong_namespace_sequence_or_inconsistent_amounts_and_draws() {
    let f = Fixture::new();
    let original = f.fact(true, 1000, 9999, None);
    let mut bad = Vec::new();
    let mut x = original.clone();
    x.sequence = 0;
    bad.push(x);
    let mut x = original.clone();
    x.actor = x.target;
    bad.push(x);
    let mut x = original.clone();
    x.actor = key(8, 13).unwrap();
    bad.push(x);
    let mut x = original.clone();
    x.actor_generation = None;
    bad.push(x);
    let mut x = original.clone();
    x.target_generation = NonZeroU64::new(1);
    bad.push(x);
    let mut x = original.clone();
    x.attack = ContentId::new("theia.dev.missing").unwrap();
    bad.push(x);
    let mut x = original.clone();
    x.draws.base = 9;
    bad.push(x);
    let mut x = original.clone();
    x.draws.glancing = Some(65);
    bad.push(x);
    let mut x = original.clone();
    x.result.outcome = Outcome::Critical;
    bad.push(x);
    let mut x = original.clone();
    x.result.starting_damage += 1;
    bad.push(x);
    let mut x = original.clone();
    x.result.armour.as_mut().unwrap().prevented += 1;
    bad.push(x);
    let mut x = original.clone();
    x.result.outcome_damage += 1;
    bad.push(x);
    let mut x = original.clone();
    x.result.blocked = 1;
    bad.push(x);
    let mut x = original.clone();
    x.result.resolved_damage += 1;
    bad.push(x);
    let mut x = original.clone();
    x.result.health_lost += 1;
    bad.push(x);
    let mut x = original.clone();
    x.result.overkill += 1;
    bad.push(x);
    let mut x = original.clone();
    x.remaining = u32::MAX;
    bad.push(x);
    let mut x = original;
    x.defender.standing = false;
    bad.push(x);
    for fact in bad {
        assert!(f.encode(&f.batch(fact)).is_err());
    }
    let mut incoming = f.fact(false, 100, 9999, None);
    incoming.target_generation = None;
    assert!(f.encode(&f.batch(incoming)).is_err());
}
#[test]
fn truncation_trailing_oversize_ordinals_rolls_state_health_and_nonfinite_positions_fail_closed() {
    let f = Fixture::new();
    let bytes = f.encode(&f.batch(f.fact(true, 1000, 9999, None))).unwrap();
    for n in 0..bytes.len() {
        assert!(f.decode(&bytes[..n]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(f.decode(&extra).is_err());
    assert!(f.decode(&vec![0; MAX_BYTES + 1]).is_err());
    for offset in [0, 4, HEADER_BYTES, HEADER_BYTES + 25] {
        let mut bad = bytes.clone();
        bad[offset] ^= 255;
        assert!(f.decode(&bad).is_err(), "offset {offset}");
    }
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 46..HEADER_BYTES + 48].copy_from_slice(&10000u16.to_le_bytes());
    assert!(f.decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 49] |= 128;
    assert!(f.decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 48] = 65;
    assert!(f.decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 26..HEADER_BYTES + 34].fill(0);
    assert!(f.decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 34..HEADER_BYTES + 42].copy_from_slice(&1u64.to_le_bytes());
    assert!(f.decode(&bad).is_err());
    for offset in [50, 54] {
        let mut bad = bytes.clone();
        bad[HEADER_BYTES + offset..HEADER_BYTES + offset + 4].fill(0);
        assert!(f.decode(&bad).is_err());
    }
    let mut bad = bytes;
    bad[HEADER_BYTES + 58..HEADER_BYTES + 66].copy_from_slice(&f64::NAN.to_bits().to_le_bytes());
    assert!(f.decode(&bad).is_err());
}
#[test]
fn every_older_codec_and_changed_exact_context_refuses_the_new_layout() {
    let f = Fixture::new();
    let batch = f.batch(f.fact(true, 1000, 9999, None));
    let bytes = f.encode(&batch).unwrap();
    assert!(Batch::decode(&bytes, &f.abilities).is_err());
    assert!(Batch::decode_melee(&bytes, &f.abilities, &f.weapons).is_err());
    assert!(
        Batch::decode_physical(&bytes, &f.abilities, &f.weapons, f.profile.physical()).is_err()
    );
    assert!(batch.encode(&f.abilities).is_err());
    assert!(batch.encode_melee(&f.abilities, &f.weapons).is_err());
    assert!(
        batch
            .encode_physical(&f.abilities, &f.weapons, f.profile.physical())
            .is_err()
    );
    let mut altered = DATA.to_vec();
    altered.push(b' ');
    assert!(Fixture::with(STATS, &altered).decode(&bytes).is_err());
    let mut altered = STATS.to_vec();
    altered.push(b' ');
    assert!(Fixture::with(&altered, DATA).decode(&bytes).is_err());
    let old = Batch {
        events: vec![Event::Cancelled {
            actor: key(7, 13).unwrap(),
            sequence: 1,
            reason: Reason::NotReady,
        }],
        ..batch
    };
    assert!(
        f.decode(
            &old.encode_physical(&f.abilities, &f.weapons, f.profile.physical())
                .unwrap()
        )
        .is_err()
    );
}
#[test]
fn twelve_full_facts_fit_the_existing_packet_budget_and_the_event_count_stays_bounded() {
    let f = Fixture::new();
    let mut batch = f.batch(f.fact(true, 1000, 9999, None));
    batch.events.clear();
    let mut health = 1000;
    for n in 0..12 {
        let mut fact = f.fact(true, health, 9999, None);
        fact.actor = key(7, 100 + n).unwrap();
        health = fact.remaining;
        batch.events.push(Event::WhiteMelee(fact));
    }
    let bytes = f.encode(&batch).unwrap();
    assert_eq!(bytes.len(), 1045);
    assert!(bytes.len() <= MAX_BYTES);
    assert_eq!(f.decode(&bytes).unwrap(), batch);
    let mut extra = f.fact(true, health, 9999, None);
    extra.actor = key(7, 999).unwrap();
    batch.events.push(Event::WhiteMelee(extra));
    assert!(f.encode(&batch).is_err());
}
#[test]
fn duplicate_actor_life_sequences_are_rejected_and_other_namespaces_are_independent() {
    let f = Fixture::new();
    let fact = f.fact(true, 1000, 9999, None);
    let mut batch = f.batch(fact.clone());
    batch.events.push(Event::WhiteMelee(fact.clone()));
    assert!(f.encode(&batch).is_err());
    let mut other = fact.clone();
    other.actor = key(7, 14).unwrap();
    batch.events[1] = Event::WhiteMelee(other);
    let mut bytes = f.encode(&batch).unwrap();
    bytes[HEADER_BYTES + FACT_BYTES + 1..HEADER_BYTES + FACT_BYTES + 9]
        .copy_from_slice(&fact.actor.entity.value().to_le_bytes());
    assert!(f.decode(&bytes).is_err());
    let mut other = fact;
    other.actor_generation = NonZeroU64::new(2);
    batch.events[1] = Event::WhiteMelee(other);
    assert!(f.encode(&batch).is_ok());
}
#[test]
fn new_batches_keep_existing_ability_death_events_and_reject_normal_physical_downgrades() {
    let f = Fixture::new();
    let mut batch = f.batch(f.fact(true, 1000, 9999, None));
    batch.events.push(Event::Cancelled {
        actor: key(7, 13).unwrap(),
        sequence: 99,
        reason: Reason::NotReady,
    });
    batch.events.push(Event::Died {
        target: key(7, 28).unwrap(),
        position: GlobalPosition::new([8000., 0., 8000.]).unwrap(),
        respawn_at: 300,
    });
    assert_eq!(f.decode(&f.encode(&batch).unwrap()).unwrap(), batch);
    let normal = f
        .profile
        .physical()
        .weapon_result(
            &ContentId::new("theia.dev.training-sword").unwrap(),
            11,
            1000,
        )
        .unwrap();
    let old = crate::simulation::combat::physical::Fact {
        attack: ContentId::new("theia.dev.training-sword-swing").unwrap(),
        actor: key(7, 13).unwrap(),
        actor_generation: NonZeroU64::new(1),
        sequence: 2,
        target: key(7, 27).unwrap(),
        target_generation: None,
        position: GlobalPosition::new([8000., 0., 8000.]).unwrap(),
        roll: 11,
        result: normal,
        remaining: 1000 - normal.health_lost,
    };
    batch.events = vec![Event::PhysicalDamage(old)];
    assert!(f.encode(&batch).is_err());
}
#[test]
fn high_level_glance_and_crushing_reconstruct_exact_outcome_amounts_and_check_draw_bounds() {
    let mut stats: serde_json::Value = serde_json::from_slice(STATS).unwrap();
    stats["player"]["physical"]["level"] = 60.into();
    stats["creatures"][0]["physical"]["level"] = 63.into();
    let mut data: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    data["player"]["weapons"][0]["skill"] = 300.into();
    data["player"]["defense_skill"] = 300.into();
    let f = Fixture::with(
        &serde_json::to_vec(&stats).unwrap(),
        &serde_json::to_vec(&data).unwrap(),
    );
    for (weapon, roll, glance, expected) in [
        (true, 1450, Some(65), Outcome::Glancing),
        (false, 1440, None, Outcome::Crushing),
    ] {
        let fact = f.fact(weapon, 100, roll, glance);
        assert_eq!(fact.result.outcome, expected);
        let batch = f.batch(fact);
        let bytes = f.encode(&batch).unwrap();
        assert_eq!(f.decode(&bytes).unwrap(), batch);
        if weapon {
            for invalid in [0, 54, 76, 255] {
                let mut bad = bytes.clone();
                bad[HEADER_BYTES + 48] = invalid;
                assert!(f.decode(&bad).is_err());
            }
        }
    }
}
#[test]
fn defender_state_bits_preserve_the_documented_independent_facts() {
    let f = Fixture::new();
    let mut fact = f.fact(true, 1000, 9999, None);
    fact.defender = DefenderState {
        standing: true,
        casting_non_melee: true,
        disarmed: true,
        usable_shield: true,
        ..Default::default()
    };
    let batch = f.batch(fact);
    let bytes = f.encode(&batch).unwrap();
    assert_eq!(bytes[HEADER_BYTES + 49], 85);
    assert_eq!(f.decode(&bytes).unwrap(), batch);
}
