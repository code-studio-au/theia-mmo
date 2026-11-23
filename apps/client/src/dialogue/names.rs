//! Friendly NPC overhead names and roles, anchored to the rendered contact.
use super::*;
use theia_core::{content::ContentId, identity::EntityKey};

pub(crate) const ANCHOR_HEIGHT: f32 = 1.15;
pub(crate) const HEIGHT: f32 = 48.;
const WIDTH: f32 = 232.;
#[derive(Component)]
struct Nameplate(ContentId);
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LayoutReady;
#[derive(Resource, Default)]
pub(crate) struct Layout(pub Vec<(EntityKey, Rect)>);

/// Prefer the projected position and use nearby horizontal lanes for crowded labels.
pub(crate) fn clear_rect(
    ideal: Rect,
    viewport: Vec2,
    occupied: impl Fn(Rect) -> bool,
) -> Option<Rect> {
    (0..=6)
        .flat_map(|step| {
            if step == 0 {
                [0., 0.]
            } else {
                [32. * step as f32, -32. * step as f32]
            }
        })
        .map(|x| Rect::from_corners(ideal.min + Vec2::X * x, ideal.max + Vec2::X * x))
        .find(|rect| {
            rect.min.x >= 4.
                && rect.max.x <= viewport.x - 4.
                && rect.min.y >= 0.
                && rect.max.y <= viewport.y
                && !occupied(*rect)
        })
}

fn overlaps(a: Rect, b: Rect) -> bool {
    let margin = Vec2::splat(3.);
    !Rect::from_corners(a.min - margin, a.max + margin)
        .intersect(b)
        .is_empty()
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Layout>()
        .add_systems(Startup, setup.after(super::setup));
    app.add_systems(
        PostUpdate,
        present
            .in_set(LayoutReady)
            .after(crate::camera::sync_camera_global)
            .before(bevy::ui::UiSystems::Layout),
    );
}
fn setup(
    mut commands: Commands,
    config: Res<Config>,
    journal: Option<Res<crate::quest_journal::Definitions>>,
) {
    for contact in config.0.contacts() {
        let quest_giver = journal
            .as_ref()
            .is_some_and(|d| d.0.catalog().quests().any(|q| q.giver() == contact.npc()));
        let (name, role) = contact.name().split_once(" - ").unwrap_or((
            contact.name(),
            if quest_giver {
                "Quest giver"
            } else {
                "Resident"
            },
        ));
        label(&mut commands, Nameplate(contact.npc().clone()), name, role);
    }
}

fn label(commands: &mut Commands, marker: Nameplate, name: &str, role: &str) {
    commands
        .spawn((
            marker,
            Node {
                position_type: PositionType::Absolute,
                width: px(WIDTH),
                height: px(HEIGHT),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(1),
        ))
        .with_children(|anchor| {
            anchor
                .spawn((
                    Node {
                        max_width: percent(100),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|plate| {
                    plate.spawn((
                        Text::new(name),
                        TextFont::from_font_size(21.)
                            .with_font_weight(bevy::text::FontWeight::BOLD),
                        TextColor(Color::WHITE),
                        TextLayout::justify(Justify::Center),
                        TextShadow {
                            offset: Vec2::splat(2.),
                            color: Color::BLACK,
                        },
                        Pickable::IGNORE,
                    ));
                    plate.spawn((
                        Text::new(role),
                        TextFont::from_font_size(16.)
                            .with_font_weight(bevy::text::FontWeight::BOLD),
                        TextColor(Color::srgb(1., 0.86, 0.56)),
                        TextLayout::justify(Justify::Center),
                        TextShadow {
                            offset: Vec2::splat(2.),
                            color: Color::BLACK,
                        },
                        Pickable::IGNORE,
                    ));
                });
        });
}
type Contact<'a> = (
    &'a ContactDefinition,
    &'a ContactKey,
    &'a Transform,
    &'a Visibility,
);
#[allow(clippy::too_many_arguments)]
fn present(
    network: Res<Network>,
    scale: Res<bevy::ui::UiScale>,
    cameras: Query<(&Camera, &GlobalTransform), crate::frontend::WorldCamera>,
    contacts: Query<Contact<'_>, With<GuideVisual>>,
    spatial: avian3d::prelude::SpatialQuery,
    mut names: Query<(Entity, &Nameplate, &Children, &mut Node)>,
    computed: Query<&ComputedNode>,
    mut layout: ResMut<Layout>,
) {
    layout.0.clear();
    let camera = cameras.single().ok().filter(|_| network.connected());
    let mut anchors = Vec::new();
    for (entity, name, children, mut node) in &mut names {
        let anchor = camera.and_then(|(camera, transform)| {
            let (_, key, npc, visibility) = contacts.iter().find(|(d, _, _, _)| d.0 == name.0)?;
            let key = key.0?;
            if *visibility == Visibility::Hidden {
                return None;
            }
            let world = npc.translation + Vec3::Y * ANCHOR_HEIGHT;
            if crate::quest_marker::occluded(&spatial, transform.translation(), world) {
                return None;
            }
            let screen = crate::fantasy_ui::projected_ui_point(
                camera.world_to_viewport(transform, world).ok()?,
                scale.0,
            )?;
            let viewport = camera.logical_viewport_size()? / scale.0;
            if screen.x < 0. || screen.x > viewport.x || screen.y < HEIGHT || screen.y > viewport.y
            {
                return None;
            }
            let width = children
                .first()
                .and_then(|e| computed.get(*e).ok())
                .map_or(160., |node| {
                    (node.size().x * node.inverse_scale_factor()).clamp(100., WIDTH)
                });
            let ideal = Rect::from_corners(
                screen - Vec2::new(width * 0.5, HEIGHT),
                screen + Vec2::X * width * 0.5,
            );
            Some((
                key,
                ideal,
                viewport,
                transform.translation().distance_squared(world),
            ))
        });
        if let Some((key, ideal, viewport, distance)) = anchor {
            anchors.push((entity, key, ideal, viewport, distance));
        } else if node.display != Display::None {
            node.display = Display::None;
        }
    }
    anchors.sort_by(|a, b| a.4.total_cmp(&b.4));
    for (entity, key, ideal, viewport, _) in anchors {
        let rect = clear_rect(ideal, viewport, |rect| {
            layout.0.iter().any(|(_, other)| overlaps(rect, *other))
        });
        let Ok((_, _, _, mut node)) = names.get_mut(entity) else {
            continue;
        };
        if let Some(rect) = rect {
            layout.0.push((key, rect));
            node.display = Display::Flex;
            node.left = px(rect.center().x - WIDTH * 0.5);
            node.top = px(rect.min.y);
        } else {
            node.display = Display::None;
        }
    }
}
