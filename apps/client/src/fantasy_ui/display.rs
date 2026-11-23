//! Display settings use physical pixels and the current monitor's native modes.
use bevy::{
    input_focus::{
        FocusCause, InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    picking::hover::Hovered,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{Activate, Button},
    window::{
        Monitor, MonitorSelection, OnMonitor, PrimaryMonitor, PrimaryWindow, RawHandleWrapper,
        VideoMode, VideoModeSelection, WindowMode,
    },
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Mode {
    Fullscreen,
    Borderless,
    Windowed,
}

impl Mode {
    pub(super) const ALL: [Self; 3] = [Self::Fullscreen, Self::Borderless, Self::Windowed];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Fullscreen => "Fullscreen",
            Self::Borderless => "Borderless",
            Self::Windowed => "Windowed",
        }
    }
}

type Size = [u32; 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Choice {
    mode: Mode,
    size: Size,
}

impl Choice {
    pub(crate) fn launch(fullscreen: bool, width: u32, height: u32) -> Self {
        Self {
            mode: if fullscreen {
                Mode::Borderless
            } else {
                Mode::Windowed
            },
            size: [width, height],
        }
    }

    pub(super) fn valid(self) -> bool {
        valid_size(self.size)
    }
}

fn valid_size([width, height]: Size) -> bool {
    (640..=7680).contains(&width) && (480..=4320).contains(&height)
}

pub(super) fn size_label([width, height]: Size) -> String {
    format!("{width} × {height}")
}

#[derive(Clone, Copy)]
struct Preview {
    before: Choice,
    until: Instant,
}

#[derive(Resource)]
pub(crate) struct State {
    defaults: Choice,
    current: Choice,
    drafted: Option<Choice>,
    preview: Option<Preview>,
    saved: Option<Choice>,
    restore_saved: bool,
    tiled: bool,
    desktop: Option<Size>,
    video_modes: Vec<VideoMode>,
    windowed: Size,
}

impl Default for State {
    fn default() -> Self {
        Self::new(Choice::launch(false, 1280, 720), None, false)
    }
}

impl State {
    pub(crate) fn new(defaults: Choice, saved: Option<Choice>, tiled: bool) -> Self {
        Self {
            defaults,
            current: defaults,
            drafted: None,
            preview: None,
            saved,
            restore_saved: saved.is_some() && !tiled,
            tiled,
            desktop: None,
            video_modes: Vec::new(),
            windowed: defaults.size,
        }
    }

    pub(super) fn saved(&self) -> Option<Choice> {
        self.saved
    }
    pub(crate) fn shortcut_available(&self) -> bool {
        !self.tiled && self.preview.is_none()
    }
    pub(crate) fn toggle(&mut self, window: &mut Window) {
        if self.shortcut_available() {
            self.clear();
            let mode = if window.mode == WindowMode::Windowed {
                Mode::Borderless
            } else {
                Mode::Windowed
            };
            self.apply(
                Choice {
                    mode,
                    size: self.windowed,
                },
                window,
            );
        }
    }
    pub(super) fn editable(&self) -> bool {
        self.shortcut_available()
    }
    pub(super) fn mode(&self) -> Mode {
        self.selection().mode
    }
    pub(super) fn label(&self) -> String {
        if self.tiled {
            return "Test layout".into();
        }
        self.mode().label().into()
    }
    pub(super) fn resolution_label(&self) -> String {
        let selected = self.selection();
        if self.tiled {
            return size_label(self.current.size);
        }
        if selected.mode == Mode::Borderless {
            return self.desktop.map_or_else(|| "Desktop".into(), size_label);
        }
        size_label(selected.size)
    }
    pub(super) fn resolution_editable(&self) -> bool {
        self.editable() && self.mode() != Mode::Borderless
    }
    pub(super) fn allows(&self, mode: Mode) -> bool {
        mode != Mode::Fullscreen || !self.video_modes.is_empty()
    }
    pub(super) fn choices(&self) -> Vec<Size> {
        let selected = self.selection();
        let mut sizes = match selected.mode {
            Mode::Windowed => {
                let bounds = self.desktop.unwrap_or([7680, 4320]);
                [
                    [640, 480],
                    [1024, 768],
                    [1280, 720],
                    [1366, 768],
                    [1600, 900],
                    [1920, 1080],
                    [1920, 1200],
                    [2560, 1440],
                    [3440, 1440],
                    [3840, 2160],
                ]
                .into_iter()
                .filter(|size| size[0] <= bounds[0] && size[1] <= bounds[1])
                .chain([self.windowed, selected.size])
                .collect()
            }
            Mode::Fullscreen => self
                .video_modes
                .iter()
                .map(|m| m.physical_size.to_array())
                .collect(),
            Mode::Borderless => Vec::new(),
        };
        sizes.sort_unstable();
        sizes.dedup();
        sizes
    }

    fn selection(&self) -> Choice {
        self.drafted.unwrap_or(self.current)
    }
    pub(super) fn changed(&self) -> bool {
        self.drafted.is_some_and(|choice| choice != self.current)
    }
    pub(super) fn clear(&mut self) {
        self.drafted = None;
    }

    pub(super) fn defaults(&mut self) {
        if self.editable() {
            self.drafted = Some(self.defaults);
        }
    }

    pub(super) fn select_mode(&mut self, mode: Mode) {
        if !self.editable() || !self.allows(mode) || self.mode() == mode {
            return;
        }
        let size = match mode {
            Mode::Fullscreen => self
                .desktop
                .filter(|&size| self.video_mode(size).is_some())
                .or_else(|| {
                    self.video_modes
                        .first()
                        .map(|mode| mode.physical_size.to_array())
                })
                .unwrap_or(self.windowed),
            Mode::Borderless | Mode::Windowed => self.windowed,
        };
        self.drafted = Some(Choice { mode, size });
    }

    pub(super) fn select_size(&mut self, size: Size) {
        if self.resolution_editable() && self.choices().contains(&size) {
            self.drafted = Some(Choice {
                mode: self.mode(),
                size,
            });
        }
    }

    fn video_mode(&self, size: Size) -> Option<VideoMode> {
        self.video_modes
            .iter()
            .copied()
            .filter(|mode| mode.physical_size.to_array() == size)
            .max_by_key(|mode| (mode.refresh_rate_millihertz, mode.bit_depth))
    }

    fn apply(&mut self, choice: Choice, window: &mut Window) {
        self.current = choice;
        window.mode = match choice.mode {
            Mode::Windowed => {
                self.windowed = choice.size;
                window.set_maximized(false);
                window
                    .resolution
                    .set_physical_resolution(choice.size[0], choice.size[1]);
                WindowMode::Windowed
            }
            Mode::Borderless => {
                self.windowed = choice.size;
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            }
            Mode::Fullscreen => {
                if let Some(video) = self.video_mode(choice.size) {
                    WindowMode::Fullscreen(
                        MonitorSelection::Current,
                        VideoModeSelection::Specific(video),
                    )
                } else {
                    self.current = Choice {
                        mode: Mode::Borderless,
                        size: self.windowed,
                    };
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                }
            }
        };
    }

    pub(super) fn preview(&mut self, window: &mut Window, now: Instant) {
        if !self.editable() {
            return;
        }
        let Some(choice) = self.drafted.take().filter(|choice| *choice != self.current) else {
            return;
        };
        if !self.allows(choice.mode)
            || (choice.mode == Mode::Fullscreen && self.video_mode(choice.size).is_none())
        {
            return;
        }
        let before = self.current;
        self.apply(choice, window);
        self.preview = Some(Preview {
            before,
            until: now + Duration::from_secs(15),
        });
    }

    fn finish(&mut self, window: &mut Window, keep: bool) {
        if let Some(preview) = self.preview.take() {
            if keep {
                self.saved = Some(self.current);
            } else {
                self.apply(preview.before, window);
            }
        }
    }

    fn observe_window(&mut self, window: &Window) {
        let size = [window.physical_width(), window.physical_height()];
        match window.mode {
            WindowMode::Windowed if valid_size(size) => {
                self.current = Choice {
                    mode: Mode::Windowed,
                    size,
                };
                self.windowed = size;
            }
            WindowMode::BorderlessFullscreen(_) => {
                self.current = Choice {
                    mode: Mode::Borderless,
                    size: self.windowed,
                };
            }
            _ => {}
        }
    }
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<State>()
        .add_systems(Startup, confirmation)
        .add_observer(confirm)
        .add_systems(Update, (sync, show_confirmation).chain());
}

type DisplayWindows<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Window,
        Option<&'static OnMonitor>,
        Option<&'static RawHandleWrapper>,
    ),
    With<PrimaryWindow>,
