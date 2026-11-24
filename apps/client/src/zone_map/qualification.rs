//! Explicit app-owned native checks. Never sends operating-system input.
use super::*;
use std::path::PathBuf;

#[derive(Resource)]
struct Run {
    output: PathBuf,
    next: usize,
    started: Option<f64>,
    restore_focus: Option<bool>,
    restore_cursor: Option<Option<Vec2>>,
}

pub(super) fn install(app: &mut App) {
    if let Some(output) = std::env::var_os("THEIA_ZONE_MAP_QUALIFY") {
        app.insert_resource(Run {
            output: output.into(),
            next: 0,
            started: None,
            restore_focus: None,
            restore_cursor: None,
        })
        .add_systems(
            PreUpdate,
            drive
                .after(bevy::input::InputSystems)
                .before(super::keyboard),
        )
        .add_systems(PostUpdate, restore_focus.after(super::present))
        .add_systems(Last, capture);
    }
}
fn drive(world: &mut World) {
    if world
        .get_resource::<crate::loading::State>()
        .is_some_and(|s| s.blocks_input())
    {
        return;
    }
    let now = world.resource::<Time>().elapsed_secs_f64();
    let mut run = world.remove_resource::<Run>().unwrap();
    let started = *run.started.get_or_insert(now);
    let t = now - started;
    let thresholds = [1., 5., 8., 10., 13., 15., 18.];
    if run.next < thresholds.len() && t >= thresholds[run.next] {
        // A focused input snapshot belongs only to this app-owned fixture.
        // Do not focus a desktop window or interfere with the user's typing.
        let mut window = world.query::<&mut Window>().single_mut(world).unwrap();
        eprintln!(
            "zone_map_fixture action={} native_focused={}",
            run.next, window.focused
        );
        run.restore_focus = Some(window.focused);
        window.focused = true;
        match run.next {
            0 | 2 | 3 => {
                world
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::KeyM);
            }
            4 => {
                world
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::Escape);
            }
            n => {
                let entity = world
                    .query::<(Entity, &Control)>()
                    .iter(world)
                    .find(|(_, c)| {
                        matches!(
                            (n, **c),
                            (1, Control::Landmark(6))
                                | (5, Control::Toggle)
                                | (6, Control::Landmark(0))
                        )
                    })
                    .unwrap()
                    .0;
                world.trigger(Activate { entity });
            }
        }
        run.next += 1;
    } else {
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyM);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Escape);
    }
    if (23. ..40.).contains(&t) && !world.resource::<Data>().regions.is_empty() {
        let index = ((t - 23.) / 2.) as usize;
        let target = regions::qualification_target(world, index);
        let mut window = world.query::<&mut Window>().single_mut(world).unwrap();
        run.restore_focus = Some(window.focused);
        run.restore_cursor = Some(window.cursor_position());
        window.focused = true;
        window.set_cursor_position(target.map(|(_, p)| p));
    }
    world.insert_resource(run);
}
fn restore_focus(mut run: ResMut<Run>, mut windows: Query<&mut Window>) {
    // Restore in PostUpdate before bevy_winit applies Window changes in Last;
    // the fixture must never issue a native focus_window request.
    if let Some(cursor) = run.restore_cursor.take() {
        windows.single_mut().unwrap().set_cursor_position(cursor);
    }
    if let Some(focused) = run.restore_focus.take() {
        windows.single_mut().unwrap().focused = focused;
    }
}
fn geometry<T: Component>(world: &mut World) -> Option<[f32; 4]> {
    let scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    let (node, transform) = world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<T>>()
        .iter(world)
        .next()?;
    let factor = crate::fantasy_ui::window_factor(node, scale);
    let size = node.size() * factor;
    let min = transform.affine().translation * factor - size * 0.5;
    Some([min.x, min.y, size.x, size.y])
}
fn labels(world: &mut World) -> Vec<[f32; 4]> {
    let scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<MapLabel>>()
        .iter(world)
        .map(|(node, transform)| {
            let factor = crate::fantasy_ui::window_factor(node, scale);
            let size = node.size() * factor;
            let min = transform.affine().translation * factor - size * 0.5;
            [min.x, min.y, size.x, size.y]
        })
        .collect()
}
fn capture(world: &mut World) {
    let run = world.resource::<Run>();
    let Some(started) = run.started else {
        return;
    };
    let t = world.resource::<Time>().elapsed_secs_f64() - started;
    let checks = [
        (3., "opened", true, 0),
        (7., "caves", true, 6),
        (9., "closed-m", false, 6),
        (12., "reopened", true, 6),
        (14., "closed-escape", false, 6),
        (17., "button-opened", true, 6),
        (21., "village", true, 0),
    ];
    let output = run.output.clone();
    std::fs::create_dir_all(&output).unwrap();
    if (24. ..40.).contains(&t) && !world.resource::<Data>().regions.is_empty() {
        let index = ((t - 23.) / 2.) as usize;
        let path = output.join(format!("hover-{index}.json"));
        if !path.exists() && t >= 24. + index as f64 * 2. {
            let expected = regions::qualification_target(world, index).map(|(id, _)| id);
            let actual = regions::qualification_hover(world);
            assert_eq!(actual, expected, "Boundary hover {index}");
            std::fs::write(
                path,
                serde_json::to_vec_pretty(
                    &serde_json::json!({"index":index,"expected":expected,"highlighted":actual}),
                )
                .unwrap(),
            )
            .unwrap();
            if index == 0 {
                world
                    .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                    .observe(bevy::render::view::screenshot::save_to_disk(
                        output.join("hover-millhaven.png"),
                    ));
            }
        }
    }
    for (at, name, open, selected) in checks {
        let path = output.join(format!("{name}.json"));
        if t < at || path.exists() {
            continue;
        }
        let ui = world.resource::<Ui>();
        assert_eq!(ui.open, open, "Map action {name}");
        assert_eq!(ui.selected, selected, "Landmark action {name}");
        let data = world.resource::<Data>().clone();
        let state = world.resource::<crate::network::Network>().state();
        let position = crate::minimap::owner_position(world, &state)
            .unwrap()
            .metres();
        let projected = data.project([position[0], position[2]]).unwrap();
        let heading = crate::minimap::player_heading(world, &state);
        let forward = (Quat::from_rotation_y(heading) * Vec3::NEG_Z).xz();
        let map_forward = world
            .query_filtered::<&bevy::ui::UiGlobalTransform, With<Player>>()
            .single(world)
            .map(|t| (t.affine().matrix2 * Vec2::NEG_Y).normalize().to_array())
            .ok();
        let minimap_forward = world.resource::<crate::minimap::Metrics>().arrow_forward;
        let report = serde_json::json!({"name":name,"open":open,"selected":selected,"position":position,
            "world_forward":forward.to_array(),"map_forward":map_forward,"minimap_forward":minimap_forward,
            "player_uv":projected.to_array(),"panel":geometry::<Root>(world),"surface":geometry::<Surface>(world),"labels":labels(world),
            "player":geometry::<Player>(world),"landmarks":data.landmarks.iter().map(|l|serde_json::json!({"name":l.name,"xz":l.xz,"uv":data.project(l.xz).unwrap().to_array()})).collect::<Vec<_>>()});
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        if open {
            world
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(
                    output.join(format!("{name}.png")),
                ));
        }
    }
}
