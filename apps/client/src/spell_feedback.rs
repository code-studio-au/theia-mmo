//! Bounded original feedback through standard Bevy meshes, materials and audio.
pub(crate) mod data;
use crate::{
    floating_origin::Frame,
    network::{Network, RemotePawn},
};
use bevy::{
    audio::{AudioPlugin, SpatialScale, Volume},
    prelude::*,
    text::FontSize,
};
use data::{Definition, Priority, Sound, Trigger};
use std::{collections::VecDeque, sync::Arc, time::Instant};
use theia_core::{
    coordinates::{GlobalPosition, LocalPosition},
    identity::EntityKey,
};
use theia_network::simulation::combat::Event;

// One audio unit per ten world metres keeps prototype cues audible at the
// ordinary third-person camera distance through Bevy/rodio attenuation.
const AUDIO_SCALE: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Admission<T> {
    Allow,
    Evict(T),
    Drop,
}
fn admit<T: Copy>(
    priority: Priority,
    active: &[(T, Priority, u64)],
    limit: usize,
    reserved: usize,
) -> Admission<T> {
    let cosmetics = active
        .iter()
        .filter(|(_, p, _)| *p == Priority::Cosmetic)
        .count();
    if priority == Priority::Cosmetic && cosmetics >= limit - reserved {
        return Admission::Drop;
    }
    if active.len() < limit {
        return Admission::Allow;
    }
    if priority == Priority::Critical
        && let Some((id, _, _)) = active
            .iter()
            .filter(|(_, p, _)| *p == Priority::Cosmetic)
            .min_by_key(|(_, _, serial)| *serial)
    {
        return Admission::Evict(*id);
    }
    Admission::Drop
}
#[derive(Clone)]
struct Request {
    style: usize,
    position: GlobalPosition,
    follow: Option<EntityKey>,
    tick: u64,
    serial: u64,
}
#[derive(Resource)]
pub(crate) struct Pending {
    definition: Arc<Definition>,
    queue: VecDeque<Request>,
    serial: u64,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub offered: u64,
    pub injected_cosmetics: u64,
    pub critical_spatial_initialized: u64,
    pub cosmetic_queue_drops: u64,
    pub critical_queue_overflow: u64,
    pub visual_spawns: u64,
    pub voice_spawns: u64,
    pub critical_visuals: u64,
    pub critical_voices: u64,
    pub critical_visuals_under_cosmetics: u64,
    pub critical_voices_under_cosmetics: u64,
    pub cosmetic_visual_drops: u64,
    pub cosmetic_voice_drops: u64,
    pub cosmetic_evictions: u64,
    pub critical_visual_overflow: u64,
    pub critical_voice_overflow: u64,
    pub max_visuals: usize,
    pub max_voices: usize,
    pub spatial_initialized: u64,
    pub critical_visible_samples: u64,
    pub position_checks: u64,
    pub max_position_error_metres: f32,
    pub rebased_visual_samples: u64,
    pub rebased_voice_samples: u64,
    pub far: u64,
    pub late: u64,
    pub last_cue: String,
}
impl Pending {
    pub fn offer(&mut self, event: &Event, tick: u64, metrics: &mut Metrics) {
        let (trigger, ability, position, follow) = match event {
            Event::Started {
                actor,
                cast,
                position,
            } => (
                Trigger::CastStart,
                Some(&cast.ability),
                *position,
                Some(*actor),
            ),
            Event::Damage {
                ability, position, ..
            } => (Trigger::Damage, Some(ability), *position, None),
            Event::Died { position, .. } => (Trigger::Death, None, *position, None),
            Event::Respawned { position, .. } => (Trigger::Respawn, None, *position, None),
            _ => return,
        };
        let style = self
            .definition
            .cues
            .iter()
            .position(|c| c.trigger == trigger && c.ability.as_ref() == ability)
            .expect("Validated cue coverage");
        metrics.offered += 1;
        // Explicit qualification only: synthetic presentation requests overlap
        // every real outcome, including damage/death, without changing authority.
        if std::env::var("THEIA_FEEDBACK_STRESS").as_deref() == Ok("1")
            && let Some(cosmetic) =
                self.definition.cues.iter().position(|c| {
                    c.trigger == Trigger::CastStart && c.priority == Priority::Cosmetic
                })
        {
            for _ in 0..256 {
                metrics.injected_cosmetics += 1;
                self.enqueue(cosmetic, position, None, tick, metrics);
            }
        }
        self.enqueue(style, position, follow, tick, metrics);
    }
    fn enqueue(
        &mut self,
        style: usize,
        position: GlobalPosition,
        follow: Option<EntityKey>,
        tick: u64,
        metrics: &mut Metrics,
    ) {
        let priority = self.definition.cues[style].priority;
        let active: Vec<_> = self
            .queue
            .iter()
            .map(|r| (r.serial, self.definition.cues[r.style].priority, r.serial))
            .collect();
        match admit(priority, &active, 64, 16) {
            Admission::Allow => {}
            Admission::Evict(serial) => {
                self.queue.retain(|r| r.serial != serial);
                metrics.cosmetic_evictions += 1;
            }
            Admission::Drop => {
                if priority == Priority::Critical {
                    metrics.critical_queue_overflow += 1;
                } else {
                    metrics.cosmetic_queue_drops += 1;
                }
                return;
            }
        }
        self.serial = self
            .serial
            .checked_add(1)
            .expect("Presentation serial exhausted");
        self.queue.push_back(Request {
            style,
            position,
            follow,
            tick,
            serial: self.serial,
        });
    }
}
#[derive(Resource, Clone)]
struct Assets {
    definition: Arc<Definition>,
    mesh: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    cast: Handle<AudioSource>,
    impact: Handle<AudioSource>,
}
#[derive(Component)]
pub(crate) struct Visual {
    request: Request,
    born: Instant,
    initial_age: f32,
    revision: u64,
}
#[derive(Component)]
struct Voice {
    request: Request,
    born: Instant,
    revision: u64,
    initialized: bool,
}
#[derive(Component)]
struct Caption;
#[derive(Resource, Default)]
struct CaptionTime(Option<Instant>);

