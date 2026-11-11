//! Reuse Sprinkles' inspector for the selected, already-open local effect.
use super::*;
use bevy::input_focus::InputFocus;
use bevy_sprinkles::asset::ParticlesAsset;
use bevy_sprinkles_editor::{
    hosting::{AuthoringHost, AuthoringUiUpdate, AuthoringUpdate},
    project::SaveProjectEvent,
    state::{ActiveSidebarTab, DirtyState, EditorState, Inspectable, Inspecting, SidebarTab},
    ui::components::inspector::EditorInspectorPanel,
    ui::widgets::text_edit::EditorTextEdit,
};

#[derive(Component)]
struct Root;
#[derive(Component)]
struct SaveEffect;
#[derive(Component)]
struct Caption;
#[derive(Component)]
struct EmitterStep(i8);
#[derive(Resource, Default)]
struct PendingEmitter(Option<(u64, u64, AssetId<ParticlesAsset>, u8, i8, bool)>);
#[derive(Default)]
struct SyncState {
    closing: bool,
    last: Option<(u64, u64, AssetId<ParticlesAsset>)>,
}
#[derive(Resource, Default)]
struct InspectorLocation {
    panel: Option<Entity>,
    original_parent: Option<Entity>,
    original_node: Option<Node>,
}
type InlineRoots<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Node), (With<Root>, Without<EditorInspectorPanel>)>;
type OriginalInspector<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static ChildOf, &'static mut Node),
    (With<EditorInspectorPanel>, Without<Root>),
>;
#[derive(bevy::ecs::system::SystemParam)]
struct InspectorUi<'w, 's> {
    roots: InlineRoots<'w, 's>,
    inspectors: OriginalInspector<'w, 's>,
    labels: Query<'w, 's, &'static mut Text, With<Caption>>,
    parents: Query<'w, 's, &'static ChildOf>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<InspectorLocation>()
        .init_resource::<PendingEmitter>()
        .add_observer(save_effect)
        .add_observer(step_emitter)
        .add_systems(
            Update,
            restore_workspace
                .after(particle_workspace::ZoneAuthoring)
                .before(AuthoringUiUpdate)
                .before(AuthoringUpdate),
        )
        .add_zone_systems(sync.after(workspace::sync).before(AuthoringUiUpdate))
        .add_zone_systems((
            inline_save_shortcut.after(keyboard),
            save_requested.after(inline_save_shortcut),
        ));
}

fn inline_save_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditorTextEdit>>,
    host: Res<AuthoringHost>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if host.inline_ui
        && !host.active
        && ctrl
        && !shift
        && keys.just_pressed(KeyCode::KeyS)
        && windows.single().is_ok_and(|window| window.focused)
        && focus.get().is_some_and(|entity| inputs.contains(entity))
    {
        perform(Action::Save, &mut session, &mut view);
    }
}

fn save_requested(
    mut session: ResMut<Session>,
    editor: Res<EditorState>,
    dirty: Res<DirtyState>,
    host: Res<AuthoringHost>,
    mut commands: Commands,
) {
    if !session.particle_save_requested {
        return;
    }
    session.particle_save_requested = false;
    if !host.active
        && editor.current_project_path.is_some()
        && (dirty.has_unsaved_changes || host.inline_ui)
    {
        commands.trigger(SaveProjectEvent);
    }
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Root,
            Node {
                display: Display::None,
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: percent(100),
                align_items: AlignItems::Center,
                column_gap: px(4),
                ..default()
            })
            .with_children(|row| {
                for (step, label) in [(-1, "‹"), (1, "›")] {
                    row.spawn((
                        EmitterStep(step),
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        theme::Variant::Ghost,
                        bevy::input_focus::tab_navigation::TabIndex(0),
                        theme::button_node(),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_child(theme::label(label, 12., theme::BODY));
                }
                row.spawn((
                    Caption,
                    theme::label("Particle emitter", 12., theme::DISPLAY),
                ));
            });
            root.spawn((
                SaveEffect,
                Button,
                ActivateOnPress,
                Hovered::default(),
                theme::Variant::Default,
                bevy::input_focus::tab_navigation::TabIndex(0),
                theme::button_node(),
                BackgroundColor(theme::Variant::Default.background(false)),
            ))
            .with_child(theme::label("Save effect", 12., theme::BODY));
        });
}

