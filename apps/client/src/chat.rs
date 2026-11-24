//! Explicit opt-in to the qualified, server-owned Say/Area chat profile.
use theia_core::chat::Area;
mod bubbles;
mod composer;
pub(crate) use composer::{Ui, row};
pub(crate) fn install(app: &mut bevy::prelude::App) {
    composer::install(app);
    bubbles::install(app);
}
pub(crate) fn snapshot(world: &mut bevy::prelude::World) -> serde_json::Value {
    let mut snapshot = composer::snapshot(world);
    if !snapshot.is_null() {
        snapshot["bubbles"] = bubbles::snapshot(world);
        snapshot["scrollback"] = crate::combat::log::snapshot(world);
        snapshot["opacity"] =
            crate::ui_theme::fade::background_opacity(world, crate::ui_theme::fade::Pane::Chat)
                .into();
    }
    snapshot
}

pub(crate) fn parse(
    args: &mut Vec<String>,
    classic: bool,
) -> Result<Option<Area>, Box<dyn std::error::Error>> {
    let Some(index) = args.iter().position(|a| a == "--chat-area") else {
        return Ok(None);
    };
    args.remove(index);
    if !classic || index >= args.len() {
        return Err("Chat requires Classic controls and one nonzero area ID".into());
    }
    let value = args.remove(index).parse()?;
    if args.iter().any(|a| a == "--chat-area") {
        return Err("Choose chat area once".into());
    }
    Ok(Some(Area::new(value).ok_or("Chat area must be nonzero")?))
}

/// Chat wraps the complete gameplay identity before the optional party suffix.
pub(crate) fn world_identity(source: [u8; 32], area: Option<Area>) -> [u8; 32] {
    area.map_or(source, |area| {
        theia_network::chat::profile_identity(source, area)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chat_profile_is_explicit_bounded_and_requires_classic() {
        for (arguments, classic) in [
            (vec!["--chat-area"], true),
            (vec!["--chat-area", "0"], true),
            (vec!["--chat-area", "4294967296"], true),
            (vec!["--chat-area", "1"], false),
            (vec!["--chat-area", "1", "--chat-area", "2"], true),
        ] {
            assert!(
                parse(
                    &mut arguments.into_iter().map(str::to_owned).collect(),
                    classic
                )
                .is_err()
            );
        }
        assert_eq!(parse(&mut vec![], false).unwrap(), None);
        let mut args = vec!["--chat-area".into(), "7".into(), "--renet-connect".into()];
        assert_eq!(parse(&mut args, true).unwrap(), Area::new(7));
        assert_eq!(args, ["--renet-connect"]);
    }

    #[test]
    fn normal_client_identity_matches_chat_and_party_authority() {
        for parties in [false, true] {
            for area in [None, Area::new(7)] {
                let mut world =
                    theia_network::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
                let source = world.identity();
                world.enable_prediction().unwrap();
                world.enable_classic().unwrap();
                if let Some(area) = area {
                    world.enable_local_chat(area).unwrap();
                }
                if parties {
                    world.enable_parties().unwrap();
                }
                let identity = crate::party::world_identity(
                    world_identity(theia_network::classic::profile_identity(source), area),
                    parties,
                );
                assert_eq!(identity, world.identity());
            }
        }
    }
}
