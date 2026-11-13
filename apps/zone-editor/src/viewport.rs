use super::*;
use bevy::{
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
    render::{
        renderer::RenderAdapterInfo,
        view::window::screenshot::{Screenshot, ScreenshotCaptured},
    },
};
#[derive(Component)]
pub(super) struct Authored(pub u64);
#[derive(Component)]
pub(super) struct ZoneCamera;
#[derive(Component)]
pub(super) struct SpawnMarker;
#[derive(Component)]
pub(super) struct Pawn;
#[derive(Resource)]
pub(super) struct Palette {
    pub(super) meshes: [Handle<Mesh>; 3],
    normal: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}
pub(super) fn index(primitive: Primitive) -> usize {
    match primitive {
        Primitive::Box => 0,
        Primitive::Sphere => 1,
        Primitive::Cylinder => 2,
    }
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let pawn = materials.add(Color::srgb(0.25, 0.65, 0.95));
    commands.insert_resource(Palette {
        meshes: [
            meshes.add(Cuboid::new(1., 1., 1.)),
            meshes.add(Sphere::new(0.5)),
            meshes.add(Cylinder::new(0.5, 1.)),
        ],
        normal: materials.add(Color::srgb(0.52, 0.60, 0.66)),
        selected: materials.add(Color::srgb(0.95, 0.61, 0.19)),
    });
    commands.spawn((
        SpawnMarker,
        Mesh3d(meshes.add(Cone::new(0.65, 2.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.9, 0.5),
            unlit: true,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        Visibility::Visible,
        bevy::light::NotShadowCaster,
        bevy::picking::Pickable::IGNORE,
    ));
    commands.spawn((
        Pawn,
        Mesh3d(meshes.add(Capsule3d::new(0.35, 1.1))),
        MeshMaterial3d(pawn),
        Visibility::Hidden,
        Transform::default(),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 14000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(20., 40., 15.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // Sprinkles uses the standard ViewportNode, which resizes this target to
    // its actual UI rectangle and routes native picking into the scene camera.
    // Keep the UI camera on the window so panels retain whole-window layout.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        bevy::ui::IsDefaultUiCamera,
    ));
    let image = images.add(Image::new_target_texture(
        1,
        1,
        bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let camera = commands
        .spawn((
            Camera3d::default(),
            ZoneCamera,
            Camera {
                order: -1,
                ..default()
            },
            bevy::camera::RenderTarget::Image(image.into()),
            bevy::core_pipeline::tonemapping::Tonemapping::ReinhardLuminance,
            Transform::from_xyz(20., 28., 30.).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id();
    commands.spawn((
        panels::Region::Canvas,
        Node {
            position_type: PositionType::Absolute,
            left: px(CANVAS_LEFT),
            right: px(0),
            top: px(workspace::HEADER),
            bottom: px(status_bar::HEIGHT),
            ..default()
        },
        bevy::ui::widget::ViewportNode::new(camera),
    ));
}
fn transform(zone: &Zone, pose: &Pose) -> Transform {
    Transform {
        translation: Vec3::from_array(zone.local(pose.position).expect("Validated pose")),
        rotation: Quat::from_rotation_y(pose.yaw_degrees.to_radians() as f32),
        scale: Vec3::from_array(pose.size.map(|v| v as f32)),
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    palette: Res<Palette>,
    mut objects: Query<(
        Entity,
        &Authored,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut spawn: Query<
        (&mut Transform, &mut Visibility),
        (With<SpawnMarker>, Without<Authored>, Without<Pawn>),
    >,
    mut pawn: Query<
        (&mut Transform, &mut Visibility),
        (With<Pawn>, Without<Authored>, Without<SpawnMarker>),
    >,
    mut last: Local<Option<u64>>,
) {
    if *last != Some(session.editor.revision()) {
        for (entity, _, _, _) in &objects {
            commands.entity(entity).despawn();
        }
        for object in &session.editor.zone().objects {
            commands.spawn((
                Authored(object.id),
                Mesh3d(palette.meshes[index(object.primitive)].clone()),
                MeshMaterial3d(palette.normal.clone()),
                transform(session.editor.zone(), &object.pose),
            ));
        }
        *last = Some(session.editor.revision());
    } else {
        for (_, authored, mut visual, mut material) in &mut objects {
            let object = session
                .editor
                .zone()
                .objects
                .iter()
                .find(|o| o.id == authored.0)
                .expect("Synced object");
            let pose = session
                .drag
                .as_ref()
                .filter(|d| d.id == object.id)
                .map_or(&object.pose, |d| &d.preview);
            *visual = transform(session.editor.zone(), pose);
            material.0 = if session.selected == Some(object.id) {
                palette.selected.clone()
            } else {
                palette.normal.clone()
            };
        }
    }
    if let Ok((mut visual, mut visibility)) = spawn.single_mut() {
        *visibility = if session.play.is_none() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        visual.translation = Vec3::from_array(
            session
                .editor
                .zone()
                .local(session.editor.zone().spawn)
                .unwrap(),
        ) + Vec3::Y * 1.2;
    }
    if let Ok((mut visual, mut visibility)) = pawn.single_mut() {
        *visibility = if let Some(play) = &session.play {
            visual.translation = Vec3::from_array(play.feet.metres()) + Vec3::Y * 0.9;
            visual.rotation = Quat::from_rotation_y(session.heading);
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<crate::viewport::ZoneCamera>>,
    objects: Query<&Authored>,
    particles: Query<&particle_anchors::ParticleAnchor>,
    scenery: Query<&scenery_scene::SceneryRoot>,
    test_players: Query<&test_players::Badge>,
    parents: Query<&ChildOf>,
    handles: Query<(&gizmo::TransformHandle, &Visibility, &Transform)>,
    ground: Query<(), With<terrain::GroundPart>>,
    mut raycast: MeshRayCast,
    library: Option<Res<scenery_library::Library>>,
    mut session: ResMut<Session>,
) {
    let previous_preview = session.placement_preview.take();
    let Ok(window) = windows.single() else {
        return;
    };
    if session.has_modal() {
        return;
    }
    if session.terrain_mode || session.path_mode || session.marker_mode || session.vegetation_mode {
        return;
    }
    if !window.focused || session.play.is_some() {
        session.drag = None;
        return;
    }
    let Some(cursor) = canvas_cursor(window, &session) else {
        if mouse.just_released(MouseButton::Left) {
            session.drag = None;
        }
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    if let Some(primitive) = session.palette {
        let filter = |entity| ground.contains(entity);
        let settings = MeshRayCastSettings::default().with_filter(&filter);
        let hit = raycast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_, hit)| ray.get_point(hit.distance));
        if let Some(hit) = hit {
            match session.editor.zone().global(hit.to_array()) {
                Ok(point) => {
                    session.placement_preview =
                        placement::preview(&session, primitive, point, previous_preview);
                    if mouse.just_pressed(MouseButton::Left) {
                        session.place_primitive(primitive, point);
                    }
                }
                Err(error) if mouse.just_pressed(MouseButton::Left) => session.status = error,
                Err(_) => {}
            }
        }
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        let scenery_id = |entity| {
            std::iter::once(entity)
                .chain(parents.iter_ancestors(entity))
                .find_map(|ancestor| scenery.get(ancestor).ok().map(|root| root.0))
        };
        let filter = |entity| {
            objects.contains(entity)
                || scenery_id(entity).is_some()
                || test_players.contains(entity)
                || particles.contains(entity)
                || ground.contains(entity)
                || handles
                    .get(entity)
                    .is_ok_and(|(_, v, _)| *v != Visibility::Hidden)
        };
        let settings = MeshRayCastSettings::default().with_filter(&filter);
        let hit = raycast
            .cast_ray(ray, &settings)
            .first()
            .map(|(entity, _)| *entity);
        if let Some((handle, _, handle_transform)) = hit.and_then(|entity| handles.get(entity).ok())
        {
            if let Some((id, pose)) = transform_target::selected(&session) {
                let local = Vec3::from_array(session.editor.zone().local(pose.position).unwrap());
                let reference = if handle.tool == gizmo::Tool::Scale {
                    handles
                        .iter()
                        .find(|(h, v, _)| {
                            h.tool == gizmo::Tool::Scale && h.head && **v != Visibility::Hidden
                        })
                        .map_or(0., |(_, _, transform)| {
                            transform.translation.distance(local)
                        })
                } else {
                    handle_transform.translation.distance(local)
                };
                if let Some(gesture) =
                    gizmo::Gesture::begin(handle.tool, ray, local, handle.axis, reference)
                {
                    session.drag = Some(Drag {
                        id,
                        before: pose.clone(),
                        preview: pose,
                        revision: session.editor.revision(),
                        gesture: gesture.with_grid(session.grid()),
                    });
                    session.status = match handle.tool {
                        gizmo::Tool::Move => "Moving world axis - release applies; Escape cancels",
                        gizmo::Tool::Rotate => "Rotating yaw - release applies; Escape cancels",
                        gizmo::Tool::Scale => "Scaling uniformly - release applies; Escape cancels",
                    }
                    .into();
                } else {
                    session.status = "Orbit the view to drag this handle".into();
                }
            }
        } else if let Some(id) = hit.and_then(scenery_id) {
            let already_selected = session.scenery_selected == Some(id);
            session.set_mode(EditorMode::Objects);
            session.scenery_selected = Some(id);
            session.status = format!("Selected scenery #{id}");
            if already_selected
                && session.gizmo_tool == gizmo::Tool::Move
                && let Some((_, pose)) = transform_target::selected(&session)
                && let Ok(local) = session.editor.zone().local(pose.position)
                && let Some(gesture) = gizmo::Gesture::plane(ray, Vec3::from_array(local))
            {
                session.drag = Some(Drag {
                    id,
                    before: pose.clone(),
                    preview: pose,
                    revision: session.editor.revision(),
                    gesture: gesture.with_grid(session.grid()),
                });
                session.status = "Moving XZ plane - release applies; Escape cancels".into();
            }
        } else if let Some(player) = hit.and_then(|entity| test_players.get(entity).ok()) {
            session.set_mode(EditorMode::Objects);
            session.test_player_selected = Some(player.0);
            session.gizmo_tool = gizmo::Tool::Move;
            session.status = format!("Selected test player #{}", player.0);
        } else if let Some(particle) = hit.and_then(|entity| particles.get(entity).ok()) {
            session.inspect_spawn = false;
            session.selected = None;
            session.particle_selected = Some(particle.0);
            session.quest_area_selected = None;
            session.spawn_group_selected = None;
            session.scenery_selected = None;
            session.test_player_selected = None;
            session.status = format!("Selected particle reference #{}", particle.0);
        } else {
            let id = hit.and_then(|entity| objects.get(entity).ok()).map(|o| o.0);
            let already_selected = id == session.selected;
            session.inspect_spawn = false;
            session.selected = id;
            session.particle_selected = None;
            session.quest_area_selected = None;
            session.spawn_group_selected = None;
            session.scenery_selected = None;
            session.test_player_selected = None;
            if already_selected
                && session.gizmo_tool == gizmo::Tool::Move
                && let Some(id) = id
            {
                let object = session
                    .editor
                    .zone()
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .unwrap();
                let local =
                    Vec3::from_array(session.editor.zone().local(object.pose.position).unwrap());
                if let Some(gesture) = gizmo::Gesture::plane(ray, local) {
                    session.drag = Some(Drag {
                        id,
                        before: object.pose.clone(),
                        preview: object.pose.clone(),
                        revision: session.editor.revision(),
                        gesture: gesture.with_grid(session.grid()),
                    });
                    session.status = "Moving XZ plane - release applies; Escape cancels".into();
                }
            }
        }
    }
    if let Some(mut drag) = session.drag.take() {
        let zone = session.editor.zone();
        if let Some(pose) = drag.gesture.preview(ray, &drag.before, session.snap)
            && pose != drag.preview
        {
            // Body, axis, rotation and scale all share candidate validation.
            let valid = Editor::new(zone.clone()).and_then(|mut candidate| {
                candidate.execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![transform_target::edit(
                        zone,
                        drag.id,
                        &pose,
                        library.as_deref(),
                    )?],
                })
            });
            match valid {
                Ok(()) => drag.preview = pose,
                Err(error) => session.status = error,
            }
        }
        if mouse.just_released(MouseButton::Left) {
            if drag.preview != drag.before {
                match transform_target::edit(
                    session.editor.zone(),
                    drag.id,
                    &drag.preview,
                    library.as_deref(),
                ) {
                    Ok(edit) => session.execute(Command::Apply {
                        expected_revision: drag.revision,
                        edits: vec![edit],
                    }),
                    Err(error) => session.status = error,
                }
            } else {
                session.status = match drag.gesture.tool() {
                    gizmo::Tool::Move => "Position unchanged",
                    gizmo::Tool::Rotate => "Yaw unchanged",
                    gizmo::Tool::Scale => "Size unchanged",
                }
                .into();
            }
        } else if mouse.pressed(MouseButton::Left) {
            session.drag = Some(drag);
        }
    }
}
#[derive(Resource)]
pub(super) struct Capture {
    pub path: PathBuf,
    pub frames: u32,
    pub view: Option<cli::CaptureView>,
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn capture(
    mut commands: Commands,
    capture: Option<ResMut<Capture>>,
    adapter: Res<RenderAdapterInfo>,
    session: Res<Session>,
    particle_editor: Res<bevy_sprinkles_editor::state::EditorState>,
    triggers: Query<(Entity, &menus::Trigger)>,
    mut requests: ResMut<workspace::Requests>,
    previews: Query<
        (&Children, &bevy_sprinkles::Particles3d),
        (
            With<particle_preview::ZoneParticlePreview>,
            With<bevy_sprinkles::runtime::ParticleSystemRuntime>,
        ),
    >,
) {
    let Some(mut capture) = capture else {
        return;
    };
    capture.frames += 1;
    if capture.frames == 30 && capture.view == Some(cli::CaptureView::ParticlePreview) {
        requests.particle_preview = true;
    }
    if capture.frames == 30 && capture.view == Some(cli::CaptureView::ParticleBind) {
        requests.bind_particle = true;
    }
    if capture.frames == 30 && capture.view == Some(cli::CaptureView::ParticleLibrary) {
        requests.preview_library_particle = true;
    }
    if capture.frames == 30
        && capture.view == Some(cli::CaptureView::PropertyActions)
        && let Some((entity, _)) = triggers
            .iter()
            .find(|(_, trigger)| trigger.0 == "Property actions")
    {
        commands.trigger(bevy::ui_widgets::MenuEvent {
            source: entity,
            action: bevy::ui_widgets::MenuAction::Open(
                bevy::input_focus::tab_navigation::NavAction::First,
            ),
        });
    }
    let capture_frame = if matches!(
        capture.view,
        Some(
            cli::CaptureView::ParticlePreview
                | cli::CaptureView::ParticleBind
                | cli::CaptureView::ParticleLibrary
        )
    ) {
        150
    } else {
        90
    };
    if capture.frames == capture_frame {
        if matches!(
            capture.view,
            Some(cli::CaptureView::ParticlePreview | cli::CaptureView::ParticleLibrary)
        ) {
            assert!(
                previews.iter().any(|(children, _)| !children.is_empty()),
                "Particle preview did not start any emitter entities"
            );
        }
        if capture.view == Some(cli::CaptureView::ParticleLibrary) {
            assert!(
                previews.iter().any(|(_, effect)| particle_editor
                    .current_project
                    .as_ref()
                    .is_none_or(|open| open.id() != effect.0.id())),
                "Library preview unexpectedly uses the open project handle"
            );
        }
        if capture.view == Some(cli::CaptureView::ParticleBind) {
            let effect = &session
                .editor
                .zone()
                .particles
                .first()
                .expect("capture particle")
                .effect;
            let path =
                bevy_sprinkles_editor::io::effect_library_path(effect).expect("valid capture ID");
            assert!(
                path.is_file(),
                "Particle library save did not create its file"
            );
            assert_eq!(particle_editor.current_project_path.as_ref(), Some(&path));
        }
        let path = capture.path.clone();
        info!(
            "Zone editor native capture: {} | {:?} | device {} | revision {} | objects {}",
            adapter.name,
            adapter.backend,
            adapter.device,
            session.editor.revision(),
            session.editor.zone().objects.len()
        );
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<bevy::app::AppExit>| {
                event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .expect("GPU pixels")
                    .save(&path)
                    .expect("Save capture");
                exit.write(bevy::app::AppExit::Success);
            },
        );
    }
    assert!(capture.frames < 600, "Capture timed out");
}

/// Opt-in read-only native qualification output; never consumed as a command.
#[derive(Resource)]
pub(super) struct Report {
    pub path: PathBuf,
    pub elapsed: f32,
}
type ReportControls<'w, 's> = Query<
    'w,
    's,
    (
        (
            Option<&'static Action>,
            Has<bevy::ui::Pressed>,
            Option<&'static ScrollPosition>,
            Option<&'static scrolling::Pane>,
            Entity,
            Option<&'static properties::FieldLabel>,
            Has<bevy::ui_widgets::MenuItem>,
            Option<&'static menus::SelectTrigger>,
            Option<&'static properties::SummaryValue>,
            Option<&'static Text>,
            Option<&'static properties::InlineInput>,
            Option<&'static bevy::text::EditableText>,
        ),
        Option<&'static naming::Control>,
        Option<&'static properties::Control>,
        Option<&'static properties::Field>,
        &'static ComputedNode,
        &'static bevy::ui::UiGlobalTransform,
        &'static Hovered,
        Option<&'static workspace::Control>,
        Option<&'static workspace::Setting>,
        Option<&'static bevy::ui_widgets::SliderValue>,
        Option<&'static bevy::ui_widgets::SliderRange>,
        Option<&'static file_picker::Control>,
        Option<&'static sections::Section>,
        Option<&'static menus::Trigger>,
        Option<&'static menus::Popup>,
    ),
>;
type ContentControls<'w, 's> = Query<
    'w,
    's,
    (
        &'static outliner::Control,
        &'static Node,
        &'static ComputedNode,
        &'static bevy::ui::UiGlobalTransform,
        &'static Hovered,
        Has<bevy::ui::InteractionDisabled>,
    ),
>;
type SceneVisuals<'w, 's> = Query<
    'w,
    's,
    (
        Has<markers::AuthoredMarker>,
        Has<terrain::BrushCursor>,
        Has<SpawnMarker>,
        &'static Visibility,
        Option<&'static Mesh3d>,
        Option<&'static Transform>,
        Option<&'static gizmo::TransformHandle>,
        Option<&'static authoring_grid::Line>,
        Has<terrain::TerrainGrid>,
        Has<placement::Ghost>,
    ),
>;
type TabControls<'w, 's> = Query<
    'w,
    's,
    (
        &'static property_tabs::Tab,
        &'static ComputedNode,
        &'static bevy::ui::UiGlobalTransform,
    ),
>;
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn report(
    mut commands: Commands,
    report: Option<ResMut<Report>>,
    time: Res<Time>,
    input: (
        Res<ButtonInput<KeyCode>>,
        Res<bevy::input_focus::InputFocus>,
        crate::particle_workspace::Report,
        Res<property_tabs::State>,
        TabControls,
        crate::export_dialog::Report,
        crate::asset_preview::Report,
        crate::quest_inspector::Report,
        crate::spawn_groups::Report,
        Query<
            (
                &ComputedNode,
                &bevy::ui::UiGlobalTransform,
                &bevy::text::EditableText,
            ),
            With<asset_browser::Search>,
        >,
        crate::account_picker::Report,
    ),
    session: Res<Session>,
    view: Res<View>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform, &Projection), With<crate::viewport::ZoneCamera>>,
    controls: ReportControls,
    scale: Res<bevy::ui::UiScale>,
    mouse: Res<ButtonInput<MouseButton>>,
    adapter: Res<RenderAdapterInfo>,
    meshes: Res<Assets<Mesh>>,
    scene_visuals: SceneVisuals,
    content: Res<outliner::Content>,
    content_controls: ContentControls,
) {
    let (
        keys,
        focus,
        particles,
        tabs,
        tab_controls,
        export_dialog,
        asset_preview,
        quests,
        spawn_groups,
        asset_search,
        account_picker,
    ) = input;
    let Some(mut report) = report else {
        return;
    };
    report.elapsed += time.delta_secs();
    let Ok(window) = windows.single() else {
        return;
    };
    if keys.just_pressed(KeyCode::F12) && window.focused {
        let path = report.path.with_extension("png");
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>| {
                event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .expect("GPU pixels")
                    .save(&path)
                    .expect("Save native capture");
            },
        );
    }
    if report.elapsed < 0.2 {
        return;
    }
    report.elapsed = 0.;
    let Ok((camera, transform, projection)) = cameras.single() else {
        return;
    };
    let project = |point: Vec3| {
        camera.world_to_viewport(transform, point).ok().map(|p| {
            session
                .ui
                .viewport_to_window(p, window.scale_factor())
                .to_array()
        })
    };
    let objects: Vec<_> = session.editor.zone().objects.iter().map(|object| {
        serde_json::json!({"id":object.id,"screen":project(Vec3::from_array(session.editor.zone().local(object.pose.position).unwrap()))})
    }).collect();
    let buttons: Vec<_> = controls.iter().filter_map(|((action,pressed,..),_,_,_,node,t,hovered,..)| {
        let action = action?;
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"action":format!("{action:?}"),"pressed":pressed,"hovered":hovered.0,"centre":(t.affine().translation * factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect();
    let mut value = serde_json::json!({"save_path":session.path.as_ref().map(|path|path.to_string_lossy()),"left_mouse_down":mouse.pressed(MouseButton::Left),"revision":session.editor.revision(),"mesh_assets":meshes.len(),"history":session.editor.history(),
        "document":session.editor.zone(),"selected":session.selected,"snap":session.snap,"status":session.status,
        "dragging":session.drag.is_some() || session.stroke.is_some(),"vegetation_mode":session.vegetation_mode,"plant_selected":session.plant_selected,"scatter_radius":session.scatter_radius,"scatter_count":session.scatter_count,
        "plants":session.editor.zone().plants.iter().map(|p|serde_json::json!({"id":p.id,"screen":project(Vec3::from_array(session.editor.zone().local(p.position).unwrap())+Vec3::Y*if p.kind==theia_zone::vegetation::PlantKind::Tree{3.2*p.scale as f32}else{0.6*p.scale as f32})})).collect::<Vec<_>>(),
        "marker_visible_entities":scene_visuals.iter().filter(|(marker,_,_,v,..)|*marker && **v != Visibility::Hidden).count(),"marker_mode":session.marker_mode,"marker_selected":session.marker_selected,"marker_palette":format!("{:?}",session.marker_palette),"terrain_mode":session.terrain_mode,"terrain_paint":session.terrain_paint,"path_mode":session.path_mode,"path_selected":session.path_selected,"path_point":session.path_point,
        "path_handles":session.editor.zone().paths.iter().find(|p|Some(p.id)==session.path_selected).map(|p|p.points.iter().map(|&point|project(Vec3::from_array(session.editor.zone().local(point).unwrap()))).collect::<Vec<_>>()),"path_draft_points":session.path_draft.as_ref().map(|d|d.path.points.len()),"brush_radius":session.brush_radius,"brush_tool":format!("{:?}",session.brush_tool),"play_feet":session.play.as_ref().map(|p| p.feet.metres()),
        "objects":objects,"buttons":buttons,"viewport":[window.width(),window.height()],
        "markers":session.editor.zone().markers.iter().map(|m|serde_json::json!({"id":m.id,"screen":project(Vec3::from_array(session.editor.zone().local(m.position).unwrap())+Vec3::Y*0.5)})).collect::<Vec<_>>(),
        "guides":session.editor.zone().paths.iter().map(|p|serde_json::json!({"id":p.id,"screen":project((Vec3::from_array(session.editor.zone().local(p.points[0]).unwrap())+Vec3::from_array(session.editor.zone().local(p.points[1]).unwrap()))/2.)})).collect::<Vec<_>>(),
        "guide_points":[project(Vec3::new(48.,0.,88.)),project(Vec3::new(64.,0.,88.)),project(Vec3::new(56.,0.,84.)),project(Vec3::new(64.,0.,80.)),project(Vec3::new(72.,0.,84.))],
        "terrain_points":[project(Vec3::new(64.,0.,64.)),project(Vec3::new(72.,0.,64.)),project(Vec3::new(48.,0.,64.))],
        "ground":[project(Vec3::new(3.,0.,4.)),project(Vec3::new(5.,0.,4.))],
        "gpu":{"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device":adapter.device}});
    value["spawn_group_selected"] = session.spawn_group_selected.into();
    value["particle_selected"] = session.particle_selected.into();
    value["exit_confirmation"] = session.exit_confirmation.into();
    value["grid"] = serde_json::json!({"enabled":view.show_grid,"spacing":authoring_grid::spacing(&session,&view),"surface":authoring_grid::surface(&session,&view),"surface_visible":scene_visuals.iter().any(|(_,_,_,v,_,_,_,_,grid,..)|grid && *v!=Visibility::Hidden),
        "lines":scene_visuals.iter().filter_map(|(_,_,_,visibility,_,transform,_,line,..)|line.map(|line|serde_json::json!({"slot":line.slot,"along_first":line.along_first,"visible":*visibility!=Visibility::Hidden,"position":transform.map(|t|t.translation.to_array()),"scale":transform.map(|t|t.scale.to_array())}))).collect::<Vec<_>>()});
    value["move_step_metres"] = session.move_step.into();
    value["middle_mouse_down"] = mouse.pressed(MouseButton::Middle).into();
    value["right_mouse_down"] = mouse.pressed(MouseButton::Right).into();
    value["window_title"] = serde_json::json!(window.title);
    value["plant_point"] = serde_json::json!(session.plant_point);
    value["show_move_handles"] = session.show_move_handles.into();
    value["camera_position"] = serde_json::json!(transform.translation().to_array());
    value["camera_projection"] = match projection {
        Projection::Orthographic(o) => {
            serde_json::json!({"kind":"Orthographic","height":match o.scaling_mode { bevy::camera::ScalingMode::FixedVertical {viewport_height}=>Some(viewport_height),_=>None }})
        }
        Projection::Perspective(p) => serde_json::json!({"kind":"Perspective","fov":p.fov}),
        _ => serde_json::json!({"kind":"Custom"}),
    };
    value["placement_preview"] = serde_json::json!(
        session
            .placement_preview
            .as_ref()
            .map(|preview| serde_json::json!({"object":preview.object,"error":preview.error}))
    );
    value["placement_ghost"] = serde_json::json!(scene_visuals.iter().find_map(|(_,_,_,visibility,_,transform,_,_,_,ghost)| {
        ghost.then(||serde_json::json!({"visible":*visibility!=Visibility::Hidden,"position":transform.map(|t|t.translation.to_array()),"scale":transform.map(|t|t.scale.to_array())}))
    }));
    value["network_playtest"] = serde_json::json!({"active":session.network_active});
    value["move_preview"] = serde_json::json!(session.drag.as_ref().map(|d| &d.preview));
    value["move_handles"] = serde_json::json!(
        scene_visuals
            .iter()
            .filter_map(|(_, _, _, visibility, mesh, transform, handle, ..)| {
                let handle = handle?;
                if !handle.head {
                    return None;
                }
                let transform = transform?;
                let mut invalid_target=None;
                let (screen,target)=if handle.tool==gizmo::Tool::Rotate {
                    let radius=transform.scale.x;
                    (transform.translation+Vec3::X*radius,transform.translation+Vec3::new(radius,0.,-radius)*std::f32::consts::FRAC_1_SQRT_2)
                } else if handle.tool==gizmo::Tool::Scale {
                    let centre=transform_target::selected(&session)
                        .and_then(|(_,pose)|session.editor.zone().local(pose.position).ok()).map(Vec3::from_array).unwrap_or(transform.translation);
                    let extent=transform.translation.distance(centre);
                    invalid_target=project(centre-Vec3::Y*extent*0.25);
                    (transform.translation,transform.translation+Vec3::Y*extent*0.5)
                } else { (transform.translation,transform.translation+Vec3::AXES[handle.axis]*2.) };
                Some(serde_json::json!({"axis":handle.axis,"tool":format!("{:?}",handle.tool),"visible":*visibility!=Visibility::Hidden,
                    "mesh":mesh.map(|m|format!("{:?}",m.0.id())),"screen":project(screen),"target":project(target),"invalid_target":invalid_target}))
            })
            .collect::<Vec<_>>()
    );
    let mut content_buttons = vec![];
    if session.play.is_none() {
        for (control, node, computed, transform, hovered, disabled) in &content_controls {
            if node.display == Display::None {
                continue;
            }
            let factor = computed.inverse_scale_factor() * scale.0;
            let (target, label) = match *control {
                outliner::Control::Row(index) => match content.row(index) {
                    Some(entry) => (format!("{:?}", entry.target), entry.label.clone()),
                    None => continue,
                },
                outliner::Control::Previous => ("Previous".into(), "Previous".into()),
                outliner::Control::Next => ("Next".into(), "Next".into()),
                outliner::Control::Filter(filter) => {
                    (format!("Filter({filter:?})"), format!("{filter:?}"))
                }
                outliner::Control::Search => ("Search".into(), "Find...".into()),
                outliner::Control::ClearSearch => ("ClearSearch".into(), "Clear search".into()),
            };
            content_buttons.push(serde_json::json!({"target":target,"label":label,"hovered":hovered.0,"disabled":disabled,
                "centre":(transform.affine().translation*factor).to_array(),"size":(computed.size()*factor).to_array()}));
        }
    }
    value["content_list"] = serde_json::json!({"visible":session.play.is_none(),"page":content.page,
        "pages":content.pages(),"count":content.entries.len(),"controls":content_buttons,
        "filter":format!("{:?}",content.filter),"total":content.total,
        "query":content.query,
        "scroll_events":content.scroll_events,"last_scroll":content.last_scroll});
    value["workspace_controls"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,_,_,menu_item,..),_,_,_,node,t,hovered,control,setting,value,range,..)| {
        let target = match (control,setting) { (Some(c),_)=>format!("{c:?}"),(_,Some(s))=>format!("{s:?}"),_=>return None };
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"target":target,"hovered":hovered.0,"centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array(),
            "menu_item":menu_item,"value":value.map(|v|v.0),"range":range.map(|r|[r.start(),r.end()])}))
    }).collect::<Vec<_>>());
    value["selectors"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,_,_,_,selector,..),_,_,_,node,t,hovered,..)| {
        let selector = selector?;
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"target":format!("{:?}",selector.kind),"menu":selector.kind.title(),
            "value":selector.kind.value(&session,&view),"enabled":selector.kind.enabled(&session,&view),"hovered":hovered.0,
            "centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["asset_preview"] = asset_preview.snapshot();
    value["attribute_summary"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,_,_,_,_,field,_,_,text),_,_,_,node,t,..)| {
        let field = field?; let text = text?;
        if node.size().min_element() <= 0. { return None; }
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"field":field.0,"value":text.value().to_string(),
            "centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["npc_quests"] = quests.snapshot(&session, scale.0);
    value["spawn_group_preview"] = spawn_groups.snapshot();
    value["spawn_group_fields"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,entity,_,_,_,_,_,field,text),_,_,_,node,t,..)| {
        let field = field?;
        if !matches!(field, properties::InlineInput::SpawnCreature | properties::InlineInput::SpawnModel | properties::InlineInput::SpawnSeed) || node.size().min_element() <= 0. { return None; }
        let text = text?;
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"field":format!("{field:?}"),"value":text.value().to_string(),"focused":focus.get()==Some(entity),
            "centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["inline_attributes"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,entity,_,_,_,_,_,field,text),_,_,_,node,t,..)| {
        let properties::InlineInput::Number(field) = field? else { return None }; let text = text?;
        if node.size().min_element() <= 0. { return None; }
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"field":field,"value":text.value().to_string(),"focused":focus.get()==Some(entity),
            "centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["inline_names"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,entity,_,_,_,_,_,field,text),_,_,_,node,t,..)| {
        if !matches!(field?, properties::InlineInput::Name) || node.size().min_element() <= 0. { return None; }
        let text = text?;
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"value":text.value().to_string(), "focused":focus.get()==Some(entity),
            "centre":(t.affine().translation*factor).to_array(), "size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["scroll_panes"] = serde_json::json!(controls.iter().filter_map(|((_,_,position,pane,..),_,_,_,node,t,hovered,..)| {
        let pane = pane?;
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"scope":format!("{pane:?}"),"position":position.map(|p|p.0.to_array()),"hovered":hovered.0,
            "centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array(),
            "limit":((node.content_size()-node.size())*factor).max(Vec2::ZERO).to_array()}))
    }).collect::<Vec<_>>());
    value["viewport_toolbar"] = serde_json::json!({
        "bounds": viewport_tools::bounds(window, &session).map(|rect| [rect.min.x, rect.min.y, rect.width(), rect.height()]),
        "owns_pointer": session.ui.overlay_owned,
        "tool": format!("{:?}", session.gizmo_tool)
    });
    value["asset_search"] = serde_json::json!(asset_search.single().ok().map(|(node, pose, text)| {
        let factor = node.inverse_scale_factor() * scale.0;
        serde_json::json!({"value":text.value().to_string(),"centre":(pose.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()})
    }));
    value["property_tabs"] = serde_json::json!({
        "active": format!("{:?}", tabs.active),
        "controls": tab_controls.iter().map(|(tab, node, pose)| {
            let factor = node.inverse_scale_factor() * scale.0;
            serde_json::json!({"tab":format!("{tab:?}"), "centre":(pose.affine().translation*factor).to_array(), "size":(node.size()*factor).to_array()})
        }).collect::<Vec<_>>()
    });
    value["ui_layout"] = serde_json::json!({
        "sidebar":[0.,workspace::HEADER,sidebar::WIDTH,window.height()-workspace::HEADER-status_bar::HEIGHT],
        "browser":[sidebar::WIDTH,workspace::HEADER,session.ui.width,session.ui.explorer_height(window.height())],
        "inspector":[sidebar::WIDTH,session.ui.properties_top(window.height()),session.ui.width,if session.ui.properties_collapsed {panels::TITLE_HEIGHT} else {window.height()-status_bar::HEIGHT-session.ui.properties_top(window.height())}],
        "panel_collapsed":{"explorer":session.ui.explorer_collapsed,"properties":session.ui.properties_collapsed},
        "canvas":[session.ui.canvas_left(),workspace::HEADER,window.width()-session.ui.canvas_left(),window.height()-workspace::HEADER-session.ui.canvas_bottom(window.height(), session.play.is_none())],
        "status_bar":[0.,window.height()-status_bar::HEIGHT,window.width(),status_bar::HEIGHT],
        "asset_browser":{"open":session.ui.assets_open && session.play.is_none(),"height":session.ui.canvas_bottom(window.height(),session.play.is_none())-status_bar::HEIGHT,"selected":session.assets.selected,"category":session.assets.category},
        "resizing":session.ui.input_owned
    });
    value["menus"] = serde_json::json!(controls.iter().filter_map(|(_,_,_,_,node,t,hovered,_,_,_,_,_,_,trigger,popup)| {
        let (name,role)=trigger.map(|m|(m.0,"button")).or_else(||popup.map(|m|(m.0,"popup")))?;
        let factor=node.inverse_scale_factor()*scale.0;
        Some(serde_json::json!({"name":name,"role":role,"hovered":hovered.0,"centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["sections"] = serde_json::json!(controls.iter().filter_map(|(_,_,_,_,node,t,hovered,_,_,_,_,_,section,..)| {
        let section=section?;
        let factor=node.inverse_scale_factor()*scale.0;
        Some(serde_json::json!({"title":section.title,"expanded":section.expanded,"hovered":hovered.0,
            "centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["file_controls"] = serde_json::json!(controls.iter().filter_map(|(_,_,_,_,node,t,hovered,_,_,_,_,control,..)| {
        let control=control?;
        let factor=node.inverse_scale_factor()*scale.0;
        let entry=match control { file_picker::Control::Row(index)=>session.file_dialog.as_ref().and_then(|d|d.row(*index)),_=>None };
        Some(serde_json::json!({"target":format!("{control:?}"),"hovered":hovered.0,"centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array(),
            "entry":entry.map(|e|serde_json::json!({"name":e.name,"path":e.path.to_string_lossy(),"directory":e.directory}))}))
    }).collect::<Vec<_>>());
    value["export_dialog"] = export_dialog.snapshot(&session, scale.0, &focus);
    value["account_picker"] = account_picker.snapshot(scale.0, &focus);
    value["particle_workspace"] = particles.snapshot();
    value["file_dialog"] = serde_json::json!(session.file_dialog.as_ref().map(|d|serde_json::json!({"mode":format!("{:?}",d.mode),"folder":d.folder.to_string_lossy(),"text":d.text,"selection":d.selection,"page":d.page,"pages":d.pages(),"overwrite":d.overwrite})));
    value["name_controls"] = serde_json::json!(controls.iter().filter_map(|(_,control,_,_,node,t,hovered,..)| {
        let control = control?;
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"action":format!("{control:?}"),"hovered":hovered.0,"centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()}))
    }).collect::<Vec<_>>());
    value["transform_controls"] = serde_json::json!(controls.iter().filter_map(|((_,_,_,_,entity,label,..),_,control,field,node,t,hovered,_,_,value,range,..)| {
        let target = match (control,field,label) {
            (Some(control),_,_) => format!("{control:?}"),
            (_,Some(field),_) => format!("Field({})",field.0),
            (_,_,Some(label)) => format!("Label({})",label.0),
            _ => return None,
        };
        let factor = node.inverse_scale_factor() * scale.0;
        Some(serde_json::json!({"target":target,"entity":entity.to_bits(),"hovered":hovered.0,"centre":(t.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array(),
            "value":value.map(|value|value.0),"range":range.map(|range|[range.start(),range.end()])}))
    }).collect::<Vec<_>>());
    value["transform_edit"] = serde_json::json!(session.transform_edit.as_ref().map(|d| serde_json::json!({"id":d.id,"fields":d.fields,"active":d.active,"selection":d.selection})));
    value["focus"] = serde_json::json!(focus.get().map(Entity::to_bits));
    value["input_elapsed"] = time.elapsed_secs_f64().into();
    value["control_down"] =
        (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)).into();
    value["shift_down"] =
        (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)).into();
    value["name_selection"] = serde_json::json!(session.name_edit.as_ref().map(|d| d.selection));
    value["name_edit"] = serde_json::json!(session.name_edit.as_ref().map(|d| &d.text));
    value["text_form_kind"] =
        serde_json::json!(session.name_edit.as_ref().map(|d| if d.is_authoring() {
            "rename"
        } else {
            "search"
        }));
    value["cursor"] = serde_json::json!(window.cursor_position().map(|p| p.to_array()));
    value["focused"] = window.focused.into();
    value["unsaved_changes"] = session.is_dirty().into();
    value["flatten_height"] = session.flatten_height.into();
    value["brush_strength"] = session.brush_strength.into();
    value["sculpt_blend"] = session.sculpt_blend.into();
    value["paint_blend"] = session.paint_blend.into();
    value["stroke_edits"] = serde_json::json!(session.stroke.as_ref().map(|s| s.commands()));
    value["stroke_progress"] = serde_json::json!(
        session
            .stroke
            .as_ref()
            .map(|s| serde_json::json!({"stamps":s.stamps(),"limit":theia_zone::MAX_EDITS}))
    );
    value["terrain_preview_heights"] = serde_json::json!(
        session
            .stroke
            .as_ref()
            .and_then(|s| s.draft.zone().terrain.as_ref().map(|p| &p.heights))
    );
    value["spawn_visual"] = serde_json::json!(scene_visuals.iter().find_map(|(_,_,spawn,visibility,handle,transform,..)| {
        if !spawn { return None; }
        let transform = transform?;
        let handle = handle?;
        let mesh = meshes.get(&handle.0)?;
        let bevy::mesh::VertexAttributeValues::Float32x3(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)? else { return None; };
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for &p in positions {
            let point = transform.transform_point(Vec3::from_array(p));
            minimum = minimum.min(point);
            maximum = maximum.max(point);
        }
        Some(serde_json::json!({"visible":*visibility!=Visibility::Hidden,"mesh":format!("{:?}",handle.0.id()),"bounds":[minimum.to_array(),maximum.to_array()],
            "screen":project(Vec3::from_array(session.editor.zone().local(session.editor.zone().spawn).ok()?)+Vec3::Y*1.2)}))
    }));
    value["brush_preview"] = serde_json::json!(scene_visuals.iter().find_map(|(_,cursor,_,visibility,handle,..)| {
        if !cursor { return None; }
        let handle = handle?;
        let mesh = meshes.get(&handle.0)?;
        let bevy::mesh::VertexAttributeValues::Float32x3(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)? else { return None; };
        let colours = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) => Some(colours),
            _ => None,
        };
        let samples: Vec<_> = colours.into_iter().flatten().enumerate().filter(|(_,colour)| colour[3]>0.).map(|(i,colour)| serde_json::json!({"index":i,"position":positions[i],"alpha":colour[3]})).collect();
        Some(serde_json::json!({"visible":*visibility!=Visibility::Hidden,"mesh":format!("{:?}",handle.0.id()),"vertices":positions.len(),"triangles":mesh.indices().map_or(0,|i|i.len()/3),"samples":samples}))
    }));
    value["brush_point"] = serde_json::json!(session.brush_point.map(|p| p.to_array()));
    value["view"] = serde_json::json!({"target":view.target.to_array(),"distance":view.distance,"yaw":view.yaw,"pitch":view.pitch,"preset":format!("{:?}",view.preset)});
    let temp = report.path.with_extension("tmp");
    if let Err(error) = std::fs::write(&temp, serde_json::to_vec_pretty(&value).unwrap())
        .and_then(|()| std::fs::rename(temp, &report.path))
    {
        warn!("Qualification report: {error}");
    }
}
