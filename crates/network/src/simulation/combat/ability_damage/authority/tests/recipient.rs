//! Recipient adapter facts, not durable or authenticated account evidence.
use super::*;
const SOLO: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-credit-solo.json"
);
const OWNER_ONLY: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-creature-credit.json");
fn rewards(rate: TickRate, bytes: &[u8]) -> (World, EntityKey) {
    let (mut w, npc) = super::credit::credit_world_profile(rate, false, 32, bytes);
    let items = crate::renet_runtime::inventory::Profile::from_json(ITEMS).unwrap();
    w.enable_rewards(Arc::new(
        crate::reward_wire::Profile::from_json(
            include_bytes!("../../../../../../../../content/gameplay/development-rewards.json"),
            &items,
        )
        .unwrap(),
    ))
    .unwrap();
    w.enable_progression(Arc::new(
        crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    (w, npc)
}
fn competing_kill(w: &mut World, npc: EntityKey) -> (u64, u64, Vec<Event>) {
    let a = join(w);
    let b = join(w);
    cast(w, a, npc, 1, "theia.dev.sword-slash");
    w.step();
    assert_eq!(w.creature_tag_owner(), w.entity_key(a));
    assert!(
        w.experience_opportunities(w.combat_events())
            .unwrap()
            .is_empty()
    );
    assert!(
        w.reward_opportunities(w.combat_events())
            .unwrap()
            .is_empty()
    );
    w.combat.as_mut().unwrap().creatures[0].current = 1;
    cast(w, b, npc, 1, "theia.dev.sword-slash");
    w.step();
    (a, b, w.combat_events().to_vec())
}
#[test]
fn explicit_solo_policy_routes_xp_and_training_rewards_to_owner_once_at_each_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = rewards(rate, SOLO);
        let (a, b, mut events) = competing_kill(&mut w, npc);
        events.extend(events.clone());
        let xp = w.experience_opportunities(&events).unwrap();
        let loot = w.reward_opportunities(&events).unwrap();
        assert_eq!(xp.len(), 1);
        assert_eq!(loot.len(), 1);
        assert_eq!(xp[0].actor, w.entity_key(a).unwrap());
        assert_eq!(loot[0].actor, xp[0].actor);
        assert_ne!(xp[0].actor, w.entity_key(b).unwrap());
        assert_eq!(
            xp[0].event.id,
            theia_core::progression::KillId::from_victim(npc)
        );
    }
}
#[test]
fn ownership_only_profile_keeps_qualified_last_hit_adapters() {
    let (mut w, npc) = rewards(TickRate::Hz30, OWNER_ONLY);
    let (a, b, events) = competing_kill(&mut w, npc);
    assert_eq!(w.creature_tag_owner(), w.entity_key(a));
    assert_eq!(
        w.experience_opportunities(&events).unwrap()[0].actor,
        w.entity_key(b).unwrap()
    );
    assert_eq!(
        w.reward_opportunities(&events).unwrap()[0].actor,
        w.entity_key(b).unwrap()
    );
}
#[test]
fn only_exact_retained_fatal_fact_authorizes_solo_recipient() {
    let (mut w, npc) = rewards(TickRate::Hz30, SOLO);
    let (a, _, events) = competing_kill(&mut w, npc);
    let original = events
        .iter()
        .find_map(|e| {
            if let Event::AbilityDamage(f) = e {
                Some(f.clone())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(original.remaining, 0);
    for field in 0..5 {
        let mut forged = original.clone();
        match field {
            0 => forged.actor = w.entity_key(a).unwrap(),
            1 => forged.sequence += 1,
            2 => forged.actor_generation = std::num::NonZeroU64::new(2).unwrap(),
            3 => forged.remaining = 1,
            _ => forged.result.damage.health_lost = 0,
        }
        let events = [Event::AbilityDamage(forged)];
        assert!(w.experience_opportunities(&events).unwrap().is_empty());
        assert!(w.reward_opportunities(&events).unwrap().is_empty());
    }
    assert_eq!(w.experience_opportunities(&events).unwrap().len(), 1);
}
#[test]
fn connected_dead_owner_keeps_solo_eligibility_departed_owner_never_falls_back() {
    let (mut w, npc) = rewards(TickRate::Hz30, SOLO);
    let (a, b, events) = competing_kill(&mut w, npc);
    let life = w.player_life_snapshot(a).unwrap().key;
    let now = w.tick;
    w.combat
        .as_mut()
        .unwrap()
        .actors
        .get_mut(&a)
        .unwrap()
        .life
        .as_mut()
        .unwrap()
        .state
        .damage(now, life, 100)
        .unwrap();
    assert_eq!(
        w.experience_opportunities(&events).unwrap()[0].actor,
        life.actor
    );
    assert_eq!(
        w.reward_opportunities(&events).unwrap()[0].actor,
        life.actor
    );
    w.leave(a);
    assert!(w.entity_key(b).is_some());
    assert!(w.experience_opportunities(&events).unwrap().is_empty());
    assert!(w.reward_opportunities(&events).unwrap().is_empty());
}
#[test]
fn old_fatal_facts_cannot_award_after_npc_replacement() {
    let (mut w, npc) = rewards(TickRate::Hz30, SOLO);
    let (_, _, events) = competing_kill(&mut w, npc);
    let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
    while w.tick < due {
        w.step();
    }
    assert_ne!(w.combat.as_ref().unwrap().creatures[0].key, npc);
    assert!(w.experience_opportunities(&events).unwrap().is_empty());
    assert!(w.reward_opportunities(&events).unwrap().is_empty());
}
#[test]
fn white_melee_fatal_fact_uses_same_owner_adapter_without_ability_assumption() {
    let (mut w, npc) = rewards(TickRate::Hz30, SOLO);
    let a = join(&mut w);
    let b = join(&mut w);
    cast(&mut w, a, npc, 1, "theia.dev.sword-slash");
    w.step();
    w.combat.as_mut().unwrap().creatures[0].current = 1;
    let mut authority = w.combat.take().unwrap();
    let weapon = authority.melee_weapon(b).unwrap().clone();
    for sequence in 1..=20 {
        authority
            .apply_white_melee(&mut w, b, npc, &weapon, sequence)
            .unwrap();
        if authority.creatures[0].current == 0 {
            break;
        }
    }
    w.combat = Some(authority);
    let events = w.combat_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e,Event::WhiteMelee(f) if f.remaining==0))
    );
    assert_eq!(
        w.experience_opportunities(events).unwrap()[0].actor,
        w.entity_key(a).unwrap()
    );
    assert_eq!(
        w.reward_opportunities(events).unwrap()[0].actor,
        w.entity_key(a).unwrap()
    );
}
