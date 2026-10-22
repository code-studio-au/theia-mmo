//! Explicit source renderer flags shared by inspection and static world imports.
use bevy::{
    gltf::GltfMaterialExtras,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use serde_json::Value;

pub(crate) fn parse(extra: &str) -> Result<Option<(bool, bool)>, &'static str> {
    let value: Value = serde_json::from_str(extra).map_err(|_| "Invalid material extras JSON")?;
    let Some(flags) = value.get("theia_render_flags") else {
        return Ok(None);
    };
    let object = flags.as_object().ok_or("Render flags must be an object")?;
    if object.len() != 2
        || object
            .keys()
            .any(|key| !matches!(key.as_str(), "cast_shadows" | "receive_shadows"))
    {
        return Err("Unsupported source render flag fields");
    }
    Ok(Some((
        flags["cast_shadows"]
            .as_bool()
            .ok_or("Cast-shadows flag must be boolean")?,
        flags["receive_shadows"]
            .as_bool()
            .ok_or("Receive-shadows flag must be boolean")?,
    )))
}

pub fn apply(
    mut commands: Commands,
    extras: Query<(Entity, &GltfMaterialExtras), Added<GltfMaterialExtras>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (entity, extra) in &extras {
        match parse(&extra.value) {
            Ok(Some((cast, receive))) => {
                let mut entity = commands.entity(entity);
                if cast {
                    entity.remove::<NotShadowCaster>();
                } else {
                    entity.insert(NotShadowCaster);
                }
                if receive {
                    entity.remove::<NotShadowReceiver>();
                } else {
                    entity.insert(NotShadowReceiver);
                }
            }
            Ok(None) => {}
            Err(error) => {
                error!("{error}");
                exit.write(AppExit::error());
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_flags_preserve_defaults_for_unmarked_materials() {
        assert_eq!(parse(r#"{"theia_source_slot":0}"#), Ok(None));
        assert_eq!(
            parse(r#"{"theia_render_flags":{"cast_shadows":false,"receive_shadows":false}}"#),
            Ok(Some((false, false)))
        );
        assert_eq!(
            parse(r#"{"theia_render_flags":{"cast_shadows":true,"receive_shadows":true}}"#),
            Ok(Some((true, true)))
        );
    }
    #[test]
    fn malformed_missing_unknown_and_numeric_flags_reject() {
        for value in [
            r#"{"theia_render_flags":null}"#,
            r#"{"theia_render_flags":true}"#,
            r#"{"theia_render_flags":{"cast_shadows":false}}"#,
            r#"{"theia_render_flags":{"cast_shadows":0,"receive_shadows":false}}"#,
            r#"{"theia_render_flags":{"cast_shadows":false,"receive_shadows":false,"unknown":true}}"#,
        ] {
            assert!(parse(value).is_err(), "{value}");
        }
    }
}
