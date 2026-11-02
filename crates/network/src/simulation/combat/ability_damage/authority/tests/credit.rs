//! Real checked-terrain World adoption; rewards and transport remain legacy.
use super::*;
const MEMBERSHIP: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-combat-membership.json");
const CREDIT: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-creature-credit.json");
fn credit_world(rate: TickRate, miss: bool, capacity: usize) -> (World, EntityKey) {
    credit_world_profile(rate, miss, capacity, CREDIT)
}
pub(super) fn credit_world_profile(
    rate: TickRate,
    miss: bool,
    capacity: usize,
    bytes: &[u8],
) -> (World, EntityKey) {
    let (mut w, npc) = inactive(rate, Some(DATA));
    let mut policy: Value = serde_json::from_slice(POLICY).unwrap();
    for ability in policy["abilities"].as_array_mut().unwrap() {
        if ability.get("miss_bp").is_some() {
            ability["miss_bp"] = json!(if miss { 10000 } else { 0 });
            ability["dodge_bp"] = json!(0);
        }
        ability["critical_bp"] = json!(0);
    }
    w.enable_ability_damage(STARTING, &serde_json::to_vec(&policy).unwrap(), ABILITIES)
        .unwrap();
    w.combat
        .as_mut()
        .unwrap()
        .ability_damage
        .as_mut()
        .unwrap()
        .random = outcomes::Random::seed_for_test([41; 32]);
    w.enable_combat_membership(MEMBERSHIP).unwrap();
    let mut profile: Value = serde_json::from_slice(bytes).unwrap();
    profile["maximum_contributors"] = json!(capacity);
    w.enable_creature_credit(&serde_json::to_vec(&profile).unwrap())
        .unwrap();
    (w, npc)
}
fn resolve(w: &mut World, h: Hit) -> crate::Result<()> {
    let mut authority = w.combat.take().unwrap();
    let result = authority.apply_ability_damage(w, &h);
    w.combat = Some(authority);
    result
}
#[test]
fn credit_opt_in_is_exact_pre_admission_and_keeps_old_worlds_unmodified() {
    let (mut old, _) = fixture(TickRate::Hz30);
    let identity = old.identity();
    assert!(old.enable_creature_credit(CREDIT).is_err());
    assert_eq!(old.identity(), identity);
    old.enable_combat_membership(MEMBERSHIP).unwrap();
    let identity = old.identity();
    assert!(old.enable_creature_credit(b"{}").is_err());
    assert_eq!(old.identity(), identity);
    let previous = old
        .combat
        .as_ref()
        .unwrap()
        .identity(old.patrol_identity.unwrap());
    let expected = crate::classic::profile_identity(
        crate::simulation::combat::credit::profile_identity(previous, CREDIT).unwrap(),
    );
    old.enable_creature_credit(CREDIT).unwrap();
    assert_eq!(old.identity(), expected);
    assert!(old.enable_creature_credit(CREDIT).is_err());
    let (mut other, _) = fixture(TickRate::Hz30);
    other.enable_combat_membership(MEMBERSHIP).unwrap();
    let mut changed = CREDIT.to_vec();
    changed.push(b' ');
    other.enable_creature_credit(&changed).unwrap();
    assert_ne!(old.identity(), other.identity());
    join(&mut other);
    assert!(other.enable_creature_credit(CREDIT).is_err());
}
#[test]
fn first_special_tags_before_competitors_fatal_hit_at_each_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = credit_world(rate, false, 32);
        let first = join(&mut w);
        let second = join(&mut w);
        let owner = w.entity_key(first).unwrap();
        cast(&mut w, first, npc, 1, "theia.dev.sword-slash");
        assert_eq!(w.creature_tag_owner(), None);
        w.step();
        assert_eq!(w.creature_tag_owner(), Some(owner));
        assert!(w.creature_kill_credit().is_none());
        w.combat.as_mut().unwrap().creatures[0].current = 1;
        cast(&mut w, second, npc, 1, "theia.dev.sword-slash");
        w.step();
        let credit = w.creature_kill_credit().unwrap();
        assert_eq!(credit.owner, Some(owner));
        assert_eq!(credit.contributions.len(), 2);
        assert_eq!(credit.contributions[1].actor, w.entity_key(second).unwrap());
        assert_eq!(credit.contributions[1].health_lost, 1);
        assert_eq!(credit.encounter.creature, npc);
        assert!(w.creature_credit_error().is_none());
    }
}
#[test]
fn valid_avoided_special_enters_combat_without_claim_or_contribution() {
    let (mut w, npc) = credit_world(TickRate::Hz30, true, 1);
    let id = join(&mut w);
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    assert_eq!(facts(&w)[0].result.damage.health_lost, 0);
    assert!(w.combat_membership(id).unwrap().in_combat());
    assert_eq!(w.creature_tag_owner(), None);
    assert!(w.creature_kill_credit().is_none());
}
#[test]
fn contributor_capacity_refuses_before_health_rng_membership_or_facts_commit() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 1);
    let a = join(&mut w);
    let b = join(&mut w);
    let h = hit(&w, a, npc);
    resolve(&mut w, h).unwrap();
    let health = w.combat.as_ref().unwrap().creatures[0].current;
    let draws = w.ability_damage_metrics().unwrap();
    let events = w.combat_events().len();
    let owner = w.creature_tag_owner();
    let h = hit(&w, b, npc);
    assert!(resolve(&mut w, h).is_err());
    assert_eq!(w.combat.as_ref().unwrap().creatures[0].current, health);
    assert_eq!(
        serde_json::to_value(w.ability_damage_metrics().unwrap()).unwrap(),
        serde_json::to_value(draws).unwrap()
    );
    assert_eq!(w.combat_events().len(), events);
    assert_eq!(w.creature_tag_owner(), owner);
    assert!(!w.combat_membership(b).unwrap().in_combat());
}
#[test]
fn owner_departure_releases_claim_and_rejects_in_flight_old_encounter() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 32);
    let a = join(&mut w);
    let b = join(&mut w);
    let h = hit(&w, a, npc);
    resolve(&mut w, h).unwrap();
    let pending = hit(&w, b, npc);
    let previous = w.creature_credit_encounter().unwrap();
    w.leave(a);
    assert_eq!(w.creature_tag_owner(), None);
    w.step();
    assert_ne!(w.creature_credit_encounter(), Some(previous));
    let health = w.combat.as_ref().unwrap().creatures[0].current;
    assert!(resolve(&mut w, pending).is_err());
    assert_eq!(w.combat.as_ref().unwrap().creatures[0].current, health);
    // Return/blocked phases cannot accept a replacement claim. Resume patrol first.
    for _ in 0..180 {
        if w.creature_credit_encounter().is_some() {
            break;
        }
        w.step();
    }
    let h = hit(&w, b, npc);
    resolve(&mut w, h).unwrap();
    assert_eq!(w.creature_tag_owner(), w.entity_key(b));
}
#[test]
fn blocked_route_reset_clears_claim_and_old_generation_cannot_retag() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 32);
    let id = join(&mut w);
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    w.step();
    assert!(w.creature_tag_owner().is_some());
    let old = hit(&w, id, npc);
    w.patrol.as_mut().unwrap().motion_blocked = true;
    w.step();
    assert_eq!(
        w.creature_pursuit_directive().unwrap().mode,
        theia_core::pursuit::Mode::Return
    );
    assert_eq!(w.creature_tag_owner(), None);
    let health = w.combat.as_ref().unwrap().creatures[0].current;
    assert!(resolve(&mut w, old).is_err());
    assert_eq!(w.combat.as_ref().unwrap().creatures[0].current, health);
    let new = hit(&w, id, npc);
    assert!(resolve(&mut w, new).is_err());
    assert_eq!(w.combat.as_ref().unwrap().creatures[0].current, health);
}
#[test]
fn resolved_death_credit_is_stable_until_fresh_npc_respawn() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 32);
    let id = join(&mut w);
    w.combat.as_mut().unwrap().creatures[0].current = 1;
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    let credit = w.creature_kill_credit().unwrap().clone();
    let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
    while w.tick + 1 < due {
        w.step();
        assert_eq!(w.creature_kill_credit(), Some(&credit));
    }
    w.step();
    assert_ne!(w.combat.as_ref().unwrap().creatures[0].key, npc);
    assert!(w.creature_kill_credit().is_none());
    assert_eq!(w.creature_tag_owner(), None);
    assert!(w.creature_credit_error().is_none());
}
#[test]
fn impact_resolution_serial_accepts_older_cast_sequence_after_newer_instant() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 32);
    let id = join(&mut w);
    let mut later = hit(&w, id, npc);
    later.sequence = 2;
    resolve(&mut w, later).unwrap();
    // Explicit first-spell landing draw; this fixture checks ordering, not miss rates.
    w.combat
        .as_mut()
        .unwrap()
        .ability_damage
        .as_mut()
        .unwrap()
        .random = outcomes::Random::seed_for_test([41; 32]);
    let mut earlier = hit(&w, id, npc);
    earlier.sequence = 1;
    earlier.ability = ContentId::new("theia.dev.arcane-bolt").unwrap();
    resolve(&mut w, earlier).unwrap();
    assert_eq!(facts(&w).len(), 2);
    assert!(facts(&w).iter().all(|f| f.result.damage.health_lost > 0));
    assert!(w.creature_credit_error().is_none());
}
#[test]
fn arcane_tags_only_after_actual_projectile_impact_at_each_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = credit_world(rate, false, 32);
        let id = join(&mut w);
        gear(&mut w, id, 2, "theia.dev.training-wand");
        cast(&mut w, id, npc, 1, "theia.dev.arcane-bolt");
        let mut launched = false;
        let mut impact = false;
        for _ in 0..rate.hz() * 3 {
            w.step();
            launched |= w
                .combat_events()
                .iter()
                .any(|e| matches!(e, Event::ProjectileLaunched { .. }));
            if !facts(&w).is_empty() {
                assert!(launched);
                impact = true;
                break;
            }
            assert_eq!(w.creature_tag_owner(), None);
        }
        assert!(impact);
        assert_eq!(w.creature_tag_owner(), w.entity_key(id));
    }
}
#[test]
fn incoming_white_does_not_tag_but_outgoing_white_contributes_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = credit_world(rate, false, 32);
        let id = join(&mut w);
        let life = w.player_life_snapshot(id).unwrap().key;
        let mut authority = w.combat.take().unwrap();
        authority
            .apply_white_retaliation(&mut w, npc, life)
            .unwrap();
        w.combat = Some(authority);
        assert_eq!(w.creature_tag_owner(), None);
        let mut authority = w.combat.take().unwrap();
        let weapon = authority.melee_weapon(id).unwrap().clone();
        for sequence in 1..=20 {
            authority
                .apply_white_melee(&mut w, id, npc, &weapon, sequence)
                .unwrap();
            if authority.events.iter().any(|e| matches!(e,Event::WhiteMelee(f) if f.actor==life.actor && f.result.health_lost>0)) { break; }
        }
        w.combat = Some(authority);
        assert_eq!(w.creature_tag_owner(), Some(life.actor));
    }
}
#[test]
fn actual_projectile_carries_old_encounter_across_owner_departure() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 32);
    let a = join(&mut w);
    let b = join(&mut w);
    let h = hit(&w, a, npc);
    resolve(&mut w, h).unwrap();
    gear(&mut w, b, 2, "theia.dev.training-wand");
    cast(&mut w, b, npc, 1, "theia.dev.arcane-bolt");
    for _ in 0..60 {
        w.step();
        if w.combat_events()
            .iter()
            .any(|e| matches!(e, Event::ProjectileLaunched { .. }))
        {
            break;
        }
    }
    assert_eq!(w.projectile_metrics().unwrap().launched, 1);
    let damage_before = w.ability_damage_metrics().unwrap().arcane.health_lost;
    w.leave(a);
    for _ in 0..60 {
        w.step();
    }
    assert_eq!(w.projectile_metrics().unwrap().impacts, 1);
    assert_eq!(
        w.ability_damage_metrics().unwrap().arcane.health_lost,
        damage_before
    );
    assert_eq!(w.creature_tag_owner(), None);
    assert!(w.ability_damage_metrics().unwrap().refusals > 0);
    assert!(w.creature_credit_error().is_none());
}
#[test]
fn player_death_alone_keeps_session_claim_and_stale_life_cannot_contribute() {
    let (mut w, npc) = credit_world(TickRate::Hz30, false, 32);
    let id = join(&mut w);
    let h = hit(&w, id, npc);
    resolve(&mut w, h).unwrap();
    let owner = w.entity_key(id).unwrap();
    let life = w.player_life_snapshot(id).unwrap().key;
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
        .damage(now, life, 100)
        .unwrap();
    assert_eq!(w.creature_tag_owner(), Some(owner));
    let h = hit(&w, id, npc);
    assert!(resolve(&mut w, h).is_err());
    // Advance only the existing life authority: a separate pursuit return/reset
    // would deliberately abandon the encounter and is tested independently.
    let due = w.combat.as_ref().unwrap().actors[&id]
        .life
        .as_ref()
        .unwrap()
        .state
        .respawn_at()
        .unwrap();
    w.advance_player_lives(due);
    assert!(w.player_alive(id));
    assert_ne!(w.player_life_snapshot(id).unwrap().key, life);
    assert_eq!(w.creature_tag_owner(), Some(owner));
    let h = hit(&w, id, npc);
    resolve(&mut w, h).unwrap();
    assert_eq!(w.creature_tag_owner(), Some(owner));
}