#[allow(clippy::too_many_arguments)]
fn step_emitter(
    event: On<Activate>,
    buttons: Query<&EmitterStep>,
    session: Res<Session>,
    editor: Res<EditorState>,
    host: Res<AuthoringHost>,
    root: Res<particle_preview::LibraryRoot>,
    location: Res<InspectorLocation>,
    parents: Query<&ChildOf>,
    mut focus: ResMut<InputFocus>,
    mut pending: ResMut<PendingEmitter>,
) {
    let Ok(step) = buttons.get(event.entity) else {
        return;
    };
    let Some(outliner::Target::Particle(id)) = outliner::selection(&session) else {
        return;
    };
    let (Some(project), Some(inspecting)) = (&editor.current_project, editor.inspecting) else {
        return;
    };
    if !host.inline_ui
        || host.active
        || session.has_modal()
        || inspecting.kind != Inspectable::Emitter
        || !particle_preview::selected_matches_open_project(&session, &editor, &root)
    {
        return;
    }
    if focus.get().is_some_and(|entity| {
        location.panel.is_some_and(|panel| {
            entity == panel || parents.iter_ancestors(entity).any(|p| p == panel)
        })
    }) {
        focus.clear();
    }
    pending.0 = Some((
        session.document_id,
        id,
        project.id(),
        inspecting.index,
        step.0,
        true,
    ));
}

fn restore_workspace(
    mut commands: Commands,
    mut host: ResMut<AuthoringHost>,
    mut location: ResMut<InspectorLocation>,
    mut pending: ResMut<PendingEmitter>,
    mut inspectors: OriginalInspector,
) {
    if !host.active {
        return;
    }
    if host.inline_ui {
        host.inline_ui = false;
    }
    if pending.0.is_some() {
        pending.0 = None;
    }
    restore_inspector(&mut commands, &mut location, &mut inspectors);
}

fn restore_inspector(
    commands: &mut Commands,
    location: &mut InspectorLocation,
    inspectors: &mut OriginalInspector,
) {
    let (Some(panel), Some(parent)) = (location.panel, location.original_parent) else {
        return;
    };
    let Ok((_, current, mut node)) = inspectors.get_mut(panel) else {
        return;
    };
    if current.parent() != parent {
        commands.entity(parent).add_child(panel);
        if let Some(original) = location.original_node.take() {
            *node = original;
        }
    }
}

fn save_effect(
    event: On<Activate>,
    buttons: Query<(), With<SaveEffect>>,
    session: Res<Session>,
    editor: Res<EditorState>,
    root: Res<particle_preview::LibraryRoot>,
    host: Res<AuthoringHost>,
    mut commands: Commands,
) {
    if buttons.contains(event.entity)
        && host.inline_ui
        && !host.active
        && !session.has_modal()
        && session.play.is_none()
        && particle_preview::selected_matches_open_project(&session, &editor, &root)
    {
        commands.trigger(SaveProjectEvent);
    }
}

