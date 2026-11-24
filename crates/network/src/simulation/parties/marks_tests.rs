use super::*;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::EntityKey,
    party::{
        Action,
        marks::{Mark, Symbol},
    },
    time::TickRate,
};
fn send(w: &mut World, id: u64, action: Action) -> Result<(), Refusal> {
    for _ in 0..w.tick_rate().hz() * 3 {
        w.step();
    }
    let v = w.party_view(id).unwrap();
    w.party_request(
        id,
        Request {
            sequence: v.sequence + 1,
            revision: v.revision,
            action,
        },
    )
}
fn marks(w: &World, id: u64) -> Vec<Mark> {
    w.party_view(id)
        .unwrap()
        .membership
        .party
        .map_or_else(Vec::new, |p| p.marks)
}
fn set(target: EntityKey, symbol: Symbol) -> Action {
    Action::Mark {
        target,
        symbol: Some(symbol),
    }
}
#[test]
fn world_marks_preserve_selection_pose_and_privacy_but_retire_on_death_and_fresh_respawn() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut w = World::with_procedural([8060., 8000.], 1.).unwrap();
        assert!(w.set_tick_rate(rate));
        w.enable_prediction().unwrap();
        w.enable_classic().unwrap();
        w.enable_parties().unwrap();
        let guide = w
            .spawn_development_guide(
                std::sync::Arc::new(
                    crate::dialogue::Profile::from_json(include_bytes!(
                        "../../../../../content/gameplay/development-dialogue.json"
                    ))
                    .unwrap(),
                ),
                [8062., 8000.],
            )
            .unwrap();
        let target = w.spawn_development_target([8060., 7997.]).unwrap();
        let mut abilities: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        abilities["abilities"][0]["cast_ms"] = 0.into();
        abilities["abilities"][0]["effect"]["amount"] = 100.into();
        w.enable_combat(&serde_json::to_vec(&abilities).unwrap(), 100, 1000)
            .unwrap();
        let owner = w.join().unwrap();
        let member = w.join().unwrap();
        let outsider = w.join().unwrap();
        let chunks = w
            .procedural
            .as_ref()
            .unwrap()
            .patch(w.global_position(owner).unwrap())
            .unwrap()
            .ids
            .iter()
            .map(|c| [c.x, c.z])
            .collect();
        assert!(w.client_ready(
            owner,
            &crate::wire::ChunkReady {
                version: crate::wire::VERSION,
                epoch: w.epoch(),
                sequence: 1,
                chunks
            }
        ));
        let member_key = w.entity_key(member).unwrap();
        super::targeting_tests::group(&mut w, owner, member_key);
        assert!(w.select_target(owner, Some(member_key)));
        for _ in 0..rate.hz() {
            w.step();
        }
        let pose = w.correction(owner).unwrap().state.pose;
        send(&mut w, owner, set(target, Symbol::Skull)).unwrap();
        assert_eq!(w.correction(owner).unwrap().state.pose, pose);
        assert_eq!(
            w.targeting_snapshot(owner).unwrap().selected_member,
            Some(member_key)
        );
        assert_eq!(marks(&w, owner), marks(&w, member));
        assert!(marks(&w, outsider).is_empty());
        assert_eq!(
            send(&mut w, outsider, set(target, Symbol::Star)),
            Err(Refusal::NotMember)
        );
        let outsider_key = w.entity_key(outsider).unwrap();
        assert_eq!(
            send(&mut w, owner, set(outsider_key, Symbol::Star)),
            Err(Refusal::MissingTarget)
        );
        let origin = w.players[&owner].position;
        w.players.get_mut(&owner).unwrap().position =
            GlobalPosition::new([10060., 0., 8000.]).unwrap();
        w.retire_invalid_marks();
        assert_eq!(marks(&w, owner).len(), 1);
        let v = w.party_view(owner).unwrap();
        assert_eq!(
            w.party_request(
                owner,
                Request {
                    sequence: v.sequence + 1,
                    revision: v.revision,
                    action: set(target, Symbol::Moon)
                }
            ),
            Err(Refusal::MissingTarget)
        );
        w.players.get_mut(&owner).unwrap().position = origin;
        let result = w.cast(
            owner,
            crate::simulation::combat::Request {
                epoch: w.epoch(),
                sequence: 1,
                target,
                ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            },
        );
        assert!(
            matches!(
                result.reply.outcome,
                crate::simulation::combat::Outcome::Started { .. }
            ),
            "{:?}",
            result.reply
        );
        w.step();
        assert!(marks(&w, owner).is_empty());
        for _ in 0..rate.hz() {
            w.step();
        }
        let fresh = w.targeting_snapshot(owner).unwrap().targets[0].key;
        assert_ne!(fresh, target);
        assert!(marks(&w, member).is_empty());
        assert_eq!(
            send(&mut w, owner, set(target, Symbol::Skull)),
            Err(Refusal::MissingTarget)
        );
        send(&mut w, owner, set(fresh, Symbol::Moon)).unwrap();
        assert!(w.remove_development_target(fresh));
        assert!(marks(&w, member).is_empty());
        send(&mut w, owner, set(guide.key, Symbol::Square)).unwrap();
        assert!(
            w.targeting_snapshot(owner)
                .unwrap()
                .targets
                .iter()
                .all(|t| t.key != guide.key)
        );
        assert!(w.retire_development_guide(guide.key));
        assert!(marks(&w, member).is_empty());
        assert_eq!(
            send(&mut w, owner, set(guide.key, Symbol::Square)),
            Err(Refusal::MissingTarget)
        );
        send(&mut w, owner, set(member_key, Symbol::Star)).unwrap();
        w.leave(member);
        assert!(marks(&w, owner).is_empty());
    }
}