>;

fn sync(
    mut state: ResMut<State>,
    mut windows: DisplayWindows,
    monitors: Query<(Entity, &Monitor, Has<PrimaryMonitor>)>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
) {
    let Ok((mut window, linked, handle)) = windows.single_mut() else {
        return;
    };
    if state.tiled {
        state.current.size = [window.physical_width(), window.physical_height()];
        return;
    }
    let monitor = linked
        .and_then(|link| monitors.get(link.0).ok())
        .or_else(|| monitors.iter().find(|(_, _, primary)| *primary))
        .or_else(|| monitors.iter().next())
        .map(|(_, monitor, _)| monitor);
    state.desktop = monitor.map(|monitor| monitor.physical_size().to_array());
    let exclusive = handle.is_some_and(|handle| {
        matches!(
            handle.get_window_handle(),
            raw_window_handle::RawWindowHandle::Xlib(_)
                | raw_window_handle::RawWindowHandle::Xcb(_)
                | raw_window_handle::RawWindowHandle::Win32(_)
        )
    });
    let modes = monitor
        .filter(|_| exclusive)
        .map_or(&[][..], |monitor| &monitor.video_modes);
    state.video_modes.clear();
    state.video_modes.extend(
        modes
            .iter()
            .copied()
            .filter(|mode| valid_size(mode.physical_size.to_array())),
    );

    if state.restore_saved && handle.is_some() && monitor.is_some() {
        state.restore_saved = false;
        if let Some(choice) = state.saved {
            state.apply(choice, &mut window);
        }
        return;
    }
    if let Some(preview) = state.preview {
        if Instant::now() >= preview.until
            || keys.is_some_and(|keys| keys.just_pressed(KeyCode::Escape))
        {
            state.finish(&mut window, false);
        }
    } else if state.drafted.is_none() {
        state.observe_window(&window);
    }
}

