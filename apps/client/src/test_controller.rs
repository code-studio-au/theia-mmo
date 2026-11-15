//! Bounded local movement scenarios; emit normal Classic input, never edit poses.
use bevy::prelude::*;
use serde::Deserialize;
use std::{collections::BTreeSet, path::PathBuf};
use theia_core::identity::EntityKey;

mod travel;

const MAX_BYTES: usize = 32 * 1024;
#[derive(Clone, Copy, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Step {
    duration_ms: u32,
    #[serde(default)]
    strafe: i16,
    #[serde(default)]
    forward: i16,
    #[serde(default)]
    turn: i8,
    #[serde(default)]
    walk: bool,
    #[serde(default)]
    jump: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Script {
    schema: u32,
    name: String,
    #[serde(default)]
    repeat: bool,
    steps: Vec<Step>,
}
impl Script {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err("Controller script exceeds 32 KiB".into());
        }
        let script: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if script.schema != 1
            || script.name.trim().is_empty()
            || script.name.len() > 80
            || script.name.chars().any(char::is_control)
            || !(1..=64).contains(&script.steps.len())
            || script.steps.iter().any(|s| {
                !(100..=60_000).contains(&s.duration_ms)
                    || !(-1000..=1000).contains(&s.strafe)
                    || !(-1000..=1000).contains(&s.forward)
                    || !(-1..=1).contains(&s.turn)
            })
            || script
                .steps
                .iter()
                .map(|s| u64::from(s.duration_ms))
                .sum::<u64>()
                > 600_000
        {
            return Err("Invalid controller script schema, steps or action bounds".into());
        }
        Ok(script)
    }
}
#[derive(Resource)]
pub(crate) struct Run {
    script: Script,
    travel: Option<travel::Travel>,
    pub active: bool,
    step: usize,
    elapsed: f64,
    context: Option<(EntityKey, u64)>,
    waiting: bool,
    last_report: f64,
}
impl Run {
    fn new(script: Script) -> Self {
        Self {
            script,
            travel: None,
            active: true,
            step: 0,
            elapsed: 0.,
            context: None,
            waiting: true,
            last_report: 0.,
        }
    }
    pub fn description(&self) -> serde_json::Value {
        serde_json::json!({"name":self.script.name,"steps":self.script.steps.len(),"repeat":self.script.repeat,"active":self.active,"travel":self.travel.as_ref().map(|t| t.description())})
    }
    fn finished(&self) -> bool {
        self.travel
            .as_ref()
            .map_or(self.step >= self.script.steps.len(), |t| t.finished())
    }
    fn release_completed_travel(&mut self) -> bool {
        if self.active
            && self
                .travel
                .as_ref()
                .is_some_and(|t| t.finished() || t.failure.is_some())
        {
            self.active = false;
            true
        } else {
            false
        }
    }
    fn toggle(&mut self) {
        self.active = !self.active;
        if self.active {
            self.step = 0;
            self.elapsed = 0.;
            self.context = None;
            if let Some(travel) = &mut self.travel {
                travel.restart();
            }
        }
    }
    fn restart_outward(&mut self) {
        if let Some(travel) = &mut self.travel {
            travel.restart_outward();
            self.active = true;
            self.step = 0;
            self.elapsed = 0.;
            self.context = None;
        }
    }
    fn sample(&mut self, seconds: f64, ready: bool) -> Option<(Step, f64)> {
        self.waiting = !ready;
        if !self.active || !ready || self.step >= self.script.steps.len() {
            return None;
        }
        let step = self.script.steps[self.step];
        let left = f64::from(step.duration_ms) / 1000. - self.elapsed;
        let dt = if seconds.is_finite() {
            seconds.clamp(0., 0.1).min(left.max(0.))
        } else {
            0.
        };
        self.elapsed += dt;
        if self.elapsed + 1e-9 >= f64::from(step.duration_ms) / 1000. {
            self.step += 1;
            self.elapsed = 0.;
            if self.step == self.script.steps.len() && self.script.repeat {
                self.step = 0;
            }
        }
        Some((step, dt))
    }
}
pub(crate) fn from_args(
    args: &mut Vec<String>,
    allowed: bool,
    source: &theia_network::procedural::Source,
) -> Result<Option<Run>, Box<dyn std::error::Error>> {
    let Some(i) = args
        .iter()
        .position(|s| s == "--test-controller" || s == "--test-route")
    else {
        return Ok(None);
    };
    let route = args.remove(i) == "--test-route";
    if !allowed
        || i >= args.len()
        || args
            .iter()
            .any(|s| s == "--test-controller" || s == "--test-route")
    {
        return Err("Test controller requires one script and an authored Classic world".into());
    }
    use std::io::Read;
    let path = PathBuf::from(args.remove(i));
    let mut bytes = Vec::new();
    std::fs::File::open(&path)?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err("Controller script exceeds 32 KiB".into());
    }
    if route {
        let (name, travel, start_paused) = travel::Travel::load(&path, &bytes, source)?;
        let mut run = Run::new(Script {
            schema: 1,
            name,
            repeat: false,
            steps: vec![],
        });
        run.travel = Some(travel);
        run.active = !start_paused;
        Ok(Some(run))
    } else {
        Ok(Some(Run::new(Script::parse(&bytes)?)))
    }
}
#[derive(Component)]
struct Toggle;
#[derive(Component)]
struct Status;
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(25),
                width: percent(50),
                top: px(12),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(30),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Toggle,
                    bevy::ui_widgets::Button,
                    Node {
                        max_width: percent(100),
                        padding: UiRect::axes(px(10), px(6)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.13, 0.15)),
                    BorderColor::all(Color::srgb(0.35, 0.36, 0.39)),
                ))
                .with_child((
                    Status,
                    Text::new("Controller: waiting [F7: manual]"),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
                    Pickable::IGNORE,
                ));
        });
}

