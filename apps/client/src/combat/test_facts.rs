//! Presentation-only typed observations. These bypass admission/decoding in unit
//! tests; World and codec qualification provide their separate authority proof.
use std::num::NonZeroU64;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    damage::{
        Range, Weapon,
        white_melee::{
            Outcome,
            impact::{self, GlancePercent, Impact},
        },
    },
    hit_outcomes::Roll,
    identity::{EntityId, EntityKey, WorldEpoch},
};
use theia_network::simulation::combat::white_melee::{DefenderState, Draws, Fact};
pub(crate) fn fact(outcome: Outcome, incoming: bool, health: u32) -> Fact {
    let owner = EntityKey {
        entity: EntityId::new(13).unwrap(),
        epoch: WorldEpoch::new(7).unwrap(),
    };
    let npc = EntityKey {
        entity: EntityId::new(27).unwrap(),
        ..owner
    };
    let (starting, base) = if incoming {
        (Range::new(4, 6).unwrap().starting_damage(5).unwrap(), 5)
    } else {
        (
            Weapon::new(Range::new(10, 12).unwrap(), 2400)
                .unwrap()
                .starting_damage(11, 28)
                .unwrap(),
            11,
        )
    };
    let impact = match outcome {
        Outcome::Miss => Impact::Miss,
        Outcome::Dodge => Impact::Dodge,
        Outcome::Parry => Impact::Parry,
        Outcome::Hit => Impact::Hit,
        Outcome::Critical => Impact::Critical,
        Outcome::Crushing => Impact::Crushing,
        Outcome::Block => Impact::Block { value: 3 },
        Outcome::Glancing => Impact::Glancing(GlancePercent::new(65).unwrap()),
    };
    let result = impact::physical_reference_v1(starting, 1, 100, 0, health, impact).unwrap();
    Fact {
        attack: ContentId::new(if incoming {
            "theia.dev.wolf-bite"
        } else {
            "theia.dev.training-sword-swing"
        })
        .unwrap(),
        actor: if incoming { npc } else { owner },
        actor_generation: if incoming { None } else { NonZeroU64::new(1) },
        sequence: 1,
        target: if incoming { owner } else { npc },
        target_generation: if incoming { NonZeroU64::new(1) } else { None },
        position: GlobalPosition::new([0., 0., 0.]).unwrap(),
        defender: DefenderState {
            standing: true,
            can_react: true,
            in_front: true,
            ..Default::default()
        },
        draws: Draws {
            base,
            outcome: Roll::new(if outcome == Outcome::Critical {
                if incoming { 1000 } else { 550 }
            } else {
                9999
            })
            .unwrap(),
            glancing: (outcome == Outcome::Glancing).then_some(65),
        },
        result,
        remaining: health - result.health_lost,
    }
}

pub(crate) fn ability_fact(
    outcome: theia_network::simulation::combat::ability_damage::Outcome,
    health: u32,
) -> theia_network::simulation::combat::ability_damage::Fact {
    use theia_core::damage::{Amount, ability_impact};
    use theia_network::simulation::combat::ability_damage::{
        Draws, Fact, Outcome as AbilityOutcome, Resolved,
    };
    let ordinary = fact(Outcome::Hit, false, 100);
    let magic = matches!(outcome, AbilityOutcome::Arcane(_));
    let starting = if magic {
        Amount::fixed(20).unwrap()
    } else {
        Weapon::new(Range::new(16, 16).unwrap(), 2400)
            .unwrap()
            .starting_damage(16, 28)
            .unwrap()
    };
    let damage = match outcome {
        AbilityOutcome::MeleeSpecial(value) => {
            ability_impact::physical_special_reference_v1(starting, value, 1, 100, 0, health)
                .unwrap()
        }
        AbilityOutcome::Arcane(value) => {
            ability_impact::magic_zero_resistance_reference_v1(starting, value, health).unwrap()
        }
    };
    Fact {
        ability: ContentId::new(if magic {
            "theia.dev.arcane-bolt"
        } else {
            "theia.dev.sword-slash"
        })
        .unwrap(),
        actor: ordinary.actor,
        actor_generation: NonZeroU64::new(1).unwrap(),
        sequence: 1,
        target: ordinary.target,
        position: ordinary.position,
        defender: ordinary.defender,
        draws: Draws {
            base: if magic { 20 } else { 11 },
            landing: Roll::new(9999).unwrap(),
            critical: Some(Roll::new(0).unwrap()),
        },
        result: Resolved { outcome, damage },
        remaining: health - damage.health_lost,
    }
}
