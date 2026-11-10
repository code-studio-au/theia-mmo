use super::*;
fn session(origin: [f64; 3]) -> Session {
    let zone = Zone {
        origin,
        spawn: [origin[0], origin[1] + 0.0625, origin[2] + 6.],
        ..default()
    };
    let mut editor = Editor::new(zone.clone()).unwrap();
    editor
        .execute(Command::Apply {
            expected_revision: 0,
            edits: vec![Edit::RenameZone {
                name: "Temporary".into(),
            }],
        })
        .unwrap();
    editor
        .execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
    let mut session = Session::new(editor, "unused.json".into(), Some(zone));
    session.placement_size = [8., 3.25, 2.5];
    session.move_step = 0.25;
    session
}
#[test]
fn preview_is_read_only_and_matches_the_eventual_shared_stamp_at_local_and_far_origins() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for primitive in [Primitive::Box, Primitive::Sphere, Primitive::Cylinder] {
            for snap in [true, false] {
                let mut session = session(origin);
                session.snap = snap;
                let source = session.editor.zone().clone();
                let hit = [origin[0] + 12.38, origin[1] + 4., origin[2] - 8.38];
                let candidate = preview(&session, primitive, hit, None).unwrap();
                assert!(candidate.error.is_none());
                let same = preview(&session, primitive, hit, Some(candidate.clone())).unwrap();
                assert_eq!(same.object, candidate.object);
                assert_eq!(session.editor.zone(), &source);
                assert_eq!(session.editor.revision(), 2);
                assert_eq!(session.editor.history(), (0, 1));
                assert_eq!(session.next_id, 1);
                session.placement_preview = Some(same);
                assert!(!session.is_dirty());
                session.place_primitive(primitive, hit);
                assert_eq!(session.editor.zone().objects, vec![candidate.object]);
                assert_eq!(session.editor.revision(), 3);
                assert_eq!(session.editor.history(), (1, 0));
                let placed = session.editor.zone().clone();
                let previous = session.placement_preview.take();
                let next = preview(&session, primitive, hit, previous).unwrap();
                assert_eq!(next.object.id, 2);
                assert!(next.error.is_none());
                assert_eq!(session.editor.zone(), &placed);
                session.placement_preview = Some(next);
                session.set_mode(EditorMode::Sculpt);
                assert!(session.placement_preview.is_none());
            }
        }
    }
}
#[test]
fn invalid_candidates_share_stamp_refusals_and_budget_or_id_changes_invalidate_the_cache() {
    let mut session = session([0.; 3]);
    let source = session.editor.zone().clone();
    let invalid = preview(&session, Primitive::Box, [511., 0., 0.], None).unwrap();
    assert!(invalid.error.as_ref().unwrap().contains("Outside bounded"));
    session.place_primitive(Primitive::Box, [511., 0., 0.]);
    assert_eq!(Some(&session.status), invalid.error.as_ref());
    assert_eq!(session.editor.zone(), &source);
    assert_eq!(session.editor.history(), (0, 1));
    assert!(preview(&session, Primitive::Box, [f64::NAN, 0., 0.], None).is_none());
    let prior = preview(&session, Primitive::Box, [12., 0., -8.], None).unwrap();
    session.next_id = u64::MAX;
    let exhausted = preview(&session, Primitive::Box, [12., 0., -8.], Some(prior)).unwrap();
    assert_eq!(exhausted.error.as_deref(), Some("Object IDs exhausted"));
    assert_eq!(session.editor.zone(), &source);
    let mut full = source.clone();
    full.objects = (1..=theia_zone::MAX_OBJECTS as u64)
        .map(|id| Object {
            id,
            name: format!("Box {id}"),
            primitive: Primitive::Box,
            pose: Pose {
                position: [8., 1., -8.],
                yaw_degrees: 0.,
                size: [2.; 3],
            },
        })
        .collect();
    session.editor = Editor::new(full.clone()).unwrap();
    session.next_id = theia_zone::MAX_OBJECTS as u64 + 1;
    let candidate = preview(&session, Primitive::Box, [12., 0., -8.], None).unwrap();
    assert_eq!(candidate.error.as_deref(), Some("Object budget exceeded"));
    session.place_primitive(Primitive::Box, [12., 0., -8.]);
    assert_eq!(Some(&session.status), candidate.error.as_ref());
    assert_eq!(session.editor.zone(), &full);
    assert_eq!(session.editor.history(), (0, 0));
}
#[test]
fn one_nonpickable_nonshadowing_ghost_reuses_palette_meshes_and_hides_without_a_candidate() {
    let mut app = App::new();
    app.insert_resource(session([128_000_000_000.125, 20.25, -128_000_000_000.5]))
        .insert_resource(View {
            target: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<bevy::input_focus::InputFocus>()
        .init_resource::<workspace::Requests>()
        .init_resource::<outliner::Content>()
        .init_resource::<bevy::picking::pointer::PointerMap>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(
            Startup,
            (viewport::setup, setup.after(viewport::setup), naming::setup),
        )
        .add_systems(Update, (naming::keyboard, sync).chain())
        .add_systems(PostUpdate, naming::finish);
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    app.update();
    let ghost = app
        .world_mut()
        .query_filtered::<Entity, With<Ghost>>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        *app.world().get::<Visibility>(ghost).unwrap(),
        Visibility::Hidden
    );
    assert!(
        app.world()
            .get::<bevy::light::NotShadowCaster>(ghost)
            .is_some()
    );
    let pickable = app.world().get::<bevy::picking::Pickable>(ghost).unwrap();
    assert!(!pickable.is_hoverable && !pickable.should_block_lower);
    let mesh_count = app.world().resource::<Assets<Mesh>>().len();
    let material_count = app.world().resource::<Assets<StandardMaterial>>().len();
    for primitive in [Primitive::Box, Primitive::Sphere, Primitive::Cylinder] {
        let origin = app.world().resource::<Session>().editor.zone().origin;
        for (offset, invalid) in [(12., false), (511., true)] {
            let candidate = preview(
                app.world().resource::<Session>(),
                primitive,
                [origin[0] + offset, origin[1], origin[2] - 8.],
                None,
            )
            .unwrap();
            assert_eq!(candidate.error.is_some(), invalid);
            let pose = candidate.object.pose.clone();
            app.world_mut().resource_mut::<Session>().palette = Some(primitive);
            app.world_mut().resource_mut::<Session>().placement_preview = Some(candidate);
            app.update();
            assert_eq!(
                *app.world().get::<Visibility>(ghost).unwrap(),
                Visibility::Visible
            );
            assert_eq!(
                app.world().get::<Mesh3d>(ghost).unwrap().0,
                app.world().resource::<viewport::Palette>().meshes[viewport::index(primitive)]
            );
            let transform = app.world().get::<Transform>(ghost).unwrap();
            assert_eq!(
                transform.scale,
                Vec3::from_array(pose.size.map(|v| v as f32))
            );
            assert_eq!(
                transform.translation,
                Vec3::from_array(
                    app.world()
                        .resource::<Session>()
                        .editor
                        .zone()
                        .local(pose.position)
                        .unwrap()
                )
            );
            let handle = &app
                .world()
                .get::<MeshMaterial3d<StandardMaterial>>(ghost)
                .unwrap()
                .0;
            let material = app
                .world()
                .resource::<Assets<StandardMaterial>>()
                .get(handle)
                .unwrap();
            assert_eq!(material.alpha_mode, AlphaMode::Blend);
            assert!(material.unlit);
            let color = material.base_color.to_srgba();
            assert_eq!(color.red > color.blue, invalid);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
            assert_eq!(
                app.world().resource::<Assets<StandardMaterial>>().len(),
                material_count
            );
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        }
    }
    let source = app.world().resource::<Session>().editor.zone().clone();
    app.world_mut().resource_mut::<workspace::Requests>().find = true;
    app.update();
    assert!(app.world().resource::<Session>().name_edit.is_some());
    assert!(
        app.world()
            .resource::<Session>()
            .placement_preview
            .is_some()
    );
    assert_eq!(
        *app.world().get::<Visibility>(ghost).unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(ghost).unwrap(),
        Visibility::Visible
    );
    for playing in [true, false] {
        app.world_mut()
            .resource_scope(|world, mut session: Mut<Session>| {
                perform(
                    Action::Play,
                    &mut session,
                    &mut world.resource_mut::<View>(),
                )
            });
        app.update();
        assert_eq!(app.world().resource::<Session>().play.is_some(), playing);
        assert!(
            app.world()
                .resource::<Session>()
                .placement_preview
                .is_some()
        );
        assert_eq!(
            *app.world().get::<Visibility>(ghost).unwrap(),
            if playing {
                Visibility::Hidden
            } else {
                Visibility::Visible
            }
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    }
    app.world_mut().resource_mut::<Session>().palette = None;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(ghost).unwrap(),
        Visibility::Hidden
    );
    app.world_mut().resource_mut::<Session>().placement_preview = None;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(ghost).unwrap(),
        Visibility::Hidden
    );
}
