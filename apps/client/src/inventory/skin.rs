//! Read-only icon/count projection of the confirmed bag. Slot actions remain in
//! the existing inventory widgets; this module never queues an operation.
use super::*;
#[derive(Component)]
pub(crate) enum Icon {
    Bag(u8),
    MainHand,
}
#[derive(Component)]
struct Count(u8);
#[derive(Component)]
pub(crate) struct Tooltip;
#[derive(Component)]
struct TooltipText;
#[derive(Resource)]
pub(super) struct PreviewHover(u8);

pub(super) fn slot_children(button: &mut ChildSpawnerCommands, slot: u8) {
    item_art(button, Icon::Bag(slot));
    button.spawn((
        Count(slot),
        text("", 13.),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(2),
            right: px(4),
            ..default()
        },
        TextShadow {
            offset: Vec2::ONE,
            color: Color::BLACK,
        },
    ));
}
fn item_art(parent: &mut ChildSpawnerCommands, icon: Icon) {
    parent.spawn((
        icon,
        ImageNode::default(),
        Pickable::IGNORE,
        Node {
            display: Display::None,
            ..crate::ui_theme::slot::art()
        },
    ));
}
pub(super) fn equipment_slot(parent: &mut ChildSpawnerCommands, label: &str) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                crate::ui_theme::slot::frame(Node {
                    width: px(36),
                    height: px(36),
                    flex_shrink: 0.,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                }),
                super::context_menu::MainHand,
                AccessibleLabel::new("Main hand equipment"),
            ))
            .with_children(|slot| item_art(slot, Icon::MainHand));
            row.spawn((EquipmentLabel, text(label, 14.)));
        });
}
pub(super) fn spawn_tooltip(commands: &mut Commands) {
    commands
        .spawn((
            Tooltip,
            crate::ui_theme::Panel,
            Pickable::IGNORE,
            GlobalZIndex(70),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(260),
                padding: UiRect::all(px(10)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(crate::fantasy_ui::PANEL),
            Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
        ))
        .with_child((TooltipText, text("", 14.)));
}
fn stack(ui: &Ui, slot: u8) -> Option<&theia_core::inventory::Stack> {
    ui.bag.as_ref()?.slots().get(usize::from(slot))?.as_ref()
}
fn description(config: &Config, ui: &Ui, slot: u8) -> Option<String> {
    let stack = stack(ui, slot)?;
    let definition = config.0.catalog().definition(stack.item())?;
    let mut value = format!(
        "{}\nCount: {}\nMaximum stack: {}",
        definition.name(),
        stack.count(),
        definition.maximum_stack()
    );
    if let Some(offer) = config.0.vendor().and_then(|v| {
        v.catalog()
            .offers()
            .find(|o| o.is_sale() && o.cost().item() == Some(stack.item()))
    }) {
        value.push_str(&format!(
            "\nSell price: {} (per {} item)",
            offer.grant().label(config.0.catalog()),
            offer.cost().count()
        ));
    }
    Some(value)
}
type SlotView<'a> = (
    Entity,
    &'a SlotButton,
    &'a Hovered,
    &'a AccessibleLabel,
    &'a ComputedNode,
    &'a bevy::ui::UiGlobalTransform,
);
type TooltipFilter = (With<Tooltip>, Without<Icon>);
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Views<'w, 's> {
    icons:
        Query<'w, 's, (&'static Icon, &'static mut ImageNode, &'static mut Node), Without<Tooltip>>,
    counts: Query<'w, 's, (&'static Count, &'static mut Text), Without<TooltipText>>,
    slots: Query<'w, 's, SlotView<'static>>,
    tooltip: Query<'w, 's, (&'static mut Node, &'static ComputedNode), TooltipFilter>,
    preview_cameras: Query<'w, 's, &'static Camera, With<Camera2d>>,
    cameras: Query<'w, 's, &'static Camera, crate::frontend::WorldCamera>,
    scale: Res<'w, bevy::ui::UiScale>,
    tooltip_text: Query<'w, 's, &'static mut Text, (With<TooltipText>, Without<Count>)>,
}
// Clear both art and descriptions when the confirmed projection disappears.
pub(super) fn present(
    mut commands: Commands,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    config: Res<Config>,
    ui: Res<Ui>,
    mut views: Views,
    preview: Option<Res<PreviewHover>>,
) {
    let Some(skin) = skin else {
        return;
    };
    for (slot, mut image, mut node) in &mut views.icons {
        let item = match slot {
            Icon::Bag(slot) => stack(&ui, *slot),
            Icon::MainHand => ui.bag.as_ref().and_then(Bag::main_hand),
        };
        let art = item.map_or_else(ImageNode::default, |s| skin.item_icon(s.item().as_str()));
        if image.image != art.image || image.rect != art.rect {
            *image = art;
        }
        let display = if item.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (slot, mut count) in &mut views.counts {
        let value = stack(&ui, slot.0)
            .filter(|s| s.count() > 1)
            .map_or(String::new(), |s| s.count().to_string());
        if count.0 != value {
            count.0 = value;
        }
    }
    let mut hovered = None;
    let mut anchor = None;
    for (entity, slot, hover, label, computed, transform) in &views.slots {
        let value = description(&config, &ui, slot.0).unwrap_or_else(|| {
            format!(
                "Backpack slot {}: {}",
                slot.0 + 1,
                if ui.bag.is_some() { "Empty" } else { "Loading" }
            )
        });
        if label.0 != value {
            commands.entity(entity).insert(AccessibleLabel::new(value));
        }
        if ui.open && (hover.0 || preview.as_ref().is_some_and(|p| p.0 == slot.0)) {
            let centre = transform.affine().translation * computed.inverse_scale_factor();
            let size = computed.size() * computed.inverse_scale_factor();
            anchor = Some(Rect::from_center_size(centre, size));
            hovered = description(&config, &ui, slot.0);
        }
    }
    if ui.open
        && let Some(preview) = preview.as_deref()
    {
        hovered = description(&config, &ui, preview.0);
    }
    let display = if hovered.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    let viewport = views
        .cameras
        .iter()
        .find_map(|c| c.logical_viewport_size())
        .or_else(|| {
            preview.as_ref()?;
            views
                .preview_cameras
                .iter()
                .find_map(Camera::logical_viewport_size)
        })
        .map(|p| p / views.scale.0);
    for (mut node, computed) in &mut views.tooltip {
        if let (Some(slot), Some(viewport)) = (anchor, viewport) {
            let size =
                (computed.size() * computed.inverse_scale_factor()).max(Vec2::new(260., 96.));
            let point = tooltip_position(slot, size, viewport);
            node.left = px(point.x);
            node.top = px(point.y);
        }
        if node.display != display {
            node.display = display;
        }
    }
    for mut text in &mut views.tooltip_text {
        let value = hovered.clone().unwrap_or_default();
        if text.0 != value {
            text.0 = value;
        }
    }
}

fn tooltip_position(slot: Rect, size: Vec2, viewport: Vec2) -> Vec2 {
    let gap = 8.;
    let x = if slot.min.x >= size.x + gap {
        slot.min.x - size.x - gap
    } else {
        slot.max.x + gap
    };
    Vec2::new(
        x.clamp(4., (viewport.x - size.x - 4.).max(4.)),
        (slot.max.y - size.y).clamp(4., (viewport.y - size.y - 4.).max(4.)),
    )
}

/// Read-only qualification observation; never participates in command admission.
pub(crate) fn tooltip_state(world: &mut World) -> serde_json::Value {
    let visible = world
        .query_filtered::<&Node, With<Tooltip>>()
        .iter(world)
        .any(|n| n.display != Display::None);
    let text = world
        .query_filtered::<&Text, With<TooltipText>>()
        .iter(world)
        .next()
        .map(|t| t.0.clone());
    let scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    let bounds = world.query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Tooltip>>()
        .iter(world).next().map(|(node, transform)| {
            let factor = crate::fantasy_ui::window_factor(node, scale);
            serde_json::json!({"point":(transform.affine().translation*factor).to_array(), "size":(node.size()*factor).to_array()})
        });
    serde_json::json!({"visible":visible,"text":text,"bounds":bounds})
}

/// Actual slot widgets and projector, with explicitly synthetic content. No
/// network resource, inventory observer, or command producer is installed.
pub(crate) fn install_preview(app: &mut App) {
    app.init_resource::<bevy::ui::UiScale>();
    let profile = Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-document-items.json"
    ))
    .unwrap();
    let items = [
        "training-stone",
        "training-token",
        "training-sword",
        "training-wand",
        "wolf-pelt",
        "roadside-orders",
    ];
    let mut slots = std::array::from_fn(|_| None);
    for (index, id) in items.into_iter().enumerate() {
        slots[index] = theia_core::inventory::Stack::new(
            profile.catalog(),
            theia_core::content::ContentId::new(&format!("theia.dev.{id}")).unwrap(),
            if index == 0 { 3 } else { 1 },
        );
    }
    let main_hand = slots[2].clone();
    let bag = Bag::from_parts(profile.catalog(), 0, slots, main_hand).unwrap();
    app.insert_resource(PreviewHover(0))
        .insert_resource(Config(profile))
        .insert_resource(Ui {
            open: true,
            bag: Some(bag),
            ..default()
        })
        .add_systems(Startup, super::setup)
        .add_systems(PostStartup, preview_setup)
        .add_systems(Update, (paint_buttons, present).chain());
}
type PreviewPanels = Or<(With<Pane>, With<EquipmentPane>)>;
fn preview_setup(
    mut panels: Query<&mut Node, PreviewPanels>,
    mut equipment: Query<&mut Text, With<EquipmentLabel>>,
) {
    for mut panel in &mut panels {
        panel.display = Display::Flex;
    }
    for mut label in &mut equipment {
        **label = "Main hand: Training Sword".into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tooltip_stays_beside_the_slot_and_inside_scaled_viewports() {
        for viewport in [
            Vec2::new(3440., 1440.),
            Vec2::new(1920., 1440.),
            Vec2::new(1280., 960.),
        ] {
            let size = Vec2::new(260., 110.);
            let slot = Rect::from_center_size(viewport - Vec2::new(42., 90.), Vec2::splat(36.));
            let p = tooltip_position(slot, size, viewport);
            assert!(p.x + size.x < slot.min.x);
            assert!(p.cmpge(Vec2::splat(4.)).all());
            assert!((p + size).cmple(viewport - Vec2::splat(4.)).all());
            let edge = Rect::from_center_size(Vec2::new(22., 22.), Vec2::splat(36.));
            assert!(tooltip_position(edge, size, viewport).x > edge.max.x);
        }
    }
    #[test]
    fn icons_counts_and_hover_follow_confirmed_projection_then_clear_on_loss() {
        let mut app = App::new();
        app.insert_resource(crate::fantasy_ui::Skin::fixture());
        install_preview(&mut app);
        app.update();
        let mut tooltip = app.world_mut().query_filtered::<&Text, With<TooltipText>>();
        assert!(tooltip.single(app.world()).unwrap().0.contains("Count: 3"));
        // A newer authoritative bag reduces the holding; selection/hover alone
        // cannot keep the old count or claim quest readiness.
        let config = app.world().resource::<Config>();
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = theia_core::inventory::Stack::new(
            config.0.catalog(),
            theia_core::content::ContentId::new("theia.dev.training-stone").unwrap(),
            2,
        );
        let bag = Bag::from_slots(config.0.catalog(), 1, slots).unwrap();
        app.world_mut().resource_mut::<Ui>().bag = Some(bag);
        app.update();
        assert!(tooltip.single(app.world()).unwrap().0.contains("Count: 2"));
        app.world_mut().resource_mut::<Ui>().bag = None;
        app.update();
        assert_eq!(tooltip.single(app.world()).unwrap().0, "");
        let world = app.world_mut();
        for node in world.query_filtered::<&Node, With<Icon>>().iter(world) {
            assert_eq!(node.display, Display::None);
        }
        for count in world.query_filtered::<&Text, With<Count>>().iter(world) {
            assert_eq!(count.0, "");
        }
        assert_eq!(
            world
                .query_filtered::<&Node, With<Tooltip>>()
                .single(world)
                .unwrap()
                .display,
            Display::None
        );
        assert!(
            !world.contains_resource::<Pending>(),
            "Preview installs no inventory command producer"
        );
    }
}
