//! Cosmetic travel from server launches. Never reports impacts or changes HP.
use bevy::prelude::*;
use std::{collections::VecDeque, time::Instant};
use theia_core::{
    abilities::Catalog,
    coordinates::GlobalPosition,
    identity::EntityKey,
    projectiles::{Flight, State},
    time::TickRate,
};
use theia_network::simulation::combat::Event;
const MAX_ACTIVE: usize = 32;
const MAX_SEEN: usize = 64;
const MAX_CATCHUP_STEPS: u32 = 8;
#[derive(Clone, Copy, PartialEq, Eq)]
struct Key(EntityKey, u64);
struct Track {
    key: Key,
    target: EntityKey,
    goal: GlobalPosition,
    flight: Flight,
    received: Instant,
    initial_age: f64,
    hz: u32,
    steps: u32,
    lifetime: f64,
    entity: Option<Entity>,
    revision: u64,
}
#[derive(Resource, Default)]
pub(crate) struct Cues {
    tracks: Vec<Track>,
    seen: VecDeque<Key>,
    pub metrics: Metrics,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub launches: u64,
    pub terminals: u64,
    pub duplicates: u64,
    pub dropped: u64,
    pub late: u64,
    pub spawns: u64,
    pub removed: u64,
    pub max_active: usize,
    pub visible_frames: u64,
    pub visible_now: usize,
    pub visible_streak: u32,
    pub travel_frames: u64,
    pub waiting_impact_frames: u64,
    pub rebased_frames: u64,
    pub position_checks: u64,
    pub maximum_conversion_error_metres: f64,
}
impl Cues {
    pub(crate) fn offer(
        &mut self,
        event: &Event,
        catalog: &Catalog,
        tick: u64,
        now_tick: u64,
        hz: u32,
    ) {
        match event {
            Event::ProjectileLaunched {
                actor,
                sequence,
                target,
                ability,
                position,
                target_position,
            } => {
                let Some(profile) = catalog
                    .ability(ability.as_str())
                    .and_then(|a| a.projectile())
                else {
                    return;
                };
                let key = Key(*actor, *sequence);
                if self.seen.contains(&key) {
                    self.metrics.duplicates += 1;
                    return;
                }
                self.remember(key);
                self.metrics.launches += 1;
                let rate = TickRate::from_hz(hz).unwrap_or_default();
                let age = now_tick.saturating_sub(tick) as f64 / f64::from(rate.hz());
                if age >= profile.lifetime().as_secs_f64() {
                    self.metrics.late += 1;
                    return;
                }
                if self.tracks.len() >= MAX_ACTIVE {
                    self.metrics.dropped += 1;
                    return;
                }
                self.tracks.push(Track {
                    key,
                    target: *target,
                    goal: *target_position,
                    flight: Flight::launch(profile, *position, rate),
                    received: Instant::now(),
                    initial_age: age,
                    hz: rate.hz(),
                    steps: 0,
                    lifetime: profile.lifetime().as_secs_f64(),
                    entity: None,
                    revision: 0,
                });
                self.metrics.max_active = self.metrics.max_active.max(self.tracks.len());
            }
            Event::AbilityDamage(fact)
                if catalog
                    .ability(fact.ability.as_str())
                    .is_some_and(|a| a.projectile().is_some()) =>
            {
                self.terminal(Key(fact.actor, fact.sequence));
            }
            Event::ProjectileEnded {
                actor,
                sequence,
                ability,
                ..
            }
            | Event::Damage {
                actor,
                sequence,
                ability,
                ..
            }
            | Event::Avoided {
                actor,
                sequence,
                ability,
                ..
            } if catalog
                .ability(ability.as_str())
                .is_some_and(|a| a.projectile().is_some()) =>
            {
                let key = Key(*actor, *sequence);
                self.terminal(key);
            }
            _ => {}
        }
    }
    fn terminal(&mut self, key: Key) {
        if self.tracks.iter().any(|t| t.key == key) {
            self.metrics.terminals += 1;
        }
        self.tracks.retain(|t| t.key != key);
        self.remember(key);
    }
    fn remember(&mut self, key: Key) {
        if self.seen.contains(&key) {
            return;
        }
        if self.seen.len() == MAX_SEEN {
            self.seen.pop_front();
        }
        self.seen.push_back(key);
    }
}
#[derive(Resource)]
struct Assets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}
#[derive(Component)]
struct Visual(Key);
pub(crate) fn install(app: &mut App) {
    app.init_resource::<Cues>()
        .add_systems(Startup, setup)
        .add_systems(Update, render.after(crate::combat::Presentation));
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<bevy::asset::Assets<Mesh>>,
    mut materials: ResMut<bevy::asset::Assets<StandardMaterial>>,
) {
    commands.insert_resource(Assets {
        mesh: meshes.add(Sphere::new(0.12).mesh().uv(12, 8)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.65, 1.),
            unlit: true,
            ..default()
        }),
    });
}
fn render(
    mut commands: Commands,
    mut cues: ResMut<Cues>,
    assets: Res<Assets>,
    frame: Res<crate::floating_origin::Frame>,
    visuals: Query<(Entity, &Visual, &ViewVisibility)>,
    targets: Query<(
        &crate::targeting::TargetVisual,
        &crate::targeting::motion::TargetMotion,
    )>,
) {
    let now = Instant::now();
    cues.metrics.visible_now = 0;
    cues.tracks.retain(|t| {
        t.flight.state() != State::Expired
            && t.initial_age + now.saturating_duration_since(t.received).as_secs_f64()
                < t.lifetime + 0.25
    });
    for (entity, visual, visible) in &visuals {
        if !cues.tracks.iter().any(|t| t.key == visual.0) {
            commands.entity(entity).despawn();
            cues.metrics.removed += 1;
        } else {
            cues.metrics.visible_frames += u64::from(visible.get());
            cues.metrics.visible_now += usize::from(visible.get());
        }
    }
    cues.metrics.visible_streak = if cues.metrics.visible_now > 0 {
        cues.metrics.visible_streak.saturating_add(1)
    } else {
        0
    };
    let Cues {
        tracks, metrics, ..
    } = &mut *cues;
    for track in tracks {
        if let Some((_, motion)) = targets
            .iter()
            .find(|(target, _)| target.key == track.target)
        {
            let mut goal = motion.global().metres();
            goal[1] += 0.6;
            if let Some(goal) = GlobalPosition::new(goal) {
                track.goal = goal;
            }
        }
        let age = track.initial_age + now.saturating_duration_since(track.received).as_secs_f64();
        let wanted = (age * f64::from(track.hz)).floor() as u32;
        for _ in 0..wanted.saturating_sub(track.steps).min(MAX_CATCHUP_STEPS) {
            if track.flight.advance(track.goal).is_none() {
                break;
            }
            track.steps += 1;
        }
        if track.flight.state() == State::Expired {
            if let Some(entity) = track.entity.take() {
                commands.entity(entity).despawn();
                metrics.removed += 1;
            }
            continue;
        }
        let global = if track.flight.state() == State::Travelling {
            metrics.travel_frames += 1;
            let mut preview = track.flight.clone();
            let fraction = (age * f64::from(track.hz) - f64::from(track.steps)).clamp(0., 1.);
            let segment = preview
                .advance(track.goal)
                .expect("Travelling preview has one segment");
            let a = segment.from.metres();
            let b = segment.to.metres();
            GlobalPosition::new(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * fraction))
                .unwrap_or(segment.from)
        } else {
            metrics.waiting_impact_frames += 1;
            track.flight.position()
        };
        let Some(local) = frame.render.to_local(global) else {
            if let Some(entity) = track.entity {
                commands.entity(entity).insert(Visibility::Hidden);
            }
            continue;
        };
        metrics.position_checks += 1;
        if let Some(restored) = frame.render.to_global(local) {
            let error = restored
                .metres()
                .into_iter()
                .zip(global.metres())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            metrics.maximum_conversion_error_metres =
                metrics.maximum_conversion_error_metres.max(error);
        }
        if track.revision != frame.revision {
            metrics.rebased_frames += 1;
            track.revision = frame.revision;
        }
        let transform = Transform::from_translation(Vec3::from_array(local.metres()));
        if let Some(entity) = track.entity {
            commands
                .entity(entity)
                .insert((transform, Visibility::Inherited));
        } else {
            track.entity = Some(
                commands
                    .spawn((
                        Visual(track.key),
                        Mesh3d(assets.mesh.clone()),
                        MeshMaterial3d(assets.material.clone()),
                        transform,
                        bevy::light::NotShadowCaster,
                        bevy::light::NotShadowReceiver,
                    ))
                    .id(),
            );
            metrics.spawns += 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::identity::{EntityId, WorldEpoch};
    fn catalog() -> Catalog {
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/gameplay/development-resource-abilities.json"
        ))
        .unwrap();
        raw["schema"] = serde_json::json!(6);
        raw["abilities"][0]["projectile"] = serde_json::json!({"speed_metres_per_second":20.,"maximum_distance_metres":45.,"lifetime_ms":3000,"target_radius_metres":0.2});
        Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap()
    }
    fn launch(catalog: &Catalog, sequence: u64) -> Event {
        let key = |id| EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        };
        Event::ProjectileLaunched {
            actor: key(1),
            sequence,
            target: key(2),
            ability: catalog.abilities()[0].id().clone(),
            position: GlobalPosition::new([15040., 25., 15040.]).unwrap(),
            target_position: GlobalPosition::new([15048., 25., 15040.]).unwrap(),
        }
    }
    #[test]
    fn cosmetic_budget_duplicates_terminal_order_and_late_launches_cannot_grow_tracks() {
        let catalog = catalog();
        let mut cues = Cues::default();
        for sequence in 1..=40 {
            cues.offer(&launch(&catalog, sequence), &catalog, 10, 10, 30);
        }
        assert_eq!(cues.tracks.len(), MAX_ACTIVE);
        assert_eq!(cues.metrics.dropped, 8);
        cues.offer(&launch(&catalog, 1), &catalog, 10, 10, 30);
        assert_eq!(cues.metrics.duplicates, 1);
        let Event::ProjectileLaunched {
            actor,
            target,
            ability,
            position,
            ..
        } = launch(&catalog, 1)
        else {
            unreachable!()
        };
        let end = Event::ProjectileEnded {
            actor,
            target,
            ability,
            sequence: 1,
            position,
            reason: theia_network::simulation::combat::ProjectileEnd::Expired,
        };
        cues.offer(&end, &catalog, 20, 20, 30);
        assert_eq!(cues.tracks.len(), MAX_ACTIVE - 1);
        cues.offer(&launch(&catalog, 1), &catalog, 10, 20, 30);
        assert_eq!(cues.tracks.len(), MAX_ACTIVE - 1);
        cues.offer(&launch(&catalog, 41), &catalog, 10, 200, 30);
        assert_eq!(cues.metrics.late, 1);
        for sequence in 42..200 {
            cues.offer(&launch(&catalog, sequence), &catalog, 10, 200, 30);
        }
        assert_eq!(cues.seen.len(), MAX_SEEN);
        assert_eq!(cues.tracks.len(), MAX_ACTIVE - 1);
    }
    #[test]
    fn terminal_before_launch_suppresses_that_cosmetic_without_inventing_an_impact() {
        let catalog = catalog();
        let mut cues = Cues::default();
        let Event::ProjectileLaunched {
            actor,
            target,
            ability,
            position,
            ..
        } = launch(&catalog, 1)
        else {
            unreachable!()
        };
        cues.offer(
            &Event::Damage {
                actor,
                target,
                ability,
                sequence: 1,
                position,
                amount: 20,
                remaining: 80,
            },
            &catalog,
            20,
            20,
            30,
        );
        cues.offer(&launch(&catalog, 1), &catalog, 10, 20, 30);
        assert!(cues.tracks.is_empty());
        assert_eq!(cues.metrics.spawns, 0);
        assert_eq!(cues.metrics.duplicates, 1);
        assert_eq!(cues.metrics.terminals, 0);
    }
    #[test]
    fn typed_arcane_hit_and_miss_end_the_matching_projectile_and_suppress_late_launch() {
        use theia_network::simulation::combat::ability_damage::Outcome;
        for outcome in [
            theia_core::damage::spell::Outcome::Hit,
            theia_core::damage::spell::Outcome::Miss,
        ] {
            let catalog = Catalog::from_json(include_bytes!(
                "../../../content/gameplay/development-classic-abilities.json"
            ))
            .unwrap();
            let fact = crate::combat::test_facts::ability_fact(Outcome::Arcane(outcome), 100);
            let launch = Event::ProjectileLaunched {
                actor: fact.actor,
                target: fact.target,
                ability: fact.ability.clone(),
                sequence: fact.sequence,
                position: fact.position,
                target_position: GlobalPosition::new([0., 0., 2.]).unwrap(),
            };
            let mut cues = Cues::default();
            cues.offer(&launch, &catalog, 10, 10, 30);
            assert_eq!(cues.tracks.len(), 1);
            cues.offer(&Event::AbilityDamage(fact), &catalog, 12, 12, 30);
            assert!(cues.tracks.is_empty());
            assert_eq!(cues.metrics.terminals, 1);
            cues.offer(&launch, &catalog, 10, 12, 30);
            assert!(cues.tracks.is_empty());
        }
    }
}
