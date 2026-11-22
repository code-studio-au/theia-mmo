//! One cosmetic XP label from committed owner snapshots; never awards experience.
use bevy::{picking::Pickable, prelude::*, text::FontSize};
use theia_network::{
    client::State,
    progression_wire::{Profile, State as Progress},
};
const LIFETIME: f64 = 1.5;
const COLOR: Color = Color::srgb(0.78, 0.25, 0.95);
#[derive(Component)]
pub(crate) struct Label;
#[derive(Resource, Default)]
struct Cursor {
    prior: Option<Progress>,
    world: Option<[u8; 32]>,
    amount: u32,
    born: Option<f64>,
}
impl Cursor {
    fn observe(&mut self, state: &State, profile: &Profile, now: f64) {
        let valid = (|| {
            let welcome = state.welcome.as_ref()?;
            let view = state.progression.as_ref()?;
            (state.error.is_none()
                && welcome.world != [0; 32]
                && state.id == Some(welcome.id)
                && view.actor.entity.value() == welcome.id
                && view.actor.epoch.value() == welcome.epoch
                && view.profile_hash == profile.hash()
                && profile.profile().progress(&view.progress).is_ok())
            .then_some((view, welcome.world))
        })();
        let Some((view, world)) = valid else {
            *self = Self::default();
            return;
        };
        if self.world != Some(world) {
            *self = Self::default();
            self.world = Some(world);
            self.prior = Some(view.clone());
            return;
        }
        let Some(prior) = self
            .prior
            .as_ref()
            .filter(|old| old.actor == view.actor && old.profile_hash == view.profile_hash)
        else {
            self.prior = Some(view.clone());
            self.born = None;
            self.amount = 0;
            return;
        };
        if view.validate_successor(view.actor, Some(prior)).is_err() {
            // Keep the last valid high-water mark; an invalid regression cannot
            // create a later cosmetic "gain" from the same already-seen XP.
            self.born = None;
            self.amount = 0;
            return;
        }
        let delta = view.progress.total() - prior.progress.total();
        if delta == 0 {
            return;
        }
        self.prior = Some(view.clone());
        self.amount = if self.visible(now).is_some() {
            self.amount.saturating_add(delta)
        } else {
            delta
        };
        self.born = Some(now);
    }
    fn visible(&self, now: f64) -> Option<(u32, f32)> {
        let age = now - self.born?;
        (self.amount > 0 && (0. ..LIFETIME).contains(&age)).then_some((self.amount, age as f32))
    }
}
fn label_position(anchor: Vec2, viewport: Vec2, age: f32) -> Vec2 {
    Vec2::new(
        (anchor.x - 110.).clamp(0., (viewport.x - 220.).max(0.)),
        (anchor.y - age * 28.).clamp(0., (viewport.y - 40.).max(0.)),
    )
}
fn widget() -> impl Bundle {
    (
        Label,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(28.),
            ..default()
        },
        TextColor(COLOR),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            width: px(220),
            display: Display::None,
            ..default()
        },
        TextLayout::justify(Justify::Center),
        GlobalZIndex(15),
    )
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Cursor>()
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(widget());
        })
        .add_systems(
            PostUpdate,
            present
                .after(crate::camera::sync_camera_global)
                .before(bevy::ui::UiSystems::Layout),
        );
}
pub(crate) fn install_preview(app: &mut App, anchor: Vec2) {
    app.add_systems(Startup, move |mut commands: Commands| {
        let mut node = Node {
            position_type: PositionType::Absolute,
            left: px(anchor.x - 110.),
            top: px(anchor.y),
            width: px(220),
            ..default()
        };
        node.display = Display::Flex;
        commands
            .spawn(widget())
            .insert((node, Text::new("XP +25 (fixture)")));
    });
}
#[allow(clippy::too_many_arguments)]
fn present(
    network: Res<crate::network::Network>,
    profile: Res<super::Profile>,
    time: Res<Time>,
    frame: Res<crate::floating_origin::Frame>,
    scale: Option<Res<bevy::ui::UiScale>>,
    cameras: Query<(&Camera, &GlobalTransform), crate::frontend::WorldCamera>,
    mut cursor: ResMut<Cursor>,
    mut labels: Query<(&mut Node, &mut Text, &mut TextColor), With<Label>>,
) {
    let state = network.state();
    let now = time.elapsed_secs_f64();
    cursor.observe(&state, &profile.0, now);
    let shown = (|| {
        let (amount, age) = cursor.visible(now)?;
        let mut global = state.authority_focus()?.metres();
        global[1] += 2.6;
        let point = frame
            .render
            .to_local(theia_core::coordinates::GlobalPosition::new(global)?)?;
        let (camera, transform) = cameras.single().ok()?;
        let p = camera
            .world_to_viewport(transform, Vec3::from_array(point.metres()))
            .ok()?;
        let size = camera.logical_viewport_size()?;
        if p.x < 0. || p.y < 0. || p.x > size.x || p.y > size.y {
            return None;
        }
        let p = crate::fantasy_ui::projected_ui_point(p, scale.as_ref().map_or(1., |s| s.0))?;
        let size = crate::fantasy_ui::projected_ui_point(size, scale.as_ref().map_or(1., |s| s.0))?;
        Some((amount, age, label_position(p, size, age)))
    })();
    for (mut node, mut text, mut color) in &mut labels {
        let display = if shown.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if let Some((amount, age, p)) = shown {
            let value = format!("XP +{amount}");
            if text.0 != value {
                text.0 = value;
            }
            node.left = px(p.x);
            node.top = px(p.y);
            color.0 = COLOR.with_alpha((1. - age / LIFETIME as f32).clamp(0., 1.));
        } else if !text.0.is_empty() {
            text.0.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (State, Profile) {
        let profile = Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let actor = state.quest_journal.as_ref().unwrap().actor;
        state.progression = Some(Progress {
            actor,
            profile_hash: profile.hash(),
            progress: theia_core::progression::State::new(profile.profile(), 0, 0).unwrap(),
        });
        (state, profile)
    }
    fn set(state: &mut State, profile: &Profile, revision: u64, total: u32) {
        state.progression.as_mut().unwrap().progress =
            theia_core::progression::State::new(profile.profile(), revision, total).unwrap();
    }
    #[test]
    fn confirmed_gains_coalesce_and_expire_without_login_or_duplicate_credit() {
        let (mut state, profile) = fixture();
        let mut cursor = Cursor::default();
        set(&mut state, &profile, 4, 100);
        cursor.observe(&state, &profile, 0.);
        assert!(
            cursor.visible(0.).is_none(),
            "Saved login progress is not a new award"
        );
        set(&mut state, &profile, 5, 125);
        cursor.observe(&state, &profile, 0.1);
        assert_eq!(cursor.visible(0.1).unwrap().0, 25);
        cursor.observe(&state, &profile, 0.2);
        assert_eq!(cursor.visible(0.2).unwrap().0, 25);
        set(&mut state, &profile, 6, 150);
        cursor.observe(&state, &profile, 0.3);
        assert_eq!(cursor.visible(0.3).unwrap().0, 50);
        assert!(cursor.visible(1.8).is_none());
        set(&mut state, &profile, 7, 175);
        cursor.observe(&state, &profile, 2.);
        assert_eq!(cursor.visible(2.).unwrap().0, 25);
    }
    #[test]
    fn reconnect_world_owner_and_profile_changes_establish_a_new_baseline() {
        for case in 0..7 {
            let (mut state, profile) = fixture();
            let mut cursor = Cursor::default();
            cursor.observe(&state, &profile, 0.);
            set(&mut state, &profile, 1, 25);
            cursor.observe(&state, &profile, 0.1);
            assert!(cursor.visible(0.1).is_some());
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.welcome.as_mut().unwrap().world = [2; 32],
                2 => {
                    state.id = Some(9);
                    state.welcome.as_mut().unwrap().id = 9;
                    state.progression.as_mut().unwrap().actor.entity =
                        theia_core::identity::EntityId::new(9).unwrap();
                }
                3 => state.progression.as_mut().unwrap().profile_hash[0] ^= 1,
                4 => state.welcome = None,
                5 => state.welcome.as_mut().unwrap().epoch += 1,
                _ => state.welcome.as_mut().unwrap().world = [0; 32],
            }
            cursor.observe(&state, &profile, 0.2);
            assert!(cursor.visible(0.2).is_none(), "case{case}");
            let (mut fresh, _) = fixture();
            set(&mut fresh, &profile, 10, 250);
            cursor.observe(&fresh, &profile, 0.3);
            assert!(cursor.visible(0.3).is_none(), "rejoin case{case}");
        }
    }
    #[test]
    fn floating_label_stays_inside_scaled_viewport_edges() {
        assert_eq!(
            label_position(Vec2::new(10., 0.), Vec2::new(1280., 720.), 1.),
            Vec2::ZERO
        );
        assert_eq!(
            label_position(Vec2::new(1270., 720.), Vec2::new(1280., 720.), 0.),
            Vec2::new(1060., 680.)
        );
        assert_eq!(
            label_position(Vec2::new(640., 360.), Vec2::new(1280., 720.), 1.),
            Vec2::new(530., 332.)
        );
    }
    #[test]
    fn regressed_and_same_revision_changed_totals_cannot_replay_experience() {
        let (mut state, profile) = fixture();
        let mut cursor = Cursor::default();
        set(&mut state, &profile, 5, 125);
        cursor.observe(&state, &profile, 0.);
        set(&mut state, &profile, 4, 100);
        cursor.observe(&state, &profile, 0.1);
        assert!(cursor.visible(0.1).is_none());
        set(&mut state, &profile, 5, 125);
        cursor.observe(&state, &profile, 0.2);
        assert!(cursor.visible(0.2).is_none());
        set(&mut state, &profile, 5, 150);
        cursor.observe(&state, &profile, 0.3);
        assert!(cursor.visible(0.3).is_none());
        set(&mut state, &profile, 6, 150);
        cursor.observe(&state, &profile, 0.4);
        assert_eq!(cursor.visible(0.4).unwrap().0, 25);
    }
}
