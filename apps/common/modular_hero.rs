//! Shared standard-material modular hero presentation for game and creation preview.
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use theia_core::appearance::Appearance;

#[derive(Component)]
struct Applied(Appearance);
#[derive(Resource, Default)]
struct Palettes(BTreeMap<String, Handle<StandardMaterial>>);

/// Returns false while the world asset is still instantiating. Caller keeps it hidden.
pub fn apply(
    world: &mut World,
    root: Entity,
    appearance: Appearance,
    source: &str,
) -> Result<bool, String> {
    if world
        .get::<Applied>(root)
        .is_some_and(|a| a.0 == appearance)
    {
        return Ok(true);
    }
    let mut entities = vec![root];
    let mut index = 0;
    while index < entities.len() {
        if let Some(children) = world.get::<Children>(entities[index]) {
            entities.extend(children.iter());
        }
        index += 1;
    }
    let parts: Vec<_> = entities
        .iter()
        .filter_map(|&e| {
            let name = world.get::<Name>(e)?.as_str();
            name.starts_with("Chr_").then(|| (e, name.to_owned()))
        })
        .collect();
    if parts.is_empty() {
        return Ok(false);
    }
    let selected: BTreeSet<_> = appearance.parts().into_iter().collect();
    if !selected
        .iter()
        .all(|name| parts.iter().any(|(_, actual)| actual == name))
    {
        return Err("Modular hero is missing a selected body part".into());
    }
    // Equipment is attached elsewhere on this skeleton. Palette changes must
    // only affect the modular body parts, never a weapon or another attachment.
    let material_entities = part_materials(world, parts.iter().map(|(e, _)| *e));
    if material_entities.is_empty() {
        return Ok(false);
    }
    let palette_path = format!("{source}://{}", appearance.palette());
    world.init_resource::<Palettes>();
    let handle = if let Some(handle) = world.resource::<Palettes>().0.get(&palette_path) {
        handle.clone()
    } else {
        let texture = world.resource::<AssetServer>().load(palette_path.clone());
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: Some(texture),
                perceptual_roughness: 0.8,
                ..default()
            });
        world
            .resource_mut::<Palettes>()
            .0
            .insert(palette_path, material.clone());
        material
    };
    let texture = world
        .resource::<Assets<StandardMaterial>>()
        .get(&handle)
        .and_then(|m| m.base_color_texture.as_ref())
        .unwrap();
    let server = world.resource::<AssetServer>();
    if matches!(
        server.load_state(texture.id()),
        bevy::asset::LoadState::Failed(_)
    ) {
        return Err("Hero palette failed to load".into());
    }
    if !server.is_loaded_with_dependencies(texture.id()) {
        return Ok(false);
    }
    for (entity, name) in parts {
        world
            .entity_mut(entity)
            .insert(if selected.contains(&name) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
    }
    for entity in material_entities {
        world
            .entity_mut(entity)
            .insert(MeshMaterial3d(handle.clone()));
    }
    world.entity_mut(root).insert(Applied(appearance));
    Ok(true)
}

/// Read-only evidence for the opt-in native qualification report.
pub fn qualification(world: &mut World) -> serde_json::Value {
    let roots: Vec<_> = world
        .query::<(Entity, &Applied)>()
        .iter(world)
        .map(|(e, a)| (e, a.0))
        .collect();
    serde_json::json!(roots.into_iter().map(|(root, appearance)| {
        let mut entities = vec![root];
        let mut index = 0;
        while index < entities.len() {
            if let Some(children) = world.get::<Children>(entities[index]) { entities.extend(children.iter()); }
            index += 1;
        }
        let parts: Vec<_> = entities.iter().filter_map(|&e| {
            let name = world.get::<Name>(e)?;
            (name.as_str().starts_with("Chr_") && world.get::<Visibility>(e) != Some(&Visibility::Hidden)).then(|| name.as_str().to_owned())
        }).collect();
        serde_json::json!({"appearance":appearance,"visible_parts":parts,"position":world.get::<GlobalTransform>(root).map(|t|t.translation().to_array())})
    }).collect::<Vec<_>>())
}

fn part_materials(world: &World, roots: impl Iterator<Item = Entity>) -> Vec<Entity> {
    let mut result = Vec::new();
    for root in roots {
        let mut entities = vec![root];
        let mut index = 0;
        while index < entities.len() {
            let entity = entities[index];
            if let Some(children) = world.get::<Children>(entity) {
                entities.extend(children.iter());
            }
            if world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_some()
            {
                result.push(entity);
            }
            index += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn body_palette_selection_excludes_equipment_on_the_same_rig() {
        let mut world = World::new();
        let rig = world.spawn_empty().id();
        let body = world.spawn(ChildOf(rig)).id();
        let skin = world
            .spawn((
                ChildOf(body),
                MeshMaterial3d::<StandardMaterial>(Handle::default()),
            ))
            .id();
        let hand = world.spawn(ChildOf(rig)).id();
        let weapon = world
            .spawn((
                ChildOf(hand),
                MeshMaterial3d::<StandardMaterial>(Handle::default()),
            ))
            .id();
        let materials = part_materials(&world, [body].into_iter());
        assert_eq!(materials, vec![skin]);
        assert!(!materials.contains(&weapon));
    }
}
