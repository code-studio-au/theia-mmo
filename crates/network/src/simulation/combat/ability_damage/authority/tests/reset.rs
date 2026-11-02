//! Explicit reset World prerequisite. No normal manifest/client adoption yet.
use super::*;
const RESET: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-return-reset.json"
);
const CREDIT: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-credit-solo.json"
);
fn reset_world(rate: TickRate, reset: bool) -> (World, EntityKey) {
    let (mut world, npc) = super::credit::credit_world_profile(rate, false, 32, CREDIT);
    if reset {
        world.enable_creature_reset(RESET).unwrap();
    }
    (world, npc)
}
fn hurt(world: &mut World, npc: EntityKey) -> u64 {
    let id = join(world);
    cast(world, id, npc, 1, "theia.dev.sword-slash");
    world.step();
    world.step();
    assert!(world.combat.as_ref().unwrap().creatures[0].current < 1000);
    id
}
fn force_return(world: &mut World) {
    world.patrol.as_mut().unwrap().motion_blocked = true;
    world.step();
}
#[test]
fn reset_requires_strict_bounded_matching_pre_admission_context() {
    use crate::simulation::combat::reset::Profile;
    assert_eq!(
        Profile::from_json(RESET).unwrap().id().as_str(),
        "theia.dev.creature-return-reset"
    );
    for (field, value) in [
        ("schema", json!(2)),
        ("ruleset", json!("classic_verified")),
        ("id", json!("")),
        ("creature", Value::Null),
        ("unknown", json!(true)),
    ] {
        let mut raw: Value = serde_json::from_slice(RESET).unwrap();
        raw[field] = value;
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    assert!(
        Profile::from_json(&vec![
            b' ';
            crate::simulation::combat::reset::MAX_PROFILE_BYTES + 1
        ])
        .is_err()
    );
    let (mut old, _) = fixture(TickRate::Hz30);
    let identity = old.identity();
    assert!(old.enable_creature_reset(RESET).is_err());
    assert_eq!(old.identity(), identity);
    let (mut world, _) = reset_world(TickRate::Hz30, false);
    let identity = world.identity();
    let mut wrong: Value = serde_json::from_slice(RESET).unwrap();
    wrong["creature"] = json!("theia.dev.other-creature");
    assert!(
        world
            .enable_creature_reset(&serde_json::to_vec(&wrong).unwrap())
            .is_err()
    );
    assert_eq!(world.identity(), identity);
    world.enable_creature_reset(RESET).unwrap();
    let identity = world.identity();
    assert!(world.enable_creature_reset(RESET).is_err());
    assert_eq!(world.identity(), identity);
    let (mut admitted, _) = reset_world(TickRate::Hz30, false);
    join(&mut admitted);
    let identity = admitted.identity();
    assert!(admitted.enable_creature_reset(RESET).is_err());
    assert_eq!(admitted.identity(), identity);
}
#[test]
fn reset_binds_exact_bytes_after_credit_in_world_identity() {
    let (mut world, _) = reset_world(TickRate::Hz30, false);
    let prior = world
        .combat
        .as_ref()
        .unwrap()
        .identity(world.patrol_identity.unwrap());
    let profile = crate::simulation::combat::reset::Profile::from_json(RESET).unwrap();
    world.enable_creature_reset(RESET).unwrap();
    assert_eq!(
        world.identity(),
        crate::classic::profile_identity(profile.identity(prior))
    );
    let (mut changed, _) = reset_world(TickRate::Hz30, false);
    let mut bytes = RESET.to_vec();
    bytes.push(b' ');
    changed.enable_creature_reset(&bytes).unwrap();
    assert_ne!(world.identity(), changed.identity());
}
#[test]
fn return_restores_health_clears_injected_effects_and_membership_once_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = reset_world(rate, true);
        let id = hurt(&mut world, npc);
        let health = world.combat.as_ref().unwrap().creatures[0].current;
        let now = world.tick;
        let source = world.entity_key(id).unwrap();
        // The current typed catalogue has no DoT ability. Inject only through
        // the trusted existing Registry to qualify cleanup of future effects.
        let cast = crate::simulation::combat::Cast {
            sequence: 1,
            ability: ContentId::new("theia.dev.sword-slash").unwrap(),
            target: npc,
            started: now,
            completes: now,
        };
        world
            .combat
            .as_mut()
            .unwrap()
            .statuses
            .as_mut()
            .unwrap()
            .apply(
                now,
                rate,
                source,
                &cast,
                &ContentId::new("theia.dev.scorch").unwrap(),
            )
            .unwrap();
        assert_eq!(world.combat_status_snapshot(id).unwrap().len(), 1);
        force_return(&mut world);
        assert_eq!(
            world.creature_pursuit_directive().unwrap().mode,
            theia_core::pursuit::Mode::Return
        );
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].key, npc);
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 1000);
        assert!(
            world.combat.as_ref().unwrap().creatures[0]
                .respawn_at
                .is_none()
        );
        assert!(world.combat_status_snapshot(id).unwrap().is_empty());
        assert!(!world.combat_membership(id).unwrap().in_combat());
        assert_eq!(world.creature_tag_owner(), None);
        assert!(world.player_life_snapshot(id).unwrap().windup.is_none());
        let metrics = world.creature_reset_metrics().unwrap();
        assert_eq!(metrics.resets, 1);
        assert_eq!(metrics.health_restored, u64::from(1000 - health));
        assert_eq!(metrics.statuses_removed, 1);
        for _ in 0..rate.hz() * 2 {
            world.step();
        }
        assert_eq!(world.creature_reset_metrics().unwrap(), metrics);
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 1000);
        assert!(world.creature_reset_error().is_none());
    }
}
#[test]
fn older_typed_mode_refuses_return_damage_but_keeps_damaged_health() {
    let (mut world, npc) = fixture(TickRate::Hz30);
    let id = hurt(&mut world, npc);
    let health = world.combat.as_ref().unwrap().creatures[0].current;
    force_return(&mut world);
    assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, health);
    let h = hit(&world, id, npc);
    let mut authority = world.combat.take().unwrap();
    assert!(authority.apply_ability_damage(&mut world, &h).is_err());
    let weapon = authority.melee_weapon(id).unwrap().clone();
    assert!(
        authority
            .apply_white_melee(&mut world, id, npc, &weapon, 2)
            .is_err()
    );
    assert_eq!(authority.creatures[0].current, health);
    world.combat = Some(authority);
    assert!(world.creature_reset_metrics().is_none());
}
#[test]
fn stale_projectile_context_stays_invalid_after_return_ends_but_new_cast_can_tag() {
    let (mut world, npc) = reset_world(TickRate::Hz30, true);
    let id = hurt(&mut world, npc);
    let old = hit(&world, id, npc);
    force_return(&mut world);
    for _ in 0..60 {
        world.step();
    }
    assert_eq!(
        world.creature_pursuit_directive().unwrap().mode,
        theia_core::pursuit::Mode::Patrol
    );
    let draws = world.ability_damage_metrics().unwrap();
    let mut authority = world.combat.take().unwrap();
    assert!(authority.apply_ability_damage(&mut world, &old).is_err());
    assert_eq!(authority.creatures[0].current, 1000);
    world.combat = Some(authority);
    assert_eq!(
        serde_json::to_value(draws).unwrap(),
        serde_json::to_value(world.ability_damage_metrics().unwrap()).unwrap()
    );
    cast(&mut world, id, npc, 2, "theia.dev.sword-slash");
    world.step();
    assert_eq!(world.creature_tag_owner(), world.entity_key(id));
    assert!(world.combat.as_ref().unwrap().creatures[0].current < 1000);
}
#[test]
fn death_and_fresh_respawn_are_not_return_heals_or_extra_credit() {
    let (mut world, npc) = reset_world(TickRate::Hz30, true);
    let id = join(&mut world);
    world.combat.as_mut().unwrap().creatures[0].current = 1;
    cast(&mut world, id, npc, 1, "theia.dev.sword-slash");
    world.step();
    assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 0);
    assert_eq!(world.creature_reset_metrics().unwrap().resets, 0);
    let credit = world.creature_kill_credit().unwrap().clone();
    let due = world.combat.as_ref().unwrap().creatures[0]
        .respawn_at
        .unwrap();
    while world.tick + 1 < due {
        world.step();
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 0);
        assert_eq!(world.creature_kill_credit(), Some(&credit));
    }
    world.step();
    assert_ne!(world.combat.as_ref().unwrap().creatures[0].key, npc);
    assert_eq!(world.creature_reset_metrics().unwrap().resets, 0);
    assert!(world.creature_kill_credit().is_none());
}
#[test]
fn invalid_health_refuses_reset_without_partial_heal_and_blocks_later_damage() {
    let (mut world, npc) = reset_world(TickRate::Hz30, true);
    let id = hurt(&mut world, npc);
    world.combat.as_mut().unwrap().creatures[0].current = 1001;
    force_return(&mut world);
    assert!(world.creature_reset_error().is_some());
    assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 1001);
    assert_eq!(world.creature_reset_metrics().unwrap().resets, 0);
    for _ in 0..60 {
        world.step();
    }
    let h = hit(&world, id, npc);
    let mut authority = world.combat.take().unwrap();
    assert!(authority.apply_ability_damage(&mut world, &h).is_err());
    assert_eq!(authority.creatures[0].current, 1001);
    let key = authority.actors[&id].life.as_ref().unwrap().state.key();
    let health = authority.actors[&id].life.as_ref().unwrap().state.health();
    assert!(
        authority
            .apply_white_retaliation(&mut world, npc, key)
            .is_err()
    );
    assert_eq!(
        authority.actors[&id].life.as_ref().unwrap().state.health(),
        health
    );
    world.combat = Some(authority);
}
