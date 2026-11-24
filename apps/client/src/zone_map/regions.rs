//! Presentation-only geographic masks. No zone streaming or gameplay ownership.
use super::*;
use std::io::Read;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct Region {
    pub(super) id: u8,
    pub(super) name: String,
}
pub(super) fn valid(regions: &[Region]) -> bool {
    regions.len() <= 20
        && regions.iter().enumerate().all(|(i, r)| {
            r.id != 0
                && !r.name.is_empty()
                && r.name.len() <= 64
                && regions[..i].iter().all(|other| other.id != r.id)
        })
}
#[derive(Resource)]
pub(super) struct Mask {
    width: usize,
    height: usize,
    ids: Vec<u8>,
    edges: Vec<bool>,
}
impl Mask {
    fn new(width: usize, height: usize, ids: Vec<u8>) -> Self {
        let mut edges = vec![false; ids.len()];
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                edges[i] = ids[i] != 0
                    && (x == 0
                        || y == 0
                        || x + 1 == width
                        || y + 1 == height
                        || ids[i - 1] != ids[i]
                        || ids[i + 1] != ids[i]
                        || ids[i - width] != ids[i]
                        || ids[i + width] != ids[i]);
            }
        }
        Self {
            width,
            height,
            ids,
            edges,
        }
    }
    pub(super) fn at(&self, uv: Vec2) -> Option<u8> {
        if !uv.is_finite() || uv.min_element() < 0. || uv.max_element() >= 1. {
            return None;
        }
        let id = self.ids[(uv.y * self.height as f32) as usize * self.width
            + (uv.x * self.width as f32) as usize];
        (id != 0).then_some(id)
    }
    fn pixels(&self, selected: Option<u8>) -> Vec<u8> {
        let mut rgba = vec![0; self.ids.len() * 4];
        if let Some(id) = selected {
            for (i, v) in self.ids.iter().enumerate() {
                if *v == id {
                    rgba[i * 4..i * 4 + 4].copy_from_slice(if self.edges[i] {
                        &[255, 219, 111, 255]
                    } else {
                        &[255, 210, 85, 35]
                    });
                }
            }
        }
        rgba
    }
}
pub(super) fn load(path: &std::path::Path, data: &Data) -> Result<Option<Mask>, String> {
    if data.regions.is_empty() {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path.join("regions.png"))
        .map_err(|e| e.to_string())?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("Boundary mask exceeds byte budget".into());
    }
    // Reuse the image-size/header admission without duplicating a decoder.
    let image = super::decode_image(&bytes)?;
    let width = image.width() as usize;
    let height = image.height() as usize;
    let rgba = image.data.ok_or("Missing boundary pixels")?;
    if rgba.len() != width * height * 4 {
        return Err("Boundary mask must be RGBA8".into());
    }
    let ids: Vec<_> = rgba.as_chunks::<4>().0.iter().map(|p| p[0]).collect();
    if ids
        .iter()
        .any(|id| *id != 0 && !data.regions.iter().any(|r| r.id == *id))
    {
        return Err("Unknown boundary region ID".into());
    }
    Ok(Some(Mask::new(width, height, ids)))
}
#[derive(Resource)]
struct Overlay {
    image: Handle<Image>,
    hovered: Option<u8>,
}
#[derive(Component)]
pub(super) struct HoverLabel;
pub(super) fn setup(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    mask: &Mask,
) -> Handle<Image> {
    let image = images.add(Image::new(
        bevy::render::render_resource::Extent3d {
            width: mask.width as u32,
            height: mask.height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        mask.pixels(None),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    ));
    commands.insert_resource(Overlay {
        image: image.clone(),
        hovered: None,
    });
    image
}
pub(super) fn present(world: &mut World, visible: bool) {
    if !world.contains_resource::<Mask>() || !world.contains_resource::<Overlay>() {
        return;
    }
    let scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    let cursor = world
        .query::<&Window>()
        .iter(world)
        .next()
        .filter(|w| w.focused && visible)
        .and_then(|w| w.cursor_position());
    let uv = cursor.and_then(|cursor| {
        let (node, transform) = world
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Surface>>()
            .iter(world)
            .next()?;
        let factor = crate::fantasy_ui::window_factor(node, scale);
        let size = node.size() * factor;
        let min = transform.affine().translation * factor - size * 0.5;
        Some((cursor - min) / size)
    });
    let hovered = uv.and_then(|uv| world.resource::<Mask>().at(uv));
    if hovered == world.resource::<Overlay>().hovered {
        return;
    }
    let pixels = world.resource::<Mask>().pixels(hovered);
    let image = world.resource::<Overlay>().image.clone();
    world
        .resource_mut::<Assets<Image>>()
        .get_mut(&image)
        .unwrap()
        .data = Some(pixels);
    world.resource_mut::<Overlay>().hovered = hovered;
    let caption = hovered
        .and_then(|id| {
            world
                .resource::<Data>()
                .regions
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.name.clone())
        })
        .unwrap_or_else(|| "Hover over a zone to see its boundary".into());
    for mut text in world
        .query_filtered::<&mut Text, With<HoverLabel>>()
        .iter_mut(world)
    {
        **text = caption.clone();
    }
}

