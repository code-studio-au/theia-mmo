//! Real World/bridge admission; owner observations here are synthetic, not SQL.
use super::*;
use crate::{
    inventory_wire::{Outcome, Reason},
    reward_wire::{Available, PickupRequest},
    simulation::World,
};
use theia_core::{
    coordinates::GlobalPosition,
    progression::KillId,
    rewards::{ClaimCommand, ClaimId, CorpseSource},
    time::TickRate,
};
const ITEMS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-equipment-items.json");
fn profile() -> Profile {
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    raw["schema"] = 2.into();
    let mut life = raw["rewards"][0].clone();
    life["id"] = "theia.dev.repeat-training-kill".into();
    life["scope"] = "per_creature_life".into();
    raw["rewards"].as_array_mut().unwrap().push(life);
    Profile::from_json(ITEMS)
        .unwrap()
        .with_creature_loot(&serde_json::to_vec(&raw).unwrap())
        .unwrap()
}
struct Fixture {
    world: World,
    id: u64,
    profile: Profile,
    peer: Peer,
    publish: watch::Sender<Observation>,
    requests: mpsc::Receiver<Request>,
    pickup: PickupRequest,
}
fn fixture() -> Fixture {
    let profile = profile();
    let (mut world, npc) =
        crate::simulation::creatures::tests::fixture_at(TickRate::Hz30, [8008., 8000.]);
    let abilities = include_bytes!("../../../../../content/gameplay/development-abilities.json");
    world.enable_prediction().unwrap();
    world
        .enable_resource_combat(
            abilities,
            100,
            1000,
            include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
        )
        .unwrap();
    let creature =
        include_bytes!("../../../../../content/gameplay/development-creature-patrol.json");
    let mut retaliation: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-creature-retaliation.json"
    ))
    .unwrap();
    retaliation["damage"] = 100.into();
    retaliation["windup_ms"] = 100.into();
    let life = crate::player_life_wire::Profile::from_json(
        include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
        creature,
        &serde_json::to_vec(&retaliation).unwrap(),
        include_bytes!("../../../../../content/gameplay/development-player-respawn.json"),
    )
    .unwrap();
    world.enable_player_life(Arc::new(life)).unwrap();
    let reward = Arc::new(profile.rewards().unwrap().clone());
    assert!(world.enable_rewards(reward.clone()).is_err());
    world.enable_creature_loot(reward.clone()).unwrap();
    let id = world.join().unwrap();
    let actor = world.entity_key(id).unwrap();
    let position = world.global_position(id).unwrap();
    let chunks = world
        .procedural()
        .unwrap()
        .patch(position)
        .unwrap()
        .ids
        .iter()
        .map(|id| [id.x, id.z])
        .collect();
    assert!(world.client_ready(
        id,
        &crate::wire::ChunkReady {
            version: crate::wire::VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks
        }
    ));
    let source = CorpseSource {
        life: KillId::from_victim(npc),
        position,
    };
    let claim = ClaimId::new([17; 16]).unwrap();
    let (publish, state) = watch::channel(Observation {
        bag: Some(Bag::from_slots(profile.catalog(), 0, std::array::from_fn(|_| None)).unwrap()),
        rewards: Some(OwnedRewards {
            generation: 1,
            available: vec![Available {
                reward: theia_core::content::ContentId::new("theia.dev.repeat-training-kill")
                    .unwrap(),
                claim,
                corpse: Some(source),
            }],
        }),
        ..Default::default()
    });
    let (requests, receiver) = mpsc::channel(1);
    let peer = Peer::new(Port {
        documents: None,
        vendor: None,
        rewards: Some(reward),
        opportunities: Some(mpsc::channel(16).0),
        catalog: profile.catalog.clone(),
        hash: profile.hash(),
        requests,
        state,
    })
    .unwrap();
    let pickup = PickupRequest {
        request: crate::reward_wire::Request {
            actor,
            sequence: 1,
            command: ClaimCommand {
                claim,
                expected_revision: 0,
            },
        },
        life: source.life,
    };
    Fixture {
        world,
        id,
        profile,
        peer,
        publish,
        requests: receiver,
        pickup,
    }
}
#[tokio::test]
async fn exact_pickup_waits_for_commit_and_shares_the_backpack_sequence() {
    let mut f = fixture();
    let (ordinary, commands) = mpsc::channel(1);
    let (pickups, pickup_rx) = mpsc::channel(1);
    let mut client = ClientState::new(Client {
        profile: f.profile.clone(),
        commands,
        pickups: Some(pickup_rx),
    })
    .unwrap();
    let actor = f.pickup.request.actor;
    let View::State(initial) = f.peer.view(actor).unwrap() else {
        panic!()
    };
    client
        .receive(
            &initial
                .encode(f.profile.catalog(), f.profile.hash())
                .unwrap(),
            actor,
        )
        .unwrap();
    f.peer.sent(*initial);
    client
        .receive_rewards(
            &f.peer
                .rewards_view(actor)
                .unwrap()
                .unwrap()
                .encode(
                    f.profile.rewards().unwrap().catalog(),
                    f.profile.rewards().unwrap().hash(),
                )
                .unwrap(),
            actor,
        )
        .unwrap();
    pickups
        .try_send(Pickup {
            command: f.pickup.request.command,
            life: f.pickup.life,
        })
        .unwrap();
    ordinary
        .try_send(
            theia_core::inventory::Command {
                expected_revision: 1,
                operation: theia_core::inventory::Operation::Move { from: 0, to: 1 },
            }
            .into(),
        )
        .unwrap();
    let bytes = client
        .next(actor, tokio::time::Instant::now())
        .unwrap()
        .unwrap();
    assert_eq!(PickupRequest::decode(&bytes).unwrap(), f.pickup);
    assert!(f.peer.request_in_world(&f.world, f.id, &bytes).unwrap());
    let accepted = f.requests.try_recv().unwrap();
    assert_eq!(accepted.command, f.pickup.request.command.into());
    assert!(!f.peer.request_in_world(&f.world, f.id, &bytes).unwrap());
    assert!(
        client
            .next(actor, tokio::time::Instant::now())
            .unwrap()
            .is_none()
    );
    assert_eq!(client.view.as_ref().unwrap().bag.revision(), 0);
    // Full bag/owner rejection leaves availability unchanged and releases only
    // the matching inventory acknowledgement, not a reward-generation message.
    f.publish.send_modify(|s| {
        s.reply = Some(crate::inventory_wire::Reply {
            sequence: accepted.sequence,
            outcome: Outcome::Rejected(Reason::Full),
        })
    });
    let View::State(refused) = f.peer.view(actor).unwrap() else {
        panic!()
    };
    client
        .receive(
            &refused
                .encode(f.profile.catalog(), f.profile.hash())
                .unwrap(),
            actor,
        )
        .unwrap();
    f.peer.sent(*refused);
    assert_eq!(client.rewards.as_ref().unwrap().available.len(), 1);
    let next = client
        .next(actor, tokio::time::Instant::now())
        .unwrap()
        .unwrap();
    assert_eq!(
        crate::inventory_wire::ItemRequest::decode(&next)
            .unwrap()
            .sequence,
        2
    );
}
#[tokio::test]
async fn forged_stale_unready_distant_and_dead_pickups_never_reach_owner() {
    for case in 0..9 {
        let mut f = fixture();
        let actor = f.pickup.request.actor;
        let mut bytes = f.pickup.encode().unwrap();
        match case {
            0 => bytes = f.pickup.request.encode().unwrap(), // legacy bypass
            1 => {
                let mut p = f.pickup;
                p.life = KillId::from_victim(actor);
                bytes = p.encode().unwrap();
            }
            2 => {
                let mut p = f.pickup;
                p.request.command.claim = ClaimId::new([18; 16]).unwrap();
                bytes = p.encode().unwrap();
            }
            3 => f.publish.send_modify(|s| s.closed = true),
            4 => f.publish.send_modify(|s| s.failed = true),
            5 => {
                assert!(f.world.client_ready(
                    f.id,
                    &crate::wire::ChunkReady {
                        version: crate::wire::VERSION,
                        epoch: f.world.epoch(),
                        sequence: 2,
                        chunks: vec![]
                    }
                ));
            }
            6 => f.publish.send_modify(|s| {
                let source = s.rewards.as_mut().unwrap().available[0]
                    .corpse
                    .as_mut()
                    .unwrap();
                let mut p = source.position.metres();
                p[1] += 3.001;
                source.position = GlobalPosition::new(p).unwrap();
            }),
            7 => f
                .publish
                .send_modify(|s| s.rewards.as_mut().unwrap().available.clear()),
            8 => {
                f.world.cast(
                    f.id,
                    crate::simulation::combat::Request {
                        epoch: f.world.epoch(),
                        sequence: 1,
                        target: f.pickup.life.victim(),
                        ability: theia_core::content::ContentId::new("theia.dev.arcane-bolt")
                            .unwrap(),
                    },
                );
                for _ in 0..100 {
                    f.world.step();
                    if !f.world.player_alive(f.id) {
                        break;
                    }
                }
                assert!(!f.world.player_alive(f.id), "Fixture did not kill player");
            }
            _ => unreachable!(),
        }
        assert!(
            f.peer.request_in_world(&f.world, f.id, &bytes).unwrap(),
            "case {case}"
        );
        assert!(f.requests.try_recv().is_err(), "case {case}");
        if case == 3 {
            assert!(matches!(f.peer.view(actor).unwrap(), View::Closed));
            continue;
        }
        if case == 4 {
            assert!(f.peer.view(actor).is_err());
            continue;
        }
        let View::State(refused) = f.peer.view(actor).unwrap() else {
            panic!("case {case}")
        };
        assert_eq!(
            refused.reply.unwrap().outcome,
            Outcome::Rejected(Reason::Conflict)
        );
        assert_eq!(refused.bag.revision(), 0);
    }
}
