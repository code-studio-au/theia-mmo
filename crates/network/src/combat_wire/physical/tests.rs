use super::*;
use crate::{
    combat_wire::{Batch, MAX_BYTES},
    simulation::{
        combat::{Event, Reason, physical::Bindings},
        creatures,
    },
};
use theia_core::{abilities, coordinates::GlobalPosition, damage::Amount, inventory, melee};
const DEFINITIONS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-physical-damage-definitions.json");
const STATS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-physical-damage-stats.json");
struct Fixture {
    abilities: abilities::Catalog,
    weapons: melee::Catalog,
    profile: Profile,
}
impl Fixture {
    fn new() -> Self {
        Self::with_stats(STATS)
    }
    fn with_stats(stats: &[u8]) -> Self {
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
        let profile = Profile::new(
            DEFINITIONS,
            stats,
            Bindings {
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
            profile,
        }
    }
    fn fact(&self, weapon: bool, health: u32) -> Fact {
        let position = GlobalPosition::new([8008., -11.9, 8000.]).unwrap();
        let generation = NonZeroU64::new(1);
        let (attack, actor, actor_generation, target, target_generation, roll, result) = if weapon {
            let item = ContentId::new("theia.dev.training-sword").unwrap();
            let (attack, _) = self.profile.weapon(&item).unwrap();
            (
                attack.clone(),
                key(7, 3).unwrap(),
                generation,
                key(7, 1).unwrap(),
                None,
                11,
                self.profile.weapon_result(&item, 11, health).unwrap(),
            )
        } else {
            (
                self.profile.creature_attack().0.clone(),
                key(7, 1).unwrap(),
                None,
                key(7, 3).unwrap(),
                generation,
                5,
                self.profile.creature_result(5, health).unwrap(),
            )
        };
        Fact {
            attack,
            actor,
            actor_generation,
            sequence: 1,
            target,
            target_generation,
            position,
            roll,
            result,
            remaining: health - result.health_lost,
        }
    }
    fn batch(&self, fact: Fact) -> Batch {
        Batch {
            epoch: 7,
            sequence: 1,
            tick: 20,
            events: vec![Event::PhysicalDamage(fact)],
        }
    }
    fn encode(&self, batch: &Batch) -> Result<Vec<u8>> {
        batch.encode_physical(&self.abilities, &self.weapons, &self.profile)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Batch> {
        Batch::decode_physical(bytes, &self.abilities, &self.weapons, &self.profile)
    }
}
#[test]
fn both_directions_round_trip_exact_facts_and_overkill_with_explicit_profile_hash() {
    let fixture = Fixture::new();
    for (weapon, health) in [(true, 1000), (true, 10), (false, 100), (false, 3)] {
        let batch = fixture.batch(fixture.fact(weapon, health));
        let bytes = fixture.encode(&batch).unwrap();
        assert!(bytes.starts_with(EVENTS_TAG));
        assert_eq!(&bytes[4..36], &fixture.profile.context_hash());
        assert_eq!(bytes.len(), HEADER_BYTES + FACT_BYTES);
        assert_eq!(fixture.decode(&bytes).unwrap(), batch);
    }
}
#[test]
fn malformed_result_identity_namespace_and_math_are_rejected_before_encoding() {
    let fixture = Fixture::new();
    let original = fixture.fact(true, 1000);
    let mut invalid = Vec::new();
    let mut fact = original.clone();
    fact.sequence = 0;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.actor = fact.target;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.actor = key(8, 3).unwrap();
    invalid.push(fact);
    let mut fact = original.clone();
    fact.actor_generation = None;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.target_generation = NonZeroU64::new(1);
    invalid.push(fact);
    let mut fact = original.clone();
    fact.attack = ContentId::new("theia.test.unknown").unwrap();
    invalid.push(fact);
    let mut fact = original.clone();
    fact.roll = 9;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.starting_damage += 1;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.armour_prevented += 1;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.resolved_damage += 1;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.health_lost += 1;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.overkill += 1;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.effective_armour += 1;
    invalid.push(fact);
    let mut fact = original.clone();
    fact.result.starting_exact = Amount::fixed(1).unwrap();
    invalid.push(fact);
    let mut fact = original.clone();
    fact.remaining = u32::MAX;
    invalid.push(fact);
    let mut fact = original;
    fact.result = fixture
        .profile
        .weapon_result(&ContentId::new("theia.dev.training-sword").unwrap(), 11, 0)
        .unwrap();
    fact.remaining = 0;
    invalid.push(fact);
    for fact in invalid {
        assert!(fixture.encode(&fixture.batch(fact)).is_err());
    }
    let mut incoming = fixture.fact(false, 100);
    incoming.target_generation = None;
    assert!(fixture.encode(&fixture.batch(incoming)).is_err());
}
#[test]
fn malformed_bytes_truncation_trailing_payload_and_unknown_ordinals_fail_closed() {
    let fixture = Fixture::new();
    let bytes = fixture
        .encode(&fixture.batch(fixture.fact(true, 1000)))
        .unwrap();
    for n in 0..bytes.len() {
        assert!(fixture.decode(&bytes[..n]).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(fixture.decode(&trailing).is_err());
    assert!(fixture.decode(&vec![0; MAX_BYTES + 1]).is_err());
    for offset in [
        0,
        4,
        HEADER_BYTES,
        HEADER_BYTES + 25,
        HEADER_BYTES + 46,
        HEADER_BYTES + 50,
        HEADER_BYTES + 54,
        HEADER_BYTES + 58,
        HEADER_BYTES + 62,
    ] {
        let mut bad = bytes.clone();
        bad[offset] ^= 0xff;
        assert!(fixture.decode(&bad).is_err(), "offset {offset}");
    }
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 26..HEADER_BYTES + 34].fill(0);
    assert!(fixture.decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[HEADER_BYTES + 34..HEADER_BYTES + 42].copy_from_slice(&1u64.to_le_bytes());
    assert!(fixture.decode(&bad).is_err());
    let mut bad = bytes;
    bad[HEADER_BYTES + 70..HEADER_BYTES + 78].copy_from_slice(&f64::NAN.to_bits().to_le_bytes());
    assert!(fixture.decode(&bad).is_err());
}
#[test]
fn legacy_layouts_and_different_exact_profile_bytes_cannot_decode_new_results() {
    let fixture = Fixture::new();
    let batch = fixture.batch(fixture.fact(true, 1000));
    let bytes = fixture.encode(&batch).unwrap();
    assert!(Batch::decode(&bytes, &fixture.abilities).is_err());
    assert!(Batch::decode_melee(&bytes, &fixture.abilities, &fixture.weapons).is_err());
    assert!(batch.encode(&fixture.abilities).is_err());
    assert!(
        batch
            .encode_melee(&fixture.abilities, &fixture.weapons)
            .is_err()
    );
    let mut alternate = STATS.to_vec();
    alternate.push(b' ');
    let other = Fixture::with_stats(&alternate);
    assert!(other.decode(&bytes).is_err());
    let old = Batch {
        events: vec![Event::Cancelled {
            actor: key(7, 3).unwrap(),
            sequence: 1,
            reason: Reason::NotReady,
        }],
        ..batch
    };
    assert!(
        fixture
            .decode(
                &old.encode_melee(&fixture.abilities, &fixture.weapons)
                    .unwrap()
            )
            .is_err()
    );
}
#[test]
fn one_packet_holds_twelve_full_results_inside_the_existing_byte_bound() {
    let fixture = Fixture::new();
    let mut health = 1000;
    let mut events = Vec::new();
    for n in 0..12 {
        let mut fact = fixture.fact(true, health);
        fact.actor = key(7, 3 + n).unwrap();
        health = fact.remaining;
        events.push(Event::PhysicalDamage(fact));
    }
    let batch = Batch {
        epoch: 7,
        sequence: 1,
        tick: 20,
        events,
    };
    let bytes = fixture.encode(&batch).unwrap();
    assert_eq!(bytes.len(), 1189);
    assert!(bytes.len() <= MAX_BYTES);
    assert_eq!(fixture.decode(&bytes).unwrap(), batch);
    let mut too_many = batch;
    let mut extra = fixture.fact(true, health);
    extra.actor = key(7, 99).unwrap();
    too_many.events.push(Event::PhysicalDamage(extra));
    assert!(fixture.encode(&too_many).is_err());
}
#[test]
fn duplicate_source_sequences_are_rejected_but_separate_actor_namespaces_remain_valid() {
    let fixture = Fixture::new();
    let fact = fixture.fact(true, 1000);
    let mut batch = fixture.batch(fact.clone());
    batch.events.push(Event::PhysicalDamage(fact.clone()));
    assert!(fixture.encode(&batch).is_err());
    let mut second = fact;
    second.actor = key(7, 4).unwrap();
    batch.events[1] = Event::PhysicalDamage(second);
    assert!(fixture.encode(&batch).is_ok());
    let mut bytes = fixture.encode(&batch).unwrap();
    let second_actor = HEADER_BYTES + FACT_BYTES + 1;
    bytes[second_actor..second_actor + 8].copy_from_slice(&3u64.to_le_bytes());
    assert!(fixture.decode(&bytes).is_err());
}
#[test]
fn explicit_mixed_ability_events_work_while_legacy_fixed_melee_is_forbidden() {
    let fixture = Fixture::new();
    let fact = fixture.fact(true, 1000);
    let mut batch = fixture.batch(fact.clone());
    batch.events.push(Event::Damage {
        ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
        actor: key(7, 4).unwrap(),
        sequence: 1,
        target: fact.target,
        position: fact.position,
        amount: 20,
        remaining: fact.remaining - 20,
    });
    assert_eq!(
        fixture.decode(&fixture.encode(&batch).unwrap()).unwrap(),
        batch
    );
    batch.events[1] = Event::MeleeDamage {
        weapon: ContentId::new("theia.dev.training-sword").unwrap(),
        actor: key(7, 4).unwrap(),
        sequence: 1,
        target: fact.target,
        position: fact.position,
        amount: 15,
        remaining: fact.remaining - 15,
    };
    assert!(fixture.encode(&batch).is_err());
}
