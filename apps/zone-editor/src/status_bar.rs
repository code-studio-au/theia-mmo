//! Shared bottom edge: transient feedback, selection context and explicit save state.
use super::*;

pub(super) const HEIGHT: f32 = 28.;
#[derive(Component)]
pub(super) struct Context;
#[derive(Component)]
pub(super) struct SaveState;

pub(super) fn spawn(commands: &mut Commands) {
    commands
        .spawn((
            panels::Region::Footer,
            GlobalZIndex(20),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(0),
                height: px(HEIGHT),
                padding: UiRect::horizontal(px(12)),
                border: UiRect::top(px(1)),
                align_items: AlignItems::Center,
                column_gap: px(16),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|bar| {
            bar.spawn((
                Summary,
                theme::label("", 12., theme::MUTED),
                TextLayout::no_wrap(),
                Node {
                    min_width: px(0),
                    flex_grow: 1.,
                    flex_basis: px(0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));
            bar.spawn((
                Context,
                theme::label("", 12., theme::MUTED),
                TextLayout::no_wrap(),
                Node {
                    min_width: px(0),
                    max_width: percent(35),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));
            bar.spawn((
                SaveState,
                theme::label("", 12., theme::BODY),
                TextLayout::no_wrap(),
                Node {
                    flex_shrink: 0.,
                    ..default()
                },
            ));
        });
}
fn single_line(value: &str, limit: usize) -> String {
    let mut chars = value.chars().map(|c| if c.is_control() { ' ' } else { c });
    let mut result: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}
pub(super) fn message(session: &Session) -> String {
    let hints = if session.play.is_some() {
        "W/S Move · A/D Turn · Q/E Strafe · Space Jump · F5 Return"
    } else if session.inline_editing {
        "Enter Apply · Esc Cancel · Ctrl S Save"
    } else if session.has_modal() {
        "Enter Confirm · Esc Cancel"
    } else if session.path_draft.is_some() {
        "Click Add point · Enter Finish · Backspace Remove point · Esc Cancel"
    } else if session.palette.is_some()
        || session.plant_palette.is_some()
        || session.marker_palette.is_some()
    {
        "Click Place · Esc Select · MMB Pan · Wheel Zoom"
    } else if session.terrain_mode {
        "Drag Sculpt/paint · Ctrl Z Undo · MMB Pan · Wheel Zoom"
    } else {
        "Click Select · Double-click Frame · Drag Transform · MMB Pan · Wheel Zoom"
    };
    let feedback = if let Some(error) = session
        .placement_preview
        .as_ref()
        .and_then(|preview| preview.error.as_deref())
    {
        error
    } else if session.status.starts_with("Saved ") {
        "File saved"
    } else {
        &session.status
    };
    format!("{}  |  {hints}", single_line(feedback, 180))
}
fn context(session: &Session) -> String {
    let zone = session.editor.zone();
    let selection = outliner::selection(session)
        .map(|target| match target {
            outliner::Target::Object(id)
            | outliner::Target::Guide(id)
            | outliner::Target::Marker(id)
            | outliner::Target::Plant(id)
            | outliner::Target::Particle(id) => {
                single_line(zone.authoring_name(id).unwrap_or("Selection"), 48)
            }
            outliner::Target::Terrain => "Terrain".into(),
            outliner::Target::Spawn => "Player spawn".into(),
        })
        .unwrap_or_else(|| "No selection".into());
    format!("{selection}  |  {} objects", zone.objects.len())
}
pub(super) fn sync(
    session: Res<Session>,
    mut contexts: Query<&mut Text, With<Context>>,
    mut states: Query<&mut Text, (With<SaveState>, Without<Context>)>,
) {
    for mut text in &mut contexts {
        **text = context(&session);
    }
    for mut text in &mut states {
        **text = if session.is_dirty() {
            "Unsaved changes · Ctrl S"
        } else {
            "Saved"
        }
        .into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_is_one_bounded_line_even_with_multiline_errors_and_unicode() {
        assert_eq!(single_line("one\ntwo\tthree", 30), "one two three");
        assert_eq!(single_line("树木🌿tree", 3), "树木🌿…");
        assert_eq!(single_line("tree", 4), "tree");
    }

    #[test]
    fn save_state_and_selection_stay_visible_beside_long_feedback() {
        let zone = Zone::default();
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::from("test.json"),
            Some(zone),
        );
        session.status = "Long feedback ".repeat(200);
        let source = session.editor.zone().clone();
        let history = session.editor.history();
        let mut app = App::new();
        app.insert_resource(session)
            .add_systems(Startup, setup_ui)
            .add_systems(Update, sync);
        crate::headless_ui::add_layout(&mut app);
        for _ in 0..3 {
            app.update();
        }
        let (node, pose) = app
            .world_mut()
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<panels::Region>>()
            .single(app.world())
            .unwrap();
        assert_eq!(node.size(), Vec2::new(3440., HEIGHT));
        assert_eq!(
            pose.affine().translation,
            Vec2::new(1720., 1440. - HEIGHT / 2.)
        );
        for (node, pose, text) in app.world_mut().query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform, &Text), Or<(With<SaveState>, With<Context>)>>().iter(app.world()) {
            assert!(node.size().min_element() > 0.);
            assert!(pose.affine().translation.y - node.size().y / 2. >= 1440. - HEIGHT);
            assert!(pose.affine().translation.y + node.size().y / 2. <= 1440.);
            assert!(pose.affine().translation.x + node.size().x / 2. <= 3440.);
            assert!(!text.contains('\n'));
        }
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), history);
        assert!(!app.world().resource::<Session>().is_dirty());
    }

    #[test]
    fn viewport_and_stacked_dock_leave_the_status_bar_clear() {
        let layout = panels::Layout::default();
        for height in [420., 720., 1440.] {
            let explorer = layout.explorer_height(height);
            let properties_top = layout.properties_top(height);
            assert!(explorer >= panels::TITLE_HEIGHT);
            assert!(properties_top + panels::TITLE_HEIGHT <= height - HEIGHT);
            let mut folded = panels::Layout::default();
            folded.properties_collapsed = true;
            assert_eq!(
                folded.properties_top(height) + panels::TITLE_HEIGHT,
                height - HEIGHT
            );
            folded.explorer_collapsed = true;
            assert_eq!(folded.explorer_height(height), panels::TITLE_HEIGHT);
        }
    }
}
