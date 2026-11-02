//! Classic manual input. Authority receives bounded movement and independent yaw.
use bevy::prelude::*;
use serde::Deserialize;
use theia_core::{facing::Facing, identity::EntityKey};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    turn_degrees_per_second: f64,
    bindings: RawBindings,
    #[serde(default)]
    targeting: theia_core::target_search::Settings,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBindings {
    forward: String,
    backward: String,
    turn_left: Option<String>,
    turn_right: Option<String>,
    strafe_left: String,
    strafe_right: String,
    jump: String,
    run_modifier: String,
}
struct Bindings {
    forward: KeyCode,
    backward: KeyCode,
    turn_left: Option<KeyCode>,
    turn_right: Option<KeyCode>,
    strafe_left: KeyCode,
    strafe_right: KeyCode,
    jump: KeyCode,
    run_modifier: KeyCode,
}
fn key(name: &str) -> Result<KeyCode, String> {
    use KeyCode::*;
    Ok(match name {
        "KeyA" => KeyA,
        "KeyB" => KeyB,
        "KeyC" => KeyC,
        "KeyD" => KeyD,
        "KeyE" => KeyE,
        "KeyF" => KeyF,
        "KeyG" => KeyG,
        "KeyH" => KeyH,
        "KeyI" => KeyI,
        "KeyJ" => KeyJ,
        "KeyK" => KeyK,
        "KeyL" => KeyL,
        "KeyM" => KeyM,
        "KeyN" => KeyN,
        "KeyO" => KeyO,
        "KeyP" => KeyP,
        "KeyQ" => KeyQ,
        "KeyR" => KeyR,
        "KeyS" => KeyS,
        "KeyT" => KeyT,
        "KeyU" => KeyU,
        "KeyV" => KeyV,
        "KeyW" => KeyW,
        "KeyX" => KeyX,
        "KeyY" => KeyY,
        "KeyZ" => KeyZ,
        "ArrowUp" => ArrowUp,
        "ArrowDown" => ArrowDown,
        "ArrowLeft" => ArrowLeft,
        "ArrowRight" => ArrowRight,
        "Space" => Space,
        "ShiftLeft" => ShiftLeft,
        "ShiftRight" => ShiftRight,
        "ControlLeft" => ControlLeft,
        "ControlRight" => ControlRight,
        _ => return Err(format!("Unsupported classic movement key: {name}")),
    })
}
#[derive(Resource)]
pub struct Controller {
    pub escape_owned_by_panel: bool,
    bindings: Bindings,
    targeting: theia_core::target_search::Settings,
    turn_radians_per_second: f64,
    yaw: f64,
    context: Option<(EntityKey, u64)>,
}
pub struct Sample {
    pub axes: [i16; 2],
    pub jump: bool,
    pub sprint: bool,
    pub keyboard_turn: f32,
}
impl Controller {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 4096 {
            return Err("Classic controls exceed 4096 bytes".into());
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if !raw.targeting.valid()
            || raw.schema != 1
            || !raw.turn_degrees_per_second.is_finite()
            || !(1. ..=720.).contains(&raw.turn_degrees_per_second)
        {
            return Err("Invalid classic controls schema or keyboard turn rate".into());
        }
        let b = raw.bindings;
        let bindings = Bindings {
            forward: key(&b.forward)?,
            backward: key(&b.backward)?,
            turn_left: b.turn_left.as_deref().map(key).transpose()?,
            turn_right: b.turn_right.as_deref().map(key).transpose()?,
            strafe_left: key(&b.strafe_left)?,
            strafe_right: key(&b.strafe_right)?,
            jump: key(&b.jump)?,
            run_modifier: key(&b.run_modifier)?,
        };
        let values = [
            Some(bindings.forward),
            Some(bindings.backward),
            bindings.turn_left,
            bindings.turn_right,
            Some(bindings.strafe_left),
            Some(bindings.strafe_right),
            Some(bindings.jump),
            Some(bindings.run_modifier),
        ];
        let mut unique = std::collections::HashSet::new();
        if values
            .into_iter()
            .flatten()
            .any(|k| matches!(k, KeyCode::KeyF | KeyCode::KeyI))
        {
            return Err(
                "Movement key conflicts with the current F interaction or I backpack binding"
                    .into(),
            );
        }
        if values.into_iter().flatten().any(|k| !unique.insert(k)) {
            return Err("Conflicting classic movement bindings".into());
        }
        Ok(Self {
            escape_owned_by_panel: false,
            bindings,
            targeting: raw.targeting,
            turn_radians_per_second: raw.turn_degrees_per_second.to_radians(),
            yaw: 0.,
            context: None,
        })
    }
    pub fn from_args(
        args: &mut Vec<String>,
        enabled: bool,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let bytes = if let Some(i) = args.iter().position(|s| s == "--controls-file") {
            use std::io::Read;
            args.remove(i);
            if !enabled || i >= args.len() {
                return Err("--controls-file requires classic controls and a file".into());
            }
            let mut bytes = Vec::new();
            std::fs::File::open(args.remove(i))?
                .take(4097)
                .read_to_end(&mut bytes)?;
            bytes
        } else {
            include_bytes!("../../../content/input/classic-controls.json").to_vec()
        };
        enabled
            .then(|| Self::parse(&bytes).map_err(Into::into))
            .transpose()
    }
    pub fn targeting(&self) -> theia_core::target_search::Settings {
        self.targeting
    }
    pub fn facing(&self) -> Facing {
        Facing::from_radians(self.yaw).expect("Finite bounded classic input yaw")
    }
    pub fn yaw(&self) -> f32 {
        self.yaw as f32
    }
    pub fn sync(&mut self, correction: Option<theia_network::prediction::Correction>) {
        if let Some(c) = correction {
            let context = (c.actor, c.input.stream_epoch);
            if self.context != Some(context) {
                self.context = Some(context);
                self.yaw = c.pose.facing.unwrap_or_default().radians();
            }
        }
    }
    pub fn sample(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        seconds: f64,
        camera_yaw: f32,
        steering: bool,
        both_buttons: bool,
        allowed: bool,
    ) -> Sample {
        let mut sample = Sample {
            axes: [0, 0],
            jump: false,
            sprint: false,
            keyboard_turn: 0.,
        };
        if !allowed {
            return sample;
        }
        let b = &self.bindings;
        let held = |key| f64::from(u8::from(keys.pressed(key)));
        let turn = b.turn_left.map_or(0., held) - b.turn_right.map_or(0., held);
        if steering && camera_yaw.is_finite() {
            self.yaw = f64::from(camera_yaw);
        } else {
            let dt = if seconds.is_finite() {
                seconds.clamp(0., 0.1)
            } else {
                0.
            };
            let delta = turn * self.turn_radians_per_second * dt;
            self.yaw += delta;
            sample.keyboard_turn = delta as f32;
        }
        self.yaw = self.yaw.rem_euclid(std::f64::consts::TAU);
        let x = held(b.strafe_right) - held(b.strafe_left) + if steering { -turn } else { 0. };
        let z = held(b.backward) - held(b.forward).max(f64::from(u8::from(both_buttons)));
        sample.axes = self
            .facing()
            .world_axes([(x.clamp(-1., 1.) * 1000.) as i16, (z * 1000.) as i16])
            .unwrap();
        sample.jump = keys.pressed(b.jump);
        sample.sprint = keys.pressed(b.run_modifier);
        sample
    }
    pub fn hint(&self) -> String {
        let b = &self.bindings;
        let label = |k: KeyCode| format!("{k:?}").replace("Key", "");
        let optional = |k: Option<KeyCode>| k.map_or("unbound".into(), label);
        format!(
            "{}/{}: move | {}/{}: turn | {}/{}: strafe\nLMB: orbit | RMB: steer | Both: forward | Wheel: zoom\n{}: jump | {}: run | Esc: close / clear target",
            label(b.forward),
            label(b.backward),
            optional(b.turn_left),
            optional(b.turn_right),
            label(b.strafe_left),
            label(b.strafe_right),
            label(b.jump),
            label(b.run_modifier)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn controller() -> Controller {
        Controller::parse(include_bytes!(
            "../../../content/input/classic-controls.json"
        ))
        .unwrap()
    }
    #[test]
    fn target_search_defaults_preserve_old_files_and_metric_radius_is_checked() {
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/input/classic-controls.json"
        ))
        .unwrap();
        raw.as_object_mut().unwrap().remove("targeting");
        let c = Controller::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(c.targeting().prefer_front);
        assert!(c.targeting().maximum_distance_metres.is_none());
        raw["targeting"] = serde_json::json!({"prefer_front":false,"maximum_distance_metres":32.5});
        let c = Controller::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(c.targeting().maximum_distance_metres, Some(32.5));
        for radius in [0., -1., 4097.] {
            raw["targeting"]["maximum_distance_metres"] = serde_json::json!(radius);
            assert!(Controller::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
        }
    }
    #[test]
    fn turn_in_place_then_strafe_and_backpedal_preserve_the_facing() {
        let mut c = controller();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyA);
        for _ in 0..60 {
            assert_eq!(
                c.sample(&keys, 1. / 120., 0., false, false, true).axes,
                [0, 0]
            );
        }
        assert_eq!(c.facing(), Facing::from_units(16384));
        keys.release(KeyCode::KeyA);
        keys.press(KeyCode::KeyE);
        assert_eq!(c.sample(&keys, 0., 2., false, false, true).axes, [0, -1000]);
        assert_eq!(c.facing().units(), 16384);
        keys.release(KeyCode::KeyE);
        keys.press(KeyCode::KeyS);
        assert_eq!(c.sample(&keys, 0., 2., false, false, true).axes, [1000, 0]);
    }
    #[test]
    fn keyboard_turn_is_render_rate_independent_and_hitches_are_bounded() {
        for hz in [30, 60, 120, 240] {
            let mut c = controller();
            let mut keys = ButtonInput::default();
            keys.press(KeyCode::KeyD);
            for _ in 0..hz {
                c.sample(&keys, 1. / f64::from(hz), 0., false, false, true);
            }
            assert_eq!(c.facing().units(), 32768);
        }
        let mut c = controller();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyA);
        let s = c.sample(&keys, 20., 0., false, false, true);
        assert!((s.keyboard_turn - std::f32::consts::PI * 0.1).abs() < 0.00001);
        c.sample(&keys, f64::NAN, f32::NAN, false, false, true);
        assert!(c.yaw().is_finite());
    }
    #[test]
    fn mouse_steers_stationary_and_both_buttons_move_forward_without_camera_target_coupling() {
        let mut c = controller();
        let keys = ButtonInput::default();
        let yaw = std::f32::consts::FRAC_PI_2;
        assert_eq!(c.sample(&keys, 0., yaw, true, false, true).axes, [0, 0]);
        assert_eq!(c.facing().units(), 16384);
        assert_eq!(c.sample(&keys, 0., yaw, true, true, true).axes, [-1000, 0]);
        c.sample(&keys, 0., 0., false, false, true);
        assert_eq!(c.facing().units(), 16384);
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyA);
        assert_eq!(c.sample(&keys, 0., yaw, true, false, true).axes, [0, 1000]);
        assert_eq!(c.facing().units(), 16384);
        assert_eq!(c.sample(&keys, 1., 0., true, true, false).axes, [0, 0]);
        assert_eq!(c.facing().units(), 16384);
    }
    #[test]
    fn bindings_allow_ad_strafe_and_reject_unknown_conflicting_or_oversized_settings() {
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/input/classic-controls.json"
        ))
        .unwrap();
        raw["bindings"]["turn_left"] = serde_json::json!("ArrowLeft");
        raw["bindings"]["turn_right"] = serde_json::json!("ArrowRight");
        raw["bindings"]["strafe_left"] = serde_json::json!("KeyA");
        raw["bindings"]["strafe_right"] = serde_json::json!("KeyD");
        let mut c = Controller::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyA);
        assert_eq!(
            c.sample(&keys, 1. / 60., 0., false, false, true).axes,
            [-1000, 0]
        );
        assert_eq!(c.facing(), Facing::default());
        raw["bindings"]["forward"] = serde_json::json!("KeyA");
        assert!(Controller::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
        raw["bindings"]["forward"] = serde_json::json!("Unknown");
        assert!(Controller::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
        assert!(Controller::parse(&vec![b' '; 4097]).is_err());
    }
}
