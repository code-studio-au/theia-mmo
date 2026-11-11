//! Shared-window host for the original Sprinkles UI and its isolated preview.
use super::*;
use bevy::ecs::system::ScheduleSystem;
use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy_sprinkles::ParticleSystemRuntime;
use bevy_sprinkles_editor::{
    ParticleAuthoringPlugin,
    hosting::{
        AuthoringHost, AuthoringUiRoot, AuthoringUiUpdate, AuthoringUpdate, ReturnToHostRequested,
    },
    viewport::{EditorCamera, EditorParticlePreview},
};
use std::collections::HashMap;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ZoneAuthoring;

#[derive(Resource, Default)]
struct Bridge {
    visible: bool,
    return_requested: bool,
    defer_return: bool,
    title: String,
    hidden_roots: HashMap<Entity, Display>,
    paused: HashMap<Entity, bool>,
}

pub(super) trait ZoneAppExt {
    fn add_zone_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;
}

impl ZoneAppExt for App {
    fn add_zone_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.add_systems(Update, systems.in_set(ZoneAuthoring))
    }
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(AuthoringHost::embedded(1))
        .init_resource::<Bridge>()
        .add_plugins(ParticleAuthoringPlugin)
        .configure_sets(Update, ZoneAuthoring.run_if(zone_active))
        .add_observer(request_return)
        .add_systems(
            Update,
            switch
                .before(ZoneAuthoring)
                .before(AuthoringUpdate)
                .before(AuthoringUiUpdate),
        );
}

pub(super) fn zone_active(host: Option<Res<AuthoringHost>>) -> bool {
    host.is_none_or(|host| !host.active)
}