#[derive(Component)]
struct ConfirmPanel;
#[derive(Component)]
struct ConfirmPrompt;
#[derive(Component)]
struct ConfirmButton(bool);

fn confirmation(mut commands: Commands) {
    commands
        .spawn((
            ConfirmPanel,
            TabGroup::modal(),
            GlobalZIndex(2000),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.60)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: px(400),
                        max_width: percent(90),
                        padding: UiRect::all(px(24)),
                        ..super::kit::column()
                    },
                    crate::ui_theme::Panel,
                    BackgroundColor(crate::ui_theme::PANEL),
                ))
                .with_children(|panel| {
                    super::kit::section(panel, "Keep display changes?");
                    panel.spawn((ConfirmPrompt, super::kit::text("", 16.)));
                    panel
                        .spawn(Node {
                            column_gap: px(10),
                            ..default()
                        })
                        .with_children(|buttons| {
                            for (keep, caption) in [(true, "Keep Changes"), (false, "Revert")] {
                                buttons
                                    .spawn((
                                        ConfirmButton(keep),
                                        Button,
                                        TabIndex(0),
                                        Hovered::default(),
                                        AccessibleLabel::new(caption),
                                        if keep {
                                            crate::ui_theme::Style::Primary
                                        } else {
                                            crate::ui_theme::Style::Secondary
                                        },
                                        Node {
                                            padding: UiRect::axes(px(16), px(9)),
                                            ..default()
                                        },
                                    ))
                                    .with_child(super::kit::text(caption, 16.));
                            }
                        });
                });
        });
}

fn confirm(
    event: On<Activate>,
    buttons: Query<&ConfirmButton, Without<InteractionDisabled>>,
    mut state: ResMut<State>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(button) = buttons.get(event.entity)
        && let Ok(mut window) = windows.single_mut()
        && window.focused
    {
        state.finish(&mut window, button.0);
    }
}

fn show_confirmation(
    mut commands: Commands,
    state: Res<State>,
    mut panels: Query<(Entity, &mut Node, &mut Visibility), With<ConfirmPanel>>,
    mut prompts: Query<&mut Text, With<ConfirmPrompt>>,
    buttons: Query<(Entity, &ConfirmButton)>,
    title_camera: Query<(Entity, &Camera), With<crate::frontend::FrontCamera>>,
    mut focus: Option<ResMut<InputFocus>>,
) {
    let display = if state.preview.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for (panel, mut node, mut visibility) in &mut panels {
        if node.display != display {
            node.display = display;
            if display == Display::Flex {
                *visibility = Visibility::Visible;
                if let Ok((camera, view)) = title_camera.single()
                    && view.is_active
                {
                    commands.entity(panel).insert(UiTargetCamera(camera));
                } else {
                    commands.entity(panel).remove::<UiTargetCamera>();
                }
                if let Some(focus) = &mut focus
                    && let Some((revert, _)) = buttons.iter().find(|(_, button)| !button.0)
                {
                    focus.set(revert, FocusCause::Navigated);
                }
            } else {
                *visibility = Visibility::Hidden;
                if let Some(focus) = &mut focus
                    && focus.get().is_some_and(|entity| buttons.contains(entity))
                {
                    focus.clear();
                }
            }
        }
    }
    if let Some(preview) = state.preview {
        let seconds = preview
            .until
            .saturating_duration_since(Instant::now())
            .as_secs_f32()
            .ceil() as u32;
        let label = format!("Reverting in {seconds} seconds.");
        for mut text in &mut prompts {
            if text.0 != label {
                text.0 = label.clone();
            }
        }
    }
}

