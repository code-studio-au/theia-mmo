//! Placement settings and shared placement/undo, without OS input or a renderer.
use super::*;

fn fixture(origin: [f64; 3]) -> App {
    let zone = Zone {
        origin,
        spawn: [origin[0], origin[1] + 0.0625, origin[2] + 6.],
        ..default()
    };
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        "unused.zone.json".into(),
        Some(zone),
    );
    session.palette = Some(Primitive::Box);
    session.move_step = 0.25;
    let mut app = App::new();
    app.insert_resource(session)
        .insert_resource(View {
            target: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        })
        .add_observer(change)
        .add_systems(Startup, (setup, crate::setup_ui))
        .add_systems(Update, (sync, crate::present).chain());
    app.world_mut().spawn(Window {
        focused: true,
        resolution: (3440, 1440).into(),
        ..default()
    });
    app.update();
    app
}
fn change_size(app: &mut App, axis: usize, value: f32) {
    let entity = app
        .world_mut()
        .query::<(Entity, &Setting)>()
        .iter(app.world())
        .find(|(_, setting)| matches!(setting, Setting::PlacementSize(i) if *i == axis))
        .unwrap()
        .0;
    app.world_mut().trigger(ValueChange {
        source: entity,
        value,
        is_final: true,
    });
    app.update();
}
#[test]
fn placement_settings_keep_source_redo_and_shape_constraints_then_place_one_shared_edit() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let mut app = fixture(origin);
        let original = app.world().resource::<Session>().editor.zone().clone();
        for primitive in [Primitive::Box, Primitive::Sphere, Primitive::Cylinder] {
            app.world_mut().resource_mut::<Session>().palette = Some(primitive);
            for (axis, value) in [(0, 8.), (1, 3.25), (2, 2.5)] {
                change_size(&mut app, axis, value);
            }
            let before = app.world().resource::<Session>();
            assert_eq!(before.editor.zone(), &original);
            assert_eq!(
                before.editor.history(),
                (0, if primitive == Primitive::Box { 0 } else { 1 })
            );
            assert!(!before.is_dirty());
            let expected_size = match primitive {
                Primitive::Box => [8., 3.25, 2.5],
                Primitive::Sphere => [8.; 3],
                Primitive::Cylinder => [8., 3.25, 8.],
            };
            assert_eq!(before.placement_dimensions(primitive), expected_size);
            let prior_revision = before.editor.revision();
            app.world_mut().resource_mut::<Session>().place_primitive(
                primitive,
                [origin[0] + 12.38, origin[1] + 4., origin[2] - 8.38],
            );
            let session = app.world().resource::<Session>();
            let object = &session.editor.zone().objects[0];
            assert_eq!(object.primitive, primitive);
            assert_eq!(object.pose.size, expected_size);
            assert_eq!(
                object.pose.position,
                [
                    origin[0] + 12.5,
                    origin[1] + expected_size[1] / 2.,
                    origin[2] - 8.5
                ]
            );
            let placed = session.editor.zone().clone();
            assert_eq!(session.editor.revision(), prior_revision + 1);
            assert_eq!(session.editor.history(), (1, 0));
            let mut oracle = Editor::new(original.clone()).unwrap();
            oracle
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::Place {
                        object: object.clone(),
                    }],
                })
                .unwrap();
            assert_eq!(oracle.zone(), &placed);
            let mut session = app.world_mut().resource_mut::<Session>();
            let revision = session.editor.revision();
            session
                .editor
                .execute(Command::Undo {
                    expected_revision: revision,
                })
                .unwrap();
            assert_eq!(session.editor.zone(), &original);
            change_size(&mut app, 0, 7.);
            let mut session = app.world_mut().resource_mut::<Session>();
            assert_eq!(session.editor.history(), (0, 1));
            let revision = session.editor.revision();
            session
                .editor
                .execute(Command::Redo {
                    expected_revision: revision,
                })
                .unwrap();
            assert_eq!(session.editor.zone(), &placed);
            let revision = session.editor.revision();
            session
                .editor
                .execute(Command::Undo {
                    expected_revision: revision,
                })
                .unwrap();
            session.selected = None;
        }
    }
}
#[test]
fn placement_sliders_obey_focus_busy_visibility_and_bounded_quarter_steps() {
    let mut app = fixture([0.; 3]);
    change_size(&mut app, 0, f32::NAN);
    assert_eq!(app.world().resource::<Session>().placement_size, [1.; 3]);
    change_size(&mut app, 0, 0.01);
    change_size(&mut app, 1, 99.);
    change_size(&mut app, 2, 1.37);
    assert_eq!(
        app.world().resource::<Session>().placement_size,
        [0.25, 32., 1.25]
    );
    app.world_mut().resource_mut::<Session>().palette = Some(Primitive::Sphere);
    change_size(&mut app, 1, 4.);
    change_size(&mut app, 2, 5.);
    assert_eq!(
        app.world().resource::<Session>().placement_size,
        [0.25, 32., 1.25]
    );
    app.world_mut().resource_mut::<Session>().palette = None;
    change_size(&mut app, 0, 3.);
    assert_eq!(
        app.world().resource::<Session>().placement_size,
        [0.25, 32., 1.25]
    );
    app.world_mut().resource_mut::<Session>().palette = Some(Primitive::Box);
    app.world_mut()
        .query::<&mut Window>()
        .single_mut(app.world_mut())
        .unwrap()
        .focused = false;
    change_size(&mut app, 0, 3.);
    assert_eq!(
        app.world().resource::<Session>().placement_size,
        [0.25, 32., 1.25]
    );
    app.world_mut()
        .query::<&mut Window>()
        .single_mut(app.world_mut())
        .unwrap()
        .focused = true;
    crate::paths::perform(
        Action::NewPath(theia_zone::paths::PathKind::Road),
        &mut app.world_mut().resource_mut::<Session>(),
    );
    change_size(&mut app, 0, 3.);
    assert_eq!(
        app.world().resource::<Session>().placement_size,
        [0.25, 32., 1.25]
    );
    assert_eq!(app.world().resource::<Session>().editor.revision(), 0);
}

