//! Explicit native qualification only: ordinary movement input and observed
//! authority state. Never changes health, creature state or combat outcomes.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use theia_core::player_life::LifeKey;
use theia_network::{client::State, wire::Intent};

#[derive(serde::Serialize)]
struct Sample {
    seconds: f32,
    owner: [f64; 3],
    creature: [f64; 3],
    health: u32,
}
#[derive(Resource)]
pub(crate) struct Run {
    variant: String,
    output: PathBuf,
    started: Option<Instant>,
    hit: Option<Instant>,
    retreat: Option<Instant>,
    attack_frames: u64,
    owner_key: Option<LifeKey>,
    npc: Option<theia_core::identity::EntityKey>,
    initial: Option<[f64; 3]>,
    position: Option<[f64; 3]>,
    creature: Option<[f64; 3]>,
    extent: Option<[f64; 2]>,
    min_health: u32,
    unexpected_life: bool,
    samples: Vec<Sample>,
    last_sample: Option<Instant>,
    last_report: Option<Instant>,
    frames: u64,
    moving_frames: u64,
    shot: bool,
    connected: bool,
}
pub(crate) fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(variant) = std::env::var("THEIA_CREATURE_PURSUIT_QUALIFY") else {
        return Ok(());
    };
    if !matches!(variant.as_str(), "A" | "B") {
        return Err("Pursuit qualification variant must be A or B".into());
    }
    let output = std::env::var_os("THEIA_GRAYBOX_QUALIFY")
        .ok_or("Pursuit qualification needs a capture directory")?;
    app.insert_resource(Run {
        variant,
        output: PathBuf::from(output),
        started: None,
        hit: None,
        retreat: None,
        attack_frames: 0,
        owner_key: None,
        npc: None,
        initial: None,
        position: None,
        creature: None,
        extent: None,
        min_health: u32::MAX,
        unexpected_life: false,
        samples: Vec::new(),
        last_sample: None,
        last_report: None,
        frames: 0,
        moving_frames: 0,
        shot: false,
        connected: false,
    })
    .add_systems(Last, observe);
    Ok(())
}
impl Run {
    pub(crate) fn intent(&mut self, state: &State) -> Intent {
        self.connected = state.error.is_none() && state.id.is_some();
        let Some(life) = state.player_life.as_ref().and_then(|p| p.current()) else {
            return Intent::default();
        };
        let Some(position) = state.authority_focus() else {
            return Intent::default();
        };
        let Some(creature) = state.targeting.as_ref().and_then(|p| p.targets.first()) else {
            return Intent::default();
        };
        if state.chunk_ack.is_none() {
            return Intent::default();
        }
        let now = Instant::now();
        let started = *self.started.get_or_insert(now);
        self.frames += 1;
        self.unexpected_life |= self.owner_key.is_some_and(|key| key != life.key)
            || self.npc.is_some_and(|key| key != creature.key);
        self.owner_key = Some(life.key);
        self.npc = Some(creature.key);
        self.initial.get_or_insert(position.metres());
        self.position = Some(position.metres());
        self.creature = Some(creature.position.metres());
        self.min_health = self.min_health.min(life.health);
        if life.health < life.maximum_health {
            self.hit.get_or_insert(now);
        }
        let x = creature.position.metres()[0];
        let extent = self.extent.get_or_insert([x, x]);
        extent[0] = extent[0].min(x);
        extent[1] = extent[1].max(x);
        if self.samples.len() < 160
            && self
                .last_sample
                .is_none_or(|last| now.duration_since(last) >= Duration::from_millis(250))
        {
            self.samples.push(Sample {
                seconds: now.duration_since(started).as_secs_f32(),
                owner: position.metres(),
                creature: creature.position.metres(),
                health: life.health,
            });
            self.last_sample = Some(now);
        }
        let seconds = now.duration_since(started).as_secs_f32();
        let axes = if self.variant == "A" {
            if self.retreat.is_none()
                && self.attack_frames >= 20
                && self
                    .hit
                    .is_some_and(|hit| now.duration_since(hit) >= Duration::from_secs(3))
            {
                self.retreat = Some(now);
            }
            if self
                .retreat
                .is_some_and(|start| now.duration_since(start) < Duration::from_secs(9))
            {
                [-1000, 0]
            } else {
                [0, 0]
            }
        } else if seconds < 11. {
            [0, 1000]
        } else {
            [0, 0]
        };
        self.moving_frames += u64::from(axes != [0, 0]);
        Intent {
            axes,
            ..Default::default()
        }
    }
    fn elapsed(&self) -> f32 {
        self.started.map_or(0., |s| s.elapsed().as_secs_f32())
    }
    fn ready(&self) -> bool {
        if self.variant == "A" {
            self.retreat
                .is_some_and(|start| start.elapsed() >= Duration::from_secs(22))
        } else {
            self.elapsed() >= 24.
        }
    }
}
fn observe(world: &mut World) {
    let Some(run) = world.get_resource::<Run>() else {
        return;
    };
    if run
        .last_report
        .is_some_and(|last| last.elapsed() < Duration::from_millis(250))
    {
        return;
    }
    let wolf = world
        .get_resource::<crate::creature_visual::Pending>()
        .map(|p| serde_json::to_value(&p.metrics).unwrap());
    let (report, output, shot, view) = {
        let mut run = world.resource_mut::<Run>();
        run.attack_frames = wolf
            .as_ref()
            .and_then(|m| m["attack_frames"].as_u64())
            .unwrap_or(0);
        run.last_report = Some(Instant::now());
        let displacement = run
            .initial
            .zip(run.position)
            .map(|(a, b)| [b[0] - a[0], b[1] - a[1], b[2] - a[2]]);
        let travelled = displacement.is_some_and(|d| {
            if run.variant == "A" {
                d[0] < -35.
            } else {
                d[2] > 45.
            }
        });
        let observed = run.extent.is_some_and(|e| e[1] - e[0] > 15.);
        let health = if run.variant == "A" {
            run.min_health > 0 && run.min_health < 100
        } else {
            run.min_health == 100
        };
        let animated = wolf.as_ref().is_some_and(|m| {
            m["transitions"][2].as_u64().unwrap_or(0) > 0
                && m["moving_frames"].as_u64().unwrap_or(0) > 100
        });
        let passed = run.ready()
            && run.connected
            && !run.unexpected_life
            && health
            && travelled
            && observed
            && animated;
        let shot = passed && !run.shot;
        if shot {
            run.shot = true;
        }
        let report = serde_json::json!({"passed":passed,"variant":run.variant,"seconds":run.elapsed(),"attacked":run.hit.is_some(),"retreat_started":run.retreat.is_some(),"connected":run.connected,"minimum_health":run.min_health,"unexpected_life":run.unexpected_life,"owner":run.position,"creature":run.creature,"creature_x_extent":run.extent,"owner_displacement":displacement,"frames":run.frames,"moving_frames":run.moving_frames,"samples":run.samples,"wolf":wolf,"scope":"Explicit native movement driver and observed authority; no physical keyboard delivery or client gameplay authority"});
        let view = if run.variant == "B" && run.elapsed() > 11. {
            run.position.zip(run.creature)
        } else {
            None
        };
        (report, run.output.clone(), shot, view)
    };
    if let Some((owner, creature)) = view
        && let Some(mut orbit) = world.get_resource_mut::<crate::camera::Orbit>()
    {
        let yaw = (-(creature[0] - owner[0])).atan2(-(creature[2] - owner[2])) as f32;
        orbit.set_angles(yaw, 0.25);
    }
    std::fs::write(
        output.join("pursuit.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .expect("Write native pursuit qualification");
    if shot {
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(output.join("pursuit.png")));
    }
}
