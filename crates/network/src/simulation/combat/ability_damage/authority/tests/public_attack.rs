//! Actual World observation only. Transport and native presentation remain next.
use super::{
    threat::{competing, fixture_threat, healthy, life},
    *,
};
use crate::creature_attack_wire::{self as wire, Applied, ClientState, Profile};
#[test]
fn public_timing_activation_is_exact_pre_admission_and_keeps_legacy_worlds_inactive() {
    let mut empty = World::default();
    let before = empty.identity();
    assert!(empty.enable_public_creature_attacks().is_err());
    assert_eq!(empty.identity(), before);
    assert!(empty.public_creature_attack_snapshot().unwrap().is_none());
    let (mut world, _) = fixture_threat(TickRate::Hz30, 32);
    assert!(world.public_creature_attack_snapshot().unwrap().is_none());
    let source = world
        .combat
        .as_ref()
        .unwrap()
        .identity(world.patrol_identity.unwrap());
    let profile = Profile::new(world.player_life_profile().unwrap().clone());
    world.enable_public_creature_attacks().unwrap();
    assert_eq!(
        world.identity(),
        crate::classic::profile_identity(profile.identity(source))
    );
    let installed = world.identity();
    assert!(world.enable_public_creature_attacks().is_err());
    assert_eq!(world.identity(), installed);
    let (mut admitted, _) = fixture_threat(TickRate::Hz30, 32);
    join(&mut admitted);
    let old = admitted.identity();
    assert!(admitted.enable_public_creature_attacks().is_err());
    assert_eq!(admitted.identity(), old);
    assert!(admitted.public_creature_attack_profile().is_none());
}
#[test]
fn public_timing_matches_the_owner_windup_without_changing_an_observers_private_state_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        world.enable_public_creature_attacks().unwrap();
        let a = join(&mut world);
        let b = join(&mut world);
        cast(&mut world, a, npc, 1, "theia.dev.sword-slash");
        let mut observed = None;
        for _ in 0..rate.hz() * 2 {
            world.step();
            healthy(&world);
            let public = world.public_creature_attack_snapshot().unwrap().unwrap();
            if public.windup.is_some() {
                observed = Some(public);
                break;
            }
        }
        let public = observed.expect("No public windup");
        let pending = public.windup.unwrap();
        let private = world.player_life_snapshot(a).unwrap().windup.unwrap();
        assert_eq!(pending.target, private.target.actor);
        assert_eq!(pending.completes, private.completes);
        assert_eq!(public.creature, npc);
        assert_eq!(public.tick, world.tick);
        let profile = world.public_creature_attack_profile().unwrap();
        assert_eq!(
            pending.completes - pending.starts,
            profile.windup_ticks(rate).unwrap()
        );
        let observer = world.player_life_snapshot(b).unwrap();
        assert!(observer.windup.is_none());
        assert_eq!(observer.health, 100);
        assert_eq!(
            wire::encode(profile, rate, &public).unwrap().len(),
            wire::MAX_FRAME_BYTES
        );
        world.patrol.as_mut().unwrap().motion_blocked = true;
        world.step();
        healthy(&world);
        assert!(
            world
                .public_creature_attack_snapshot()
                .unwrap()
                .unwrap()
                .windup
                .is_none()
        );
    }
}
#[test]
fn public_cancel_transfer_death_and_new_npc_cannot_replay_an_old_windup_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture_threat(rate, 32);
        world.enable_public_creature_attacks().unwrap();
        let (a, b) = competing(&mut world, npc);
        assert!(
            world
                .public_creature_attack_snapshot()
                .unwrap()
                .unwrap()
                .windup
                .is_none()
        );
        let mut observed = None;
        for _ in 0..rate.hz() * 3 {
            world.step();
            healthy(&world);
            let p = world.public_creature_attack_snapshot().unwrap().unwrap();
            if p.windup.is_some() {
                observed = Some(p);
                break;
            }
        }
        let old = observed.expect("Transferred victim never received a new windup");
        assert_eq!(old.windup.unwrap().target, life(&world, b).actor);
        assert!(world.player_life_snapshot(a).unwrap().windup.is_none());
        let profile = world.public_creature_attack_profile().unwrap().clone();
        let mut receiver =
            ClientState::new(profile.clone(), npc.epoch, rate, world.identity()).unwrap();
        let pending = wire::encode(&profile, rate, &old).unwrap();
        receiver.apply(&pending).unwrap();
        world.combat.as_mut().unwrap().creatures[0].current = 1;
        cast(&mut world, a, npc, 2, "theia.dev.sword-slash");
        world.step();
        healthy(&world);
        let dead = world.public_creature_attack_snapshot().unwrap().unwrap();
        assert!(dead.windup.is_none());
        receiver
            .apply(&wire::encode(&profile, rate, &dead).unwrap())
            .unwrap();
        let due = world.combat.as_ref().unwrap().creatures[0]
            .respawn_at
            .unwrap();
        while world.tick < due {
            world.step();
            healthy(&world);
        }
        let fresh = world.public_creature_attack_snapshot().unwrap().unwrap();
        assert_ne!(fresh.creature, npc);
        assert!(fresh.windup.is_none());
        receiver
            .apply(&wire::encode(&profile, rate, &fresh).unwrap())
            .unwrap();
        assert_eq!(receiver.apply(&pending).unwrap(), Applied::Stale);
        assert!(receiver.active_at(world.tick).is_none());
    }
}
#[test]
fn public_timing_refuses_an_authority_error_instead_of_presenting_a_stale_attack() {
    let (mut world, _) = fixture_threat(TickRate::Hz30, 32);
    world.enable_public_creature_attacks().unwrap();
    world
        .combat
        .as_mut()
        .unwrap()
        .threat
        .as_mut()
        .unwrap()
        .error = Some("trusted fixture failure".into());
    assert!(world.public_creature_attack_snapshot().is_err());
}
