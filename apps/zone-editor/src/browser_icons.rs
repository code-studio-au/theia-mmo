//! Original pinned Sprinkles/Remix artwork; see third_party/sprinkles/PROVENANCE.md.
use super::*;
use bevy::asset::{embedded_asset, load_embedded_asset};

#[derive(Resource)]
pub(super) struct Icons {
    pub folder: Handle<Image>,
    pub file: Handle<Image>,
}

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "browser_icons/ri-file-line.png");
    embedded_asset!(app, "browser_icons/ri-folder-open-line.png");
    let server = app.world().resource::<AssetServer>();
    let icons = Icons {
        folder: load_embedded_asset!(server, "browser_icons/ri-folder-open-line.png"),
        file: load_embedded_asset!(server, "browser_icons/ri-file-line.png"),
    };
    app.insert_resource(icons);
}
