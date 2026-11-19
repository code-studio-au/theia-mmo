//! Actual World/checked terrain. Direct prepared hits are trusted fixtures for
//! resolution atomicity, not client damage input or ability timing evidence.
use super::*;
use theia_core::{player_life::LifeKey, pursuit::Mode};
const THREAT: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-creature-threat.json");
const RESET: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-return-reset.json"
);
const SOLO: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-credit-solo.json"
);
pub(super) fn fixture_threat(rate: TickRate, capacity: usize) -> (World, EntityKey) {
    fixture_outcomes(rate, capacity, false)
}
fn fixture_outcomes(rate: TickRate, capacity: usize, miss: bool) -> (World, EntityKey) {
    let (mut world, npc) = super::credit::credit_world_profile(rate, miss, 32, SOLO);
    world.enable_creature_reset(RESET).unwrap();
    let mut policy: Value = serde_json::from_slice(THREAT).unwrap();
    policy["maximum_participants"] = json!(capacity);
    world
        .enable_creature_threat(&serde_json::to_vec(&policy).unwrap())
        .unwrap();
    (world, npc)
}
pub(super) fn life(world: &World, id: u64) -> LifeKey {
    world.combat.as_ref().unwrap().actors[&id]
        .life
        .as_ref()
        .unwrap()
        .state
        .key()
}
fn resolve(world: &mut World, hit: Hit) -> crate::Result<()> {
    let mut authority = world.combat.take().unwrap();
    let result = authority.apply_ability_damage(world, &hit);
    world.combat = Some(authority);
    result
}
pub(super) fn healthy(world: &World) {
    assert!(
        world.creature_threat_error().is_none(),
        "{:?}",
        world.creature_threat_error()
    );
    assert!(
        world.player_life_error().is_none(),
        "{:?}",
        world.player_life_error()
    );
    assert!(world.creature_reset_error().is_none());
}
pub(super) fn competing(world: &mut World, npc: EntityKey) -> (u64, u64) {
    let a = join(world);
    let b = join(world);
    cast(world, a, npc, 1, "theia.dev.sword-slash");
    for _ in 0..2 {
        world.step();
        healthy(world);
    }
    assert_eq!(world.creature_tag_owner(), world.entity_key(a));
    assert_eq!(
        world.creature_pursuit_directive().unwrap().target,
        Some(life(world, a)),
        "directive={:?} policy={:?} threat={:?}",
        world.creature_pursuit_directive(),
        world.combat.as_ref().unwrap().threat_policy_target(),
        world.creature_threat_entries()
    );
    for sequence in 1..=8 {
        let mut h = hit(world, b, npc);
        h.sequence = sequence;
        resolve(world, h).unwrap();
        world.step();
        healthy(world);
        if world.creature_pursuit_directive().unwrap().target == Some(life(world, b)) {
            return (a, b);
        }
    }
    panic!("Competing effective damage never changed the victim")
}
#[test]
fn threat_activation_is_exact_exclusive_and_pre_admission() {
    let (mut old, _) = super::credit::credit_world_profile(TickRate::Hz30, false, 32, SOLO);
    let before = old.identity();
    assert!(old.enable_creature_threat(THREAT).is_err());
    assert_eq!(old.identity(), before);
    old.enable_creature_reset(RESET).unwrap();
    let before = old.identity();
    assert!(old.enable_creature_threat(b"{}").is_err());
    assert_eq!(old.identity(), before);
    old.enable_creature_threat(THREAT).unwrap();
    assert_ne!(old.identity(), before);
    let installed = old.identity();
    assert!(old.enable_creature_threat(THREAT).is_err());
    assert_eq!(old.identity(), installed);
    let (mut other, _) = super::credit::credit_world_profile(TickRate::Hz30, false, 32, SOLO);
    other.enable_creature_reset(RESET).unwrap();
    let mut padded = THREAT.to_vec();
    padded.push(b' ');
    other.enable_creature_threat(&padded).unwrap();
    assert_ne!(other.identity(), installed);
    let (mut admitted, _) = super::credit::credit_world_profile(TickRate::Hz30, false, 32, SOLO);
    admitted.enable_creature_reset(RESET).unwrap();
    join(&mut admitted);
    let before = admitted.identity();
    assert!(admitted.enable_creature_threat(THREAT).is_err());
    assert_eq!(admitted.identity(), before);
}
#[test]
fn competing_damage_changes_victim_without_stealing_credit_and_retains_both_memberships_all_rates()
{
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        let (a, b) = competing(&mut world, npc);
        healthy(&world);
        assert_eq!(world.creature_tag_owner(), world.entity_key(a));
        assert_eq!(
            world.combat.as_ref().unwrap().threat_policy_target(),
            Some(life(&world, b))
        );
        assert!(world.combat_membership(a).unwrap().in_combat());
        assert!(world.combat_membership(b).unwrap().in_combat());
        let metrics = world.creature_threat_metrics().unwrap();
        assert!(metrics.transfers >= 1);
        let lost = 1000 - world.combat.as_ref().unwrap().creatures[0].current;
        assert_eq!(metrics.health_lost, u64::from(lost));
        assert_eq!(
            world
                .creature_threat_entries()
                .unwrap()
                .iter()
                .map(|e| e.damage_threat)
                .sum::<u64>(),
            u64::from(lost)
        );
    }
}
#[test]
fn saturated_threat_and_replayed_casts_leave_health_entropy_credit_and_table_unchanged() {
    let (mut world, npc) = fixture_threat(TickRate::Hz30, 1);
    let a = join(&mut world);
    let b = join(&mut world);
    cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
    world.step();
    healthy(&world);
    let health = world.combat.as_ref().unwrap().creatures[0].current;
    let table = world.creature_threat_entries().unwrap().to_vec();
    let owner = world.creature_tag_owner();
    let draws = world.ability_damage_metrics().unwrap();
    let h = hit(&world, b, npc);
    assert!(resolve(&mut world, h).is_err());
    let replay = cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
    assert!(
        replay.events.is_empty(),
        "A replay may return its cached acknowledgement but cannot resolve damage again"
    );
    assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, health);
    assert_eq!(world.creature_threat_entries().unwrap(), table);
    assert_eq!(world.creature_tag_owner(), owner);
    let after = world.ability_damage_metrics().unwrap();
    assert_eq!(
        (draws.base_draws, draws.landing_draws, draws.critical_draws),
        (after.base_draws, after.landing_draws, after.critical_draws)
    );
    healthy(&world);
}
#[test]
fn disconnect_transfers_to_the_remaining_life_without_returning_or_reassigning_credit() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        let (a, b) = competing(&mut world, npc);
        let retired = life(&world, b);
        let anchor = world.creature_pursuit_directive().unwrap().anchor;
        world.leave(b);
        world.step();
        healthy(&world);
        let directive = world.creature_pursuit_directive().unwrap();
        assert_eq!(directive.target, Some(life(&world, a)));
        assert_eq!(directive.anchor, anchor);
        assert!(matches!(directive.mode, Mode::Chase | Mode::Attack));
        assert!(
            world
                .creature_threat_entries()
                .unwrap()
                .iter()
                .all(|e| e.life != retired)
        );
        assert_eq!(world.creature_tag_owner(), world.entity_key(a));
    }
}
#[test]
fn return_clears_the_table_in_the_transition_frame_and_stale_encounters_cannot_restore_it() {
    let (mut world, npc) = fixture_threat(TickRate::Hz30, 32);
    let a = join(&mut world);
    cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
    world.step();
    world.step();
    healthy(&world);
    let mut stale = hit(&world, a, npc);
    stale.sequence = 2;
    assert!(!world.creature_threat_entries().unwrap().is_empty());
    world.patrol.as_mut().unwrap().motion_blocked = true;
    world.step();
    healthy(&world);
    assert!(matches!(
        world.creature_pursuit_directive().unwrap().mode,
        Mode::Return | Mode::Blocked
    ));
    assert!(
        world.creature_threat_entries().unwrap().is_empty(),
        "Returning NPC retained stale threat until a later frame"
    );
    assert!(resolve(&mut world, stale).is_err());
    assert!(world.creature_threat_entries().unwrap().is_empty());
    assert!(!world.combat_membership(a).unwrap().in_combat());
}

