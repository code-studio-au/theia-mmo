//! Sprinkles text_edit/vector_edit presentation over retained exact f64 fields.
use super::*;

pub(super) fn spawn(panel: &mut ChildSpawnerCommands) {
    let mut heading = theme::font(10.);
    heading.weight = bevy::text::FontWeight::MEDIUM;
    panel.spawn((
        FieldGroup::PositionHeading,
        Text::new("Position · world metres"),
        heading.clone(),
        TextColor(theme::MUTED),
        Pickable::IGNORE,
    ));
    for index in 0..4 {
        field(panel, index, false);
    }
    panel.spawn((
        FieldGroup::SizeHeading,
        Text::new("Size · metres"),
        heading.clone(),
        TextColor(theme::MUTED),
        Pickable::IGNORE,
    ));
    panel
        .spawn((
            FieldGroup::Tail,
            Node {
                width: percent(100),
                column_gap: px(12),
                flex_shrink: 0.,
                ..default()
            },
        ))
        .with_children(|row| {
            for index in 4..7 {
                field(row, index, true);
            }
        });
}

fn field(parent: &mut ChildSpawnerCommands, index: usize, compact: bool) {
    let font = theme::input_font();
    parent
        .spawn((
            FieldRow(index),
            Visibility::Inherited,
            Node {
                width: if compact { Val::Auto } else { percent(100) },
                height: px(28),
                min_height: px(28),
                min_width: px(0),
                flex_grow: if compact { 1. } else { 0. },
                flex_basis: if compact { px(0) } else { Val::Auto },
                flex_shrink: 0.,
                column_gap: px(6),
                padding: UiRect::axes(px(6), px(0)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(2)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|row| {
            row.spawn((
                FieldLabel(index),
                numeric::Handle(index),
                Text::new("X"),
                theme::font(12.),
                TextColor(theme::BODY.with_alpha(0.5)),
                TextLayout::justify(Justify::Center),
                Hovered::default(),
                Node {
                    width: px(16),
                    height: percent(100),
                    padding: UiRect::vertical(px(5)),
                    flex_shrink: 0.,
                    ..default()
                },
            ));
            row.spawn((
                Field(index),
                FramedInput,
                TextInput,
                SelectAllOnFocus,
                TabIndex(index as i32),
                Hovered::default(),
                EditableText {
                    max_characters: Some(64),
                    allow_newlines: false,
                    ..default()
                },
                Node {
                    flex_grow: 1.,
                    min_width: px(0),
                    // Center the text line itself inside the 28 px outer frame.
                    // A full-height text node leaves its glyphs at the top edge.
                    height: Val::Auto,
                    min_height: px(16),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderColor::all(Color::NONE),
                BackgroundColor(Color::NONE),
                font,
                TextColor(theme::BODY),
                TextLayout::no_wrap(),
                theme::text_cursor(),
            ));
            row.spawn((
                FieldUnit(index),
                Text::new("m"),
                theme::font(11.),
                TextColor(theme::MUTED),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn prefix(target: &Target, index: usize) -> Option<(&'static str, f32, &'static str)> {
    if index >= target.labels().len() {
        return None;
    }
    if matches!(target, Target::Path { point: None, .. }) {
        return Some(("Width", 44., "m"));
    }
    if matches!(target, Target::SpawnGroup(_, _)) {
        return Some(match index {
            0 => ("X", 16., "m"),
            1 => ("Z", 16., "m"),
            2 => ("Min Y", 44., "m"),
            3 => ("Max Y", 44., "m"),
            4 => ("Alive", 44., ""),
            5 => ("Respawn", 58., "s"),
            _ => ("Roaming", 58., "m"),
        });
    }
    if index < 3 {
        return Some((["X", "Y", "Z"][index], 16., "m"));
    }
    match (target, index) {
        (Target::TestPlayer(_), 3) => Some(("Delay", 44., "ms")),
        (Target::TestPlayer(_), 4) => Some(("Jitter", 44., "ms")),
        (Target::TestPlayer(_), 5) => Some(("Loss", 44., "%")),
        (Target::TestPlayer(_), 6) => Some(("Seed", 44., "")),
        (Target::Path { .. }, 3) => Some(("Width", 44., "m")),
        (_, 3) => Some(("Yaw", 28., "°")),
        (Target::Object(_), 4..=6) => Some((["X", "Y", "Z"][index - 4], 16., "m")),
        (Target::Plant(_), 4) => Some(("Scale", 40., "")),
        (Target::Particle(_), 4) => Some(("Scale", 40., "")),
        (Target::Marker(_), 4) => Some(("Radius", 44., "m")),
        _ => None,
    }
}
