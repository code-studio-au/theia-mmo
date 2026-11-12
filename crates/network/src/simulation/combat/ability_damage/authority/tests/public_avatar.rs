//! Actual World life projection; direct trusted life damage isolates visual
//! lifecycle from attack timing. Transport/native adoption remains separate.
use super::{
    threat::{fixture_threat, healthy},
    *,
};
use crate::avatar_life_wire::{self as wire, Profile};
#[test]
fn avatar_activation_is_exact_pre_admission_and_leaves_inactive_worlds_unchanged() {
    let mut empty = World::default();
    let before = empty.identity();
    assert!(empty.enable_public_avatar_life().is_err());
    assert_eq!(empty.identity(), before);
    assert!(empty.public_avatar_life_snapshot().unwrap().is_none());
    let (mut world, _) = fixture_threat(TickRate::Hz30, 32);
    world.enable_public_creature_attacks().unwrap();
    let source = world
        .combat
        .as_ref()
        .unwrap()
        .identity(world.patrol_identity.unwrap());
    let profile = Profile::new(world.player_life_profile().unwrap().clone());
    world.enable_public_avatar_life().unwrap();
    assert_eq!(
        world.identity(),
        crate::classic::profile_identity(profile.identity(source))
    );
    let current = world.identity();
    assert!(world.enable_public_avatar_life().is_err());
    assert_eq!(world.identity(), current);
    assert!(
        world
            .public_avatar_life_snapshot()
            .unwrap()
            .unwrap()
            .actors
            .is_empty()
    );
    let (mut world, _) = fixture_threat(TickRate::Hz30, 32);
    join(&mut world);
    let before = world.identity();
    assert!(world.enable_public_avatar_life().is_err());
    assert_eq!(world.identity(), before);
    assert!(world.public_avatar_life_profile().is_none());
}
#[test]
fn actual_death_and_respawn_project_public_identity_without_touching_other_players_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, _) = fixture_threat(rate, 32);
        world.enable_public_avatar_life().unwrap();
        let a = join(&mut world);
        let b = join(&mut world);
        let initial = world.public_avatar_life_snapshot().unwrap().unwrap();
        assert_eq!(initial.actors.len(), 2);
        assert!(initial.actors.iter().all(|a| !a.dead));
        let a_key = world.entity_key(a).unwrap();
        let b_private = world.player_life_snapshot(b).unwrap();
        let now = world.tick;
        let life = world
            .combat
            .as_mut()
            .unwrap()
            .actors
            .get_mut(&a)
            .unwrap()
            .life
            .as_mut()
            .unwrap();
        let key = life.state.key();
        life.state.damage(now, key, 1000).unwrap();
        let dead = world.public_avatar_life_snapshot().unwrap().unwrap();
        let record = dead.actors.iter().find(|p| p.key.actor == a_key).unwrap();
        assert!(record.dead);
        assert_eq!(record.key, key);
        assert_eq!(world.player_life_snapshot(b).unwrap(), b_private);
        let respawn = world.player_life_snapshot(a).unwrap().respawn_at.unwrap();
        while world.tick < respawn {
            world.step();
            healthy(&world);
        }
        let fresh = world.public_avatar_life_snapshot().unwrap().unwrap();
        let record = fresh.actors.iter().find(|p| p.key.actor == a_key).unwrap();
        assert!(!record.dead);
        assert_eq!(record.key.generation.get(), key.generation.get() + 1);
        assert_eq!(world.player_life_snapshot(a).unwrap().key, record.key);
        let bytes =
            wire::encode(world.public_avatar_life_profile().unwrap(), rate, &fresh).unwrap();
        assert_eq!(bytes.len(), wire::HEADER_BYTES + 2 * wire::ACTOR_BYTES);
        world.leave(b);
        let left = world.public_avatar_life_snapshot().unwrap().unwrap();
        assert_eq!(left.actors.len(), 1);
        assert_eq!(left.actors[0].key.actor, a_key);
    }
}
#[test]
fn unavailable_life_authority_refuses_public_roster_instead_of_stale_visual_truth() {
    let (mut world, _) = fixture_threat(TickRate::Hz30, 32);
    world.enable_public_avatar_life().unwrap();
    join(&mut world);
    world.combat.as_mut().unwrap().pve = None;
    assert!(world.public_avatar_life_snapshot().is_err());
}

#[test]
fn public_roster_covers_the_actual_bounded_connected_player_set() {
    let (mut world, _) = fixture_threat(TickRate::Hz30, 32);
    world.enable_public_avatar_life().unwrap();
    let actors: Vec<_> = (0..wire::MAX_ACTORS).map(|_| join(&mut world)).collect();
    let view = world.public_avatar_life_snapshot().unwrap().unwrap();
    assert_eq!(
        view.actors
            .iter()
            .map(|a| a.key.actor.entity.value())
            .collect::<Vec<_>>(),
        actors
    );
    assert_eq!(
        wire::encode(
            world.public_avatar_life_profile().unwrap(),
            TickRate::Hz30,
            &view
        )
        .unwrap()
        .len(),
        wire::MAX_FRAME_BYTES
    );
    world
        .combat
        .as_mut()
        .unwrap()
        .threat
        .as_mut()
        .unwrap()
        .error = Some("trusted threat fixture failure".into());
    assert!(world.public_avatar_life_snapshot().is_err());
}
