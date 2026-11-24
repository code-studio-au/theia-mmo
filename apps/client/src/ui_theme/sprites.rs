//! Approved regenerated artwork, embedded so installed clients need no source-tree paths.
use bevy::{
    prelude::*,
    sprite::{BorderRect, TextureSlicer},
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Manifest {
    assets: BTreeMap<String, Sprite>,
}
#[derive(Deserialize)]
struct Sprite {
    rect: [f32; 4],
    atlas: usize,
    nine_slice: Option<[f32; 4]>,
    pixels_per_ui_unit: f32,
}
#[derive(Resource)]
pub(super) struct Sprites {
    manifest: Manifest,
    images: Vec<Handle<Image>>,
}
impl Sprites {
    #[cfg(test)]
    pub(super) fn fixture() -> Self {
        Self {
            manifest: serde_json::from_str(include_str!(
                "../../../../content/ui/sprites-v3/manifest.json"
            ))
            .unwrap(),
            images: vec![Handle::default(); 9],
        }
    }
    pub(super) fn load(images: &mut Assets<Image>) -> Self {
        let bytes: [&[u8]; 9] = [
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-00.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-01.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-02.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-03.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-04.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-05.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-06.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-07.png"),
            include_bytes!("../../../../content/ui/sprites-v3/atlases/ui-08.png"),
        ];
        Self {
            manifest: serde_json::from_str(include_str!(
                "../../../../content/ui/sprites-v3/manifest.json"
            ))
            .expect("Qualified sprite manifest"),
            images: bytes.into_iter().map(|b| images.add(decode(b))).collect(),
        }
    }
    pub(super) fn node(&self, name: &str) -> ImageNode {
        let s = &self.manifest.assets[name];
        let [x0, y0, x1, y1] = s.rect;
        ImageNode {
            image: self.images[s.atlas].clone(),
            rect: Some(Rect::new(x0, y0, x1, y1)),
            visual_box: bevy::ui::VisualBox::BorderBox,
            image_mode: s.nine_slice.map_or(
                NodeImageMode::Stretch,
                |[left, top, right, bottom]| {
                    NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect {
                            min_inset: Vec2::new(left, top),
                            max_inset: Vec2::new(right, bottom),
                        },
                        max_corner_scale: 1. / s.pixels_per_ui_unit,
                        ..default()
                    })
                },
            ),
            ..default()
        }
    }
}
pub(crate) fn decode(bytes: &[u8]) -> Image {
    Image::from_buffer(
        bytes,
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::linear(),
        default(),
    )
    .expect("Qualified embedded Theia artwork")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_manifest_keeps_all_control_states_inside_their_atlas() {
        let m: Manifest = serde_json::from_str(include_str!(
            "../../../../content/ui/sprites-v3/manifest.json"
        ))
        .unwrap();
        assert_eq!(m.assets.len(), 416);
        for s in m.assets.values() {
            assert!(s.atlas < 9);
            assert_eq!(s.pixels_per_ui_unit, 4.);
            assert!(s.rect[0] >= 0. && s.rect[1] >= 0. && s.rect[2] <= 2048. && s.rect[3] <= 2048.);
        }
        for family in [
            "button.primary",
            "button.secondary",
            "input.standard",
            "tab.chamfered",
            "control.close",
            "checkbox",
            "slot.empty",
        ] {
            for state in [
                "default", "hover", "pressed", "selected", "focused", "disabled",
            ] {
                assert!(m.assets.contains_key(&format!("{family}.{state}")));
            }
        }
    }
}