pub(super) fn snapshot(world: &mut World, scale: f32) -> serde_json::Value {
    let confirmation: Vec<_> = world
        .query::<(&ConfirmButton, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world)
        .filter(|(_, node, _)| node.size().min_element() > 0.)
        .map(|(button, node, transform)| serde_json::json!({
            "name": if button.0 { "keep-display" } else { "revert-display" },
            "point": (transform.affine().translation * super::window_factor(node, scale)).to_array(),
        }))
        .collect();
    serde_json::json!(confirmation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_confirmed_display_changes_save_and_timeout_restores_previous_size() {
        let initial = Choice::launch(false, 1600, 900);
        let mut state = State::new(initial, None, false);
        let mut window = Window {
            resolution: bevy::window::WindowResolution::new(1600, 900),
            ..default()
        };
        let now = Instant::now();
        state.select_size([1280, 720]);
        assert!(state.changed());
        assert_eq!(window.physical_width(), 1600);
        state.preview(&mut window, now);
        assert_eq!(window.physical_width(), 1280);
        assert_eq!(state.preview.unwrap().until, now + Duration::from_secs(15));
        assert_eq!(state.saved(), None);
        assert!(!state.shortcut_available());
        state.finish(&mut window, false);
        assert_eq!(window.physical_width(), 1600);
        assert_eq!(state.saved(), None);

        state.select_mode(Mode::Borderless);
        state.preview(&mut window, now);
        state.finish(&mut window, true);
        assert_eq!(state.saved(), Some(Choice::launch(true, 1600, 900)));
        assert_eq!(
            window.mode,
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        );
        state.toggle(&mut window);
        assert_eq!(window.mode, WindowMode::Windowed);
        assert_eq!(window.physical_width(), 1600);
        assert_eq!(state.saved(), Some(Choice::launch(true, 1600, 900)));
    }

    #[test]
    fn exclusive_resolutions_follow_the_monitor_and_tiles_preserve_their_layout() {
        let initial = Choice::launch(false, 1600, 900);
        let mut state = State::new(initial, None, false);
        state.select_mode(Mode::Fullscreen);
        assert!(!state.changed());
        let mode = VideoMode {
            physical_size: UVec2::new(1920, 1080),
            bit_depth: 24,
            refresh_rate_millihertz: 60_000,
        };
        let best = VideoMode {
            refresh_rate_millihertz: 120_000,
            ..mode
        };
        state.video_modes = vec![mode, best];
        state.desktop = Some(mode.physical_size.to_array());
        state.select_mode(Mode::Fullscreen);
        assert_eq!(state.choices(), [[1920, 1080]]);
        state.select_size([1280, 720]);
        assert_eq!(state.selection().size, [1920, 1080]);
        let mut window = Window::default();
        state.preview(&mut window, Instant::now());
        assert_eq!(
            window.mode,
            WindowMode::Fullscreen(
                MonitorSelection::Current,
                VideoModeSelection::Specific(best)
            )
        );

        let mut tile = State::new(initial, Some(Choice::launch(true, 1920, 1080)), true);
        tile.select_mode(Mode::Borderless);
        tile.select_size([1280, 720]);
        tile.toggle(&mut window);
        assert!(!tile.changed());
        assert!(!tile.restore_saved);
        assert!(!tile.editable());
    }

    #[test]
    fn confirmation_renders_above_the_title_screen_and_uses_default_camera_in_game() {
        let mut app = App::new();
        let mut state = State::default();
        let mut window = Window::default();
        state.select_mode(Mode::Borderless);
        state.preview(&mut window, Instant::now());
        app.insert_resource(state)
            .add_systems(Update, show_confirmation);
        let camera = app
            .world_mut()
            .spawn((crate::frontend::FrontCamera, Camera::default()))
            .id();
        let panel = app
            .world_mut()
            .spawn((
                ConfirmPanel,
                Visibility::Hidden,
                Node {
                    display: Display::None,
                    ..default()
                },
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<UiTargetCamera>(panel).unwrap().0, camera);
        app.world_mut()
            .resource_mut::<State>()
            .finish(&mut window, false);
        app.update();
        app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = false;
        let mut state = app.world_mut().resource_mut::<State>();
        state.select_mode(Mode::Borderless);
        state.preview(&mut window, Instant::now());
        app.update();
        assert!(app.world().get::<UiTargetCamera>(panel).is_none());
    }
}