/// Read-only painted button bounds for owned native input qualification.
pub(crate) fn button_bounds(world: &mut World) -> Option<[f32; 4]> {
    let ui_scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Toggle>>()
        .iter(world)
        .next()
        .map(|(node, transform)| {
            let factor = crate::fantasy_ui::window_factor(node, ui_scale);
            let size = node.size() * factor;
            let centre = transform.affine().translation * factor;
            [
                centre.x - size.x / 2.,
                centre.y - size.y / 2.,
                size.x,
                size.y,
            ]
        })
}
fn activate(
    event: On<bevy::ui_widgets::Activate>,
    buttons: Query<(), With<Toggle>>,
    mut run: ResMut<Run>,
) {
    if buttons.contains(event.entity) {
        run.toggle();
    }
}
fn toggle(keys: Res<ButtonInput<KeyCode>>, windows: Query<&Window>, mut run: ResMut<Run>) {
    if keys.just_pressed(KeyCode::F7) && windows.single().is_ok_and(|w| w.focused) {
        if run.travel.is_some() && keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            run.restart_outward();
        } else {
            run.toggle();
        }
    }
}
fn ready(
    state: &theia_network::client::State,
    patch: Option<&theia_network::procedural::Patch>,
) -> bool {
    let Some(c) = state.correction else {
        return false;
    };
    let Some(ack) = state
        .correction_ready
        .as_ref()
        .filter(|ack| ack.sequence == c.state.ready_sequence)
    else {
        return false;
    };
    let ready: BTreeSet<_> = ack
        .chunks
        .iter()
        .filter(|id| {
            state
                .chunk_report
                .as_ref()
                .is_some_and(|r| r.chunks.contains(id))
        })
        .map(|&[x, z]| theia_core::coordinates::ChunkId { x, z })
        .collect();
    state.error.is_none()
        && patch.is_some_and(|patch| patch.covers(c.state.pose.feet, Some(&ready)))
}
#[allow(clippy::too_many_arguments)]
fn drive(
    time: Res<Time>,
    mut run: ResMut<Run>,
    network: Res<crate::network::Network>,
    gray: Res<crate::graybox::Graybox>,
    scene: Res<crate::authored_scene::Scene>,
    character: Option<Res<crate::reference_scene::Ready>>,
    mut controller: ResMut<crate::classic_controls::Controller>,
    mut intent: ResMut<crate::movement::MoveIntent>,
    mut orbit: ResMut<crate::camera::Orbit>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !run.active {
        report(
            &mut run,
            &time,
            &network.state(),
            controller.facing().radians(),
            false,
        );
        return;
    }
    if run.travel.is_some()
        && (controller.manual_movement_pressed(&keys)
            || buttons.pressed(MouseButton::Left)
            || buttons.pressed(MouseButton::Right))
    {
        run.active = false;
        run.travel.as_mut().unwrap().cancel("Manual takeover");
    }
    intent.direction = Vec2::ZERO;
    intent.jump = false;
    intent.sprint = false;
    let state = network.state();
    if let Some(c) = state.correction {
        let context = (c.state.actor, c.state.input.stream_epoch);
        if run.context != Some(context) {
            if run.context.is_some()
                && let Some(travel) = &mut run.travel
            {
                travel.cancel("Player incarnation or input stream changed");
            }
            run.step = 0;
            run.elapsed = 0.;
            run.context = Some(context);
        }
        controller.sync(Some(c.state));
    }
    let allowed = network.can_act()
        && scene.ready()
        && character.is_some()
        && ready(&state, gray.patch.as_deref());
    let sample = if run.travel.is_some() {
        run.waiting = !allowed;
        let active = run.active;
        let admitted = run.context.is_some();
        let travel = run.travel.as_mut().unwrap();
        if state.error.is_some() {
            travel.cancel("Network error");
        }
        if admitted && !network.can_act() {
            travel.cancel("Player cannot act");
        }
        if active && allowed {
            let feet = network
                .prediction
                .as_ref()
                .and_then(|p| p.pose())
                .map(|p| p.feet)
                .or_else(|| state.correction.map(|c| c.state.pose.feet));
            feet.and_then(|feet| {
                travel.sample(
                    time.delta_secs_f64(),
                    feet,
                    controller.facing(),
                    controller.keyboard_turn_rate(),
                )
            })
        } else {
            travel.wait(time.delta_secs_f64());
            None
        }
    } else {
        run.sample(time.delta_secs_f64(), allowed)
    };
    if let Some((step, seconds)) = sample {
        let sample = controller.sample_script(
            [step.strafe, -step.forward],
            step.turn,
            step.walk,
            step.jump,
            seconds,
        );
        intent.direction = Vec2::new(
            f32::from(sample.axes[0]) / 1000.,
            f32::from(sample.axes[1]) / 1000.,
        );
        intent.jump = sample.jump;
        intent.sprint = sample.sprint;
        if !buttons.pressed(MouseButton::Left) {
            let angles = orbit.angles();
            orbit.set_angles(angles.x + sample.keyboard_turn, angles.y);
        }
    }
    // Emit the terminal result before returning control to ordinary input on
    // the next frame. A completed route must not swallow subsequent quest input.
    let released = run.release_completed_travel();
    report(
        &mut run,
        &time,
        &state,
        controller.facing().radians(),
        released,
    );
}
// Continue observing authoritative corrections after input ownership is released;
// the terminal predicted arrival can precede its server acknowledgement.
fn report(
    run: &mut Run,
    time: &Time,
    state: &theia_network::client::State,
    facing: f64,
    force: bool,
) {
    if force || time.elapsed_secs_f64() - run.last_report >= 1. {
        run.last_report = time.elapsed_secs_f64();
        eprintln!(
            "{}",
            serde_json::json!({"test_controller":{"name":run.script.name,"active":run.active,"step":run.step,"waiting":run.waiting,"finished":run.finished(),"travel":run.travel.as_ref().map(|t| t.description()),"feet":state.correction.map(|c|c.state.pose.feet.metres()),"facing":facing}})
        );
    }
}
fn status(run: Res<Run>, mut labels: Query<&mut Text, With<Status>>) {
    for mut text in &mut labels {
        text.0 = if let Some(reason) = run.travel.as_ref().and_then(|t| t.failure) {
            format!("{} | Stopped: {reason} [F7: restart]", run.script.name)
        } else if let Some(travel) = run.travel.as_ref().filter(|travel| travel.finished()) {
            format!(
                "{} | Arrived, manual control [F7: {} | Shift+F7: outward]",
                run.script.name,
                travel.next_action()
            )
        } else if !run.active {
            format!("{} | Manual [F7: restart script]", run.script.name)
        } else if run.waiting {
            format!("{} | Waiting for world [F7: manual]", run.script.name)
        } else if run.finished() {
            format!("{} | Finished [F7: manual]", run.script.name)
        } else if let Some(travel) = &run.travel {
            format!(
                "{} | Waypoint {} [F7: manual]",
                run.script.name,
                travel.index + 1
            )
        } else {
            format!(
                "{} | Step {}/{} [F7: manual]",
                run.script.name,
                run.step + 1,
                run.script.steps.len()
            )
        };
    }
}
pub(crate) fn install(app: &mut App, run: Run) {
    app.insert_resource(run)
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_systems(Startup, setup)
        .add_observer(activate)
        .add_systems(
            PreUpdate,
            toggle.before(crate::input).after(bevy::input::InputSystems),
        )
        .add_systems(
            PreUpdate,
            drive.after(crate::input).after(crate::graybox::autoplay),
        )
        .add_systems(Update, status);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn script() -> Script {
        Script::parse(br#"{"schema":1,"name":"Test route","steps":[{"duration_ms":200,"forward":1000},{"duration_ms":100,"turn":1}]}"#).unwrap()
    }
    #[test]
    fn unready_and_manual_do_not_consume_steps_and_finished_input_stops() {
        let mut run = Run::new(script());
        for _ in 0..10 {
            assert!(run.sample(0.1, false).is_none());
        }
        assert_eq!(run.step, 0);
        assert_eq!(run.elapsed, 0.);
        assert_eq!(run.sample(0.1, true).unwrap().0.forward, 1000);
        run.toggle();
        assert!(run.sample(0.1, true).is_none());
        run.toggle(); // Reattachment starts this bounded script from the beginning.
        assert_eq!(run.step, 0);
        assert_eq!(run.elapsed, 0.);
        run.sample(0.1, true);
        run.sample(0.1, true);
        assert_eq!(run.sample(0.1, true).unwrap().0.turn, 1);
        assert!(run.sample(0.1, true).is_none());
        assert_eq!(run.step, 2);
    }
    #[test]
    fn bounded_delta_preserves_turn_duration_and_repeat_never_runs_unbounded_steps() {
        let mut script = script();
        script.repeat = true;
        let mut run = Run::new(script);
        let mut turn_seconds = 0.;
        for _ in 0..30 {
            let (step, seconds) = run.sample(100., true).unwrap();
            if step.turn != 0 {
                turn_seconds += seconds;
            }
        }
        assert!((turn_seconds - 1.).abs() < 1e-9);
        assert_eq!(run.step, 0);
        assert_eq!(run.sample(f64::NAN, true).unwrap().1, 0.);
    }
    #[test]
    fn script_budgets_and_action_ranges_refuse_before_runtime() {
        let source = serde_json::json!({"schema":1,"name":"Test","steps":[{"duration_ms":100}]});
        for (field, value) in [
            ("duration_ms", serde_json::json!(0)),
            ("duration_ms", serde_json::json!(60001)),
            ("forward", serde_json::json!(1001)),
            ("strafe", serde_json::json!(-1001)),
            ("turn", serde_json::json!(2)),
            ("teleport", serde_json::json!([0, 0, 0])),
        ] {
            let mut invalid = source.clone();
            invalid["steps"][0][field] = value;
            assert!(Script::parse(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
        for steps in [
            vec![],
            vec![serde_json::json!({"duration_ms":100}); 65],
            vec![serde_json::json!({"duration_ms":60_000}); 11],
        ] {
            let mut invalid = source.clone();
            invalid["steps"] = serde_json::json!(steps);
            assert!(Script::parse(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
        assert!(Script::parse(&vec![b' '; MAX_BYTES + 1]).is_err());
    }
    #[test]
    fn toggle_key_requires_owned_focus_and_button_returns_manual_control() {
        use bevy::ui_widgets::Activate;
        let mut app = App::new();
        app.insert_resource(Run::new(script()))
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, toggle)
            .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: false,
                ..default()
            })
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F7);
        app.update();
        assert!(app.world().resource::<Run>().active);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F7);
            keys.clear();
            keys.press(KeyCode::F7);
        }
        app.update();
        assert!(!app.world().resource::<Run>().active);
        let button = app.world_mut().spawn(Toggle).id();
        app.world_mut().trigger(Activate { entity: button });
        assert!(app.world().resource::<Run>().active);
    }
}
