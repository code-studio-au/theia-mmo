use super::*;
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityKey, WorldEpoch},
    party::Action,
    time::TickRate,
};

fn send(world: &mut World, actor: u64, action: Action) {
    let view = world.party_view(actor).unwrap();
    world
        .party_request(
            actor,
            Request {
                sequence: view.sequence + 1,
                revision: view.revision,
                action,
            },
        )
        .unwrap();
}
pub(in crate::simulation) fn group(world: &mut World, owner: u64, member: EntityKey) {
    send(world, owner, Action::Invite(member));
    let id = world
        .party_view(member.entity.value())
        .unwrap()
        .membership
        .incoming
        .unwrap()
        .id;
    send(world, member.entity.value(), Action::Accept(id));
}
#[test]
fn party_selection_preserves_pose_and_interest_but_not_departure_or_incarnation_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        world.enable_parties().unwrap();
        let enemy = world.spawn_development_target([8064., 8000.]).unwrap();
        let owner = world.join().unwrap();
        let member = world.join().unwrap();
        let outsider = world.join().unwrap();
        let key = world.entity_key(member).unwrap();
        assert!(!world.select_target(owner, Some(key)));
        group(&mut world, owner, key);
        let pose = world.correction(owner).unwrap().state.pose;
        assert!(world.select_target(owner, Some(enemy)));
        assert!(world.select_target(owner, Some(key)));
        assert_eq!(world.correction(owner).unwrap().state.pose, pose);
        assert!(!world.select_target(owner, world.entity_key(owner)));
        assert!(!world.select_target(outsider, Some(key)));
        assert!(!world.select_target(owner, world.entity_key(outsider)));
        let foreign = EntityKey {
            epoch: WorldEpoch::new(key.epoch.value() + 1).unwrap(),
            ..key
        };
        assert!(!world.select_target(owner, Some(foreign)));
        let selected = world.targeting_snapshot(owner).unwrap();
        assert_eq!(selected.selected_member, Some(key));
        assert_eq!(selected.selected, None);
        assert_eq!(
            selected.targets.iter().map(|t| t.key).collect::<Vec<_>>(),
            vec![enemy]
        );
        // Controlled authoritative placement beyond ordinary spatial relevance.
        world.players.get_mut(&member).unwrap().position =
            GlobalPosition::new([10060., 0., 8000.]).unwrap();
        assert!(!crate::targeting::relevant(
            world.global_position(owner).unwrap(),
            world.global_position(member).unwrap()
        ));
        assert!(world.select_target(owner, None));
        assert!(
            world.select_target(owner, Some(key)),
            "Selection request must accept the distant current member"
        );
        world.step();
        assert_eq!(
            world.targeting_snapshot(owner).unwrap().selected_member,
            Some(key)
        );
        assert!(world.select_target(owner, None));
        assert!(
            world.select_target(owner, Some(key)),
            "Party frame can select outside spatial interest"
        );
        assert!(world.remove_development_target(enemy));
        world.step();
        assert_eq!(
            world.targeting_snapshot(owner).unwrap().selected_member,
            Some(key)
        );
        send(&mut world, owner, Action::Promote(key));
        assert_eq!(
            world.targeting_snapshot(owner).unwrap().selected_member,
            Some(key)
        );
        send(&mut world, member, Action::Leave);
        let selected = world.targeting_snapshot(owner).unwrap();
        assert!(selected.selected.is_none() && selected.selected_member.is_none());
        assert!(!world.select_target(owner, Some(key)));
        // Expire the short membership-command rate window before regrouping.
        for _ in 0..rate.hz() * 2 {
            world.step();
        }
        group(&mut world, owner, key);
        assert!(world.select_target(owner, Some(key)));
        send(&mut world, owner, Action::Kick(key));
        assert!(
            world
                .targeting_snapshot(owner)
                .unwrap()
                .selected_member
                .is_none()
        );
        group(&mut world, owner, key);
        assert!(world.select_target(owner, Some(key)));
        world.leave(member);
        assert!(
            world
                .targeting_snapshot(owner)
                .unwrap()
                .selected_member
                .is_none()
        );
        let replacement = world.join().unwrap();
        assert_ne!(world.entity_key(replacement), Some(key));
        assert!(!world.select_target(owner, Some(key)));
        assert!(
            world
                .targeting_snapshot(replacement)
                .unwrap()
                .selected_member
                .is_none()
        );
    }
}
