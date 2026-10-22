//! CPU-only workload; never presented as network/rendered-client capacity.
use std::{hint::black_box, time::Instant};
use theia_core::time::TickRate;
use theia_network::{
    simulation::World,
    wire::{Input, Intent, MAX_PLAYERS, VERSION, encode},
};

fn main() -> theia_network::Result<()> {
    for hz in [20, 30, 60] {
        let mut world = World::at_rate(TickRate::from_hz(hz).unwrap());
        let players: Vec<_> = (0..MAX_PLAYERS).map(|_| world.join().unwrap()).collect();
        let mut samples = Vec::new();
        let mut bytes = 0;
        // Ten seconds of simulated time; this loop does not measure wall-clock scheduling.
        for sequence in 1..=u64::from(hz) * 10 {
            let started = Instant::now();
            for &id in &players {
                let axes = if sequence % 20 < 10 {
                    [1000, 0]
                } else {
                    [-1000, 0]
                };
                world.input(
                    id,
                    Input {
                        version: VERSION,
                        epoch: world.epoch(),
                        sequence,
                        intent: Intent {
                            jump: false,
                            axes,
                            sprint: false,
                        },
                    },
                );
            }
            bytes += black_box(encode(&world.step())?).len();
            samples.push(started.elapsed().as_nanos());
        }
        samples.sort_unstable();
        println!(
            "{}",
            serde_json::json!({
                "scope":"CPU graybox authority and JSON encoding; no live clients, transport, terrain or GPU",
                "tick_hz":hz,"players":players.len(),"steps":samples.len(),
                "p50_ns":samples[samples.len()/2],"p95_ns":samples[samples.len()*95/100],
                "max_ns":samples[samples.len()-1],"encoded_bytes":bytes
            })
        );
    }
    Ok(())
}
