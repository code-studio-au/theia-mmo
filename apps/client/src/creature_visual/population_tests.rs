use super::{attack_cue, tests::key};
use std::{num::NonZeroU64, sync::Arc};
use theia_core::{player_life::LifeKey, time::TickRate};
use theia_network::{
    client::State,
    creature_attack_wire::{
        Observation, Profile, Windup,
        population::{self as wire, Frame, PopulationProfile, PopulationState},
    },
    player_life_wire,
    simulation::combat::player_life,
    wire::{Player, Snapshot, VERSION, Welcome},
};

fn fixture(rate: TickRate) -> (State, Arc<PopulationProfile>, Frame) {
    let life = Arc::new(
        player_life_wire::Profile::from_json(
            include_bytes!("../../../../content/gameplay/development-character-stats.json"),
            include_bytes!(
                "../../../../content/gameplay/development-creature-deflect-pursuit.json"
            ),
            include_bytes!("../../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    );
    let profile = Arc::new(PopulationProfile::new(Arc::new(Profile::new(life.clone()))));
    let frame = Frame {
        epoch: key(1).epoch,
        tick: 100,
        actors: [(1, 20), (2, 30)]
            .into_iter()
            .map(|(actor, target)| Observation {
                profile: profile.source().hash(),
                creature: key(actor),
                tick: 100,
                windup: Some(Windup {
                    target: key(target),
                    starts: 100,
                    completes: 100 + profile.source().windup_ticks(rate).unwrap(),
                }),
            })
            .collect(),
    };
    // Keep a private windup deliberately live to expose accidental fallback after
    // the public stream cancels, expires or retires this exact creature.
    let own_life = LifeKey {
        actor: key(40),
        generation: NonZeroU64::new(1).unwrap(),
    };
    let private = player_life::Observation {
        profile: life.hash(),
        key: own_life,
        tick: 100,
        movement_stream: 1,
        health: life.life().stats().maximum_health(),
        maximum_health: life.life().stats().maximum_health(),
        respawn_at: None,
        combat: None,
        windup: Some(player_life::Windup {
            creature: key(1),
            target: own_life,
            completes: frame.actors[0].windup.unwrap().completes,
        }),
    };
    let mut private_state =
        player_life_wire::ClientState::new(life.clone(), key(40), rate, [7; 32]).unwrap();
    private_state
        .apply(&player_life_wire::encode(&life, rate, &private).unwrap())
        .unwrap();
    let state = State {
        id: Some(40),
        welcome: Some(Welcome {
            id: 40,
            epoch: 1,
            world: [7; 32],
            origin: [8000., 0., 8000.],
            terrain_stream: true,
        }),
        public_population_attack: Some(
            PopulationState::new(profile.clone(), key(1).epoch, rate, [7; 32]).unwrap(),
        ),
        player_life: Some(private_state),
        snapshot: Some(Snapshot {
            version: VERSION,
            epoch: 1,
            origin: [8000., 0., 8000.],
            tick: 100,
            tick_hz: rate.hz(),
            players: [(20, [5., 9.]), (30, [9., 5.]), (40, [0., 0.])]
                .into_iter()
                .map(|(id, xz)| Player {
                    id,
                    xz,
                    y: 1.,
                    applied_sequence: 0,
                    facing: None,
                })
                .collect(),
        }),
        ..Default::default()
    };
    (state, profile, frame)
}

fn apply(state: &mut State, profile: &PopulationProfile, rate: TickRate, frame: &Frame) {
    state
        .public_population_attack
        .as_mut()
        .unwrap()
        .apply(&wire::encode(profile, rate, frame).unwrap())
        .unwrap();
}

#[test]
fn independent_population_cues_face_exact_remote_victims_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut state, profile, mut frame) = fixture(rate);
        apply(&mut state, &profile, rate, &frame);
        for (actor, position) in [(1, [8005., 1., 8009.]), (2, [8009., 1., 8005.])] {
            let cue = attack_cue(&state, key(actor)).unwrap();
            assert!(cue.remote);
            assert_eq!(cue.target.unwrap().metres(), position);
            assert_eq!(cue.serial, frame.actors[0].windup.unwrap().completes);
            assert!(cue.remaining > 0.);
        }
        assert!(attack_cue(&state, key(99)).is_none());
        state.snapshot.as_mut().unwrap().players.remove(0);
        assert!(attack_cue(&state, key(1)).unwrap().target.is_none());
        assert!(attack_cue(&state, key(2)).unwrap().target.is_some());
        // A transfer to this observer must switch only that NPC's target.
        frame.tick += 1;
        for actor in &mut frame.actors {
            actor.tick = frame.tick;
        }
        frame.actors[0].windup.as_mut().unwrap().target = key(40);
        apply(&mut state, &profile, rate, &frame);
        let own = attack_cue(&state, key(1)).unwrap();
        assert!(!own.remote);
        assert_eq!(own.target, state.authority_focus());
        assert!(attack_cue(&state, key(2)).unwrap().remote);
        state.snapshot.as_mut().unwrap().epoch = 2;
        assert!(attack_cue(&state, key(2)).unwrap().target.is_none());
    }
}

#[test]
fn public_population_cancellation_retirement_and_empty_frames_suppress_private_fallback() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut state, profile, mut frame) = fixture(rate);
        assert!(
            attack_cue(&state, key(1)).is_none(),
            "Awaiting public frame suppresses private data"
        );
        apply(&mut state, &profile, rate, &frame);
        frame.tick += 1;
        for actor in &mut frame.actors {
            actor.tick = frame.tick;
        }
        frame.actors[0].windup = None;
        apply(&mut state, &profile, rate, &frame);
        assert!(attack_cue(&state, key(1)).is_none());
        assert!(attack_cue(&state, key(2)).is_some());
        frame.tick += 1;
        frame.actors.remove(0);
        frame.actors[0].tick = frame.tick;
        apply(&mut state, &profile, rate, &frame);
        assert!(attack_cue(&state, key(1)).is_none());
        assert!(attack_cue(&state, key(2)).is_some());
        frame.tick += 1;
        frame.actors.clear();
        apply(&mut state, &profile, rate, &frame);
        assert!(attack_cue(&state, key(1)).is_none());
        assert!(attack_cue(&state, key(2)).is_none());
        state.public_population_attack = None;
        assert!(
            attack_cue(&state, key(1)).is_some(),
            "Retained private-only profile still works"
        );
    }
}

#[test]
fn late_population_cues_expire_at_the_latest_authority_snapshot() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut state, profile, frame) = fixture(rate);
        apply(&mut state, &profile, rate, &frame);
        let end = frame.actors[0].windup.unwrap().completes;
        state.snapshot.as_mut().unwrap().tick = end - 1;
        for actor in [1, 2] {
            assert_eq!(
                attack_cue(&state, key(actor)).unwrap().remaining,
                1. / rate.hz() as f32
            );
        }
        state.snapshot.as_mut().unwrap().tick = end;
        for actor in [1, 2] {
            assert!(attack_cue(&state, key(actor)).is_none());
        }
    }
}
