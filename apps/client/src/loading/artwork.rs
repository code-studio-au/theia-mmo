//! Zone artwork is presentation-only; admitted package identities stay unchanged.
use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(crate) enum Artwork {
    #[default]
    Valley,
    Millhaven,
}
impl Artwork {
    pub(crate) fn for_package(root: &std::path::Path) -> theia_network::Result<Self> {
        use theia_network::content::{MANIFEST_FILE, Manifest};
        Ok(
            Manifest::load_optional(&root.join(MANIFEST_FILE))?.map_or(Self::Valley, |manifest| {
                Self::for_id(manifest.content_id().as_str())
            }),
        )
    }
    fn for_id(id: &str) -> Self {
        match id {
            // Millhaven's current editor export retains its original stable ID.
            "theia.zone.starter-valley" | "theia.zone.millhaven" => Self::Millhaven,
            _ => Self::Valley,
        }
    }
    pub(super) fn bytes(self) -> &'static [u8] {
        match self {
            Self::Valley => include_bytes!("../../../../content/ui/loading/theia-valley-v1.png"),
            Self::Millhaven => {
                include_bytes!("../../../../content/ui/loading/millhaven-valley-sunset.png")
            }
        }
    }
    pub(super) fn aspect(self) -> f32 {
        match self {
            Self::Valley => 1672. / 941.,
            Self::Millhaven => 1671. / 941.,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn millhaven_art_is_scoped_to_exact_zone_ids_and_both_embedded_images_decode() {
        assert_eq!(
            Artwork::for_id("theia.zone.starter-valley"),
            Artwork::Millhaven
        );
        assert_eq!(Artwork::for_id("theia.zone.millhaven"), Artwork::Millhaven);
        for other in ["theia.zone.other", "theia.zone.millhaven-annex", ""] {
            assert_eq!(Artwork::for_id(other), Artwork::Valley);
        }
        for art in [Artwork::Valley, Artwork::Millhaven] {
            let image = Image::from_buffer(
                art.bytes(),
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::NONE,
                true,
                bevy::image::ImageSampler::default(),
                bevy::asset::RenderAssetUsages::default(),
            )
            .unwrap();
            assert!((image.width() as f32 / image.height() as f32 - art.aspect()).abs() < 0.0001);
        }
    }
}
