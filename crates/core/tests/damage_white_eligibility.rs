use theia_core::damage::white_melee::eligibility::{
    AttackerCapabilities, Combatant, DefenderFacts, Kind, physical_standing_reference_v1 as resolve,
};
fn actor(kind: Kind, level: u8) -> Combatant {
    Combatant::new(kind, level).unwrap()
}
fn capabilities() -> AttackerCapabilities {
    AttackerCapabilities {
        critical: true,
        glancing: true,
        crushing: true,
    }
}
fn facts() -> DefenderFacts {
    DefenderFacts {
        standing: true,
        can_react: true,
        in_front: true,
        can_dodge: true,
        can_parry: true,
        can_block: true,
        usable_melee_weapon: true,
        usable_shield: true,
        casting_non_melee: false,
        disarmed: false,
    }
}
#[test]
fn behind_player_cannot_dodge_parry_or_block_but_creature_can_dodge() {
    let behind = DefenderFacts {
        in_front: false,
        ..facts()
    };
    let p = actor(Kind::Player, 60);
    let npc = actor(Kind::UncontrolledCreature, 63);
    let player = resolve(npc, p, capabilities(), behind).unwrap();
    assert!(!player.dodge && !player.parry && !player.block);
    let creature = resolve(p, npc, capabilities(), behind).unwrap();
    assert!(creature.dodge && !creature.parry && !creature.block);
    let front = resolve(npc, p, capabilities(), facts()).unwrap();
    assert!(front.dodge && front.parry && front.block);
}
#[test]
fn player_parry_and_block_require_distinct_usable_gear() {
    let p = actor(Kind::Player, 1);
    let npc = actor(Kind::UncontrolledCreature, 1);
    for weapon in [false, true] {
        for shield in [false, true] {
            let gear = DefenderFacts {
                usable_melee_weapon: weapon,
                usable_shield: shield,
                ..facts()
            };
            let eligible = resolve(npc, p, capabilities(), gear).unwrap();
            assert_eq!(eligible.parry, weapon);
            assert_eq!(eligible.block, shield);
            assert!(eligible.dodge);
        }
    }
    let npc_caps = resolve(
        p,
        npc,
        capabilities(),
        DefenderFacts {
            usable_melee_weapon: false,
            usable_shield: false,
            ..facts()
        },
    )
    .unwrap();
    assert!(npc_caps.parry && npc_caps.block);
    let wolf = resolve(
        p,
        npc,
        capabilities(),
        DefenderFacts {
            can_parry: false,
            can_block: false,
            ..facts()
        },
    )
    .unwrap();
    assert!(!wolf.parry && !wolf.block);
    for (a, d) in [(p, npc), (npc, p)] {
        let disarmed = resolve(
            a,
            d,
            capabilities(),
            DefenderFacts {
                disarmed: true,
                ..facts()
            },
        )
        .unwrap();
        assert!(!disarmed.parry && disarmed.dodge);
    }
}
#[test]
fn lost_reaction_or_nonmelee_cast_suppresses_defensive_avoidance_only() {
    let p = actor(Kind::Player, 60);
    let npc = actor(Kind::UncontrolledCreature, 63);
    for state in [
        DefenderFacts {
            can_react: false,
            ..facts()
        },
        DefenderFacts {
            casting_non_melee: true,
            ..facts()
        },
    ] {
        for (a, d) in [(p, npc), (npc, p)] {
            let eligible = resolve(a, d, capabilities(), state).unwrap();
            assert!(!eligible.dodge && !eligible.parry && !eligible.block);
            assert!(eligible.critical);
        }
    }
}
#[test]
fn glancing_and_crushing_have_different_actor_categories_and_capabilities() {
    let p = actor(Kind::Player, 60);
    let npc = actor(Kind::UncontrolledCreature, 63);
    let outgoing = resolve(p, npc, capabilities(), facts()).unwrap();
    assert!(outgoing.glancing && !outgoing.crushing);
    let incoming = resolve(npc, p, capabilities(), facts()).unwrap();
    assert!(incoming.crushing && !incoming.glancing);
    for (a, d) in [(p, p), (npc, npc)] {
        let e = resolve(a, d, capabilities(), facts()).unwrap();
        assert!(!e.glancing && !e.crushing);
    }
    let disabled = resolve(p, npc, AttackerCapabilities::default(), facts()).unwrap();
    assert!(!disabled.glancing && !disabled.critical && !disabled.crushing);
    assert!(disabled.dodge);
}
#[test]
fn reference_glancing_gate_excludes_level_ten_and_lower_creatures() {
    let p = actor(Kind::Player, 1);
    for level in 1..=63 {
        let e = resolve(
            p,
            actor(Kind::UncontrolledCreature, level),
            capabilities(),
            facts(),
        )
        .unwrap();
        assert_eq!(e.glancing, level > 10);
    }
}
#[test]
fn unsupported_levels_and_nonstanding_targets_are_refused() {
    for kind in [Kind::Player, Kind::UncontrolledCreature] {
        for level in [0, 64, 80, u8::MAX] {
            assert!(Combatant::new(kind, level).is_none());
        }
    }
    let p = actor(Kind::Player, 1);
    let npc = actor(Kind::UncontrolledCreature, 1);
    assert!(
        resolve(
            p,
            npc,
            capabilities(),
            DefenderFacts {
                standing: false,
                ..facts()
            }
        )
        .is_err()
    );
}