// Deliberately seed exact arithmetic at the World consumer boundary. Damage
// production is exercised above; these cases isolate authoritative reach mapping
// and equality without hoping random damage happens to land on a boundary.
fn seed_threat(world: &mut World, a: u64, b: u64, challenger: u32) {
    use theia_core::threat::{Eligible, Table};
    let a = life(world, a);
    let b = life(world, b);
    let runtime = world.combat.as_mut().unwrap().threat.as_mut().unwrap();
    let mut table = Table::new(runtime.profile.clone(), runtime.npc);
    table.add_damage(a, 100).unwrap();
    table
        .select(&[Eligible {
            life: a,
            in_melee: true,
        }])
        .unwrap();
    table.add_damage(b, challenger).unwrap();
    runtime.table = table;
}
fn join_at(world: &mut World, x: f64, z: f64) -> u64 {
    let point = GlobalPosition::new([x, 0., z]).unwrap();
    let patch = world.procedural.as_ref().unwrap().patch(point).unwrap();
    let mut p = patch.try_spawn(x, z).unwrap().metres();
    p[1] += f64::from(crate::world::PAWN_Y);
    let id = world
        .join_restored(GlobalPosition::new(p).unwrap())
        .unwrap();
    ready(world, id, 1, true);
    gear(world, id, 1, "theia.dev.training-sword");
    id
}
#[test]
fn world_uses_current_metric_reach_and_strict_melee_ranged_thresholds_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for (x, threshold, in_melee) in [(8008., 110, true), (8018., 130, false)] {
            let (mut world, npc) = fixture_threat(rate, 32);
            let a = join(&mut world);
            let b = join_at(&mut world, x, 8002.);
            cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
            world.step();
            world.step();
            healthy(&world);
            assert_eq!(
                world
                    .combat
                    .as_ref()
                    .unwrap()
                    .threat_candidate(&world, life(&world, b))
                    .unwrap()
                    .in_melee,
                in_melee
            );
            seed_threat(&mut world, a, b, threshold);
            world.step();
            healthy(&world);
            assert_eq!(
                world.creature_pursuit_directive().unwrap().target,
                Some(life(&world, a))
            );
            let b_life = life(&world, b);
            world
                .combat
                .as_mut()
                .unwrap()
                .threat
                .as_mut()
                .unwrap()
                .table
                .add_damage(b_life, 1)
                .unwrap();
            world.step();
            healthy(&world);
            assert_eq!(
                world.creature_pursuit_directive().unwrap().target,
                Some(b_life)
            );
            assert_eq!(world.creature_tag_owner(), world.entity_key(a));
        }
    }
}
#[test]
fn avoided_hostile_attempt_registers_zero_and_overkill_adds_only_actual_health_lost() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut missed, npc) = fixture_outcomes(rate, 32, true);
        let a = join(&mut missed);
        cast(&mut missed, a, npc, 1, "theia.dev.sword-slash");
        missed.step();
        healthy(&missed);
        assert_eq!(missed.combat.as_ref().unwrap().creatures[0].current, 1000);
        let entries = missed.creature_threat_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].damage_threat, 0);
        assert_eq!(entries[0].life, life(&missed, a));
        assert_eq!(missed.creature_tag_owner(), None);
        let (mut world, npc) = fixture_threat(rate, 32);
        let a = join(&mut world);
        world.combat.as_mut().unwrap().creatures[0].current = 1;
        cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
        world.step();
        healthy(&world);
        assert_eq!(world.creature_threat_metrics().unwrap().health_lost, 1);
        assert!(world.creature_threat_entries().unwrap().is_empty());
        assert_eq!(
            world.creature_kill_credit().unwrap().contributions[0].health_lost,
            1
        );
        let stale = hit(&world, a, npc);
        let due = world.combat.as_ref().unwrap().creatures[0]
            .respawn_at
            .unwrap();
        while world.tick < due {
            world.step();
            healthy(&world);
        }
        let fresh = world.combat.as_ref().unwrap().creatures[0].key;
        assert_ne!(fresh, npc);
        assert!(world.creature_threat_entries().unwrap().is_empty());
        assert!(resolve(&mut world, stale).is_err());
        assert_eq!(world.creature_threat_metrics().unwrap().health_lost, 1);
        let fresh_hit = hit(&world, a, fresh);
        resolve(&mut world, fresh_hit).unwrap();
        assert_eq!(
            world.creature_threat_entries().unwrap()[0].life,
            life(&world, a)
        );
        healthy(&world);
    }
}
#[test]
fn dead_victim_is_removed_and_respawn_never_inherits_its_previous_life_threat() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        let (a, b) = competing(&mut world, npc);
        let old = life(&world, b);
        let now = world.tick;
        world
            .combat
            .as_mut()
            .unwrap()
            .actors
            .get_mut(&b)
            .unwrap()
            .life
            .as_mut()
            .unwrap()
            .state
            .damage(now, old, 100)
            .unwrap();
        let due = world.combat.as_ref().unwrap().actors[&b]
            .life
            .as_ref()
            .unwrap()
            .state
            .respawn_at()
            .unwrap();
        world.step();
        healthy(&world);
        assert_eq!(
            world.creature_pursuit_directive().unwrap().target,
            Some(life(&world, a))
        );
        assert!(
            world
                .creature_threat_entries()
                .unwrap()
                .iter()
                .all(|e| e.life != old)
        );
        while world.tick < due {
            world.step();
            healthy(&world);
        }
        let fresh = life(&world, b);
        assert_ne!(fresh, old);
        assert!(
            world
                .creature_threat_entries()
                .unwrap()
                .iter()
                .all(|e| e.life.actor != fresh.actor)
        );
        assert_eq!(world.creature_tag_owner(), world.entity_key(a));
    }
}
#[test]
fn moving_into_melee_can_transfer_existing_threat_without_new_damage_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        let a = join(&mut world);
        let b = join_at(&mut world, 8018., 8002.);
        cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
        world.step();
        world.step();
        healthy(&world);
        seed_threat(&mut world, a, b, 120);
        world.step();
        healthy(&world);
        assert_eq!(
            world.creature_pursuit_directive().unwrap().target,
            Some(life(&world, a))
        );
        let health_lost = world.creature_threat_metrics().unwrap().health_lost;
        let mut transferred = false;
        for _ in 0..rate.hz() * 3 {
            let player = &world.players[&b];
            let state = player.commands.as_ref().unwrap().state();
            let request = Commands {
                epoch: world.epoch(),
                stream_epoch: state.stream_epoch,
                ready_sequence: player.ready_sequence,
                commands: vec![Command {
                    sequence: state.applied_sequence + 1,
                    tick: world.tick + 1,
                    intent: Intent {
                        axes: [-1000, 0],
                        sprint: true, // Explicit v3 run; reach remains the behavior under test.
                        facing: Some(Facing::from_units(0)),
                        ..Default::default()
                    },
                }],
            };
            assert_eq!(world.command(b, &request), 1);
            world.step();
            healthy(&world);
            if world.creature_pursuit_directive().unwrap().target == Some(life(&world, b)) {
                transferred = true;
                break;
            }
        }
        assert!(transferred, "position={:?}", world.global_position(b));
        assert!(
            world
                .combat
                .as_ref()
                .unwrap()
                .threat_candidate(&world, life(&world, b))
                .unwrap()
                .in_melee
        );
        assert_eq!(
            world.creature_threat_metrics().unwrap().health_lost,
            health_lost
        );
        assert_eq!(world.creature_tag_owner(), world.entity_key(a));
    }
}
#[test]
fn tag_owner_departure_retires_old_credit_casts_but_does_not_erase_remaining_victim_threat() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        let (a, b) = competing(&mut world, npc);
        let old = hit(&world, b, npc);
        let b_life = life(&world, b);
        let remaining_threat = world
            .creature_threat_entries()
            .unwrap()
            .iter()
            .find(|e| e.life == b_life)
            .unwrap()
            .damage_threat;
        world.leave(a);
        world.step();
        healthy(&world);
        assert_eq!(world.creature_tag_owner(), None);
        assert_eq!(
            world.creature_pursuit_directive().unwrap().target,
            Some(b_life)
        );
        assert_eq!(world.creature_threat_entries().unwrap().len(), 1);
        assert_eq!(
            world.creature_threat_entries().unwrap()[0].damage_threat,
            remaining_threat
        );
        let health = world.combat.as_ref().unwrap().creatures[0].current;
        assert!(resolve(&mut world, old).is_err());
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, health);
        let fresh = hit(&world, b, npc);
        resolve(&mut world, fresh).unwrap();
        assert_eq!(world.creature_tag_owner(), world.entity_key(b));
        assert!(world.creature_threat_entries().unwrap()[0].damage_threat > remaining_threat);
        healthy(&world);
    }
}