fn request_return(
    _: On<ReturnToHostRequested>,
    mut bridge: ResMut<Bridge>,
    mut focus: ResMut<InputFocus>,
) {
    focus.clear();
    bridge.return_requested = true;
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn switch(
    mut commands: Commands,
    mut bridge: ResMut<Bridge>,
    mut host: ResMut<AuthoringHost>,
    mut requests: ResMut<workspace::Requests>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    mut roots: Query<(Entity, &mut Node, Has<AuthoringUiRoot>), Without<ChildOf>>,
    mut cameras: Query<(&mut Camera, Has<EditorCamera>, Has<viewport::ZoneCamera>)>,
    mut previews: Query<(Entity, &mut ParticleSystemRuntime), With<EditorParticlePreview>>,
    mut windows: Query<(Entity, &mut Window)>,
) {
    if bridge.defer_return {
        host.active = false;
        bridge.defer_return = false;
    } else if std::mem::take(&mut bridge.return_requested) {
        // Give original blur/commit handlers one active frame before suspending.
        bridge.defer_return = true;
    }
    if std::mem::take(&mut requests.particle_workspace) && !workspace::busy(&session) {
        host.active = true;
    }
    if host.active != bridge.visible {
        focus.clear();
        if let Ok((entity, mut window)) = windows.single_mut() {
            commands
                .entity(entity)
                .insert(bevy::window::CursorIcon::default());
            if host.active {
                bridge.title = window.title.clone();
                window.title = "Theia — Particle workspace (Sprinkles)".into();
            } else {
                window.title = bridge.title.clone();
            }
        }
        session.status = if host.active {
            "Sprinkles particle workspace"
        } else {
            "Returned to zone authoring"
        }
        .into();
        bridge.visible = host.active;
    }
    for (entity, mut node, authoring) in &mut roots {
        if authoring {
            let display = if host.active {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        } else if host.active {
            bridge.hidden_roots.entry(entity).or_insert(node.display);
            if node.display != Display::None {
                node.display = Display::None;
            }
        } else if let Some(display) = bridge.hidden_roots.remove(&entity) {
            node.display = display;
        }
    }
    for (mut camera, particle, zone) in &mut cameras {
        if particle {
            camera.is_active = host.active;
        }
        if zone {
            camera.is_active = !host.active;
        }
    }
    for (entity, mut runtime) in &mut previews {
        if host.active {
            if let Some(paused) = bridge.paused.remove(&entity) {
                runtime.paused = paused;
            }
        } else {
            bridge.paused.entry(entity).or_insert(runtime.paused);
            if !runtime.paused {
                runtime.paused = true;
            }
        }
    }
    bridge
        .hidden_roots
        .retain(|entity, _| roots.contains(*entity));
    bridge.paused.retain(|entity, _| previews.contains(*entity));
}

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(super) struct Report<'w, 's> {
    host: Res<'w, AuthoringHost>,
    state: Res<'w, bevy_sprinkles_editor::state::EditorState>,
    dirty: Res<'w, bevy_sprinkles_editor::state::DirtyState>,
    assets: Res<'w, Assets<bevy_sprinkles::asset::ParticlesAsset>>,
    runtime: Query<'w, 's, &'static ParticleSystemRuntime, With<EditorParticlePreview>>,
    fields: Query<
        'w,
        's,
        (
            Entity,
            &'static bevy::text::EditableText,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
        With<bevy_sprinkles_editor::ui::widgets::text_edit::EditorTextEdit>,
    >,
    parents: Query<'w, 's, &'static ChildOf>,
    bindings: Query<'w, 's, &'static bevy_sprinkles_editor::ui::components::binding::FieldBinding>,
    buttons: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
            Has<bevy_sprinkles_editor::hosting::ReturnToHostButton>,
            Has<bevy_sprinkles_editor::ui::components::playback_controls::PlayPauseButton>,
            Option<&'static Hovered>,
        ),
        With<bevy_sprinkles_editor::ui::widgets::button::CompatButton>,
    >,
    emitters: Query<
        'w,
        's,
        (
            &'static bevy_sprinkles::EmitterEntity,
            Option<&'static bevy::camera::visibility::RenderLayers>,
        ),
    >,
    cameras: Query<
        'w,
        's,
        (
            &'static Camera,
            Has<EditorCamera>,
            Has<viewport::ZoneCamera>,
        ),
    >,
}

impl Report<'_, '_> {
    pub(super) fn snapshot(&self) -> serde_json::Value {
        use bevy_sprinkles_editor::ui::components::binding::FieldAccessor;
        let project = self
            .state
            .current_project
            .as_ref()
            .and_then(|handle| self.assets.get(handle));
        let fields: Vec<_> = self.fields.iter().filter(|(_,_,node,_)|node.size().min_element()>0.).map(|(entity,text,node,pose)| {
            let field = self.parents.iter_ancestors(entity).find_map(|parent|self.bindings.get(parent).ok()).map(|binding|match &binding.accessor {
                FieldAccessor::Direct(path)=>path.clone(),
                FieldAccessor::VariantField{path,field_name}=>format!("{path}.{field_name}"),
            });
            serde_json::json!({"field":field,"value":text.value().to_string(),"centre":(pose.translation*node.inverse_scale_factor()).to_array(),"size":(node.size()*node.inverse_scale_factor()).to_array()})
        }).collect();
        let buttons: Vec<_> = self.buttons.iter().filter(|(node,..)|node.size().min_element()>0.).filter_map(|(node,pose,back,play,hovered)| {
            if !back && !play {return None;}
            Some(serde_json::json!({"kind":if back {"back"} else {"play"},"centre":(pose.translation*node.inverse_scale_factor()).to_array(),"hovered":hovered.is_some_and(|h|h.0)}))
        }).collect();
        serde_json::json!({"active":self.host.active,"dirty":self.dirty.has_unsaved_changes,"project":project.and_then(|asset|serde_json::to_value(asset).ok()),"fields":fields,"buttons":buttons,
            "runtime":self.runtime.iter().map(|r|serde_json::json!({"paused":r.paused,"loop":r.force_loop})).collect::<Vec<_>>(),
            "emitter_layers":self.emitters.iter().filter(|(emitter,_)|self.runtime.contains(emitter.parent_system)).map(|(_,layers)|layers.cloned().unwrap_or_default().iter().collect::<Vec<_>>()).collect::<Vec<_>>(),
            "cameras":self.cameras.iter().filter(|(_,source,zone)|*source||*zone).map(|(camera,source,_)|serde_json::json!({"kind":if source {"particle"} else {"zone"},"active":camera.is_active})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_isolates_real_zone_shortcuts_restores_roots_focus_cameras_and_manual_pause() {
        for origin in [[0.; 3], [128000000000.125, 20.25, -128000000000.5]] {
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
            let mut app = App::new();
            app.insert_resource(Session::new(
                editor,
                "unused.zone.json".into(),
                Some(zone.clone()),
            ))
            .insert_resource(View {
                target: Vec3::ZERO,
                yaw: 0.,
                pitch: 0.8,
                distance: 32.,
                preset: camera_views::Preset::Perspective,
                show_grid: true,
            })
            .insert_resource(AuthoringHost::embedded(1))
            .init_resource::<Bridge>()
            .init_resource::<workspace::Requests>()
            .init_resource::<outliner::Content>()
            .init_resource::<InputFocus>()
            .init_resource::<ButtonInput<KeyCode>>()
            .configure_sets(Update, ZoneAuthoring.run_if(zone_active))
            .add_zone_systems(crate::keyboard)
            .add_observer(request_return)
            .add_systems(Update, switch.before(ZoneAuthoring));
            let window = app
                .world_mut()
                .spawn(Window {
                    focused: true,
                    title: "Zone test".into(),
                    ..default()
                })
                .id();
            let zone_root = app.world_mut().spawn(Node::default()).id();
            let source_root = app
                .world_mut()
                .spawn((
                    AuthoringUiRoot,
                    Node {
                        display: Display::None,
                        ..default()
                    },
                ))
                .id();
            let zone_camera = app
                .world_mut()
                .spawn((viewport::ZoneCamera, Camera::default()))
                .id();
            let source_camera = app
                .world_mut()
                .spawn((
                    EditorCamera,
                    Camera {
                        is_active: false,
                        ..default()
                    },
                ))
                .id();
            let preview = app
                .world_mut()
                .spawn((EditorParticlePreview, ParticleSystemRuntime::default()))
                .id();
            app.update();
            app.world_mut()
                .resource_mut::<InputFocus>()
                .set(zone_root, bevy::input_focus::FocusCause::Pressed);
            app.world_mut()
                .resource_mut::<workspace::Requests>()
                .particle_workspace = true;
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyY);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
            assert_eq!(
                app.world().get::<Node>(zone_root).unwrap().display,
                Display::None
            );
            assert_eq!(
                app.world().get::<Node>(source_root).unwrap().display,
                Display::Flex
            );
            assert!(app.world().resource::<InputFocus>().get().is_none());
            assert!(!app.world().get::<Camera>(zone_camera).unwrap().is_active);
            assert!(app.world().get::<Camera>(source_camera).unwrap().is_active);
            assert!(
                !app.world()
                    .get::<ParticleSystemRuntime>(preview)
                    .unwrap()
                    .paused
            );
            app.world_mut()
                .get_mut::<ParticleSystemRuntime>(preview)
                .unwrap()
                .paused = true;
            app.world_mut().trigger(ReturnToHostRequested);
            app.update();
            assert!(app.world().resource::<AuthoringHost>().active);
            app.update();
            assert!(!app.world().resource::<AuthoringHost>().active);
            assert_eq!(
                app.world().get::<Node>(zone_root).unwrap().display,
                Display::Flex
            );
            assert_eq!(
                app.world().get::<Node>(source_root).unwrap().display,
                Display::None
            );
            assert_eq!(
                app.world().get::<Window>(window).unwrap().title,
                "Zone test"
            );
            assert!(app.world().get::<Camera>(zone_camera).unwrap().is_active);
            assert!(!app.world().get::<Camera>(source_camera).unwrap().is_active);
            assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
            assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyY);
            keys.clear();
            keys.press(KeyCode::KeyY);
            app.update();
            assert_eq!(
                app.world().resource::<Session>().editor.zone().name,
                "Temporary"
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.world_mut()
                .resource_mut::<workspace::Requests>()
                .particle_workspace = true;
            app.update();
            assert!(
                app.world()
                    .get::<ParticleSystemRuntime>(preview)
                    .unwrap()
                    .paused
            );
        }
    }
}
