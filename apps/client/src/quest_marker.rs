//! One bounded guide marker derived only from the owner's committed journal.
//! This presentation never grants interaction entitlement or mutates progress.
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::{prelude::*, text::FontSize};
use theia_core::{coordinates::GlobalPosition, quests::lifecycle::Phase};

const MAX_DISTANCE: f64 = 60.;
const WIDTH: f32 = 224.;
const HEIGHT: f32 = 68.;

#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub phase_frames: [u64; 3],
    pub visible_frames: [u64; 3],
    pub renderable_frames: [u64; 3],
    pub phase_now: Option<usize>,
    pub occluded_frames: u64,
    pub projected_frames: u64,
    pub visible_now: bool,
    pub last_font_px: f32,
}
#[derive(Component)]
struct Marker;

pub(crate) fn install(app: &mut App) {
    app.init_resource::<Metrics>()
        .add_systems(Startup, setup)
        .add_systems(Last, observe)
        .add_systems(
            PostUpdate,
            present
                .after(crate::camera::sync_camera_global)
                .after(avian3d::prelude::PhysicsSystems::StepSimulation)
                .before(bevy::ui::UiSystems::Layout),
        );
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Marker,
        Text::new(""),
        TextFont::default(),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            width: px(WIDTH),
            height: px(HEIGHT),
            ..default()
        },
        Visibility::Hidden,
        bevy::picking::Pickable::IGNORE,
    ));
}
fn appearance(phase: Phase) -> Option<(usize, &'static str, Color)> {
    match phase {
        Phase::Available => Some((0, "!\nQuest available", Color::srgb(1., 0.83, 0.15))),
        Phase::Active => Some((1, "...\nQuest in progress", Color::srgb(0.8, 0.85, 0.9))),
        Phase::Ready => Some((2, "?\nTurn in quest", Color::srgb(1., 0.83, 0.15))),
        Phase::TurnedIn => None,
    }
}
/// Refuse disconnected, foreign-owner, unknown/unloaded guide and stale profiles.
fn observed_context(
    state: &theia_network::client::State,
    actor: theia_core::identity::EntityKey,
) -> Option<(GlobalPosition, f64)> {
    if state.error.is_some() {
        return None;
    }
    let welcome = state.welcome.as_ref()?;
    let presence = state.guide.as_ref()?;
    let guide = presence.guide?;
    if actor != presence.actor
        || actor.epoch.value() != welcome.epoch
        || Some(actor.entity.value()) != state.id
        || guide.key.epoch != actor.epoch
        || guide.key.entity == actor.entity
    {
        return None;
    }
    let chunk = guide.position.chunk(128)?;
    let report = state.chunk_report.as_ref()?;
    if report.epoch != welcome.epoch
        || !report.valid()
        || !report.chunks.contains(&[chunk.x, chunk.z])
    {
        return None;
    }
    let player = state.authority_focus()?.metres();
    let position = guide.position.metres();
    let distance = player
        .into_iter()
        .zip(position)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    if distance > MAX_DISTANCE {
        return None;
    }
    Some((guide.position, distance))
}
fn observed(
    state: &theia_network::client::State,
    profile: &theia_network::quest_lifecycle_wire::Profile,
) -> Option<(Phase, GlobalPosition, f64)> {
    let journal = state
        .quest_lifecycle
        .as_ref()
        .filter(|j| j.profile_hash == profile.hash())?;
    let (position, distance) = observed_context(state, journal.actor)?;
    let phase = journal.progress.progress(profile.profile()).ok()?.phase;
    Some((phase, position, distance))
}
fn observed_journal(
    state: &theia_network::client::State,
    profile: &theia_network::quest_journal::Profile,
) -> Option<(Phase, GlobalPosition, f64)> {
    use theia_core::quests::journal::Status;
    let journal = state
        .quest_journal
        .as_ref()
        .filter(|j| j.profile_hash == profile.hash())?;
    journal
        .validate_successor(profile, journal.actor, None)
        .ok()?;
    let (position, distance) = observed_context(state, journal.actor)?;
    let ready = profile.catalog().quests().any(|q| {
        journal
            .journal
            .progress(q.id())
            .is_some_and(|p| p.status() == Status::Ready)
    });
    let available = profile.catalog().quests().any(|q| {
        journal.journal.progress(q.id()).is_none()
            && q.prerequisites().iter().all(|id| {
                journal
                    .journal
                    .progress(id)
                    .is_some_and(|p| p.status() == Status::Completed)
            })
    });
    let active = journal
        .journal
        .quests()
        .any(|(_, p)| p.status() == Status::Active);
    let phase = if ready {
        Phase::Ready
    } else if available {
        Phase::Available
    } else if active {
        Phase::Active
    } else {
        Phase::TurnedIn
    };
    Some((phase, position, distance))
}