#[test]
fn passive_body_pull_registers_zero_and_attacks_without_awarding_a_tag_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let fixture = inactive_stats_perception(
            rate,
            Some(DATA),
            include_bytes!(
                "../../../../../../../../content/gameplay/development-physical-damage-stats.json"
            ),
            12.,
        );
        let (mut world, _) = super::credit::credit_authority(fixture, false, 32, SOLO);
        world.enable_creature_reset(RESET).unwrap();
        world.enable_creature_threat(THREAT).unwrap();
        let a = join(&mut world);
        assert!(world.creature_threat_entries().unwrap().is_empty());
        let mut windup = false;
        for _ in 0..rate.hz() * 2 {
            world.step();
            healthy(&world);
            windup |= world.player_life_snapshot(a).unwrap().windup.is_some();
            if world.white_melee_metrics().unwrap().incoming.attempts > 0 {
                break;
            }
        }
        assert!(windup);
        assert!(world.white_melee_metrics().unwrap().incoming.attempts > 0);
        assert_eq!(
            world.creature_pursuit_directive().unwrap().target,
            Some(life(&world, a))
        );
        let entries = world.creature_threat_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].life, life(&world, a));
        assert_eq!(entries[0].damage_threat, 0);
        let metrics = world.creature_threat_metrics().unwrap();
        assert_eq!(metrics.damage_facts, 0);
        assert_eq!(metrics.health_lost, 0);
        assert_eq!(world.creature_tag_owner(), None);
        assert!(world.combat_membership(a).unwrap().in_combat());
    }
}
