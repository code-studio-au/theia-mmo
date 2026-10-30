//! Opt-in standard Bevy presentation of authoritative, nonblocking targets.
use crate::{network::Network, scene::PawnAssets};
use bevy::{prelude::*, text::FontSize};
use theia_core::identity::{EntityId, EntityKey};
mod motion;
mod qualification;
#[derive(Component)]
pub(crate) struct TargetVisual {
    pub key: EntityKey,
}
#[derive(Component)]
struct TargetHud;
#[derive(Resource)]
struct Assets {
    mesh: Handle<Mesh>,
    normal: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}
pub fn install(app: &mut App, moving_speed: Option<f32>) {
    motion::install(app, moving_speed);
    app.add_systems(Startup, setup.after(crate::scene::setup))
        .add_systems(
            Update,
            (controls, present).chain().after(crate::network::present),
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
fn controls(keys: Res<ButtonInput<KeyCode>>, network: Res<Network>) {
    if !network.connected() {
        return;
    }
    let state = network.state();
    let Some(targeting) = &state.targeting else {
        return;
    };
    if keys.just_pressed(KeyCode::Backspace) {
        network.request_target(None);
    } else if keys.just_pressed(KeyCode::Tab) {
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
) {
    let motion::Presentation {
        network,
        frame,
        assets,
        time,
        mut metrics,
    } = scene;
    let state = network.state();
    let view = state.targeting.as_ref().filter(|_| state.error.is_none());
    let mut existing = std::collections::BTreeSet::new();
    for (entity, visual, motion, mut transform, mut material, mut visibility) in &mut visuals {
        let target = view.and_then(|v| v.targets.iter().find(|t| t.key == visual.key));
        let Some(target) = target else {
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
    if let Ok(mut text) = hud.single_mut() {
        **text = match view {
            Some(v) if v.selected.is_some() => {
                "Target: Training target\nTab: cycle | Backspace: clear".into()
            }
            Some(v) => format!(
                "No target | {} nearby\nTab: select | Backspace: clear",
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
