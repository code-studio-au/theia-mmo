//! Nonblocking test-pawn authoring badges; actual players run in separate clients.
use super::*;
use std::collections::BTreeMap;
use theia_zone::test_players::{NetworkConditions, TestPlayer};

#[derive(Component)]
pub(super) struct Badge(pub u64);

pub(super) fn menu(parent: &mut ChildSpawnerCommands) {
    for (action, label) in [
        (Action::TestPlayerEnabled, "Toggle enabled client"),
        (Action::TestPlayerController(false), "Controller: Manual"),
        (
            Action::TestPlayerController(true),
            "Controller: Walk and turn",
        ),
        (Action::TestPlayerRoute(1), "Controller: Next patrol path"),
        (
            Action::TestPlayerRoute(-1),
            "Controller: Previous patrol path",
        ),
    ] {
        menus::spawn_items(parent, &[(workspace::Control::Command(action), label, "")]);
    }
}
pub(super) fn configure(action: Action, session: &mut Session) {
    let Some(mut player) = session
        .editor
        .zone()
        .test_players
        .iter()
        .find(|p| Some(p.id) == session.test_player_selected)
        .cloned()
    else {
        return;
    };
    match action {
        Action::TestPlayerEnabled => player.enabled = !player.enabled,
        Action::TestPlayerController(script) => {
            player.controller = script.then(|| "test.walk-turn".into())
        }
        Action::TestPlayerRoute(direction) => {
            let paths: Vec<_> = session
                .editor
                .zone()
                .paths
                .iter()
                .filter_map(|path| path.patrol_route_id())
                .collect();
            if paths.is_empty() {
                session.status = "Create a patrol path before assigning test travel".into();
                return;
            }
            let index = paths
                .iter()
                .position(|id| Some(id) == player.controller.as_ref());
            let next = index.map_or(if direction < 0 { paths.len() - 1 } else { 0 }, |index| {
                (index as isize + isize::from(direction.signum())).rem_euclid(paths.len() as isize)
                    as usize
            });
            player.controller = Some(paths[next].clone());
        }
        _ => return,
    }
    session.edit(vec![Edit::PutTestPlayer { player }]);
}
#[derive(Resource, Default)]
pub(super) struct Scene {
    mesh: Option<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
    entities: BTreeMap<u64, Entity>,
}
pub(super) fn add(session: &mut Session) {
    let Some(slot) = (1..=theia_zone::test_players::MAX_TEST_PLAYERS as u8).find(|slot| {
        session
            .editor
            .zone()
            .test_players
            .iter()
            .all(|p| p.client_id != *slot)
    }) else {
        session.status = "All four test player slots are occupied".into();
        return;
    };
    let id = session.next_id;
    if id == u64::MAX {
        session.status = "Authoring IDs exhausted".into();
        return;
    }
    let mut position = session.editor.zone().spawn;
    position[0] -= f64::from(slot - 1) * 2.;
    let revision = session.editor.revision();
    session.edit(vec![Edit::PutTestPlayer {
        player: TestPlayer {
            id,
            name: format!("Test player {slot}"),
            client_id: slot,
            enabled: true,
            position,
            controller: None,
            network: NetworkConditions::default(),
        },
    }]);
    if session.editor.revision() != revision {
        session.set_mode(EditorMode::Objects);
        session.test_player_selected = Some(id);
        session.gizmo_tool = gizmo::Tool::Move;
        session.ui.properties_collapsed = false;
        session.status =
            format!("Added test player {slot}; move its feet with XYZ handles or Attributes");
    }
}
pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    mut scene: ResMut<Scene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if scene.mesh.is_none() {
        scene.mesh = Some(meshes.add(Capsule3d::new(0.3, 1.2)));
        scene.materials = [
            Color::srgb(0.15, 0.65, 1.),
            Color::srgb(1., 0.65, 0.1),
            Color::srgb(0.35, 0.35, 0.4),
        ]
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            })
        })
        .to_vec();
    }
    let zone = session.editor.zone();
    scene.entities.retain(|id, entity| {
        if zone.test_players.iter().any(|p| p.id == *id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
    for player in &zone.test_players {
        let position = group_selection::preview(
            &session,
            player.id,
            &group_selection::pose(player.position, 0., 1.),
        )
        .position;
        let transform = Transform::from_translation(
            Vec3::from_array(zone.local(position).expect("Validated test player")) + Vec3::Y * 0.9,
        );
        let selected = session.test_player_selected == Some(player.id);
        let material = scene.materials[if selected {
            1
        } else if player.enabled {
            0
        } else {
            2
        }]
        .clone();
        let visibility = if session.play.is_some() {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if let Some(entity) = scene.entities.get(&player.id) {
            commands
                .entity(*entity)
                .insert((transform, MeshMaterial3d(material), visibility));
        } else {
            let entity = commands
                .spawn((
                    Badge(player.id),
                    Mesh3d(scene.mesh.as_ref().unwrap().clone()),
                    MeshMaterial3d(material),
                    transform,
                    visibility,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            scene.entities.insert(player.id, entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn session(origin: [f64; 3]) -> Session {
        let zone = Zone {
            origin,
            spawn: origin.map(|v| v + 0.1),
            ..default()
        };
        Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone),
        )
    }
    #[test]
    fn patrol_assignment_cycles_only_patrols_and_is_undoable_without_moving_player() {
        use theia_zone::paths::{GuidePath, PathKind};
        let mut session = session([0.; 3]);
        add(&mut session);
        let before = session.editor.zone().clone();
        configure(Action::TestPlayerRoute(1), &mut session);
        assert_eq!(session.editor.zone(), &before);
        for (id, kind) in [
            (100, PathKind::Road),
            (101, PathKind::Patrol),
            (102, PathKind::Patrol),
        ] {
            session.edit(vec![Edit::PutPath {
                path: GuidePath {
                    id,
                    name: format!("Path {id}"),
                    kind,
                    width_metres: 2.,
                    points: vec![[0., 0., 0.], [0., 0., -5.]],
                },
            }]);
        }
        let position = session.editor.zone().test_players[0].position;
        for (direction, expected) in [(1, 101), (1, 102), (1, 101), (-1, 102)] {
            configure(Action::TestPlayerRoute(direction), &mut session);
            assert_eq!(
                session.editor.zone().test_players[0].controller,
                Some(format!("theia.zone.patrol.{expected}"))
            );
            assert_eq!(session.editor.zone().test_players[0].position, position);
        }
        session.execute(Command::Undo {
            expected_revision: session.editor.revision(),
        });
        assert_eq!(
            session.editor.zone().test_players[0].controller.as_deref(),
            Some("theia.zone.patrol.101")
        );
        configure(Action::TestPlayerController(false), &mut session);
        assert!(session.editor.zone().test_players[0].controller.is_none());
    }
    #[test]
    fn add_duplicate_translate_configuration_and_delete_retain_selection_and_saved_document() {
        for origin in [[0.; 3], [128_000_000_000., 0., -128_000_000_000.]] {
            let mut session = session(origin);
            let saved = session.saved_zone.clone();
            add(&mut session);
            let id = session.test_player_selected.unwrap();
            assert_eq!(
                outliner::selection(&session),
                Some(outliner::Target::TestPlayer(id))
            );
            assert!(workspace::Control::GizmoTool(gizmo::Tool::Move).enabled(&session));
            assert!(!workspace::Control::GizmoTool(gizmo::Tool::Rotate).enabled(&session));
            assert!(!workspace::Control::GizmoTool(gizmo::Tool::Scale).enabled(&session));
            configure(Action::TestPlayerController(true), &mut session);
            configure(Action::TestPlayerEnabled, &mut session);
            let (selected, mut pose) = transform_target::selected(&session).unwrap();
            pose.position[0] += 0.125;
            let edit =
                transform_target::edit(session.editor.zone(), selected, &pose, None).unwrap();
            session.edit(vec![edit]);
            assert_eq!(session.test_player_selected, Some(id));
            assert_eq!(
                session.editor.zone().test_players[0].position,
                pose.position
            );
            selection::duplicate(&mut session);
            let copy = session.test_player_selected.unwrap();
            assert_ne!(copy, id);
            assert_eq!(session.editor.zone().test_players[1].client_id, 2);
            assert_eq!(session.selected, None);
            assert!(workspace::Control::Command(Action::AddTestPlayer).enabled(&session));
            add(&mut session);
            add(&mut session);
            assert_eq!(
                session
                    .editor
                    .zone()
                    .test_players
                    .iter()
                    .map(|p| p.client_id)
                    .collect::<Vec<_>>(),
                vec![1, 2, 3, 4]
            );
            assert!(!workspace::Control::Command(Action::AddTestPlayer).enabled(&session));
            let before = session.editor.zone().clone();
            add(&mut session);
            assert_eq!(session.editor.zone(), &before);
            session.test_player_selected = Some(copy);
            session.edit(vec![Edit::DeleteTestPlayer { id: copy }]);
            assert_eq!(session.test_player_selected, None);
            session.execute(Command::Undo {
                expected_revision: session.editor.revision(),
            });
            assert_eq!(session.editor.zone(), &before);
            assert_eq!(session.saved_zone, saved);
        }
    }
    #[test]
    fn badges_hide_for_offline_play_and_release_deleted_entities() {
        let mut session = session([0.; 3]);
        add(&mut session);
        let id = session.test_player_selected.unwrap();
        let mut app = App::new();
        app.insert_resource(session)
            .init_resource::<Scene>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Update, sync);
        app.update();
        let entity = app.world().resource::<Scene>().entities[&id];
        assert_eq!(
            app.world().get::<Transform>(entity).unwrap().translation,
            Vec3::new(0.1, 1., 0.1)
        );
        // Test-pawn badges add no world collision to the ordinary offline sandbox.
        let play =
            playtest::Playtest::new(app.world().resource::<Session>().editor.zone()).unwrap();
        app.world_mut().resource_mut::<Session>().play = Some(play);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Hidden
        );
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::DeleteTestPlayer { id }]);
        app.update();
        assert!(app.world().get_entity(entity).is_err());
        assert!(app.world().resource::<Scene>().entities.is_empty());
    }
}
