//! Arrange independent development clients with standard Bevy window APIs.
use bevy::{
    prelude::*,
    window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowPosition},
};

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct View {
    index: u32,
    count: u32,
}
impl View {
    fn parse(value: &str) -> Result<Self, String> {
        let (index, count) = value
            .split_once('/')
            .ok_or("Expected test window INDEX/COUNT")?;
        let index = index
            .parse::<u32>()
            .map_err(|_| "Invalid test window index")?;
        let count = count
            .parse::<u32>()
            .map_err(|_| "Invalid test window count")?;
        if !(1..=2).contains(&count) || !(1..=count).contains(&index) {
            return Err("Test windows require 1 <= index <= count <= 2".into());
        }
        Ok(Self { index, count })
    }
    fn title(self, actor: Option<u64>) -> String {
        let owner = actor.map_or_else(|| "Connecting".into(), |id| format!("Player {id}"));
        format!(
            "Theia network playtest - {owner} - view {}/{}",
            self.index, self.count
        )
    }
}

pub(crate) fn from_args(args: &mut Vec<String>) -> Result<Option<View>, String> {
    let Some(index) = args.iter().position(|arg| arg == "--test-window") else {
        return Ok(None);
    };
    args.remove(index);
    if index >= args.len() {
        return Err("Missing test window INDEX/COUNT".into());
    }
    View::parse(&args.remove(index)).map(Some)
}

pub(crate) fn install(app: &mut App, view: View) {
    app.insert_resource(view)
        // Keep the other camera live even when keyboard focus is elsewhere.
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_systems(Update, arrange);
}

fn arrange(
    view: Res<View>,
    monitors: Query<(&Monitor, Has<PrimaryMonitor>)>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    network: Option<Res<crate::network::Network>>,
    mut placed: Local<bool>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let title = view.title(network.as_deref().and_then(|n| n.local_actor_id()));
    if window.title != title {
        window.title = title;
    }
    if *placed {
        return;
    }
    let Some((monitor, _)) = monitors
        .iter()
        .find(|(_, primary)| *primary)
        .or_else(|| monitors.iter().next())
    else {
        return;
    };
    let Some((size, position)) = layout(
        *view,
        monitor.physical_size(),
        monitor.physical_position,
        UVec2::new(
            window.resolution.physical_width(),
            window.resolution.physical_height(),
        ),
    ) else {
        return;
    };
    window.resolution.set_physical_resolution(size.x, size.y);
    window.position = WindowPosition::At(position);
    *placed = true;
    println!(
        "{}",
        serde_json::json!({"test_window":{"index":view.index,"count":view.count,
        "physical_size":size.to_array(),"physical_position":position.to_array(),
        "monitor_size":monitor.physical_size().to_array(),"monitor_position":monitor.physical_position.to_array()}})
    );
}

/// Leave room for decorations/taskbars. Stack on tall displays; cascade only
/// where two minimum-size clients cannot fit. Never reposition after the user moves it.
fn layout(view: View, monitor: UVec2, origin: IVec2, preferred: UVec2) -> Option<(UVec2, IVec2)> {
    const GAP: u32 = 32;
    if monitor.x < 384 || monitor.y < 336 {
        return None;
    }
    let available = monitor - UVec2::new(GAP * 2, GAP * 3);
    let (size, offset) = if view.count == 1 {
        let size = preferred.min(available);
        (size, (monitor - size) / 2)
    } else if monitor.x >= 640 * 2 + GAP * 3 {
        let size = preferred.min(UVec2::new((monitor.x - GAP * 3) / 2, available.y));
        let start = (monitor.x - (size.x * 2 + GAP)) / 2;
        (
            size,
            UVec2::new(
                start + (view.index - 1) * (size.x + GAP),
                (monitor.y - size.y) / 2,
            ),
        )
    } else if monitor.y >= 480 * 2 + GAP * 5 {
        let size = preferred.min(UVec2::new(available.x, (monitor.y - GAP * 5) / 2));
        let start = (monitor.y - (size.y * 2 + GAP * 2)) / 2;
        (
            size,
            UVec2::new(
                (monitor.x - size.x) / 2,
                start + (view.index - 1) * (size.y + GAP * 2),
            ),
        )
    } else {
        let size = preferred.min(available - UVec2::splat(GAP));
        (size, UVec2::splat(GAP * view.index))
    };
    Some((size, origin.checked_add(offset.as_ivec2())?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arguments_are_bounded_and_do_not_consume_other_client_arguments() {
        for invalid in ["", "0/2", "3/2", "1/0", "1/3", "1/1/1", "-1/2"] {
            assert!(View::parse(invalid).is_err());
        }
        let mut args = vec![
            "--test-window".into(),
            "2/2".into(),
            "--renet-connect".into(),
        ];
        assert_eq!(
            from_args(&mut args).unwrap(),
            Some(View { index: 2, count: 2 })
        );
        assert_eq!(args, ["--renet-connect"]);
        assert!(from_args(&mut vec!["--test-window".into()]).is_err());
    }
    #[test]
    fn layouts_stay_on_the_selected_monitor_and_separate_when_space_allows() {
        for monitor in [
            UVec2::new(3440, 1440),
            UVec2::new(1920, 1080),
            UVec2::new(1080, 1920),
            UVec2::new(1280, 720),
        ] {
            for origin in [IVec2::ZERO, IVec2::new(-3440, -500)] {
                let a = layout(
                    View { index: 1, count: 2 },
                    monitor,
                    origin,
                    UVec2::new(1280, 720),
                )
                .unwrap();
                let b = layout(
                    View { index: 2, count: 2 },
                    monitor,
                    origin,
                    UVec2::new(1280, 720),
                )
                .unwrap();
                for (size, position) in [a, b] {
                    assert!(position.cmpge(origin).all());
                    assert!(
                        (position + size.as_ivec2())
                            .cmple(origin + monitor.as_ivec2())
                            .all()
                    );
                }
                if monitor != UVec2::new(1280, 720) {
                    assert!(a.1.x + a.0.x as i32 <= b.1.x || a.1.y + a.0.y as i32 <= b.1.y);
                }
            }
        }
        assert!(
            layout(
                View { index: 1, count: 1 },
                UVec2::ZERO,
                IVec2::ZERO,
                UVec2::new(1280, 720)
            )
            .is_none()
        );
    }
}
