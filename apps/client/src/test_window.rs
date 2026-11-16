//! Arrange independent development clients with standard Bevy window APIs.
use bevy::{
    prelude::*,
    window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowPosition, WindowResized},
};

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct View {
    index: u32,
    count: u32,
    work_area: Option<(IVec2, UVec2)>,
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
        if !(1..=4).contains(&count) || !(1..=count).contains(&index) {
            return Err("Test windows require 1 <= index <= count <= 4".into());
        }
        Ok(Self {
            index,
            count,
            work_area: None,
        })
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
    let mut view = View::parse(&args.remove(index))?;
    if let Some(index) = args.iter().position(|arg| arg == "--test-work-area") {
        args.remove(index);
        if index >= args.len() {
            return Err("Missing desktop work area X,Y,WIDTH,HEIGHT".into());
        }
        let value = args.remove(index);
        let values = value
            .split(',')
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Invalid desktop work area")?;
        if values.len() != 4
            || values[0].unsigned_abs() > 1_000_000
            || values[1].unsigned_abs() > 1_000_000
            || !(1..=32768).contains(&values[2])
            || !(1..=32768).contains(&values[3])
        {
            return Err("Desktop work area is out of bounds".into());
        }
        view.work_area = Some((
            IVec2::new(values[0], values[1]),
            UVec2::new(values[2] as u32, values[3] as u32),
        ));
    }
    Ok(Some(view))
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
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    network: Option<Res<crate::network::Network>>,
    mut placed: Local<bool>,
    mut resizing: Local<Option<(UVec2, IVec2)>>,
    mut resized: MessageReader<WindowResized>,
) {
    let Ok((entity, mut window)) = windows.single_mut() else {
        return;
    };
    let title = view.title(network.as_deref().and_then(|n| n.local_actor_id()));
    if window.title != title {
        window.title = title;
    }
    if *placed {
        return;
    }
    if let Some((size, position)) = *resizing {
        // Native resizing is asynchronous. Positioning against the old height
        // lets a desktop panel push the bottom row upward before resize lands.
        let scale = window.resolution.scale_factor();
        if resized.read().any(|event| {
            event.window == entity
                && (event.width * scale).round() as u32 == size.x
                && (event.height * scale).round() as u32 == size.y
        }) {
            window.position = WindowPosition::At(position);
            *resizing = None;
            *placed = true;
        }
        return;
    }
    resized.clear();
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
    window.resize_constraints.min_width = size.x as f32;
    window.resize_constraints.min_height = size.y as f32;
    if window.resolution.physical_width() == size.x && window.resolution.physical_height() == size.y
    {
        window.position = WindowPosition::At(position);
        *placed = true;
    } else {
        window.resolution.set_physical_resolution(size.x, size.y);
        *resizing = Some((size, position));
    }
    println!(
        "{}",
        serde_json::json!({"test_window":{
            "index":view.index,"count":view.count,"physical_size":size.to_array(),
            "physical_position":position.to_array(),"monitor_size":monitor.physical_size().to_array(),
            "monitor_position":monitor.physical_position.to_array(),"work_area":view.work_area.map(|(p,s)| [p.x,p.y,s.x as i32,s.y as i32])
        }})
    );
}