#[allow(clippy::too_many_arguments)]
fn sync(
    session: Res<Session>,
    mut editor: ResMut<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    root_path: Res<particle_preview::LibraryRoot>,
    mut host: ResMut<AuthoringHost>,
    mut sidebar: ResMut<ActiveSidebarTab>,
    tabs: Res<property_tabs::State>,
    mut ui: InspectorUi,
    mut location: ResMut<InspectorLocation>,
    mut commands: Commands,
    dirty: Res<DirtyState>,
    mut requests: ResMut<workspace::Requests>,
    preview: Res<particle_preview::State>,
    mut focus: ResMut<InputFocus>,
    mut pending: ResMut<PendingEmitter>,
    mut sync_state: Local<SyncState>,
) {
    let selection = match outliner::selection(&session) {
        Some(outliner::Target::Particle(id)) => Some(id),
        _ => None,
    };
    let project = editor.current_project.clone();
    if location.panel.is_none()
        && let Ok((panel, parent, _)) = ui.inspectors.single_mut()
    {
        location.panel = Some(panel);
        location.original_parent = Some(parent.parent());
    }
    let visible = !host.active
        && !session.ui.properties_collapsed
        && !session.has_modal()
        && tabs.active == property_tabs::Tab::Attributes
        && location.panel.is_some()
        && ui.roots.single().is_ok()
        && selection.is_some()
        && particle_preview::selected_matches_open_project(&session, &editor, &root_path)
        && project
            .as_ref()
            .and_then(|h| assets.get(h))
            .is_some_and(|asset| !asset.emitters.is_empty());
    if host.inline_ui && !visible && !sync_state.closing {
        sync_state.closing = true;
        pending.0 = None;
        if focus.get().is_some_and(|entity| {
            ui.roots.iter().any(|(root, _)| {
                entity == root || ui.parents.iter_ancestors(entity).any(|p| p == root)
            })
        }) {
            focus.clear();
        }
        return;
    }
    sync_state.closing = false;
    if host.inline_ui != visible {
        host.inline_ui = visible;
    }
    for (_, mut node) in &mut ui.roots {
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    if !visible {
        pending.0 = None;
        restore_inspector(&mut commands, &mut location, &mut ui.inspectors);
        sync_state.last = None;
        return;
    }
    if let (Some(panel), Ok((target, _))) = (location.panel, ui.roots.single())
        && let Ok((_, parent, mut node)) = ui.inspectors.get_mut(panel)
        && parent.parent() != target
    {
        location.original_node = Some(node.clone());
        node.width = percent(100);
        node.min_width = px(0);
        node.height = Val::Auto;
        node.min_height = px(0);
        node.flex_shrink = 0.;
        node.overflow = Overflow::visible();
        node.border = UiRect::ZERO;
        commands.entity(target).add_child(panel);
    }
    let project = project.expect("visible project");
    let asset = assets.get(&project).expect("visible asset");
    if editor.inspecting.is_none_or(|i| {
        i.kind != Inspectable::Emitter || usize::from(i.index) >= asset.emitters.len()
    }) {
        editor.inspecting = Some(Inspecting {
            kind: Inspectable::Emitter,
            index: 0,
        });
    }
    if sidebar.0 != SidebarTab::Outliner {
        sidebar.0 = SidebarTab::Outliner;
    }
    if let Some((document, placement, asset_id, original, step, wait)) = pending.0 {
        let current = editor.inspecting.as_ref().map(|value| value.index);
        if document != session.document_id
            || Some(placement) != selection
            || asset_id != project.id()
            || current != Some(original)
        {
            pending.0 = None;
        } else if wait {
            pending.0 = Some((document, placement, asset_id, original, step, false));
        } else {
            let count = asset.emitters.len().min(usize::from(u8::MAX) + 1) as i32;
            let next = (i32::from(original) + i32::from(step)).rem_euclid(count) as u8;
            editor.inspecting = Some(Inspecting {
                kind: Inspectable::Emitter,
                index: next,
            });
            pending.0 = None;
        }
    }
    let index = editor
        .inspecting
        .as_ref()
        .map_or(0, |i| usize::from(i.index));
    for mut label in &mut ui.labels {
        let caption = format!(
            "Emitter {} of {}{}",
            index + 1,
            asset.emitters.len(),
            if dirty.has_unsaved_changes {
                " · unsaved"
            } else {
                ""
            }
        );
        if **label != caption {
            **label = caption;
        }
    }
    let key = (
        session.document_id,
        selection.expect("visible selection"),
        project.id(),
    );
    if sync_state.last.as_ref() != Some(&key) {
        if !preview.previews_open_project(key.0, key.1, &project) {
            requests.particle_preview = true;
        }
        sync_state.last = Some(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input_focus::FocusCause;
    use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
    use std::time::{SystemTime, UNIX_EPOCH};
    use theia_zone::particles::ParticlePlacement;

    #[derive(Resource, Default)]
    struct Saves(u32);

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn selected_project_opens_inline_preview_saves_and_blurs_before_hiding() {
        let dir = std::env::temp_dir().join(format!(
            "theia-inline-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("library")).unwrap();
        let path = dir.join("library/vfx.fire.ron");
        std::fs::write(&path, b"fixture").unwrap();
        let mut zone = Zone {
            schema: 8,
            ..Zone::default()
        };
        zone.particles.push(ParticlePlacement {
            id: 7,
            name: "Fire".into(),
            effect: "vfx.fire".into(),
            position: [0.; 3],
            yaw_degrees: 0.,
            scale: 1.,
        });
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            dir.join("zone.json"),
            Some(zone),
        );
        session.particle_selected = Some(7);
        let mut assets = Assets::<ParticlesAsset>::default();
        let effect = assets.add(ParticlesAsset::new(
            "Fire".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default(), EmitterData::default()],
            vec![],
            false,
            default(),
        ));
        let mut editor = EditorState::default();
        editor.current_project = Some(effect);
        editor.current_project_path = Some(path);
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(assets)
            .insert_resource(editor)
            .insert_resource(particle_preview::LibraryRoot(dir.clone()))
            .insert_resource(AuthoringHost::embedded(1))
            .init_resource::<ActiveSidebarTab>()
            .init_resource::<DirtyState>()
            .init_resource::<property_tabs::State>()
            .init_resource::<workspace::Requests>()
            .init_resource::<particle_preview::State>()
            .init_resource::<InputFocus>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(View {
                target: Vec3::ZERO,
                yaw: 0.,
                pitch: 0.,
                distance: 10.,
                preset: camera_views::Preset::Perspective,
                show_grid: true,
            })
            .init_resource::<Saves>()
            .add_observer(|_: On<SaveProjectEvent>, mut saves: ResMut<Saves>| saves.0 += 1)
            .add_plugins(plugin);
        app.world_mut().spawn(Window::default());
        let root = app
            .world_mut()
            .spawn((
                Root,
                Node {
                    display: Display::None,
                    ..default()
                },
            ))
            .id();
        let original_parent = app.world_mut().spawn(Node::default()).id();
        let inspector = app
            .world_mut()
            .spawn((
                EditorInspectorPanel,
                Node {
                    width: px(320),
                    height: percent(100),
                    ..default()
                },
                ChildOf(original_parent),
            ))
            .id();
        let field = app
            .world_mut()
            .spawn((EditorTextEdit, ChildOf(inspector)))
            .id();
        let button = app.world_mut().spawn((SaveEffect, ChildOf(root))).id();
        let next = app.world_mut().spawn((EmitterStep(1), ChildOf(root))).id();
        app.update();
        assert!(app.world().resource::<AuthoringHost>().inline_ui);
        assert!(!app.world().resource::<AuthoringHost>().active);
        assert_eq!(
            app.world().get::<Node>(root).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<ChildOf>(inspector).unwrap().parent(),
            root
        );
        assert_eq!(
            app.world().get::<Node>(inspector).unwrap().width,
            percent(100)
        );
        assert_eq!(
            app.world()
                .resource::<EditorState>()
                .inspecting
                .unwrap()
                .index,
            0
        );
        assert!(
            app.world()
                .resource::<workspace::Requests>()
                .particle_preview
        );
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(field, FocusCause::Navigated);
        app.world_mut().trigger(Activate { entity: next });
        assert_eq!(app.world().resource::<InputFocus>().get(), None);
        app.update();
        assert_eq!(
            app.world()
                .resource::<EditorState>()
                .inspecting
                .unwrap()
                .index,
            0
        );
        app.update();
        assert_eq!(
            app.world()
                .resource::<EditorState>()
                .inspecting
                .unwrap()
                .index,
            1
        );
        app.world_mut().trigger(Activate { entity: button });
        app.update();
        assert_eq!(app.world().resource::<Saves>().0, 1);
        app.world_mut()
            .resource_mut::<Session>()
            .particle_save_requested = true;
        app.update();
        assert_eq!(app.world().resource::<Saves>().0, 2);
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(field, FocusCause::Navigated);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlLeft);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyS);
        app.update();
        assert_eq!(app.world().resource::<Saves>().0, 3);
        assert!(dir.join("zone.json").is_file());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(field, FocusCause::Navigated);
        app.world_mut().trigger(Activate { entity: next });
        app.world_mut().resource_mut::<Session>().particle_selected = None;
        app.update();
        assert!(app.world().resource::<PendingEmitter>().0.is_none());
        assert!(app.world().resource::<AuthoringHost>().inline_ui);
        assert_eq!(app.world().resource::<InputFocus>().get(), None);
        app.world_mut().trigger(Activate { entity: button });
        assert_eq!(app.world().resource::<Saves>().0, 3);
        app.update();
        assert!(!app.world().resource::<AuthoringHost>().inline_ui);
        assert_eq!(
            app.world().get::<Node>(root).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<ChildOf>(inspector).unwrap().parent(),
            original_parent
        );
        assert_eq!(app.world().get::<Node>(inspector).unwrap().width, px(320));
        app.world_mut().resource_mut::<Session>().particle_selected = Some(7);
        app.update();
        assert_eq!(
            app.world().get::<ChildOf>(inspector).unwrap().parent(),
            root
        );
        app.world_mut().resource_mut::<AuthoringHost>().active = true;
        app.update();
        assert_eq!(
            app.world().get::<ChildOf>(inspector).unwrap().parent(),
            original_parent
        );
        assert!(!app.world().resource::<AuthoringHost>().inline_ui);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
