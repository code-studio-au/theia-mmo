//! Presentation of committed owner progression; never derives combat stats.
use theia_network::{client::State, progression_wire::Profile};

pub(crate) fn owner_level(state: &State, profile: &Profile) -> Option<u8> {
    let welcome = state.welcome.as_ref()?;
    let view = state.progression.as_ref()?;
    if state.error.is_some()
        || welcome.world == [0; 32]
        || state.id != Some(welcome.id)
        || view.actor.entity.value() != welcome.id
        || view.actor.epoch.value() != welcome.epoch
        || view.profile_hash != profile.hash()
    {
        return None;
    }
    profile
        .profile()
        .progress(&view.progress)
        .ok()
        .map(|p| p.level)
}

/// Identity must belong to the admitted owner in the current roster.
pub(in crate::combat) fn owner_identity(
    state: &State,
) -> Option<&theia_network::player_identity_wire::Actor> {
    let welcome = state.welcome.as_ref()?;
    let snapshot = state.snapshot.as_ref()?;
    let view = state.player_identity.as_ref()?;
    if state.error.is_some()
        || state.id != Some(welcome.id)
        || snapshot.epoch != welcome.epoch
        || !snapshot.players.iter().any(|p| p.id == welcome.id)
    {
        return None;
    }
    let owner = theia_core::identity::EntityKey {
        epoch: theia_core::identity::WorldEpoch::new(welcome.epoch)?,
        entity: theia_core::identity::EntityId::new(welcome.id)?,
    };
    view.validate_successor(owner, welcome.world, None).ok()?;
    view.actors.iter().find(|a| a.key == owner)
}

use bevy::{picking::Pickable, prelude::*};

// The portrait draws above the tracks and covers their left end caps.
const PORTRAIT_OVERLAP: f32 = 18.;
const TRACK_OVERLAP: f32 = 5.;

