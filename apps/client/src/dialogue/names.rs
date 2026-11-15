//! Friendly NPC overhead names and roles, anchored to the rendered contact.
use super::*;
use theia_core::content::ContentId;
#[derive(Component)]
struct Nameplate(ContentId);
pub(super) fn install(app: &mut App) {
    app.add_systems(Startup, setup.after(super::setup));
    app.add_systems(
        PostUpdate,
        present
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
        commands.spawn((
            Nameplate(contact.npc().clone()),
            Text::new(format!("{name}\n<{role}>")),
            TextFont {
                font_size: FontSize::Px(16.),
                ..default()
            },
            TextColor(Color::srgb(0.25, 1., 0.22)),
            TextLayout::justify(Justify::Center),
            TextShadow {
                offset: Vec2::ONE,
                color: Color::BLACK,
            },
            Node {
                position_type: PositionType::Absolute,
                width: px(220),
                height: px(46),
                display: Display::None,
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(1),
        ));
    }
}
fn present(
    network: Res<Network>,
    scale: Res<bevy::ui::UiScale>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    contacts: Query<(&ContactDefinition, &Transform, &Visibility), With<GuideVisual>>,
    spatial: avian3d::prelude::SpatialQuery,
    mut names: Query<(&Nameplate, &mut Node)>,
) {
    let camera = cameras.single().ok();
    for (name, mut node) in &mut names {
        let anchor = camera.and_then(|(camera, transform)| {
            if !network.connected() {
                return None;
            }
            let (_, npc, visibility) = contacts.iter().find(|(d, _, _)| d.0 == name.0)?;
            if *visibility == Visibility::Hidden {
                return None;
            }
            let world = npc.translation + Vec3::Y * 1.15;
            if crate::quest_marker::occluded(&spatial, transform.translation(), world) {
                return None;
            }
            let screen = crate::fantasy_ui::projected_ui_point(
                camera.world_to_viewport(transform, world).ok()?,
                scale.0,
            )?;
            let viewport = camera.logical_viewport_size()? / scale.0;
            (screen.x >= 110.
                && screen.x <= viewport.x - 110.
                && screen.y >= 46.
                && screen.y <= viewport.y)
                .then_some(screen)
        });
        node.display = if anchor.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(p) = anchor {
            node.left = px(p.x - 110.);
            node.top = px(p.y - 46.);
        }
    }
}
