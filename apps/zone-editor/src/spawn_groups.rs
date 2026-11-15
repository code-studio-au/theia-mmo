//! Population draft controls and selected polygon preview; no runtime spawning.
use super::*;
use theia_zone::spawn_groups::SpawnGroup;

pub(super) struct Context {
    pub path: PathBuf,
    pub generation: u64,
    definition: theia_network::simulation::creatures::Definition,
}
impl Context {
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"path":self.path,"id":self.definition.id().as_str()})
    }
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        use std::io::Read;
        use theia_network::simulation::creatures;
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .map_err(|e| e.to_string())?
            .take(creatures::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        let definition = creatures::Definition::from_json(&bytes).map_err(|e| e.to_string())?;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Ok(Self {
            path,
            definition,
            generation: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        })
    }
}
fn loaded_definition(
    session: &Session,
) -> Option<&theia_network::simulation::creatures::Definition> {
    session
        .creature_context
        .as_ref()
        .map(|c| &c.definition)
        .or_else(|| session.quest_context.as_ref().map(|c| c.creature()))
}

pub(super) fn can_bind_creature(session: &Session) -> bool {
    session.play.is_none()
        && loaded_definition(session).is_some_and(|definition| {
            session.editor.zone().spawn_groups.iter().any(|g| {
                Some(g.id) == session.spawn_group_selected && g.creature != definition.id().as_str()
            })
        })
}
pub(super) fn bind_creature(session: &mut Session) {
    if !can_bind_creature(session) {
        return;
    }
    let creature = loaded_definition(session).unwrap().id().as_str().to_owned();
    let mut group = session
        .editor
        .zone()
        .spawn_groups
        .iter()
        .find(|g| Some(g.id) == session.spawn_group_selected)
        .unwrap()
        .clone();
    group.creature = creature;
    session.edit(vec![Edit::PutSpawnGroup { group }]);
}
pub(super) fn inspection(session: &Session) -> String {
    let mut report = inspect_binding(
        session.editor.zone(),
        session.spawn_group_selected,
        loaded_definition(session),
    );
    if let Some(context) = &session.creature_context {
        report.push_str(&format!("\nCreature file: {}", context.path.display()));
    }
    report
}
fn inspect_binding(
    zone: &Zone,
    id: Option<u64>,
    definition: Option<&theia_network::simulation::creatures::Definition>,
) -> String {
    let Some(id) = id else {
        return String::new();
    };
    let travel = zone
        .spawn_groups
        .iter()
        .find(|g| g.id == id)
        .and_then(|g| g.patrol_path)
        .and_then(|id| zone.paths.iter().find(|p| p.id == id))
        .map_or_else(String::new, |path| {
            format!(
                "\nPatrol: {} (#{}) · out and back\n1.5 m/s · pauses 2–6 s · checked at export",
                path.name, path.id
            )
        });
    let Some(definition) = definition else {
        return format!(
            "Load gameplay to inspect its creature, or load a creature definition directly{travel}"
        );
    };
    let mut report = format!("Loaded creature: {}", definition.id().as_str());
    match theia_network::simulation::population::authoring::binding(zone, id, definition.clone()) {
        Ok(binding) => report.push_str(&format!("\n{} definition resolved · respawn {} s\nPlacement still needs world/navigation checks", if binding.definition().aggro().is_some() { "Pursuing" } else { "Passive" }, binding.respawn_ms() / 1000)),
        Err(error) => report.push_str(&format!("\nPopulation: {error}")),
    }
    report.push_str(&travel);
    report
}

