//! Server-only fixed-step authority. Inputs never choose timestep, speed or position.
use crate::wire::{Input, Intent, MAX_PLAYERS, Player, Snapshot, TICK_HZ, VERSION};
use std::collections::BTreeMap;

pub const INPUT_TIMEOUT_TICKS: u64 = 10;
#[derive(Default)]
pub struct World {
    tick: u64,
    next_id: u64,
    players: BTreeMap<u64, Actor>,
    terrain: Option<crate::terrain::Terrain>,
}
struct Actor {
    player: Player,
    intent: Intent,
    input_tick: u64,
}
impl World {
    pub fn with_terrain(terrain: crate::terrain::Terrain) -> Self {
        Self {
            terrain: Some(terrain),
            ..Self::default()
        }
    }
    pub fn identity(&self) -> [u8; 32] {
        self.terrain
            .as_ref()
            .map_or(crate::terrain::GRAYBOX_ID, |t| t.identity)
    }
    pub fn join(&mut self) -> Option<u64> {
        if self.players.len() >= MAX_PLAYERS {
            return None;
        }
        self.next_id = self.next_id.checked_add(1)?;
        let id = self.next_id;
        self.players.insert(
            id,
            Actor {
                player: Player {
                    id,
                    xz: [((id - 1) % 4) as f32 * 2., 0.],
                    y: self.terrain.as_ref().map_or(crate::world::PAWN_Y, |t| {
                        t.spawn_y([((id - 1) % 4) as f32 * 2., 0.])
                    }),
                    applied_sequence: 0,
                },
                intent: Intent::default(),
                input_tick: self.tick,
            },
        );
        Some(id)
    }
    pub fn leave(&mut self, id: u64) {
        self.players.remove(&id);
    }
    pub fn input(&mut self, id: u64, input: Input) -> bool {
        let Some(actor) = self.players.get_mut(&id) else {
            return false;
        };
        if input.version != VERSION
            || !input.intent.valid()
            || input.sequence <= actor.player.applied_sequence
        {
            return false;
        }
        actor.player.applied_sequence = input.sequence;
        actor.intent = input.intent;
        actor.input_tick = self.tick;
        true
    }
    pub fn step(&mut self) -> Snapshot {
        self.tick += 1;
        for actor in self.players.values_mut() {
            if self.tick - actor.input_tick > INPUT_TIMEOUT_TICKS {
                actor.intent = Intent::default();
            }
            if let Some(terrain) = &self.terrain {
                actor.player.xz =
                    terrain.move_player(actor.player.xz, actor.intent, 1. / TICK_HZ as f32);
                actor.player.y = terrain.height(actor.player.xz).expect("Legal terrain pose")
                    + crate::world::PAWN_Y;
            } else {
                actor.player.xz =
                    crate::world::move_player(actor.player.xz, actor.intent, 1. / TICK_HZ as f32);
            }
        }
        Snapshot {
            version: VERSION,
            tick: self.tick,
            players: self.players.values().map(|a| a.player.clone()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::WORLD_LIMIT;
    fn moving(sequence: u64, axes: [i16; 2]) -> Input {
        Input {
            version: VERSION,
            sequence,
            intent: Intent {
                axes,
                sprint: false,
            },
        }
    }
    #[test]
    fn terrain_players_pass_through_and_snapshots_follow_surface_height() {
        let a = [-24., -7.2, -24.];
        let b = [24., 7.2, -24.];
        let c = [24., 7.2, 24.];
        let d = [-24., -7.2, 24.];
        let bytes=serde_json::to_vec(&serde_json::json!({"schema":1,"source_origin":[0,0,0],"visual_sha256":vec![0;32],"triangles":[[a,c,b],[a,d,c]]})).unwrap();
        let mut world = World::with_terrain(crate::terrain::Terrain::decode(&bytes).unwrap());
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        let mut crossed = false;
        for sequence in 1..=30 {
            world.input(b, moving(sequence, [-1000, 0]));
            let s = world.step();
            for p in &s.players {
                assert!((p.y - (p.xz[0] * 0.3 + 0.9)).abs() < 1e-5);
            }
            let first = s.players.iter().find(|p| p.id == a).unwrap();
            let second = s.players.iter().find(|p| p.id == b).unwrap();
            crossed |= (second.xz[0] - first.xz[0]).abs() < 0.1;
            if sequence == 30 {
                assert!(second.xz[0] < first.xz[0] - 2.9);
            }
        }
        assert!(crossed, "Players share terrain without blocking each other");
    }
    #[test]
    fn speed_diagonal_stale_timeout_and_bounds_are_server_owned() {
        let mut world = World::default();
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        for sequence in 1..=30 {
            assert!(world.input(a, moving(sequence, [1000, 1000])));
            assert!(world.input(b, moving(sequence, [1000, 0])));
            // Even arbitrarily many accepted/replayed packets cannot run an extra step.
            assert!(!world.input(a, moving(sequence, [-1000, 0])));
            world.step();
        }
        let snapshot = world.step();
        let diagonal = snapshot.players[0].xz;
        assert!((diagonal[0].hypot(diagonal[1]) - 5. * 31. / 30.).abs() < 0.001);
        let before = snapshot.players[1].xz;
        for _ in 0..30 {
            world.step();
        }
        let stopped = world.step().players[1].xz;
        for _ in 0..30 {
            world.step();
        }
        assert_eq!(stopped, world.step().players[1].xz);
        assert!(stopped[0] - before[0] < 2.);
        for seq in 31..1000 {
            world.input(a, moving(seq, [1000, 1000]));
            world.step();
        }
        assert_eq!(world.step().players[0].xz, [WORLD_LIMIT; 2]);
    }
    #[test]
    fn capacity_disconnect_non_reused_ids_and_invalid_inputs() {
        let mut world = World::default();
        let first = world.join().unwrap();
        assert!(!world.input(first, moving(1, [1001, 0])));
        assert!(!world.input(999, moving(1, [0, 0])));
        for _ in 1..MAX_PLAYERS {
            world.join().unwrap();
        }
        assert!(world.join().is_none());
        world.leave(first);
        assert!(world.join().unwrap() > MAX_PLAYERS as u64);
        assert!(world.step().players.iter().all(|p| p.id != first));
    }

    #[test]
    fn players_can_overlap_and_pass_through_each_other() {
        let mut world = World::default();
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        for sequence in 1..=12 {
            world.input(a, moving(sequence, [1000, 0]));
            world.input(b, moving(sequence, [-1000, 0]));
            let snapshot = world.step();
            if sequence == 6 {
                assert!((snapshot.players[0].xz[0] - snapshot.players[1].xz[0]).abs() < 0.00001);
            }
            if sequence == 12 {
                assert!((snapshot.players[0].xz[0] - 2.).abs() < 0.00001);
                assert!(snapshot.players[1].xz[0].abs() < 0.00001);
            }
        }
    }
}