pub(super) fn qualification_target(world: &mut World, index: usize) -> Option<(u8, Vec2)> {
    let id = world.resource::<Data>().regions.get(index)?.id;
    let mask = world.get_resource::<Mask>()?;
    let matches: Vec<_> = mask
        .ids
        .iter()
        .enumerate()
        .filter(|(_, v)| **v == id)
        .map(|(i, _)| i)
        .collect();
    let i = *matches.get(matches.len() / 2)?;
    let uv = Vec2::new((i % mask.width) as f32 + 0.5, (i / mask.width) as f32 + 0.5)
        / Vec2::new(mask.width as f32, mask.height as f32);
    let scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    let (node, transform) = world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Surface>>()
        .iter(world)
        .next()?;
    let factor = crate::fantasy_ui::window_factor(node, scale);
    let size = node.size() * factor;
    Some((
        id,
        transform.affine().translation * factor - size * 0.5 + uv * size,
    ))
}
pub(super) fn qualification_hover(world: &World) -> Option<u8> {
    world.get_resource::<Overlay>().and_then(|o| o.hovered)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hover_uses_irregular_mask_and_clears_over_sea_and_outside() {
        let mask = Mask::new(4, 3, vec![1, 1, 0, 2, 1, 0, 2, 2, 0, 2, 2, 2]);
        assert_eq!(mask.at(Vec2::new(0.1, 0.1)), Some(1));
        assert_eq!(mask.at(Vec2::new(0.6, 0.1)), None);
        assert_eq!(mask.at(Vec2::new(0.6, 0.6)), Some(2));
        assert_eq!(mask.at(Vec2::ONE), None);
        assert_eq!(mask.at(Vec2::splat(f32::NAN)), None);
        assert!(mask.pixels(None).iter().all(|v| *v == 0));
        let pixels = mask.pixels(Some(1));
        assert_eq!(&pixels[..4], &[255, 219, 111, 255]);
        assert_eq!(&pixels[12..16], &[0; 4]);
    }
    #[test]
    fn regional_map_startup_builds_hover_label_without_duplicate_components() {
        let (data, image) =
            super::super::decode_map(super::super::MAP_JSON, super::super::MAP_PNG).unwrap();
        let mut app = App::new();
        app.insert_resource(data)
            .insert_resource(super::super::MapImage(image))
            .insert_resource(Mask::new(2, 2, vec![1, 1, 0, 2]))
            .init_resource::<Assets<Image>>()
            .add_systems(Update, super::super::setup);
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<&Text, With<HoverLabel>>()
                .iter(world)
                .count(),
            1
        );
    }
    #[test]
    fn duplicate_and_zero_ids_are_rejected() {
        let r = || Region {
            id: 1,
            name: "Millhaven".into(),
        };
        assert!(!valid(&[r(), r()]));
        assert!(!valid(&[Region {
            id: 0,
            name: "Sea".into()
        }]));
        assert!(valid(&[r()]));
    }
}