#[test]
fn sized_placement_uses_terrain_support_and_refuses_invalid_or_out_of_bounds_stamps() {
    for origin in [[0., 0.25, 0.], [128_000_000_000., 20.25, -128_000_000_000.]] {
        let mut app = fixture(origin);
        let mut zone = app.world().resource::<Session>().editor.zone().clone();
        let mut patch = theia_zone::terrain::HeightPatch::flat([
            (origin[0] / 128.) as i64,
            (origin[2] / 128.) as i64,
        ]);
        let axis = patch.axis();
        for z in 1..axis - 1 {
            for x in 1..axis - 1 {
                patch.heights[z * axis + x] = 2.;
            }
        }
        zone.schema = 2;
        zone.terrain = Some(patch);
        app.world_mut().resource_mut::<Session>().editor = Editor::new(zone).unwrap();
        change_size(&mut app, 0, 8.);
        change_size(&mut app, 1, 3.25);
        change_size(&mut app, 2, 2.5);
        let mut session = app.world_mut().resource_mut::<Session>();
        session.place_primitive(
            Primitive::Box,
            [origin[0] + 32., origin[1] + 9., origin[2] + 32.],
        );
        assert_eq!(
            session.editor.zone().objects[0].pose.position,
            [origin[0] + 32., origin[1] + 2. + 3.25 / 2., origin[2] + 32.]
        );
        let source = session.editor.zone().clone();
        for hit in [
            [origin[0] + 511., origin[1], origin[2]],
            [f64::NAN, origin[1], origin[2]],
        ] {
            session.place_primitive(Primitive::Box, hit);
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.revision(), 1);
            assert_eq!(session.editor.history(), (1, 0));
        }
        session.next_id = u64::MAX;
        session.place_primitive(
            Primitive::Sphere,
            [origin[0] + 24., origin[1], origin[2] + 24.],
        );
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.status, "Object IDs exhausted");
    }
}
#[test]
fn placement_controls_fit_properties_for_each_shape_and_hide_when_selecting() {
    let mut app = fixture([0.; 3]);
    {
        let mut session = app.world_mut().resource_mut::<Session>();
        session
            .editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Long courtyard ".repeat(5),
                }],
            })
            .unwrap();
        session.placement_size = [32.; 3];
    }
    crate::headless_ui::add_layout(&mut app);
    for primitive in [
        Some(Primitive::Box),
        Some(Primitive::Sphere),
        Some(Primitive::Cylinder),
        None,
    ] {
        app.world_mut().resource_mut::<Session>().palette = primitive;
        let candidate = primitive.and_then(|p| {
            placement::preview(app.world().resource::<Session>(), p, [511., 0., 0.], None)
        });
        app.world_mut().resource_mut::<Session>().placement_preview = candidate;

        for _ in 0..3 {
            app.update();
        }
        for (text, node, transform) in app
            .world_mut()
            .query::<(&Text, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
            .iter(app.world())
        {
            if text.0.starts_with("Click sliders to focus") {
                let centre = transform.affine().translation;
                let half = node.size() * 0.5;
                assert!(
                    centre.y - half.y >= workspace::HEADER && centre.y + half.y <= 1440.,
                    "footer overflow: {centre:?} {half:?}"
                );
            }
        }
        for (setting, node, transform) in app
            .world_mut()
            .query::<(&Setting, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
            .iter(app.world())
        {
            let Setting::PlacementSize(axis) = setting else {
                continue;
            };
            let visible = match primitive {
                Some(Primitive::Box) => true,
                Some(Primitive::Sphere) => *axis == 0,
                Some(Primitive::Cylinder) => *axis < 2,
                None => false,
            };
            if !visible {
                assert_eq!(node.size(), Vec2::ZERO);
                continue;
            }
            assert!(node.size().min_element() > 0.);
            let centre = transform.affine().translation;
            let half = node.size() * 0.5;
            assert!(
                centre.x - half.x >= sidebar::WIDTH + outliner::WIDTH
                    && centre.x + half.x <= CANVAS_LEFT
            );
            assert!(centre.y - half.y >= workspace::HEADER && centre.y + half.y <= 1440.);
        }
    }
    let before: Vec<_> = app
        .world_mut()
        .query::<(Entity, &SettingRow, &Node)>()
        .iter(app.world())
        .map(|(entity, _, node)| (entity, node.display))
        .collect();
    let source = app.world().resource::<Session>().editor.zone().clone();
    let preferences = app.world().resource::<Session>().placement_size;
    app.world_mut()
        .resource_scope(|world, mut session: Mut<Session>| {
            perform(
                Action::Play,
                &mut session,
                &mut world.resource_mut::<View>(),
            )
        });
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().resource::<Session>().play.is_some());
    for (_, computed) in app
        .world_mut()
        .query::<(&Setting, &ComputedNode)>()
        .iter(app.world())
    {
        assert_eq!(computed.size(), Vec2::ZERO);
    }
    for (node,transform) in app.world_mut().query_filtered::<(&ComputedNode,&bevy::ui::UiGlobalTransform),Or<(With<Caption>,With<Info>,With<SettingsHint>)>>().iter(app.world()) {
        let centre=transform.affine().translation;let half=node.size()*0.5;
        assert!(node.size().min_element()>0.);
        assert!(centre.x-half.x>=sidebar::WIDTH+outliner::WIDTH && centre.x+half.x<=CANVAS_LEFT);
        assert!(centre.y-half.y>=workspace::HEADER && centre.y+half.y<=1440.);
    }
    app.world_mut()
        .resource_scope(|world, mut session: Mut<Session>| {
            perform(
                Action::Play,
                &mut session,
                &mut world.resource_mut::<View>(),
            )
        });
    for _ in 0..3 {
        app.update();
    }
    let after: Vec<_> = app
        .world_mut()
        .query::<(Entity, &SettingRow, &Node)>()
        .iter(app.world())
        .map(|(entity, _, node)| (entity, node.display))
        .collect();
    assert_eq!(before, after);
    assert_eq!(
        app.world().resource::<Session>().placement_size,
        preferences
    );
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
}