/// Tile the monitor within the desktop work area, distributing odd pixels.
/// Position once; later user movement is retained.
fn layout(view: View, monitor: UVec2, origin: IVec2, preferred: UVec2) -> Option<(UVec2, IVec2)> {
    let (monitor, origin) = if let Some((area_origin, area_size)) = view.work_area {
        let start = origin.max(area_origin);
        let end = (origin + monitor.as_ivec2()).min(area_origin + area_size.as_ivec2());
        ((end - start).max(IVec2::ZERO).as_uvec2(), start)
    } else {
        (monitor, origin)
    };
    if monitor.min_element() == 0 {
        return None;
    }
    if view.count == 1 {
        let size = preferred.min(monitor);
        return Some((size, origin + ((monitor - size) / 2).as_ivec2()));
    }
    let rows = if view.count <= 2 { 1 } else { 2 };
    let column = (view.index - 1) % 2;
    let row = (view.index - 1) / 2;
    let start = UVec2::new(monitor.x * column / 2, monitor.y * row / rows);
    let end = UVec2::new(monitor.x * (column + 1) / 2, monitor.y * (row + 1) / rows);
    Some((end - start, origin + start.as_ivec2()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arguments_are_bounded_and_do_not_consume_other_client_arguments() {
        for invalid in ["", "0/2", "3/2", "1/0", "1/5", "5/4", "1/1/1", "-1/2"] {
            assert!(View::parse(invalid).is_err());
        }
        let mut args = vec![
            "--test-window".into(),
            "2/2".into(),
            "--renet-connect".into(),
        ];
        assert_eq!(
            from_args(&mut args).unwrap(),
            Some(View {
                index: 2,
                count: 2,
                work_area: None
            })
        );
        assert_eq!(args, ["--renet-connect"]);
        assert!(from_args(&mut vec!["--test-window".into()]).is_err());
    }
    #[test]
    fn layouts_tile_two_and_four_clients_inside_the_monitor() {
        let monitor = UVec2::new(3440, 1440);
        for origin in [IVec2::ZERO, IVec2::new(-3440, -500)] {
            let size = UVec2::new(1720, 1440);
            let a = layout(
                View {
                    index: 1,
                    count: 2,
                    work_area: None,
                },
                monitor,
                origin,
                size,
            )
            .unwrap();
            let b = layout(
                View {
                    index: 2,
                    count: 2,
                    work_area: None,
                },
                monitor,
                origin,
                size,
            )
            .unwrap();
            assert_eq!(a, (size, origin));
            assert_eq!(b, (size, origin + IVec2::new(1720, 0)));
            for index in 1..=4 {
                assert!(View::parse(&format!("{index}/4")).is_ok());
                assert_eq!(
                    layout(
                        View {
                            index,
                            count: 4,
                            work_area: None
                        },
                        monitor,
                        origin,
                        monitor
                    ),
                    Some((
                        UVec2::new(1720, 720),
                        origin
                            + IVec2::new(
                                ((index - 1) % 2 * 1720) as i32,
                                ((index - 1) / 2 * 720) as i32,
                            )
                    ))
                );
            }
            assert_eq!(
                layout(
                    View {
                        index: 1,
                        count: 1,
                        work_area: None
                    },
                    monitor,
                    origin,
                    monitor
                ),
                Some((monitor, origin))
            );
        }
        assert_eq!(
            layout(
                View {
                    index: 1,
                    count: 1,
                    work_area: None
                },
                UVec2::ZERO,
                IVec2::ZERO,
                monitor
            ),
            None
        );
    }
    #[test]
    fn desktop_panels_are_excluded_from_both_layouts() {
        let monitor = UVec2::new(3440, 1440);
        for count in [2, 4] {
            for index in 1..=count {
                let view = View {
                    index,
                    count,
                    work_area: Some((IVec2::ZERO, UVec2::new(3440, 1400))),
                };
                let height = if count == 2 { 1400 } else { 700 };
                assert_eq!(
                    layout(view, monitor, IVec2::ZERO, monitor),
                    Some((
                        UVec2::new(1720, height),
                        IVec2::new(
                            ((index - 1) % 2 * 1720) as i32,
                            ((index - 1) / 2 * height) as i32
                        )
                    ))
                );
            }
        }
        let view = View::parse("2/2").unwrap();
        let mut args = vec![
            "--test-window".into(),
            "2/2".into(),
            "--test-work-area".into(),
            "0,40,3440,1400".into(),
        ];
        let parsed = from_args(&mut args).unwrap().unwrap();
        assert!(args.is_empty());
        assert_eq!(parsed.index, view.index);
        assert_eq!(
            layout(parsed, monitor, IVec2::ZERO, monitor),
            Some((UVec2::new(1720, 1400), IVec2::new(1720, 40)))
        );
    }
    #[test]
    fn odd_monitor_dimensions_have_no_gaps_or_overlap() {
        let monitor = UVec2::new(3441, 1441);
        let mut area = 0;
        for index in 1..=4 {
            let (size, position) = layout(
                View {
                    index,
                    count: 4,
                    work_area: None,
                },
                monitor,
                IVec2::ZERO,
                monitor,
            )
            .unwrap();
            assert_eq!(position.x, if index % 2 == 1 { 0 } else { 1720 });
            assert_eq!(position.y, if index <= 2 { 0 } else { 720 });
            assert!((position.as_uvec2() + size).cmple(monitor).all());
            area += size.x * size.y;
        }
        assert_eq!(area, monitor.x * monitor.y);
    }
}