pub fn install(app: &mut App, definition: Definition) {
    let definition = Arc::new(definition);
    app.insert_resource(Pending {
        definition,
        queue: VecDeque::with_capacity(64),
        serial: 0,
    })
    .init_resource::<Metrics>()
    .init_resource::<CaptionTime>()
    .add_systems(Startup, setup.after(crate::scene::setup))
    .add_systems(Update, tick.after(crate::combat::Presentation))
    .add_systems(Last, sync_audio);
}
pub fn audio_plugin() -> AudioPlugin {
    AudioPlugin {
        global_volume: Volume::Linear(0.3).into(),
        default_spatial_scale: SpatialScale::new(AUDIO_SCALE),
    }
}
fn setup(world: &mut World) {
    let definition = world.resource::<Pending>().definition.clone();
    let mesh = world
        .resource_mut::<bevy::asset::Assets<Mesh>>()
        .add(Sphere::new(1.).mesh().uv(12, 8));
    let mut materials = Vec::new();
    for cue in &definition.cues {
        let [r, g, b, a] = cue.colour;
        materials.push(
            world
                .resource_mut::<bevy::asset::Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgba(r, g, b, a),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
        );
    }
    let mut audio = world.resource_mut::<bevy::asset::Assets<AudioSource>>();
    let cast = audio.add(AudioSource {
        bytes: Arc::from(include_bytes!("../../../content/gameplay/audio/cast.wav").as_slice()),
    });
    let impact = audio.add(AudioSource {
        bytes: Arc::from(include_bytes!("../../../content/gameplay/audio/impact.wav").as_slice()),
    });
    world.insert_resource(Assets {
        definition,
        mesh,
        materials,
        cast,
        impact,
    });
    let cameras: Vec<_> = world
        .query_filtered::<Entity, With<Camera3d>>()
        .iter(world)
        .collect();
    assert_eq!(
        cameras.len(),
        1,
        "Feedback expects the normal single game camera"
    );
    world
        .entity_mut(cameras[0])
        .insert(SpatialListener::new(0.4));
    world.spawn((
        Caption,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            bottom: px(24),
            padding: UiRect::all(px(10)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.85)),
    ));
}
fn position(world: &mut World, request: &Request, frame: &Frame) -> GlobalPosition {
    if let Some(actor) = request.follow {
        let network = world.resource::<Network>();
        let state = network.state();
        if state
            .welcome
            .as_ref()
            .is_some_and(|w| w.epoch == actor.epoch.value())
        {
            if state.id == Some(actor.entity.value()) {
                if let Some(p) = network.rendered_global() {
                    return p;
                }
            } else {
                for (pawn, transform) in world.query::<(&RemotePawn, &Transform)>().iter(world) {
                    if pawn.actor_id() == actor.entity.value()
                        && let Some(local) = LocalPosition::new(transform.translation.to_array())
                        && let Some(global) = frame.render.to_global(local)
                    {
                        return global;
                    }
                }
            }
        }
    }
    request.position
}
fn visual_leases(world: &mut World, definition: &Definition) -> Vec<(Entity, Priority, u64)> {
    world
        .query::<(Entity, &Visual)>()
        .iter(world)
        .map(|(id, v)| {
            (
                id,
                definition.cues[v.request.style].priority,
                v.request.serial,
            )
        })
        .collect()
}
fn voice_leases(world: &mut World, definition: &Definition) -> Vec<(Entity, Priority, u64)> {
    world
        .query::<(Entity, &Voice)>()
        .iter(world)
        .map(|(id, v)| {
            (
                id,
                definition.cues[v.request.style].priority,
                v.request.serial,
            )
        })
        .collect()
}
fn reserve(
    world: &mut World,
    priority: Priority,
    leases: &mut Vec<(Entity, Priority, u64)>,
    limit: usize,
    reserved: usize,
    metrics: &mut Metrics,
    visual: bool,
) -> bool {
    match admit(priority, leases, limit, reserved) {
        Admission::Allow => true,
        Admission::Evict(id) => {
            world.despawn(id);
            leases.retain(|(other, _, _)| *other != id);
            metrics.cosmetic_evictions += 1;
            true
        }
        Admission::Drop => {
            match (priority, visual) {
                (Priority::Critical, true) => metrics.critical_visual_overflow += 1,
                (Priority::Critical, false) => metrics.critical_voice_overflow += 1,
                (Priority::Cosmetic, true) => metrics.cosmetic_visual_drops += 1,
                (Priority::Cosmetic, false) => metrics.cosmetic_voice_drops += 1,
            }
            false
        }
    }
}
fn tick(world: &mut World) {
    let assets = world.resource::<Assets>().clone();
    let definition = &assets.definition;
    let mut pending = world.remove_resource::<Pending>().unwrap();
    let mut metrics = world.remove_resource::<Metrics>().unwrap();
    // Frame is temporarily removed only while resolving presentation positions;
    // authority and networking are never called mutably or awaited here.
    let frame = world.remove_resource::<Frame>().unwrap();
    let state = world.resource::<Network>().state();
    let now_tick = state
        .snapshot
        .as_ref()
        .map_or(0, |s| s.tick)
        .max(state.combat.as_ref().map_or(0, |s| s.tick));
    let hz = state
        .snapshot
        .as_ref()
        .map_or(30., |s| f64::from(s.tick_hz));
    let visuals: Vec<_> = world
        .query::<(Entity, &Visual)>()
        .iter(world)
        .map(|(e, v)| (e, v.request.clone(), v.born, v.initial_age, v.revision))
        .collect();
    for (id, request, born, initial_age, revision) in visuals {
        let cue = &definition.cues[request.style];
        let age = initial_age + born.elapsed().as_secs_f32();
        if age >= cue.duration_seconds {
            world.despawn(id);
            continue;
        }
        let point = position(world, &request, &frame);
        let Some(local) = frame.render.to_local(point) else {
            world.despawn(id);
            continue;
        };
        let mut entity = world.entity_mut(id);
        let mut transform = entity.get_mut::<Transform>().unwrap();
        transform.translation = Vec3::from_array(local.metres());
        let phase = age / cue.duration_seconds;
        transform.scale = Vec3::splat(cue.radius_metres * (1. + phase) * (1. - phase));
        metrics.position_checks += 1;
        // Measure global/local round-trip precision, not a comparison of the
        // just-written transform with the same local value.
        let recovered = frame
            .render
            .to_global(LocalPosition::new(transform.translation.to_array()).unwrap())
            .unwrap()
            .metres();
        let error = recovered
            .into_iter()
            .zip(point.metres())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        metrics.max_position_error_metres = metrics.max_position_error_metres.max(error as f32);
        if revision != frame.revision {
            metrics.rebased_visual_samples += 1;
        }
    }
    let voices: Vec<_> = world
        .query::<(Entity, &Voice, Option<&SpatialAudioSink>)>()
        .iter(world)
        .map(|(e, v, s)| {
            (
                e,
                v.request.clone(),
                v.born,
                v.revision,
                v.initialized,
                s.is_some(),
            )
        })
        .collect();
    for (id, request, born, revision, initialized, has_sink) in voices {
        let cue = &definition.cues[request.style];
        if born.elapsed().as_secs_f32() > cue.sound.seconds() / cue.pitch + 0.25 {
            world.despawn(id);
            continue;
        }
        if has_sink && !initialized {
            world.entity_mut(id).get_mut::<Voice>().unwrap().initialized = true;
            metrics.spatial_initialized += 1;
            if cue.priority == Priority::Critical {
                metrics.critical_spatial_initialized += 1;
            }
        }
        if let Some(local) = frame.render.to_local(position(world, &request, &frame)) {
            world
                .entity_mut(id)
                .get_mut::<Transform>()
                .unwrap()
                .translation = Vec3::from_array(local.metres());
        }
        if revision != frame.revision {
            metrics.rebased_voice_samples += 1;
        }
    }
    let mut visuals = visual_leases(world, definition);
    let mut voices = voice_leases(world, definition);
    let cosmetics = std::env::var("THEIA_FEEDBACK_COSMETICS").as_deref() != Ok("0");
    while let Some(request) = pending.queue.pop_front() {
        let cue = &definition.cues[request.style];
        let priority = cue.priority;
        let age = now_tick.saturating_sub(request.tick) as f64 / hz;
        if age > definition.max_age_seconds {
            metrics.late += 1;
            continue;
        }
        if let Some(focus) = state.authority_focus() {
            let a = focus.metres();
            let b = request.position.metres();
            if a.into_iter()
                .zip(b)
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f64>()
                > definition.max_distance_metres.powi(2)
            {
                metrics.far += 1;
                continue;
            }
        }
        if priority == Priority::Cosmetic && !cosmetics {
            metrics.cosmetic_visual_drops += 1;
            metrics.cosmetic_voice_drops += 1;
            continue;
        }
        let Some(local) = frame.render.to_local(position(world, &request, &frame)) else {
            metrics.far += 1;
            continue;
        };
        metrics.last_cue = cue.id.as_str().into();
        if priority == Priority::Critical {
            let captions: Vec<_> = world
                .query_filtered::<Entity, With<Caption>>()
                .iter(world)
                .collect();
            for id in captions {
                **world.entity_mut(id).get_mut::<Text>().unwrap() = cue.label.clone();
            }
            world.resource_mut::<CaptionTime>().0 = Some(Instant::now());
        }
        if (age as f32) < cue.duration_seconds
            && reserve(
                world,
                priority,
                &mut visuals,
                definition.visual_limit,
                definition.reserved_visuals,
                &mut metrics,
                true,
            )
        {
            let entity = world
                .spawn((
                    Visual {
                        request: request.clone(),
                        born: Instant::now(),
                        initial_age: age as f32,
                        revision: frame.revision,
                    },
                    Mesh3d(assets.mesh.clone()),
                    bevy::light::NotShadowCaster,
                    bevy::light::NotShadowReceiver,
                    MeshMaterial3d(assets.materials[request.style].clone()),
                    Transform::from_translation(Vec3::from_array(local.metres()))
                        .with_scale(Vec3::splat(cue.radius_metres)),
                ))
                .id();
            visuals.push((entity, priority, request.serial));
            metrics.visual_spawns += 1;
            if priority == Priority::Critical {
                metrics.critical_visuals += 1;
                metrics.critical_visuals_under_cosmetics +=
                    u64::from(visuals.iter().any(|(_, p, _)| *p == Priority::Cosmetic));
            }
        }
        if reserve(
            world,
            priority,
            &mut voices,
            definition.voice_limit,
            definition.reserved_voices,
            &mut metrics,
            false,
        ) {
            let source = match cue.sound {
                Sound::Cast => assets.cast.clone(),
                Sound::Impact => assets.impact.clone(),
            };
            let entity = world
                .spawn((
                    Voice {
                        request: request.clone(),
                        born: Instant::now(),
                        revision: frame.revision,
                        initialized: false,
                    },
                    AudioPlayer::new(source),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Linear(cue.gain))
                        .with_speed(cue.pitch),
                    Transform::from_translation(Vec3::from_array(local.metres())),
                ))
                .id();
            voices.push((entity, priority, request.serial));
            metrics.voice_spawns += 1;
            if priority == Priority::Critical {
                metrics.critical_voices += 1;
                metrics.critical_voices_under_cosmetics +=
                    u64::from(voices.iter().any(|(_, p, _)| *p == Priority::Cosmetic));
            }
        }
    }
    metrics.max_visuals = metrics.max_visuals.max(visuals.len());
    metrics.max_voices = metrics.max_voices.max(voices.len());
    if world
        .resource::<CaptionTime>()
        .0
        .is_some_and(|time| time.elapsed().as_secs_f32() > 1.)
    {
        let captions: Vec<_> = world
            .query_filtered::<Entity, With<Caption>>()
            .iter(world)
            .collect();
        for id in captions {
            world.entity_mut(id).get_mut::<Text>().unwrap().0.clear();
        }
        world.resource_mut::<CaptionTime>().0 = None;
    }
    world.insert_resource(frame);
    world.insert_resource(pending);
    world.insert_resource(metrics);
}
pub(crate) fn critical_visible(world: &mut World) -> bool {
    let Some(assets) = world.get_resource::<Assets>().cloned() else {
        return false;
    };
    world
        .query::<(&Visual, &ViewVisibility)>()
        .iter(world)
        .any(|(v, shown)| {
            shown.get() && assets.definition.cues[v.request.style].priority == Priority::Critical
        })
}
/// Apply final camera/ear positions after all PostUpdate camera corrections.
/// This uses Bevy's spatial sink API, not a custom audio mixer.
fn sync_audio(
    listener: Query<(&SpatialListener, &GlobalTransform)>,
    voices: Query<(&Transform, &SpatialAudioSink), With<Voice>>,
    visuals: Query<(&Visual, &ViewVisibility)>,
    assets: Res<Assets>,
    mut metrics: ResMut<Metrics>,
) {
    metrics.critical_visible_samples += visuals
        .iter()
        .filter(|(visual, visible)| {
            visible.get()
                && assets.definition.cues[visual.request.style].priority == Priority::Critical
        })
        .count() as u64;
    let Ok((listener, global)) = listener.single() else {
        return;
    };
    for (transform, sink) in &voices {
        sink.set_emitter_position(transform.translation * AUDIO_SCALE);
        sink.set_ears_position(
            global.transform_point(listener.left_ear_offset) * AUDIO_SCALE,
            global.transform_point(listener.right_ear_offset) * AUDIO_SCALE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (theia_core::abilities::Catalog, Definition) {
        let catalog = theia_core::abilities::Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        let definition = Definition::from_json(
            include_bytes!("../../../content/gameplay/development-spell-feedback.json"),
            &catalog,
        )
        .unwrap();
        (catalog, definition)
    }
    #[test]
    fn cosmetic_saturation_cannot_evict_critical_leases_and_critical_admission_reclaims_cosmetic_capacity()
     {
        for (limit, reserved) in [(64, 16), (32, 8)] {
            let mut active = Vec::new();
            for serial in 0..1000 {
                if admit(Priority::Cosmetic, &active, limit, reserved) == Admission::Allow {
                    active.push((serial, Priority::Cosmetic, serial));
                }
            }
            assert_eq!(active.len(), limit - reserved);
            for serial in 1000..1000 + limit as u64 {
                match admit(Priority::Critical, &active, limit, reserved) {
                    Admission::Allow => {}
                    Admission::Evict(id) => {
                        assert_eq!(
                            active.iter().find(|(old, _, _)| *old == id).unwrap().1,
                            Priority::Cosmetic
                        );
                        active.retain(|(old, _, _)| *old != id);
                    }
                    Admission::Drop => panic!("Cosmetic load denied critical admission"),
                }
                active.push((serial, Priority::Critical, serial));
                assert!(active.len() <= limit);
            }
            assert!(active.iter().all(|(_, p, _)| *p == Priority::Critical));
            assert_eq!(
                admit(Priority::Cosmetic, &active, limit, reserved),
                Admission::Drop
            );
            assert_eq!(
                admit(Priority::Critical, &active, limit, reserved),
                Admission::Drop,
                "Critical-only overflow must be explicit"
            );
        }
    }
    #[test]
    fn real_pending_cue_queue_retains_damage_under_cosmetic_flood_and_reports_critical_only_overflow()
     {
        let (catalog, definition) = fixture();
        let mut pending = Pending {
            definition: Arc::new(definition),
            queue: VecDeque::new(),
            serial: 0,
        };
        let mut metrics = Metrics::default();
        let key = EntityKey {
            epoch: theia_core::identity::WorldEpoch::new(1).unwrap(),
            entity: theia_core::identity::EntityId::new(1).unwrap(),
        };
        let point = GlobalPosition::new([8064., 1., 8000.]).unwrap();
        let cosmetic = Event::Started {
            actor: key,
            cast: theia_network::simulation::combat::Cast {
                sequence: 1,
                ability: catalog.abilities()[0].id().clone(),
                target: key,
                started: 0,
                completes: 30,
            },
            position: point,
        };
        for _ in 0..1000 {
            pending.offer(&cosmetic, 0, &mut metrics);
        }
        assert_eq!(pending.queue.len(), 48);
        assert_eq!(metrics.cosmetic_queue_drops, 952);
        let critical = Event::Damage {
            actor: key,
            sequence: 1,
            target: key,
            ability: catalog.abilities()[0].id().clone(),
            position: point,
            amount: 20,
            remaining: 80,
        };
        for _ in 0..64 {
            pending.offer(&critical, 0, &mut metrics);
        }
        assert_eq!(pending.queue.len(), 64);
        assert_eq!(metrics.critical_queue_overflow, 0);
        assert_eq!(metrics.cosmetic_evictions, 48);
        pending.offer(&critical, 0, &mut metrics);
        assert_eq!(metrics.critical_queue_overflow, 1);
    }
    #[test]
    fn original_pcm_assets_decode_through_bevys_standard_wav_decoder() {
        use bevy::audio::Source;
        for (bytes, seconds) in [
            (
                include_bytes!("../../../content/gameplay/audio/cast.wav").as_slice(),
                0.12,
            ),
            (
                include_bytes!("../../../content/gameplay/audio/impact.wav").as_slice(),
                0.18,
            ),
        ] {
            let source = AudioSource {
                bytes: Arc::from(bytes),
            };
            let decoder = source.decoder();
            assert_eq!(decoder.channels().get(), 1);
            assert_eq!(decoder.sample_rate().get(), 16000);
            assert!((decoder.total_duration().unwrap().as_secs_f64() - seconds).abs() < 0.0001);
            let samples: Vec<_> = decoder.collect();
            assert!(samples.iter().all(|v| v.is_finite() && v.abs() < 0.2));
            assert!(samples.iter().any(|v| v.abs() > 0.01));
        }
    }
    #[test]
    fn actual_feedback_data_rejects_unknown_mappings_and_unbounded_or_gameplay_fields() {
        let (catalog, definition) = fixture();
        assert_eq!(definition.cues.len(), 4);
        let original: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/gameplay/development-spell-feedback.json"
        ))
        .unwrap();
        let mut cases = Vec::new();
        for (field, value) in [
            ("visual_limit", 65),
            ("reserved_visuals", 0),
            ("voice_limit", 33),
            ("reserved_voices", 33),
            ("max_age_ms", 10000),
        ] {
            let mut bad = original.clone();
            bad[field] = value.into();
            cases.push(bad);
        }
        let mut bad = original.clone();
        bad["cues"][0]["damage"] = 100000.into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][0]["gain"] = 0.31.into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][0]["colour"][3] = 0.into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][0]["radius_metres"] = 5.into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][0]["duration_ms"] = 0.into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][0]["ability"] = "theia.unknown".into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][2]["ability"] = "theia.dev.arcane-bolt".into();
        cases.push(bad);
        let mut bad = original.clone();
        bad["cues"][1] = bad["cues"][0].clone();
        cases.push(bad);
        for bad in cases {
            assert!(Definition::from_json(&serde_json::to_vec(&bad).unwrap(), &catalog).is_err());
        }
        assert!(Definition::from_json(&vec![b' '; data::MAX_BYTES + 1], &catalog).is_err());
    }
}