pub(super) fn bounds(group: &SpawnGroup) -> ([f64; 3], [f64; 3]) {
    let points = group.volume.polygon();
    let min = [
        points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
        group.volume.min_y(),
        points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
    ];
    let max = [
        points
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max),
        group.volume.max_y(),
        points
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max),
    ];
    (min, max)
}
pub(super) fn centre(group: &SpawnGroup) -> [f64; 3] {
    let (min, max) = bounds(group);
    std::array::from_fn(|i| min[i] + (max[i] - min[i]) * 0.5)
}
pub(super) fn add(session: &mut Session) {
    let id = session.next_id;
    if id == u64::MAX {
        session.status = "Authoring IDs exhausted".into();
        return;
    }
    let zone = session.editor.zone();
    let lower = std::array::from_fn::<_, 3, _>(|i| {
        zone.origin[i]
            + (zone.spawn[i] - zone.origin[i] - 8.)
                .clamp(-theia_zone::ZONE_EXTENT, theia_zone::ZONE_EXTENT - 16.)
    });
    let volume = match theia_core::spawn_volume::PolygonVolume::new(
        vec![
            [lower[0], lower[2]],
            [lower[0] + 16., lower[2]],
            [lower[0] + 16., lower[2] + 16.],
            [lower[0], lower[2] + 16.],
        ],
        lower[1],
        lower[1] + 16.,
    ) {
        Ok(volume) => volume,
        Err(error) => {
            session.status = error;
            return;
        }
    };
    let revision = session.editor.revision();
    session.edit(vec![Edit::PutSpawnGroup {
        group: SpawnGroup {
            id,
            name: format!("NPC spawn zone {id}"),
            enabled: true,
            creature: "theia.dev.training-creature".into(),
            model: None,
            volume,
            target_alive: 3,
            respawn_seconds: 30,
            roam_radius_metres: 4.,
            patrol_path: None,
            seed: 42,
        },
    }]);
    if session.editor.revision() != revision {
        session.set_mode(EditorMode::Objects);
        session.spawn_group_selected = Some(id);
        session.ui.properties_collapsed = false;
        session.status =
            "NPC spawn draft added · configure in Attributes; editor Play/export is not connected"
                .into();
    }
}
pub(super) fn toggle(session: &mut Session) {
    if let Some(original) = session
        .editor
        .zone()
        .spawn_groups
        .iter()
        .find(|g| Some(g.id) == session.spawn_group_selected)
    {
        let mut group = original.clone();
        group.enabled = !group.enabled;
        session.edit(vec![Edit::PutSpawnGroup { group }]);
    }
}
pub(super) fn boundary_enabled(session: &Session, control: workspace::Control) -> bool {
    use workspace::Control;
    let Some(group) = session
        .editor
        .zone()
        .spawn_groups
        .iter()
        .find(|g| Some(g.id) == session.spawn_group_selected)
    else {
        return false;
    };
    if session.play.is_some() {
        return false;
    }
    let selected = session
        .spawn_boundary_point
        .filter(|i| *i < group.volume.polygon().len());
    match control {
        Control::SpawnBoundaryInsert => {
            selected.is_some()
                && group.volume.polygon().len() < theia_core::spawn_volume::MAX_VERTICES
        }
        Control::SpawnBoundaryRemove => selected.is_some() && group.volume.polygon().len() > 3,
        _ => true,
    }
}
pub(super) fn boundary_action(session: &mut Session, control: workspace::Control) {
    use theia_zone::spawn_groups::BoundaryEdit;
    use workspace::Control;
    if !boundary_enabled(session, control) {
        return;
    }
    let group = session
        .editor
        .zone()
        .spawn_groups
        .iter()
        .find(|g| Some(g.id) == session.spawn_group_selected)
        .unwrap();
    let count = group.volume.polygon().len();
    let point = session.spawn_boundary_point.filter(|i| *i < count);
    let edit = match control {
        Control::SpawnBoundaryWhole => {
            session.spawn_boundary_point = None;
            return;
        }
        Control::SpawnBoundaryPoint(direction) => {
            let slot = point.map_or(0, |i| i + 1);
            let next = if direction < 0 {
                (slot + count) % (count + 1)
            } else {
                (slot + 1) % (count + 1)
            };
            session.spawn_boundary_point = next.checked_sub(1);
            return;
        }
        Control::SpawnBoundaryInsert => {
            let i = point.unwrap();
            let a = group.volume.polygon()[i];
            let b = group.volume.polygon()[(i + 1) % count];
            BoundaryEdit::Insert {
                after: i,
                position: std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * 0.5),
            }
        }
        Control::SpawnBoundaryRemove => BoundaryEdit::Remove {
            index: point.unwrap(),
        },
        _ => return,
    };
    let id = group.id;
    let revision = session.editor.revision();
    session.edit(vec![Edit::EditSpawnBoundary { id, edit }]);
    if session.editor.revision() != revision {
        session.spawn_boundary_point = Some(match control {
            Control::SpawnBoundaryInsert => point.unwrap() + 1,
            _ => point.unwrap().min(count - 2),
        });
    }
}
#[derive(Component)]
struct Edge(usize);
#[derive(Resource, Default)]
struct Scene {
    signature: Option<(u64, u64, Option<u64>, bool)>,
}
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_zone_systems(sync);
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1., 1., 1.));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1., 0.65, 0.1),
        unlit: true,
        ..default()
    });
    for i in 0..theia_core::spawn_volume::MAX_VERTICES * 3 {
        commands.spawn((
            Edge(i),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            Pickable::IGNORE,
            bevy::light::NotShadowCaster,
        ));
    }
    commands.init_resource::<Scene>();
}
fn edges(zone: &Zone, group: &SpawnGroup) -> Result<Vec<Transform>, String> {
    let polygon = group.volume.polygon();
    let low = polygon
        .iter()
        .map(|p| {
            zone.local([p[0], group.volume.min_y(), p[1]])
                .map(Vec3::from_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let high = polygon
        .iter()
        .map(|p| {
            zone.local([p[0], group.volume.max_y(), p[1]])
                .map(Vec3::from_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (min, max) = bounds(group);
    let width =
        ((0..3).map(|i| max[i] - min[i]).fold(0., f64::max) * 0.002).clamp(0.04, 0.25) as f32;
    let mut transforms = Vec::new();
    for i in 0..polygon.len() {
        let j = (i + 1) % polygon.len();
        for (a, b) in [(low[i], low[j]), (high[i], high[j]), (low[i], high[i])] {
            let delta = b - a;
            transforms.push(
                Transform::from_translation((a + b) * 0.5)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, delta.normalize()))
                    .with_scale(Vec3::new(width, delta.length(), width)),
            );
        }
    }
    Ok(transforms)
}
fn sync(
    session: Res<Session>,
    mut scene: ResMut<Scene>,
    mut preview: Query<(&Edge, &mut Transform, &mut Visibility)>,
) {
    let signature = (
        session.document_id,
        session.editor.revision(),
        session.spawn_group_selected,
        session.play.is_some(),
    );
    if scene.signature == Some(signature) {
        return;
    }
    let zone = session.editor.zone();
    let transforms = if session.play.is_none() {
        zone.spawn_groups
            .iter()
            .find(|g| Some(g.id) == session.spawn_group_selected)
            .and_then(|g| edges(zone, g).ok())
    } else {
        None
    };
    for (edge, mut transform, mut visibility) in &mut preview {
        if let Some(value) = transforms.as_ref().and_then(|v| v.get(edge.0)) {
            *transform = *value;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    scene.signature = Some(signature);
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Report<'w, 's> {
    edges: Query<'w, 's, &'static Visibility, With<Edge>>,
}
impl Report<'_, '_> {
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"visible_edges": self.edges.iter().filter(|v| **v == Visibility::Visible).count(), "pool_size": self.edges.iter().count()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standalone_assignment_uses_explicit_context_and_undo_preserves_the_loaded_type() {
        let mut session = session([0.; 3], 0.);
        add(&mut session);
        let mut group = session.editor.zone().spawn_groups[0].clone();
        group.creature = "theia.creature.unresolved".into();
        session.edit(vec![Edit::PutSpawnGroup { group }]);
        let original = session.editor.zone().clone();
        session.creature_context = Some(
            Context::load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content/gameplay/development-creature-passive.json"),
            )
            .unwrap(),
        );
        assert!(session.quest_context.is_none());
        // An explicitly chosen standalone type takes precedence over a later
        // quest template; the inspector shows its source file.
        session.quest_context = Some(
            quest_inspector::Context::load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content/development-gameplay-classic-quest-areas.json"),
            )
            .unwrap(),
        );
        assert!(can_bind_creature(&session));
        bind_creature(&mut session);
        assert!(!can_bind_creature(&session));
        assert!(inspection(&session).contains("Passive definition resolved"));
        assert!(inspection(&session).contains("Creature file:"));
        let mut expected = original.clone();
        expected.spawn_groups[0].creature = "theia.dev.training-creature".into();
        assert_eq!(session.editor.zone(), &expected);
        session.execute(Command::Undo {
            expected_revision: session.editor.revision(),
        });
        assert_eq!(session.editor.zone(), &original);
        assert!(can_bind_creature(&session));
    }
    #[test]
    fn loaded_creature_assignment_is_explicit_undoable_and_preserves_other_group_fields() {
        let mut session = session([0.; 3], 0.);
        add(&mut session);
        assert!(!can_bind_creature(&session));
        assert!(inspection(&session).contains("Load gameplay"));
        session.quest_context = Some(
            quest_inspector::Context::load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content/development-gameplay-classic-quest-areas.json"),
            )
            .unwrap(),
        );
        let mut group = session.editor.zone().spawn_groups[0].clone();
        group.creature = "theia.creature.unresolved".into();
        session.edit(vec![Edit::PutSpawnGroup {
            group: group.clone(),
        }]);
        assert!(can_bind_creature(&session));
        assert!(inspection(&session).contains("does not match"));
        let saved = session.saved_zone.clone();
        let mut view = View {
            target: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        };
        perform(Action::BindSpawnCreature, &mut session, &mut view);
        let expected = session
            .quest_context
            .as_ref()
            .unwrap()
            .creature()
            .id()
            .as_str();
        let mut changed = group.clone();
        changed.creature = expected.into();
        assert_eq!(session.editor.zone().spawn_groups[0], changed);
        assert!(!can_bind_creature(&session));
        assert!(inspection(&session).contains("Pursuing definition resolved"));
        assert_eq!(session.saved_zone, saved);
        session.execute(Command::Undo {
            expected_revision: session.editor.revision(),
        });
        assert_eq!(session.editor.zone().spawn_groups[0], group);
        session.spawn_group_selected = None;
        assert!(!can_bind_creature(&session));
    }
    #[test]
    fn population_inspection_reports_runtime_limits_without_changing_authoring() {
        let mut session = session([0.; 3], 0.);
        add(&mut session);
        let definition = theia_network::simulation::creatures::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-creature-patrol.json"
        ))
        .unwrap()
        .definition()
        .clone();
        let mut zone = session.editor.zone().clone();
        let id = session.spawn_group_selected;
        assert!(
            inspect_binding(&zone, id, Some(&definition)).contains("Passive definition resolved")
        );
        zone.spawn_groups[0].model = Some("theia.model.wolf".into());
        assert!(
            inspect_binding(&zone, id, Some(&definition))
                .contains("model binding is not connected")
        );
        zone.spawn_groups[0].model = None;
        zone.spawn_groups[0].target_alive = 17;
        assert!(inspect_binding(&zone, id, Some(&definition)).contains("16-target"));
        zone.spawn_groups[0].target_alive = 5;
        zone.spawn_groups[0].enabled = false;
        assert!(inspect_binding(&zone, id, Some(&definition)).contains("disabled"));
        assert!(session.editor.zone().spawn_groups[0].enabled);
        assert_eq!(session.editor.zone().spawn_groups[0].target_alive, 3);
    }
    fn session(origin: [f64; 3], offset: f64) -> Session {
        let zone = Zone {
            origin,
            spawn: origin.map(|v| v + offset),
            ..default()
        };
        Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone),
        )
    }
    #[test]
    fn add_toggle_copy_delete_and_undo_preserve_original_document_and_edge_spawns() {
        for offset in [-510., 0., 510.] {
            let mut session = session([128_000_000_000.; 3], offset);
            let source = session.editor.zone().clone();
            add(&mut session);
            let original = session.editor.zone().spawn_groups[0].clone();
            assert!(
                original
                    .volume
                    .contains(theia_core::coordinates::GlobalPosition::new(source.spawn).unwrap())
            );
            assert_eq!(session.spawn_group_selected, Some(original.id));
            toggle(&mut session);
            assert!(!session.editor.zone().spawn_groups[0].enabled);
            if offset == 510. {
                let before = session.editor.zone().clone();
                selection::duplicate(&mut session);
                assert_eq!(session.editor.zone(), &before);
                session.edit(vec![Edit::Translate {
                    id: original.id,
                    offset: [-1., 0., -1.],
                }]);
            }
            selection::duplicate(&mut session);
            assert_eq!(session.editor.zone().spawn_groups.len(), 2);
            let copy = session.spawn_group_selected.unwrap();
            assert_ne!(copy, original.id);
            session.edit(vec![Edit::DeleteSpawnGroup { id: copy }]);
            assert_eq!(session.spawn_group_selected, None);
            let revision = session.editor.revision();
            session.execute(Command::Undo {
                expected_revision: revision,
            });
            assert_eq!(session.editor.zone().spawn_groups.len(), 2);
            assert_eq!(session.saved_zone.as_ref(), Some(&source));
        }
        let mut edge = session([theia_core::coordinates::MAX_GLOBAL_METRES; 3], 0.);
        add(&mut edge);
        assert!(edge.editor.zone().spawn_groups.is_empty());
    }
    #[test]
    fn boundary_controls_wrap_close_edges_and_refuse_invalid_topology() {
        use workspace::Control;
        let mut session = session([0.; 3], 0.);
        add(&mut session);
        let original = session.editor.zone().clone();
        boundary_action(&mut session, Control::SpawnBoundaryPoint(-1));
        assert_eq!(session.spawn_boundary_point, Some(3));
        boundary_action(&mut session, Control::SpawnBoundaryInsert);
        assert_eq!(session.spawn_boundary_point, Some(4));
        assert_eq!(
            session.editor.zone().spawn_groups[0].volume.polygon()[4],
            [-8., 0.]
        );
        boundary_action(&mut session, Control::SpawnBoundaryPoint(1));
        assert_eq!(session.spawn_boundary_point, None);
        assert!(!boundary_enabled(&session, Control::SpawnBoundaryRemove));
        boundary_action(&mut session, Control::SpawnBoundaryPoint(-1));
        boundary_action(&mut session, Control::SpawnBoundaryRemove);
        assert_eq!(session.editor.zone(), &original);
        boundary_action(&mut session, Control::SpawnBoundaryRemove);
        assert_eq!(
            session.editor.zone().spawn_groups[0].volume.polygon().len(),
            3
        );
        assert!(!boundary_enabled(&session, Control::SpawnBoundaryRemove));
        let before = session.editor.zone().clone();
        boundary_action(&mut session, Control::SpawnBoundaryRemove);
        assert_eq!(session.editor.zone(), &before);
        let mut group = before.spawn_groups[0].clone();
        group.volume = theia_core::spawn_volume::PolygonVolume::new(
            vec![[0., 0.], [0.25, 0.], [0., 1.]],
            -2.,
            2.,
        )
        .unwrap();
        session.edit(vec![Edit::PutSpawnGroup { group }]);
        session.spawn_boundary_point = Some(0);
        let before = session.editor.zone().clone();
        let revision = session.editor.revision();
        boundary_action(&mut session, Control::SpawnBoundaryInsert);
        assert_eq!(session.editor.zone(), &before);
        assert_eq!(session.editor.revision(), revision);
        assert_eq!(session.spawn_boundary_point, Some(0));
    }
    #[test]
    fn polygon_preview_uses_bounded_shared_assets_no_collision_and_hides_with_selection() {
        let mut session = session([128_000_000_000.125; 3], 0.);
        add(&mut session);
        let original = session.editor.zone().clone();
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(session)
            .add_plugins(plugin);
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<avian3d::prelude::Collider>>()
                .iter(app.world())
                .count(),
            0
        );
        let mut query = app.world_mut().query::<(&Edge, &Transform, &Visibility)>();
        assert_eq!(
            query
                .iter(app.world())
                .filter(|(_, _, v)| **v == Visibility::Visible)
                .count(),
            12
        );
        for (_, t, v) in query.iter(app.world()) {
            if *v == Visibility::Visible {
                assert!(t.translation.abs().max_element() <= 8.);
                assert_eq!(t.scale.y, 16.);
            }
        }
        app.world_mut()
            .resource_mut::<Session>()
            .spawn_group_selected = None;
        app.update();
        assert!(
            query
                .iter(app.world())
                .all(|(_, _, v)| *v == Visibility::Hidden)
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &original);
    }
}
