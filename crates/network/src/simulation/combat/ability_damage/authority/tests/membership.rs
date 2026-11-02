//! Actual opt-in World state/commit tests using the parent's checked fixture.
//! No transport/client membership flag, permissions or native gameplay claim.
use super::*;
const MEMBERSHIP: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-combat-membership.json");
fn member_world(rate: TickRate) -> (World, EntityKey) {
    let (mut w, npc) = fixture(rate);
    w.enable_combat_membership(MEMBERSHIP).unwrap();
    (w, npc)
}
fn fixed_special(miss: bool) -> (World, EntityKey) {
    let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
    let mut policy: Value = serde_json::from_slice(POLICY).unwrap();
    policy["abilities"][0]["miss_bp"] = json!(if miss { 10000 } else { 0 });
    policy["abilities"][0]["dodge_bp"] = json!(0);
    policy["abilities"][0]["critical_bp"] = json!(0);
    w.enable_ability_damage(STARTING, &serde_json::to_vec(&policy).unwrap(), ABILITIES)
        .unwrap();
    w.enable_combat_membership(MEMBERSHIP).unwrap();
    (w, npc)
}
#[test]
fn membership_requires_exact_pre_admission_context_and_changes_world_identity() {
    let (mut old, _) = inactive(TickRate::Hz30, Some(DATA));
    let previous = old.identity();
    assert!(old.enable_combat_membership(MEMBERSHIP).is_err());
    assert_eq!(old.identity(), previous);
    let (mut w, _) = fixture(TickRate::Hz30);
    let before = w.identity();
    assert!(w.enable_combat_membership(b"{}").is_err());
    assert_eq!(w.identity(), before);
    let prior = w
        .combat
        .as_ref()
        .unwrap()
        .identity(w.patrol_identity.unwrap());
    let expected = crate::classic::profile_identity(
        crate::simulation::combat::membership::profile_identity(prior, MEMBERSHIP).unwrap(),
    );
    w.enable_combat_membership(MEMBERSHIP).unwrap();
    assert_eq!(w.identity(), expected);
    assert_ne!(before, expected);
    assert!(w.enable_combat_membership(MEMBERSHIP).is_err());
    assert_eq!(w.identity(), expected);
    let (mut other, _) = fixture(TickRate::Hz30);
    let mut changed = MEMBERSHIP.to_vec();
    changed.push(b' ');
    other.enable_combat_membership(&changed).unwrap();
    assert_ne!(other.identity(), expected);
    let id = join(&mut other);
    let identity = other.identity();
    assert!(other.enable_combat_membership(MEMBERSHIP).is_err());
    assert_eq!(other.identity(), identity);
    assert!(!other.combat_membership(id).unwrap().in_combat());
}
#[test]
fn selection_paused_autoattack_and_rejected_abilities_do_not_enter_combat() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = member_world(rate);
        let id = join(&mut w);
        assert!(w.select_target(id, Some(npc)));
        turn(&mut w, id, 32768);
        engage(&mut w, id, Some(npc), 1);
        w.step();
        assert_eq!(w.white_melee_metrics().unwrap().outgoing.attempts, 0);
        assert!(!w.combat_membership(id).unwrap().in_combat());
        assert!(matches!(
            cast(&mut w, id, npc, 1, "theia.dev.sword-slash")
                .reply
                .outcome,
            CastOutcome::Rejected(Reason::NotFacing)
        ));
        assert!(!w.combat_membership(id).unwrap().in_combat());
        assert!(w.combat_membership_error().is_none());
    }
}
#[test]
fn avoided_special_still_enters_combat_and_actual_pursuit_and_incoming_swings_supply_reasons() {
    let (mut w, npc) = fixed_special(true);
    let id = join(&mut w);
    assert!(matches!(
        cast(&mut w, id, npc, 1, "theia.dev.sword-slash")
            .reply
            .outcome,
        CastOutcome::Started { .. }
    ));
    assert!(!w.combat_membership(id).unwrap().in_combat());
    w.step();
    let f = facts(&w).pop().unwrap();
    assert_eq!(f.result.damage.health_lost, 0);
    assert_eq!(f.remaining, 1000);
    let first = w.combat_membership(id).unwrap();
    assert!(first.outgoing_hostility && first.in_combat());
    w.step();
    assert!(w.combat_membership(id).unwrap().active_encounter);
    for _ in 0..60 {
        w.step();
        if w.white_melee_metrics().unwrap().incoming.attempts > 0 {
            break;
        }
    }
    assert!(w.combat_membership(id).unwrap().incoming_hostility);
    assert!(w.combat_membership_error().is_none());
}
#[test]
fn arcane_membership_starts_at_impact_not_cast_start_or_projectile_launch() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = member_world(rate);
        let id = join(&mut w);
        gear(&mut w, id, 2, "theia.dev.training-wand");
        cast(&mut w, id, npc, 1, "theia.dev.arcane-bolt");
        assert!(!w.combat_membership(id).unwrap().in_combat());
        let mut launched = false;
        let mut impacted = false;
        for _ in 0..rate.hz() * 3 {
            w.step();
            launched |= w
                .combat_events()
                .iter()
                .any(|e| matches!(e, Event::ProjectileLaunched { .. }));
            if !facts(&w).is_empty() {
                assert!(launched);
                assert!(w.combat_membership(id).unwrap().outgoing_hostility);
                impacted = true;
                break;
            }
            assert!(!w.combat_membership(id).unwrap().in_combat());
        }
        assert!(impacted);
    }
}
#[test]
fn confirmed_blocked_route_return_releases_recent_hostility_and_active_encounter() {
    let (mut w, npc) = fixed_special(true);
    let id = join(&mut w);
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    w.step();
    assert!(w.combat_membership(id).unwrap().active_encounter);
    w.patrol.as_mut().unwrap().motion_blocked = true;
    w.step();
    assert_eq!(
        w.creature_pursuit_directive().unwrap().mode,
        theia_core::pursuit::Mode::Return
    );
    assert!(!w.combat_membership(id).unwrap().in_combat());
    assert!(w.combat_membership_error().is_none());
}
#[test]
fn kill_and_fresh_npc_respawn_do_not_leave_or_inherit_membership() {
    let (mut w, npc) = fixed_special(false);
    let id = join(&mut w);
    w.combat.as_mut().unwrap().creatures[0].current = 1;
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    assert_eq!(facts(&w)[0].remaining, 0);
    assert!(!w.combat_membership(id).unwrap().in_combat());
    let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
    while w.tick < due {
        w.step();
        assert!(!w.combat_membership(id).unwrap().in_combat());
    }
    assert_ne!(w.combat.as_ref().unwrap().creatures[0].key, npc);
    assert!(w.combat_membership_error().is_none());
}
#[test]
fn player_death_respawn_and_disconnect_clear_the_exact_old_life() {
    let (mut w, npc) = fixed_special(true);
    let id = join(&mut w);
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    assert!(w.combat_membership(id).unwrap().in_combat());
    let key = w.player_life_snapshot(id).unwrap().key;
    let now = w.tick;
    w.combat
        .as_mut()
        .unwrap()
        .actors
        .get_mut(&id)
        .unwrap()
        .life
        .as_mut()
        .unwrap()
        .state
        .damage(now, key, 100)
        .unwrap();
    assert!(w.combat_membership(id).is_none());
    for _ in 0..100 {
        w.step();
        if w.player_alive(id) {
            break;
        }
    }
    assert!(w.player_alive(id));
    assert_ne!(w.player_life_snapshot(id).unwrap().key, key);
    assert!(!w.combat_membership(id).unwrap().in_combat());
    w.leave(id);
    assert!(w.combat_membership(id).is_none());
    let fresh = join(&mut w);
    assert_ne!(id, fresh);
    assert!(!w.combat_membership(fresh).unwrap().in_combat());
    assert!(w.combat_membership_error().is_none());
}
#[test]
fn two_players_keep_separate_hostility_and_only_the_actual_pursuit_target_is_pinned() {
    let (mut w, npc) = fixed_special(true);
    let a = join(&mut w);
    let b = join(&mut w);
    cast(&mut w, a, npc, 1, "theia.dev.sword-slash");
    cast(&mut w, b, npc, 1, "theia.dev.sword-slash");
    w.step();
    assert!(w.combat_membership(a).unwrap().outgoing_hostility);
    assert!(w.combat_membership(b).unwrap().outgoing_hostility);
    for _ in 0..180 {
        w.step();
    }
    assert_eq!(
        w.creature_pursuit_directive()
            .unwrap()
            .target
            .unwrap()
            .actor,
        w.entity_key(a).unwrap()
    );
    assert!(w.combat_membership(a).unwrap().active_encounter);
    assert!(!w.combat_membership(b).unwrap().in_combat());
    assert!(w.combat_membership_error().is_none());
}
#[test]
fn membership_deadline_refusal_precedes_ability_white_and_incoming_health_rng_commits() {
    let (mut w, npc) = member_world(TickRate::Hz30);
    let id = join(&mut w);
    let h = hit(&w, id, npc);
    let key = w.player_life_snapshot(id).unwrap().key;
    w.tick = u64::MAX - 10;
    let mut authority = w.combat.take().unwrap();
    let creature_health = authority.creatures[0].current;
    assert!(authority.apply_ability_damage(&mut w, &h).is_err());
    assert_eq!(
        authority
            .ability_damage
            .as_ref()
            .unwrap()
            .random
            .base_draws(),
        0
    );
    let weapon = authority.melee_weapon(id).unwrap().clone();
    assert!(
        authority
            .apply_white_melee(&mut w, id, npc, &weapon, 1)
            .is_err()
    );
    assert!(authority.apply_white_retaliation(&mut w, npc, key).is_err());
    assert_eq!(authority.creatures[0].current, creature_health);
    assert_eq!(
        authority.actors[&id].life.as_ref().unwrap().state.health(),
        100
    );
    assert!(authority.events.is_empty());
    w.combat = Some(authority);
    assert_eq!(w.white_melee_metrics().unwrap().base_draws, 0);
    assert!(!w.combat_membership(id).unwrap().in_combat());
}