/// Share the same committed owner/guide/prerequisite checks with the local map.
pub(crate) fn guide_observation(
    state: &theia_network::client::State,
    lifecycle: Option<&theia_network::quest_lifecycle_wire::Profile>,
    journal: Option<&theia_network::quest_journal::Profile>,
) -> Option<(Phase, GlobalPosition, f64)> {
    match (lifecycle, journal) {
        (Some(profile), None) => observed(state, profile),
        (None, Some(profile)) => observed_journal(state, profile),
        _ => None,
    }
}

fn anchor(position: GlobalPosition, frame: &crate::floating_origin::Frame) -> Option<Vec3> {
    let mut p = position.metres();
    p[1] += 1.9;
    Some(Vec3::from_array(
        frame.render.to_local(GlobalPosition::new(p)?)?.metres(),
    ))
}
fn font_size(distance: f64) -> f32 {
    (25. - distance as f32 * 0.18).round().clamp(15., 24.)
}
fn occluded(spatial: &SpatialQuery, eye: Vec3, point: Vec3) -> bool {
    let ray = point - eye;
    let Ok(direction) = Dir3::new(ray) else {
        return true;
    };
    spatial
        .cast_ray(
            eye,
            direction,
            (ray.length() - 0.05).max(0.),
            true,
            &SpatialQueryFilter::from_mask(
                theia_network::physics::SOLID | theia_network::physics::GROUND,
            ),
        )
        .is_some()
}
#[allow(clippy::too_many_arguments)]
fn present(
    network: Res<crate::network::Network>,
    definitions: Option<Res<crate::quest_lifecycle::Definitions>>,
    journal: Option<Res<crate::quest_journal::Definitions>>,
    frame: Res<crate::floating_origin::Frame>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial: SpatialQuery,
    mut labels: Query<
        (
            &mut Text,
            &mut TextFont,
            &mut TextColor,
            &mut Node,
            &mut Visibility,
        ),
        With<Marker>,
    >,
    mut metrics: ResMut<Metrics>,
) {
    metrics.visible_now = false;
    metrics.phase_now = None;
    let Ok((mut text, mut font, mut color, mut node, mut visibility)) = labels.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    let state = network.state();
    let observed = guide_observation(
        &state,
        definitions.as_ref().map(|d| d.0.as_ref()),
        journal.as_ref().map(|d| d.0.as_ref()),
    );
    let Some((phase, guide, distance)) = observed else {
        return;
    };
    let Some((index, caption, tint)) = appearance(phase) else {
        return;
    };
    metrics.phase_frames[index] += 1;
    let (Some(point), Ok((camera, transform))) = (anchor(guide, &frame), cameras.single()) else {
        return;
    };
    let Ok(screen) = camera.world_to_viewport(transform, point) else {
        return;
    };
    let Some(viewport) = camera.logical_viewport_size() else {
        return;
    };
    if screen.x < WIDTH * 0.5
        || screen.x > viewport.x - WIDTH * 0.5
        || screen.y < HEIGHT
        || screen.y > viewport.y
    {
        return;
    }
    let eye = transform.translation();
    if occluded(&spatial, eye, point) {
        metrics.occluded_frames += 1;
        return;
    }
    if text.0 != caption {
        text.0 = caption.to_owned();
    }
    if color.0 != tint {
        color.0 = tint;
    }
    let size = font_size(distance);
    if !matches!(font.font_size, FontSize::Px(current) if current == size) {
        font.font_size = FontSize::Px(size);
    }
    metrics.last_font_px = size;
    node.left = px(screen.x - WIDTH * 0.5);
    node.top = px(screen.y - HEIGHT);
    *visibility = Visibility::Inherited;
    metrics.visible_now = true;
    metrics.phase_now = Some(index);
    metrics.projected_frames += 1;
    metrics.visible_frames[index] += 1;
}

