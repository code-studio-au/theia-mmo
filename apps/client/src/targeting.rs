//! Opt-in standard Bevy presentation of authoritative, nonblocking targets.
use crate::{network::Network, scene::PawnAssets};
use bevy::{prelude::*, text::FontSize};
use theia_core::identity::{EntityId, EntityKey};
pub(crate) mod motion;
mod qualification;
#[derive(Component)]
pub(crate) struct TargetVisual {
    pub key: EntityKey,
}
#[derive(Component)]
struct TargetHud;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Presentation;
#[derive(Resource)]
struct Assets {
    mesh: Handle<Mesh>,
    normal: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}
pub fn install(app: &mut App, moving_speed: Option<f32>) {
    motion::install(app, moving_speed);
    crate::world_input::install(app);
    app.add_systems(Startup, setup.after(crate::scene::setup))
        .add_systems(
            Update,
            (controls, present.in_set(Presentation))
                .chain()
                .after(crate::network::present),
        );
    qualification::install(app);
}
fn setup(
    mut commands: Commands,
    pawns: Res<PawnAssets>,
    mut materials: ResMut<bevy::asset::Assets<StandardMaterial>>,
) {
    commands.insert_resource(Assets {
        mesh: pawns.mesh.clone(),
        normal: materials.add(Color::srgb(0.55, 0.24, 0.65)),
        selected: materials.add(Color::srgb(1., 0.8, 0.1)),
    });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(108),
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
        ))
        .with_child((
            TargetHud,
            Text::new("Targets loading..."),
            TextFont {
                font_size: FontSize::Px(18.),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}
fn next_target(selected: Option<EntityId>, ids: &[EntityId]) -> Option<EntityId> {
    if ids.is_empty() {
        return None;
    }
    let next = selected
        .and_then(|id| ids.iter().position(|&other| other == id))
        .map_or(0, |i| (i + 1) % ids.len());
    Some(ids[next])
}
pub(crate) fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    network: Res<Network>,
    classic: Option<Res<crate::classic_controls::Controller>>,
    inventory: Option<Res<crate::inventory::Ui>>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
    mut selection: ResMut<crate::world_input::Selection>,
    windows: Query<&Window>,
) {
    if !network.connected() || windows.single().is_ok_and(|w| !w.focused) {
        return;
    }
    let state = network.state();
    let Some(targeting) = &state.targeting else {
        return;
    };
    let panel = classic.as_ref().is_some_and(|c| c.escape_owned_by_panel)
        || inventory.as_ref().is_some_and(|u| u.open)
        || dialogue
            .as_ref()
            .is_some_and(|u| u.open || u.pending || u.close_when_open);
    if classic.is_some() && panel {
        return;
    }
    if keys.just_pressed(KeyCode::Backspace)
        || (classic.is_some() && !panel && keys.just_pressed(KeyCode::Escape))
    {
        network.request_target(None);
    } else if keys.just_pressed(KeyCode::Tab) {
        if let Some(classic) = classic {
            use theia_core::target_search::{self, Candidate, Direction, Observer};
            let Some(correction) = state.correction else {
                return;
            };
            let owner = correction.state;
            let candidates: Vec<_> = targeting
                .targets
                .iter()
                .map(|target| Candidate {
                    key: target.key,
                    position: target.position,
                    eligible: target.position.chunk(128).is_some_and(|id| {
                        state
                            .chunk_report
                            .as_ref()
                            .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
                    }) && !state.combat.as_ref().is_some_and(|c| {
                        c.targets
                            .iter()
                            .any(|h| h.entity == target.key.entity && h.current == 0)
                    }),
                })
                .collect();
            let selected = targeting.selected.map(|entity| EntityKey {
                epoch: owner.actor.epoch,
                entity,
            });
            let direction = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                Direction::Reverse
            } else {
                Direction::Forward
            };
            if let Ok(Some(key)) = target_search::cycle(
                classic.targeting(),
                Observer {
                    epoch: owner.actor.epoch,
                    position: owner.pose.feet,
                    facing: classic.facing(),
                },
                selected,
                &candidates,
                direction,
            ) {
                network.request_target(Some(key.entity));
                selection.friendly = None;
            }
            return;
        }
        let ids: Vec<_> = targeting
            .targets
            .iter()
            .filter(|t| {
                t.position.chunk(128).is_some_and(|id| {
                    state
                        .chunk_report
                        .as_ref()
                        .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
                })
            })
            .map(|t| t.key.entity)
            .collect();
        if let Some(target) = next_target(targeting.selected, &ids) {
            network.request_target(Some(target));
        }
    }
}
type VisualComponents<'a> = (
    Entity,
    &'a TargetVisual,
    Option<&'a mut motion::TargetMotion>,
    &'a mut Transform,
    &'a mut MeshMaterial3d<StandardMaterial>,
    &'a mut Visibility,
);
fn present(
    mut commands: Commands,
    scene: motion::Presentation,
    mut visuals: Query<VisualComponents<'_>>,
    mut hud: Query<&mut Text, With<TargetHud>>,
    selection: Res<crate::world_input::Selection>,
    classic: Option<Res<crate::classic_controls::Controller>>,
) {
    let motion::Presentation {
        network,
        frame,
        assets,
        time,
        mut metrics,
        creatures,
    } = scene;
    let state = network.state();
    let view = state.targeting.as_ref().filter(|_| state.error.is_none());
    let mut existing = std::collections::BTreeSet::new();
    for (entity, visual, motion, mut transform, mut material, mut visibility) in &mut visuals {
        let target = view.and_then(|v| v.targets.iter().find(|t| t.key == visual.key));
        let Some(target) = target else {
            if let Some(position) = creatures.as_ref().and_then(|c| c.corpse(visual.key)) {
                let shown = position.chunk(128).is_some_and(|id| {
                    state
                        .chunk_report
                        .as_ref()
                        .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
                });
                if let Some(local) = frame.render.to_local(position) {
                    transform.translation = Vec3::from_array(local.metres());
                    *visibility = if shown {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                } else {
                    *visibility = Visibility::Hidden;
                }
                continue;
            }
            commands.entity(entity).despawn();
            continue;
        };
        existing.insert(target.key.entity);
        let global = if let Some(mut motion) = motion {
            motion.observe(
                target.position,
                view.unwrap().tick,
                state.snapshot.as_ref().map_or(30, |s| s.tick_hz),
                time.delta_secs(),
                &mut metrics,
            )
        } else {
            target.position
        };
        let shown = target.position.chunk(128).is_some_and(|id| {
            state
                .chunk_report
                .as_ref()
                .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
        });
        let shown = shown
            && global.chunk(128).is_some_and(|id| {
                state
                    .chunk_report
                    .as_ref()
                    .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
            });
        if let Some(local) = frame.render.to_local(global) {
            transform.translation = Vec3::from_array(local.metres());
            *visibility = if shown {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            *visibility = Visibility::Hidden;
        }
        material.0 = if view.unwrap().selected == Some(target.key.entity) {
            assets.selected.clone()
        } else {
            assets.normal.clone()
        };
    }
    if let Some(view) = view {
        for target in &view.targets {
            if existing.contains(&target.key.entity) {
                continue;
            }
            let Some(local) = frame.render.to_local(target.position) else {
                continue;
            };
            let shown = target.position.chunk(128).is_some_and(|id| {
                state
                    .chunk_report
                    .as_ref()
                    .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
            });
            let mut entity = commands.spawn((
                TargetVisual { key: target.key },
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(if view.selected == Some(target.key.entity) {
                    assets.selected.clone()
                } else {
                    assets.normal.clone()
                }),
                Transform::from_translation(Vec3::from_array(local.metres())),
                if shown {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ));
            if metrics.enabled {
                entity.insert(motion::TargetMotion::new(
                    target.key,
                    target.position,
                    view.tick,
                ));
            }
        }
    }
    let cycle_hint = if classic.is_some() {
        "Tab / Shift+Tab"
    } else {
        "Tab"
    };
    if let Ok(mut text) = hud.single_mut() {
        if selection.friendly.is_some() && state.error.is_none() {
            **text = "Target: Guide\nF / right-click: interact".into();
            return;
        }
        **text = match view {
            Some(v) if v.selected.is_some() => {
                if creatures.is_some() {
                    format!("Target: Wolf\n{cycle_hint} | Backspace: clear")
                } else {
                    format!("Target: Training target\n{cycle_hint} | Backspace: clear")
                }
            }
            Some(v) => format!(
                "No target | {} known\n{cycle_hint} | Backspace: clear",
                v.targets.len()
            ),
            None if state.error.is_some() => "Disconnected".into(),
            None => "Targets loading...".into(),
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycling_recovers_from_removed_selection_and_wraps_without_selecting_unknown_ids() {
        let a = EntityId::new(1).unwrap();
        let b = EntityId::new(2).unwrap();
        let removed = EntityId::new(3).unwrap();
        assert_eq!(next_target(None, &[]), None);
        assert_eq!(next_target(None, &[a, b]), Some(a));
        assert_eq!(next_target(Some(a), &[a, b]), Some(b));
        assert_eq!(next_target(Some(b), &[a, b]), Some(a));
        assert_eq!(next_target(Some(removed), &[a, b]), Some(a));
    }
}
