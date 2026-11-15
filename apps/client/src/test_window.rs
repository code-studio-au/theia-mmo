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

/// Preserve requested test resolution, even when two clients must overlap.
/// Position only once; window-manager/user movement remains authoritative later.
fn layout(view: View, monitor: UVec2, origin: IVec2, preferred: UVec2) -> Option<(UVec2, IVec2)> {
    if monitor.min_element() == 0 {
        return None;
    }
    let size = UVec2::new(preferred.x.max(1920), preferred.y);
    let spare = UVec2::new(
        monitor.x.saturating_sub(size.x),
        monitor.y.saturating_sub(size.y),
    );
    let offset = if view.count == 1 {
        spare / 2
    } else {
        UVec2::new((view.index - 1) * spare.x, spare.y / 2)
    };
    Some((size, origin + offset.as_ivec2()))
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
    fn layouts_preserve_resolution_and_overlap_when_needed() {
        let monitor = UVec2::new(3440, 1440);
        for origin in [IVec2::ZERO, IVec2::new(-3440, -500)] {
            let size = UVec2::new(1920, 1440);
            let a = layout(View { index: 1, count: 2 }, monitor, origin, size).unwrap();
            let b = layout(View { index: 2, count: 2 }, monitor, origin, size).unwrap();
            assert_eq!(a, (size, origin));
            assert_eq!(b, (size, origin + IVec2::new(1520, 0)));
            assert_eq!(
                layout(View { index: 1, count: 1 }, monitor, origin, monitor),
                Some((monitor, origin))
            );
        }
        assert_eq!(
            layout(
                View { index: 1, count: 1 },
                UVec2::ZERO,
                IVec2::ZERO,
                monitor
            ),
            None
        );
    }
}