fn observe(
    labels: Query<
        (
            &ComputedNode,
            &InheritedVisibility,
            Option<&bevy::text::TextLayoutInfo>,
        ),
        With<Marker>,
    >,
    mut metrics: ResMut<Metrics>,
) {
    if let Some(index) = metrics.phase_now
        && let Ok((node, visible, glyphs)) = labels.single()
        && visible.get()
        && node.size().min_element() > 0.
        && glyphs.is_some_and(|g| !g.glyphs.is_empty())
    {
        metrics.renderable_frames[index] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::coordinates::LocalFrame;
    use theia_core::identity::{EntityId, EntityKey, WorldEpoch};
    use theia_network::{client::State, quest_lifecycle_wire::Profile, wire};
    #[test]
    fn actual_avian_occlusion_hides_walls_and_terrain_without_character_blocking() {
        use avian3d::prelude::Collider;
        use bevy::{ecs::system::SystemState, time::TimeUpdateStrategy};
        use std::time::Duration;
        use theia_network::physics::{GROUND, PlacedCollider, SOLID, StaticScene};
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            crate::physics::WorldPhysicsPlugin,
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.finish();
        app.cleanup();
        app.update();
        for (layer, blocked) in [(SOLID, true), (GROUND, true), (0, false)] {
            let scene = StaticScene::new(vec![PlacedCollider {
                shape: Collider::cuboid(2., 4., 2.),
                center: Vec3::new(0., 2., 5.),
                layer,
            }]);
            crate::physics::replace_scene(app.world_mut(), &scene, u64::from(layer) + 1);
            app.update();
            let mut system = SystemState::<SpatialQuery>::new(app.world_mut());
            let spatial = system.get(app.world()).unwrap();
            assert_eq!(
                occluded(&spatial, Vec3::new(0., 2., 0.), Vec3::new(0., 2., 10.)),
                blocked
            );
            assert!(!occluded(
                &spatial,
                Vec3::new(4., 2., 0.),
                Vec3::new(4., 2., 10.)
            ));
            // Geometry behind the marker does not occlude it.
            assert!(!occluded(
                &spatial,
                Vec3::new(0., 2., 0.),
                Vec3::new(0., 2., 2.)
            ));
        }
    }
    fn fixture() -> (State, Profile) {
        let profile = Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-quest.json"
        ))
        .unwrap();
        let actor = EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(3).unwrap(),
        };
        let mut state = State {
            id: Some(3),
            welcome: Some(wire::Welcome {
                terrain_stream: true,
                id: 3,
                epoch: 7,
                world: [1; 32],
                origin: [0.; 3],
            }),
            snapshot: Some(wire::Snapshot {
                version: wire::VERSION,
                epoch: 7,
                tick: 1,
                tick_hz: 30,
                origin: [0.; 3],
                players: vec![wire::Player {
                    facing: None,
                    id: 3,
                    xz: [0., 0.],
                    y: 0.9,
                    applied_sequence: 0,
                }],
            }),
            quest_lifecycle: Some(theia_network::quest_lifecycle_wire::State {
                actor,
                profile_hash: profile.hash(),
                progress: theia_core::quests::lifecycle::State::available(profile.profile()),
            }),
            guide: Some(theia_network::dialogue_wire::Presence {
                actor,
                profile_hash: [1; 32],
                guide: Some(theia_network::dialogue::Guide {
                    key: EntityKey {
                        entity: EntityId::new(2).unwrap(),
                        ..actor
                    },
                    position: GlobalPosition::new([4., 0.9, 0.]).unwrap(),
                }),
            }),
            ..default()
        };
        state.chunk_report = Some(wire::ChunkReady {
            version: wire::VERSION,
            epoch: 7,
            sequence: 1,
            chunks: vec![[0, 0]],
        });
        (state, profile)
    }
    #[test]
    fn marker_follows_committed_progress_including_reconstructed_rejoin_state() {
        use theia_core::quests::lifecycle::{State as Journal, Status};
        let (mut state, profile) = fixture();
        assert_eq!(observed(&state, &profile).unwrap().0, Phase::Available);
        for (revision, status, count, phase) in [
            (1, Status::Accepted, 0, Phase::Active),
            (3, Status::Accepted, 2, Phase::Active),
            (4, Status::Accepted, 3, Phase::Ready),
            (5, Status::TurnedIn, 3, Phase::TurnedIn),
        ] {
            state.quest_lifecycle.as_mut().unwrap().progress =
                Journal::restore(profile.profile(), revision, status, count).unwrap();
            assert_eq!(observed(&state, &profile).unwrap().0, phase);
        }
        // Sending an action has no optimistic effect on the observed phase.
        state.quest_lifecycle_pending = true;
        assert_eq!(observed(&state, &profile).unwrap().0, Phase::TurnedIn);
    }
    #[test]
    fn unknown_disconnected_unloaded_foreign_and_distant_guides_have_no_marker() {
        let (state, profile) = fixture();
        let mut cases = Vec::new();
        let mut s = state.clone();
        s.error = Some("Disconnected".into());
        cases.push(s);
        let mut s = state.clone();
        s.guide.as_mut().unwrap().guide = None;
        cases.push(s);
        let mut s = state.clone();
        s.quest_lifecycle = None;
        cases.push(s);
        let mut s = state.clone();
        s.chunk_report.as_mut().unwrap().chunks.clear();
        cases.push(s);
        let mut s = state.clone();
        s.chunk_report.as_mut().unwrap().epoch = 8;
        cases.push(s);
        let mut s = state.clone();
        s.guide.as_mut().unwrap().actor.entity = EntityId::new(4).unwrap();
        cases.push(s);
        let mut s = state.clone();
        s.quest_lifecycle.as_mut().unwrap().profile_hash = [5; 32];
        cases.push(s);
        let mut s = state.clone();
        s.welcome.as_mut().unwrap().epoch = 8;
        cases.push(s);
        let mut s = state.clone();
        s.guide.as_mut().unwrap().guide.as_mut().unwrap().key.epoch = WorldEpoch::new(8).unwrap();
        cases.push(s);
        let mut s = state.clone();
        s.guide.as_mut().unwrap().guide.as_mut().unwrap().position =
            GlobalPosition::new([60.01, 0.9, 0.]).unwrap();
        cases.push(s);
        for (index, s) in cases.iter().enumerate() {
            assert!(observed(s, &profile).is_none(), "case {index}");
            assert!(
                guide_observation(s, Some(&profile), None).is_none(),
                "map case {index}"
            );
        }
        let mut s = state;
        s.guide.as_mut().unwrap().guide.as_mut().unwrap().position =
            GlobalPosition::new([60., 0.9, 0.]).unwrap();
        assert!(observed(&s, &profile).is_some());
        assert!(guide_observation(&s, Some(&profile), None).is_some());
        assert!(guide_observation(&s, None, None).is_none());
    }
    #[test]
    fn phase_shape_and_text_distinguish_progress_and_completion() {
        assert!(appearance(Phase::Available).unwrap().1.starts_with('!'));
        assert!(appearance(Phase::Active).unwrap().1.starts_with("..."));
        assert!(appearance(Phase::Ready).unwrap().1.starts_with('?'));
        assert!(appearance(Phase::TurnedIn).is_none());
        assert_eq!(font_size(0.), 24.);
        assert_eq!(font_size(60.), 15.);
    }
    #[test]
    fn anchor_preserves_large_global_coordinates_across_render_rebases() {
        let p = GlobalPosition::new([128_000_000_123.25, 20., -128_000_000_012.5]).unwrap();
        let mut frame = crate::floating_origin::Frame::default();
        assert!(anchor(p, &frame).is_none());
        frame.render = LocalFrame::new(
            GlobalPosition::new([128_000_000_000., 0., -128_000_000_000.]).unwrap(),
        );
        let first = anchor(p, &frame).unwrap();
        frame.render = LocalFrame::new(
            GlobalPosition::new([128_000_000_128., 0., -128_000_000_128.]).unwrap(),
        );
        let second = anchor(p, &frame).unwrap();
        assert!((first - second - Vec3::new(128., 0., -128.)).length() < 0.0001);
        assert!((first.y - 21.9).abs() < 0.00001);
    }
    #[test]
    fn expanded_marker_prioritizes_ready_then_available_and_uses_committed_history() {
        use crate::quest_journal::tests::{completed_introduction, fixture, set_journal};
        use theia_core::quests::{catalog::EventKey, journal::Status};
        let (mut state, profile) = fixture();
        let id = |s: &str| theia_core::content::ContentId::new(s).unwrap();
        assert_eq!(
            observed_journal(&state, &profile).unwrap().0,
            Phase::Available
        );
        state.quest_journal_pending = true;
        assert_eq!(
            observed_journal(&state, &profile).unwrap().0,
            Phase::Available,
            "Pending acceptance is not committed progress"
        );
        state.quest_journal_pending = false;
        let mut journal = completed_introduction(&profile);
        set_journal(&mut state, journal.clone());
        assert_eq!(
            observed_journal(&state, &profile).unwrap().0,
            Phase::Available,
            "Completed first quest unlocks the next offer"
        );
        let quest = id("theia.dev.quest-follow-up");
        let npc = id("theia.dev.guide");
        journal = journal
            .propose_accept(profile.catalog(), &quest, &npc)
            .unwrap()
            .journal;
        set_journal(&mut state, journal.clone());
        assert_eq!(observed_journal(&state, &profile).unwrap().0, Phase::Active);
        for _ in 0..3 {
            journal = journal
                .propose_trusted_event(
                    profile.catalog(),
                    &EventKey::Kill(id("theia.dev.training-creature")),
                )
                .unwrap()
                .journal;
        }
        journal = journal
            .propose_trusted_event(
                profile.catalog(),
                &EventKey::NpcInteraction {
                    npc: npc.clone(),
                    interaction: id("theia.dev.interaction.training"),
                },
            )
            .unwrap()
            .journal;
        set_journal(&mut state, journal.clone());
        assert_eq!(observed_journal(&state, &profile).unwrap().0, Phase::Ready);
        journal = journal
            .propose_turn_in(profile.catalog(), &quest, &npc, 5)
            .unwrap()
            .journal;
        set_journal(&mut state, journal);
        assert_eq!(
            state
                .quest_journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&quest)
                .unwrap()
                .status(),
            Status::Completed
        );
        assert_eq!(
            observed_journal(&state, &profile).unwrap().0,
            Phase::TurnedIn
        );
        state.quest_journal.as_mut().unwrap().profile_hash = [9; 32];
        assert!(observed_journal(&state, &profile).is_none());
    }
}
