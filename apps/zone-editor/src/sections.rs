//! Collapsible inspector sections adapted from Sprinkles' panel_section.rs.
//! Upstream 0e978f5, MIT; see third_party/sprinkles/PROVENANCE.md.
use super::*;

#[derive(Component)]
pub(super) struct Section {
    pub title: &'static str,
    pub body: Entity,
    pub expanded: bool,
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
struct Chevron;

pub(super) fn spawn(
    parent: &mut ChildSpawnerCommands,
    title: &'static str,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Root,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                border: UiRect::bottom(px(1)),
                flex_shrink: 0.,
                ..default()
            },
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|section| {
            let header = section
                .spawn((
                    Button,
                    theme::Variant::Ghost,
                    ActivateOnPress,
                    Hovered::default(),
                    Node {
                        width: percent(100),
                        min_height: px(32),
                        padding: UiRect::axes(px(4), px(8)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(theme::Variant::Ghost.background(false)),
                    children![(text(title, 14.)), (Chevron, text("▾", 14.))],
                ))
                .observe(toggle)
                .id();
            let body = section
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    padding: UiRect::bottom(px(12)),
                    ..default()
                })
                .with_children(body)
                .id();
            section.commands().entity(header).insert(Section {
                title,
                body,
                expanded: true,
            });
        });
}

fn toggle(
    event: On<Activate>,
    session: Res<Session>,
    windows: Query<&Window>,
    mut sections: Query<(&mut Section, &Children)>,
    mut nodes: Query<&mut Node>,
    mut chevrons: Query<&mut Text, With<Chevron>>,
    mut focus: Option<ResMut<bevy::input_focus::InputFocus>>,
) {
    if workspace::busy(&session) || !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    let Ok((mut section, children)) = sections.get_mut(event.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(section.body) else {
        return;
    };
    section.expanded = !section.expanded;
    node.display = if section.expanded {
        Display::Flex
    } else {
        Display::None
    };
    // A hidden field must never retain keyboard ownership. Section activation
    // itself is read-only and cannot create history or change tool preferences.
    if let Some(focus) = focus.as_mut() {
        focus.set(event.entity, bevy::input_focus::FocusCause::Navigated);
    }
    for child in children.iter() {
        if let Ok(mut text) = chevrons.get_mut(child) {
            **text = if section.expanded { "▾" } else { "▸" }.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input_focus::InputFocus;

    #[test]
    fn section_activation_retains_exact_source_redo_preferences_and_keyboard_access() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            let zone = Zone {
                origin,
                spawn: [origin[0], origin[1] + 0.1, origin[2]],
                ..default()
            };
            let mut editor = Editor::new(zone.clone()).unwrap();
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::SetSpawn {
                        position: [origin[0] + 4., origin[1] + 0.1, origin[2]],
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
                "unused.json".into(),
                Some(zone.clone()),
            ));
            app.world_mut().resource_mut::<Session>().placement_size = [8.25, 3.5, 2.25];
            let window = app
                .world_mut()
                .spawn(Window {
                    focused: true,
                    ..default()
                })
                .id();
            let body = app.world_mut().spawn(Node::default()).id();
            let hidden_field = app.world_mut().spawn_empty().id();
            let chevron = app.world_mut().spawn((Chevron, Text::new("▾"))).id();
            let header = app
                .world_mut()
                .spawn(Section {
                    title: "Tool settings",
                    body,
                    expanded: true,
                })
                .add_child(chevron)
                .observe(toggle)
                .id();
            app.insert_resource(InputFocus::from_entity(hidden_field));
            for expected in [false, true, false, true] {
                app.world_mut().trigger(Activate { entity: header });
                assert_eq!(
                    app.world().get::<Section>(header).unwrap().expanded,
                    expected
                );
                assert_eq!(
                    app.world().get::<Node>(body).unwrap().display,
                    if expected {
                        Display::Flex
                    } else {
                        Display::None
                    }
                );
                assert_eq!(app.world().resource::<InputFocus>().get(), Some(header));
                let session = app.world().resource::<Session>();
                assert_eq!(session.editor.zone(), &zone);
                assert_eq!(session.editor.revision(), 2);
                assert_eq!(session.editor.history(), (0, 1));
                assert_eq!(session.placement_size, [8.25, 3.5, 2.25]);
                assert!(!session.is_dirty());
            }
            app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
            app.world_mut().trigger(Activate { entity: header });
            assert!(app.world().get::<Section>(header).unwrap().expanded);
            app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
            app.world_mut().resource_mut::<Session>().play =
                Some(playtest::Playtest::new(&zone).unwrap());
            app.world_mut().trigger(Activate { entity: header });
            assert!(app.world().get::<Section>(header).unwrap().expanded);
        }
    }
}