pub(in crate::combat) fn heading(
    parent: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    name: &str,
) {
    parent
        .spawn((
            Node {
                min_height: px(30),
                margin: UiRect::left(px(-PORTRAIT_OVERLAP)),
                padding: UiRect::new(px(10. + PORTRAIT_OVERLAP), px(10), px(0), px(0)),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|heading| {
            heading.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(3),
                    right: px(3),
                    top: px(3),
                    bottom: px(3),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                Pickable::IGNORE,
            ));
            heading.spawn((
                crate::ui_theme::Sprite("bar.frame"),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            heading.spawn((
                marker,
                super::text(name, 19.),
                crate::fantasy_ui::hud_shadow(),
                TextLayout::no_wrap(),
                Pickable::IGNORE,
            ));
        });
}

#[derive(Component)]
pub(super) enum Level {
    Player,
    Target,
}

#[derive(Component)]
pub(super) struct WolfPortrait;

#[derive(Component)]
pub(super) struct PlayerPortrait(pub Handle<Image>);

pub(in crate::combat) fn portrait(
    parent: &mut ChildSpawnerCommands,
    atlas: Option<Handle<Image>>,
    player: bool,
) {
    let size = if player { 92. } else { 86. };
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(-3),
                top: px(0),
                width: px(size),
                height: px(size),
                ..default()
            },
            ZIndex(1),
            Pickable::IGNORE,
        ))
        .with_children(|portrait| {
            portrait.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(9),
                    top: percent(9),
                    width: percent(82),
                    height: percent(82),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                Pickable::IGNORE,
                BackgroundColor(crate::fantasy_ui::PANEL),
            ));
            portrait.spawn((
                crate::ui_theme::Sprite("icon.character"),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(25),
                    top: percent(25),
                    width: percent(50),
                    height: percent(50),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            if let Some(image) = atlas {
                let mut picture = portrait.spawn((
                    ImageNode::new(image.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(9),
                        top: percent(9),
                        width: percent(82),
                        height: percent(82),
                        border_radius: BorderRadius::MAX,
                        display: Display::None,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                if player {
                    picture.insert(PlayerPortrait(image));
                } else {
                    picture.insert(WolfPortrait);
                }
            }
            portrait.spawn((
                crate::ui_theme::Sprite("portrait.frame"),
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            portrait
                .spawn((
                    if player { Level::Player } else { Level::Target },
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(0),
                        bottom: px(0),
                        width: px(28),
                        height: px(28),
                        border_radius: BorderRadius::MAX,
                        border: UiRect::all(px(1)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.03, 0.045, 0.067)),
                    BorderColor::all(crate::fantasy_ui::GOLD),
                    Pickable::IGNORE,
                ))
                .with_child((super::text("", 16.), Pickable::IGNORE));
        });
}

#[derive(Clone, Copy)]
pub(crate) enum Pool {
    Health,
    Mana,
    Hostile,
    TargetMana,
}

#[derive(Component)]
pub(super) struct TargetManaFrame;

pub(super) const DISABLED_MANA: Color = Color::srgb(0.37, 0.41, 0.46);

pub(in crate::combat) fn bar(
    parent: &mut ChildSpawnerCommands,
    pool: Pool,
    track: impl Bundle,
    fill: impl Bundle,
    label: impl Bundle,
) {
    let height = match pool {
        Pool::Health | Pool::Hostile => 24.,
        Pool::Mana | Pool::TargetMana => 21.,
    };
    parent
        .spawn((
            track,
            Node {
                margin: UiRect::new(px(-PORTRAIT_OVERLAP), px(0), px(-TRACK_OVERLAP), px(0)),
                height: px(height),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            pool_art(bar, pool, fill, ());
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_child((
                label,
                super::text("—", 14.),
                crate::fantasy_ui::hud_shadow(),
                Pickable::IGNORE,
            ));
        });
}

/// The dark track, fill and gold frame shared by HUD and overhead vitals.
pub(crate) fn pool_art(
    bar: &mut ChildSpawnerCommands,
    pool: Pool,
    fill: impl Bundle,
    frame_marker: impl Bundle,
) {
    use bevy::ui::{BackgroundGradient, ColorStop, LinearGradient};
    bar.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(5),
            right: px(5),
            top: px(3),
            bottom: px(3),
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.025, 0.04, 0.96)),
        Pickable::IGNORE,
    ))
    .with_children(|track| {
        let mut fill = track.spawn((
            fill,
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
        ));
        match pool {
            Pool::Health => fill.insert(crate::ui_theme::Sprite("bar.fill-health")),
            Pool::Mana | Pool::TargetMana => fill.insert((
                crate::ui_theme::Sprite("bar.fill-mana"),
                BackgroundColor(Color::srgb(0.14, 0.39, 0.87)),
            )),
            Pool::Hostile => fill.insert(BackgroundGradient(vec![
                LinearGradient::new(
                    LinearGradient::TO_BOTTOM,
                    vec![
                        ColorStop::percent(Color::srgb(1., 0.37, 0.33), 0),
                        ColorStop::percent(Color::srgb(0.8, 0.035, 0.05), 40),
                        ColorStop::percent(Color::srgb(0.44, 0.012, 0.055), 100),
                    ],
                )
                .into(),
            ])),
        };
    });
    let mut frame = bar.spawn((
        frame_marker,
        crate::ui_theme::Sprite("bar.frame"),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Pickable::IGNORE,
    ));
    if matches!(pool, Pool::TargetMana) {
        frame.insert((TargetManaFrame, crate::ui_theme::Muted));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_name_requires_current_admission_snapshot_and_actor() {
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let actor = state.quest_journal.as_ref().unwrap().actor;
        let name =
            theia_core::character_name::CharacterName::parse("Abcdefghijklmnopqrst").unwrap();
        state.player_identity = Some(theia_network::player_identity_wire::State {
            epoch: actor.epoch.value(),
            tick: 1,
            world: state.welcome.as_ref().unwrap().world,
            actors: vec![theia_network::player_identity_wire::Actor {
                appearance: Default::default(),
                key: actor,
                name: Some(name.clone()),
            }],
        });
        assert_eq!(owner_identity(&state).unwrap().name.as_ref(), Some(&name));
        let original = state.clone();
        for case in 0..9 {
            state = original.clone();
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.id = Some(999),
                2 => state.snapshot = None,
                3 => state.snapshot.as_mut().unwrap().players.clear(),
                4 => state.snapshot.as_mut().unwrap().epoch += 1,
                5 => state.player_identity.as_mut().unwrap().world = [2; 32],
                6 => state.player_identity.as_mut().unwrap().epoch += 1,
                7 => state.player_identity.as_mut().unwrap().actors.clear(),
                8 => state.player_identity.as_mut().unwrap().actors[0].name = None,
                _ => unreachable!(),
            }
            assert!(
                owner_identity(&state).is_none_or(|a| a.name.is_none()),
                "case {case}"
            );
        }
    }
    #[test]
    fn committed_level_requires_the_current_owner_epoch_and_profile() {
        let profile = Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let actor = state.quest_journal.as_ref().unwrap().actor;
        state.progression = Some(theia_network::progression_wire::State {
            actor,
            profile_hash: profile.hash(),
            progress: theia_core::progression::State::new(profile.profile(), 4, 125).unwrap(),
        });
        assert_eq!(owner_level(&state, &profile), Some(2));
        let original = state.clone();
        for case in 0..7 {
            state = original.clone();
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.welcome = None,
                2 => state.progression = None,
                3 => state.id = Some(actor.entity.value() + 1),
                4 => state.welcome.as_mut().unwrap().epoch += 1,
                5 => state.welcome.as_mut().unwrap().world = [0; 32],
                6 => state.progression.as_mut().unwrap().profile_hash = [7; 32],
                _ => unreachable!(),
            }
            assert_eq!(owner_level(&state, &profile), None, "case {case}");
        }
    }
}