#[test]
fn sprinkles_browser_inspector_and_tools_have_separate_full_height_regions() {
    use bevy::ecs::system::RunSystemOnce;
    let mut app = fixture([128_000_000_000.125, 20.25, -128_000_000_000.5]);
    app.world_mut().run_system_once(outliner::setup).unwrap();
    app.add_observer(sidebar::activate)
        .add_systems(Update, outliner::sync);
    crate::headless_ui::add_layout(&mut app);
    let source = app.world().resource::<Session>().editor.zone().clone();
    app.add_systems(Update, (panels::apply, terrain::palette).chain());
    for (browser_width, inspector_width) in [(300., 300.), (480., 600.), (450., 350.)] {
        app.world_mut().resource_mut::<Session>().ui.browser = browser_width;
        app.world_mut().resource_mut::<Session>().ui.inspector = inspector_width;
        for mode in [
            EditorMode::Objects,
            EditorMode::Sculpt,
            EditorMode::Paint,
            EditorMode::Guides,
            EditorMode::Markers,
            EditorMode::Plants,
        ] {
            let button = app.world_mut().query_filtered::<(Entity, &Control), With<sidebar::RailButton>>()
                .iter(app.world()).find(|(_, control)| matches!(control, Control::Mode(candidate) if *candidate == mode)).unwrap().0;
            app.world_mut().trigger(Activate { entity: button });
            assert_eq!(super::mode(app.world().resource::<Session>()), mode);
            for _ in 0..3 {
                app.update();
            }
            let rail_buttons: Vec<_> = app.world_mut()
                .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<sidebar::RailButton>>()
                .iter(app.world()).map(|(node, pose)| (node.size(), pose.affine().translation)).collect();
            assert_eq!(rail_buttons.len(), 6);
            for (size, centre) in rail_buttons {
                assert!(size.x > 0. && size.y >= 40.);
                assert!(centre.x - size.x / 2. >= 0. && centre.x + size.x / 2. <= sidebar::WIDTH);
                assert!(centre.y - size.y / 2. >= HEADER && centre.y + size.y / 2. <= 1440.);
            }
            let (browser, browser_pose) = app
            .world_mut()
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<outliner::Root>>()
            .single(app.world())
            .unwrap();
            assert_eq!(
                browser.size(),
                Vec2::new(browser_width, 1440. - HEADER - 52.)
            );
            assert_eq!(
                browser_pose.affine().translation,
                Vec2::new(
                    sidebar::WIDTH + browser_width / 2.,
                    HEADER + browser.size().y / 2.
                )
            );
            let (inspector, inspector_pose) = app
                .world_mut()
                .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Inspector>>()
                .single(app.world())
                .unwrap();
            assert_eq!(inspector.size(), Vec2::new(inspector_width, 1440. - HEADER));
            assert_eq!(
                inspector_pose.affine().translation.x,
                sidebar::WIDTH + browser_width + inspector_width / 2.
            );
            let mut visible = 0;
            for (style,node,pose) in app.world_mut().query_filtered::<(&Node,&ComputedNode,&bevy::ui::UiGlobalTransform),With<PaletteSlot>>().iter(app.world()) {
            if style.display==Display::None {continue;}
            visible+=1;
            let p=pose.affine().translation;let half=node.size()/2.;
            assert!(node.size().min_element()>0.);
            assert!(p.x-half.x>=sidebar::WIDTH+browser_width && p.x+half.x<=sidebar::WIDTH+browser_width+inspector_width);
            assert!(p.y-half.y>=HEADER && p.y+half.y<=1440.);
        }
            assert!(visible > 0);
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 0));
        }
    }
}
