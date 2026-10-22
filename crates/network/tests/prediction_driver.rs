use std::time::{Duration, Instant};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    client::{State, TransportStats},
    commands::Timeline,
    kinematic::Motion,
    prediction::{Correction, Pose},
    prediction_driver::Driver,
    prediction_wire::Corrected,
    procedural::{self, Patch},
    wire::{ChunkReady, Intent, VERSION},
};
fn state(patch: &Patch, now: Instant) -> State {
    let mut state = State {
        correction: Some(Corrected {
            speed: 1,
            state: Correction {
                world: procedural::IDENTITY,
                actor: EntityKey {
                    epoch: WorldEpoch::new(7).unwrap(),
                    entity: EntityId::new(1).unwrap(),
                },
                rate: TickRate::Hz30,
                input: Timeline::new(1, TickRate::Hz30, 0).unwrap().state(),
                pose: Pose {
                    feet: patch.spawn(8000., 8000.),
                    motion: Motion::default().snapshot(),
                },
                ready_sequence: 1,
            },
        }),
        correction_received: Some(now),
        chunk_report: Some(ChunkReady {
            version: VERSION,
            epoch: 7,
            sequence: 1,
            chunks: patch.ids.iter().map(|id| [id.x, id.z]).collect(),
        }),
        ..State::default()
    };
    state.correction_ready = state.chunk_report.clone();
    state
}
#[test]
fn stale_clocks_do_not_emit_commands_and_fresh_truth_resumes_without_sequence_reuse() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::default();
    let intent = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    driver.tick(&state, intent, Some(&patch), now).unwrap();
    let sent = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(40),
        )
        .unwrap()
        .unwrap()
        .packet
        .unwrap();
    let sequence = sent.commands.last().unwrap().sequence;
    let stale = driver
        .tick(&state, intent, Some(&patch), now + Duration::from_secs(3))
        .unwrap()
        .unwrap();
    assert!(stale.packet.is_none());
    let packet = state.correction.as_mut().unwrap();
    packet.state.input.tick = 90;
    packet.state.input.fresh_tick = 90;
    state.correction_received = Some(now + Duration::from_secs(3));
    let driven = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(3040),
        )
        .unwrap()
        .unwrap();
    let resumed = driven.packet.unwrap();
    assert!(
        resumed
            .commands
            .iter()
            .all(|c| c.sequence > sequence && c.tick > 90)
    );
    assert!(driver.stats.max_history <= 64);
}
#[test]
fn unsupported_latency_and_missing_collision_cannot_generate_unbounded_clock_debt() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::default();
    state.transport = Some(TransportStats {
        rtt_ms: 10000.,
        ..TransportStats::default()
    });
    let result = driver
        .tick(&state, Intent::default(), Some(&patch), now)
        .unwrap()
        .unwrap();
    assert!(result.packet.is_none());
    assert_eq!(driver.stats.predicted_steps, 0);
    state.transport = None;
    let result = driver
        .tick(
            &state,
            Intent::default(),
            None,
            now + Duration::from_millis(40),
        )
        .unwrap()
        .unwrap();
    assert!(result.packet.is_none());
    assert!(driver.stats.max_history <= 64);
}

#[test]
fn pending_readiness_replacement_preserves_only_still_loaded_authorized_overlap() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::default();
    state.chunk_report.as_mut().unwrap().sequence = 2;
    state.chunk_report.as_mut().unwrap().chunks =
        procedural::neighbours(ChunkId { x: 63, z: 62 }, 1)
            .iter()
            .map(|id| [id.x, id.z])
            .collect();
    let initial = state.correction.unwrap().state.pose.feet.metres()[0];
    let moving = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    driver.tick(&state, moving, Some(&patch), now).unwrap();
    let driven = driver
        .tick(
            &state,
            moving,
            Some(&patch),
            now + Duration::from_millis(40),
        )
        .unwrap()
        .unwrap();
    assert!(driven.pose.feet.metres()[0] > initial);
    assert_eq!(
        driven.packet.unwrap().ready_sequence,
        1,
        "Command describes observed authority permission"
    );
    state.chunk_report.as_mut().unwrap().sequence = 3;
    state.chunk_report.as_mut().unwrap().chunks.clear();
    let held = driver
        .tick(
            &state,
            moving,
            Some(&patch),
            now + Duration::from_millis(80),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        held.pose.feet.metres()[0],
        driven.pose.feet.metres()[0],
        "Withdrawn live geometry cannot be predicted into"
    );
}
#[test]
fn correction_focus_can_precede_the_snapshot_across_a_continent_teleport() {
    use theia_network::wire::{Player, Snapshot};
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    state.id = Some(1);
    state.snapshot = Some(Snapshot {
        version: VERSION,
        epoch: 7,
        origin: [8000., 0., 8000.],
        tick: 10,
        tick_hz: 30,
        players: vec![Player {
            id: 1,
            xz: [0., 0.],
            y: 1.,
            applied_sequence: 0,
        }],
    });
    let correction = state.correction.as_mut().unwrap();
    correction.state.input.tick = 11;
    correction.state.pose.feet = GlobalPosition::new([15800., 25., 180.]).unwrap();
    assert_eq!(
        state.authority_focus().unwrap().metres(),
        [15800., 25. + f64::from(theia_network::world::PAWN_Y), 180.]
    );
    state.snapshot.as_mut().unwrap().tick = 12;
    assert_eq!(
        state.authority_focus().unwrap().metres(),
        [8000., 1., 8000.]
    );
}
