//! Read-only hover descriptions. Static search regions never reveal live spawns.
use super::{DISPLAY, Map, Metrics, Surface, guidance, projection};
use bevy::{picking::Pickable, prelude::*, ui::RelativeCursorPosition};
use theia_core::quests::{journal::Status, lifecycle::Phase};
use theia_network::{client::State, quest_journal::Profile};

#[derive(Component)]
pub(crate) struct Tooltip;
#[derive(Component)]
struct Body;
const MAX_LINES: usize = 6;

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Tooltip,
            Pickable::IGNORE,
            GlobalZIndex(70),
            Node {
                position_type: PositionType::Absolute,
                right: px(DISPLAY + 65.),
                top: px(126),
                width: px(280),
                padding: UiRect::all(px(10)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(crate::fantasy_ui::PANEL),
            Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
        ))
        .with_child((Body, super::label("", 14.)));
}
fn cursor_point(cursor: &RelativeCursorPosition, circular: bool) -> Option<[f32; 2]> {
    let p = cursor.normalized?;
    if !cursor.cursor_over || !p.is_finite() || p.abs().max_element() > 0.5 {
        return None;
    }
    let point = ((p + Vec2::splat(0.5)) * DISPLAY).to_array();
    if circular && !projection::circle_contains(point, 0.) {
        return None;
    }
    Some(point)
}
fn contact(state: &State, profile: Option<&Profile>, phase: Phase) -> String {
    let heading = match phase {
        Phase::Available => "Quest contact — Available quests",
        Phase::Ready => "Quest contact — Ready to turn in",
        Phase::Active => "Quest contact — Quests in progress",
        Phase::TurnedIn => "Quest contact",
    };
    let mut lines = vec![heading.to_owned()];
    if let (Some(profile), Some(view)) = (profile, &state.quest_journal) {
        let candidates = profile.catalog().quests().filter(|q| {
            let progress = view.journal.progress(q.id());
            match phase {
                Phase::Ready => progress.is_some_and(|p| p.status() == Status::Ready),
                Phase::Active => progress.is_some_and(|p| p.status() == Status::Active),
                Phase::Available => {
                    progress.is_none()
                        && q.prerequisites().iter().all(|id| {
                            view.journal
                                .progress(id)
                                .is_some_and(|p| p.status() == Status::Completed)
                        })
                }
                Phase::TurnedIn => false,
            }
        });
        for (i, q) in candidates.enumerate() {
            if i == MAX_LINES {
                lines.push("More quests available in dialogue".into());
                break;
            }
            lines.push(q.title().to_owned());
        }
    }
    lines.join("\n")
}
fn search(
    state: &State,
    profile: &Profile,
    view: projection::View,
    point: [f32; 2],
) -> Option<String> {
    let regions = guidance::regions(state, profile);
    let centre = view.centre.metres();
    let dx = (f64::from(point[0]) / f64::from(DISPLAY) * 2. - 1.)
        * projection::RADIUS
        * if view.north_positive_z { -1. } else { 1. };
    let dz = (f64::from(point[1]) / f64::from(DISPLAY) * 2. - 1.)
        * projection::RADIUS
        * if view.north_positive_z { -1. } else { 1. };
    let hit: Vec<_> = regions
        .into_iter()
        .filter(|area| {
            let lo = area.min().metres();
            let hi = area.max().metres();
            lo[0] - centre[0] <= dx
                && dx <= hi[0] - centre[0]
                && lo[2] - centre[2] <= dz
                && dz <= hi[2] - centre[2]
        })
        .collect();
    if hit.is_empty() {
        return None;
    }
    let journal = &state.quest_journal.as_ref()?.journal;
    let mut lines = vec!["Quest search area".to_owned()];
    let mut count = 0;
    for q in profile.catalog().quests() {
        let Some(p) = journal
            .progress(q.id())
            .filter(|p| p.status() == Status::Active)
        else {
            continue;
        };
        let Some(stage) = q.stages().find(|s| s.id() == p.stage()) else {
            continue;
        };
        for objective in stage.objectives() {
            if p.count(objective.id()).unwrap_or(0) >= objective.required() {
                continue;
            }
            let linked = hit.iter().any(|a| a.map_objectives().contains(objective.id()) ||
                matches!(objective.event(),theia_core::quests::catalog::EventKey::ReachArea(id) if id==a.id()));
            if linked {
                if count == MAX_LINES {
                    lines.push("More objectives in the quest journal".into());
                    return Some(lines.join("\n"));
                }
                lines.push(format!(
                    "{}: {}/{}",
                    objective.label(),
                    p.count(objective.id()).unwrap_or(0),
                    objective.required()
                ));
                count += 1;
            }
        }
    }
    lines.push("Search region; spawns and drops are not guaranteed".into());
    Some(lines.join("\n"))
}
pub(super) fn set(world: &mut World, text: Option<String>) {
    for mut node in world
        .query_filtered::<&mut Node, With<Tooltip>>()
        .iter_mut(world)
    {
        let display = if text.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for mut body in world
        .query_filtered::<&mut Text, With<Body>>()
        .iter_mut(world)
    {
        let value = text.as_deref().unwrap_or("");
        if body.0 != value {
            body.0 = value.into();
        }
    }
    world.resource_mut::<Metrics>().hover_text = text;
}
pub(super) fn present(world: &mut World) {
    let text = (|| {
        if !world.resource::<Metrics>().visible {
            return None;
        }
        let circular = world.contains_resource::<crate::fantasy_ui::Skin>();
        let cursor = world
            .query_filtered::<&RelativeCursorPosition, With<Surface>>()
            .single(world)
            .ok()?;
        let point = cursor_point(cursor, circular)?;
        // Root visibility and the same owner/profile/observed guide checks as the pin.
        let state = world.resource::<crate::network::Network>().state();
        let profile = world
            .get_resource::<crate::quest_journal::Definitions>()
            .map(|d| d.0.as_ref());
        if let Some(guide) = world.resource::<Metrics>().guide_point
            && Vec2::from_array(point).distance(Vec2::from_array(guide)) <= 14.
            && let Some((phase, _, _)) = crate::quest_marker::guide_observation(
                &state,
                world
                    .get_resource::<crate::quest_lifecycle::Definitions>()
                    .map(|d| d.0.as_ref()),
                profile,
            )
        {
            return Some(contact(&state, profile, phase));
        }
        search(&state, profile?, world.resource::<Map>().view?, point)
    })();
    set(world, text);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hover_projection_handles_ui_scaling_and_round_corners_in_relative_space() {
        let cursor = |x, y| RelativeCursorPosition {
            cursor_over: true,
            normalized: Some(Vec2::new(x, y)),
        };
        assert_eq!(cursor_point(&cursor(0., 0.), true), Some([88., 88.]));
        assert!(cursor_point(&cursor(-0.49, -0.49), true).is_none());
        assert!(cursor_point(&cursor(-0.49, -0.49), false).is_some());
        assert!(cursor_point(&cursor(f32::NAN, 0.), false).is_none());
        assert!(cursor_point(&RelativeCursorPosition::default(), true).is_none());
    }
    #[test]
    fn contact_lists_only_currently_available_quests() {
        let (state, profile) = crate::quest_journal::tests::fixture();
        let available = contact(&state, Some(&profile), Phase::Available);
        assert!(available.contains("Meet the guide"));
        assert!(!available.contains("Defeat three training creatures"));
        assert!(!available.contains("Training on the road"));
        let ready = contact(&state, Some(&profile), Phase::Ready);
        assert!(!ready.contains("Meet the guide"));
    }
    #[test]
    fn hiding_a_tooltip_clears_text_and_keeps_it_noninteractive() {
        let mut world = World::new();
        world.init_resource::<Metrics>();
        let panel = world
            .spawn((Tooltip, Node::default(), Pickable::IGNORE))
            .id();
        let body = world.spawn((Body, Text::new(""))).id();
        set(&mut world, Some("Confirmed quest progress".into()));
        assert_eq!(world.get::<Node>(panel).unwrap().display, Display::Flex);
        assert_eq!(
            world.get::<Text>(body).unwrap().0,
            "Confirmed quest progress"
        );
        set(&mut world, None);
        assert_eq!(world.get::<Node>(panel).unwrap().display, Display::None);
        assert!(world.get::<Text>(body).unwrap().0.is_empty());
        assert!(world.resource::<Metrics>().hover_text.is_none());
        assert!(!world.get::<Pickable>(panel).unwrap().is_hoverable);
    }
    #[test]
    fn search_hover_uses_validated_active_geometry_and_clears_for_foreign_owner() {
        let (mut state, profile) = super::super::guidance::tests::fixture();
        let view = projection::View {
            north_positive_z: false,
            centre: theia_core::coordinates::GlobalPosition::new([7864., 0., 8000.]).unwrap(),
        };
        let text = search(&state, &profile, view, [88., 88.]).unwrap();
        assert!(text.contains("0/1") && text.contains("western overlook"));
        assert!(search(&state, &profile, view, [170., 88.]).is_none());
        state.welcome.as_mut().unwrap().epoch += 1;
        assert!(search(&state, &profile, view, [88., 88.]).is_none());
    }
}
