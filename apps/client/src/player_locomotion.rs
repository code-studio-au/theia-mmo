//! Optional authored direction clips. Presentation never turns or moves an actor.
use crate::movement::Gait;
use bevy::prelude::*;
use theia_core::coordinates::GlobalPosition;

pub(super) const FIRST: usize = 10; // Preserve the four deferred dodge slots.
pub(super) const COUNT: usize = 14;
pub(super) const POSES: usize = FIRST + COUNT;
pub(super) const NAMES: [&str; COUNT] = [
    "WalkBack",
    "WalkLeft",
    "WalkRight",
    "WalkForwardLeft",
    "WalkForwardRight",
    "WalkBackLeft",
    "WalkBackRight",
    "RunBack",
    "RunLeft",
    "RunRight",
    "RunForwardLeft",
    "RunForwardRight",
    "RunBackLeft",
    "RunBackRight",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Direction {
    Forward,
    ForwardRight,
    Right,
    BackRight,
    Back,
    BackLeft,
    Left,
    ForwardLeft,
}
const DIRECTIONS: [Direction; 7] = [
    Direction::Back,
    Direction::Left,
    Direction::Right,
    Direction::ForwardLeft,
    Direction::ForwardRight,
    Direction::BackLeft,
    Direction::BackRight,
];

pub(super) fn append_slots(
    names: &mut Vec<&'static str>,
    mut exists: impl FnMut(&str) -> bool,
) -> Result<[Option<usize>; COUNT], String> {
    let count = NAMES.iter().filter(|name| exists(name)).count();
    if !matches!(count, 0 | COUNT) {
        return Err("Directional locomotion requires all fourteen inspected clips".into());
    }
    let mut slots = [None; COUNT];
    if count != 0 {
        for (i, name) in NAMES.into_iter().enumerate() {
            slots[i] = Some(names.len());
            names.push(name);
        }
    }
    Ok(slots)
}
pub(super) fn base_name(clip: usize) -> Option<&'static str> {
    match clip {
        0..4 => Some(["Idle", "Walk", "Run", "Jump"][clip]),
        FIRST..POSES => Some(NAMES[clip - FIRST]),
        _ => None,
    }
}
pub(super) fn moving(clip: usize) -> bool {
    matches!(clip, 1 | 2 | FIRST..POSES)
}
pub(super) fn gait_index(clip: usize) -> Option<usize> {
    match clip {
        0..4 => Some(clip),
        FIRST..POSES => Some(if clip - FIRST < 7 { 1 } else { 2 }),
        _ => None,
    }
}
fn direction(clip: usize) -> Option<Direction> {
    match clip {
        1 | 2 => Some(Direction::Forward),
        FIRST..POSES => Some(DIRECTIONS[(clip - FIRST) % 7]),
        _ => None,
    }
}
pub(super) fn gait(speed: f32, enabled: bool) -> Gait {
    if !enabled {
        return Gait::from_speed(speed);
    }
    let speeds = theia_network::classic::ground_speeds();
    let midpoint = (speeds.walk_metres_per_second + speeds.backward_run_metres_per_second) / 2.;
    if !speed.is_finite() || speed <= 0.1 {
        Gait::Stand
    } else if speed > midpoint {
        Gait::Run
    } else {
        Gait::Walk
    }
}
pub(super) fn choose(
    gait: Gait,
    segment: Option<[GlobalPosition; 2]>,
    rotation: Quat,
    previous: Option<usize>,
) -> usize {
    let base = gait.index();
    if gait == Gait::Stand
        || !rotation.is_finite()
        || (rotation.length_squared() - 1.).abs() > 0.001
    {
        return base;
    }
    let Some([from, to]) = segment.map(|v| v.map(|p| p.metres())) else {
        return base;
    };
    let (x, z) = (to[0] - from[0], to[2] - from[2]);
    let length = x.hypot(z);
    if length <= 1e-9 {
        return base;
    }
    // Difference in checked globals first; f32 sees only a bounded unit vector.
    let local = rotation.inverse() * Vec3::new((x / length) as f32, 0., (z / length) as f32);
    let angle = local.x.atan2(-local.z);
    let old = previous.and_then(direction);
    let keep = old.filter(|d| {
        let centre = *d as u8 as f32 * std::f32::consts::FRAC_PI_4;
        let error = (angle - centre + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        error.abs() <= 28.5_f32.to_radians() // 22.5° sector plus 6° hysteresis.
    });
    let selected = keep.unwrap_or_else(|| {
        [
            Direction::Forward,
            Direction::ForwardRight,
            Direction::Right,
            Direction::BackRight,
            Direction::Back,
            Direction::BackLeft,
            Direction::Left,
            Direction::ForwardLeft,
        ][((angle / std::f32::consts::FRAC_PI_4).round() as i32).rem_euclid(8) as usize]
    });
    DIRECTIONS
        .iter()
        .position(|d| *d == selected)
        .map_or(base, |i| FIRST + i + if gait == Gait::Run { 7 } else { 0 })
}
pub(super) fn phase(time: f32, old_duration: f32, new_duration: f32) -> Option<f32> {
    (time.is_finite()
        && old_duration.is_finite()
        && new_duration.is_finite()
        && old_duration > 0.
        && new_duration > 0.)
        .then(|| time.rem_euclid(old_duration) / old_duration * new_duration)
}

#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub available: bool,
    pub local_frames: [u64; 16],
    pub remote_frames: [u64; 16],
    pub local_changes: u64,
    pub remote_changes: u64,
    pub phase_preservations: u64,
}
impl Metrics {
    pub(super) fn record(&mut self, local: bool, clip: usize, changed: bool) {
        let Some(direction) = direction(clip) else {
            return;
        };
        let gait = gait_index(clip).unwrap() - 1;
        let counts = if local {
            &mut self.local_frames
        } else {
            &mut self.remote_frames
        };
        counts[gait * 8 + direction as usize] += 1;
        self.local_changes += u64::from(local && changed);
        self.remote_changes += u64::from(!local && changed);
    }
}

pub(crate) fn qualification(world: &mut World) -> serde_json::Value {
    let roots: Vec<_> = world
        .query::<&super::Visual>()
        .iter(world)
        .filter(|v| v.kind == 0 && v.clip.is_some())
        .take(theia_network::wire::MAX_PLAYERS)
        .map(|v| {
            (
                v.owner,
                v.clip.unwrap(),
                v.grip_active,
                v.attack_sequence,
                v.players.clone(),
            )
        })
        .collect();
    let state = world
        .get_resource::<crate::network::Network>()
        .map(|n| n.state());
    let mut actors = Vec::new();
    for (owner, clip, grip, attack, players) in roots {
        let local = world.get::<crate::movement::LocalPawn>(owner).is_some();
        let actor = if local {
            state.as_ref().and_then(|s| s.id)
        } else {
            world
                .get::<crate::network::RemotePawn>(owner)
                .map(|p| p.actor_id())
        };
        let catalog = world.resource::<super::Catalog>();
        let index = base_name(clip).map(|_| {
            if attack.is_some() {
                catalog
                    .grip
                    .as_ref()
                    .unwrap()
                    .attack
                    .as_ref()
                    .unwrap()
                    .base(clip)
            } else if grip {
                catalog.grip.as_ref().unwrap().base(clip)
            } else {
                catalog.index(0, clip)
            }
        });
        let time = index.and_then(|index| {
            players.iter().find_map(|e| {
                world
                    .get::<AnimationPlayer>(*e)
                    .and_then(|p| p.animation(index))
                    .map(|a| a.seek_time())
            })
        });
        let name = base_name(clip).unwrap_or(match clip {
            4 => "Death",
            5 => "SwordIdle",
            _ => "DeferredDodge",
        });
        actors.push(
            serde_json::json!({"actor":actor,"local":local,"clip":name,"grip":grip,"attack":attack,
            "base_time":time,"speed":world.get::<crate::movement::Motion>(owner).map(|m|m.0),
            "forward":world.get::<Transform>(owner).map(|t|(t.rotation*Vec3::NEG_Z).to_array())}),
        );
    }
    serde_json::json!({"metrics":world.get_resource::<Metrics>(),"actors":actors})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn segment(x: f32, z: f32, yaw: f32) -> [GlobalPosition; 2] {
        let delta = Quat::from_rotation_y(yaw) * Vec3::new(x, 0., z);
        let start = [1e12 - 100., 0., 1e12 - 100.];
        [
            GlobalPosition::new(start).unwrap(),
            GlobalPosition::new([
                start[0] + f64::from(delta.x),
                0.,
                start[2] + f64::from(delta.z),
            ])
            .unwrap(),
        ]
    }
    #[test]
    fn all_eight_directions_use_displayed_body_basis_and_checked_global_differences() {
        for yaw in [0., std::f32::consts::FRAC_PI_2, std::f32::consts::PI, 1.123] {
            for (x, z, suffix) in [
                (0., -1., ""),
                (1., -1., "ForwardRight"),
                (1., 0., "Right"),
                (1., 1., "BackRight"),
                (0., 1., "Back"),
                (-1., 1., "BackLeft"),
                (-1., 0., "Left"),
                (-1., -1., "ForwardLeft"),
            ] {
                for (gait, prefix) in [(Gait::Walk, "Walk"), (Gait::Run, "Run")] {
                    let clip = choose(
                        gait,
                        Some(segment(x, z, yaw)),
                        Quat::from_rotation_y(yaw),
                        None,
                    );
                    assert_eq!(base_name(clip), Some(format!("{prefix}{suffix}").as_str()));
                    assert!(moving(clip));
                    assert_eq!(gait_index(clip), Some(gait.index()));
                    assert!(!(6..10).contains(&clip));
                }
            }
        }
    }
    #[test]
    fn sector_hysteresis_prevents_boundary_chatter_without_tracking_targets() {
        let at = |degrees: f32, previous| {
            choose(
                Gait::Walk,
                Some(segment(
                    degrees.to_radians().sin(),
                    -degrees.to_radians().cos(),
                    0.,
                )),
                Quat::IDENTITY,
                previous,
            )
        };
        assert_eq!(base_name(at(24., None)), Some("WalkForwardRight"));
        assert_eq!(at(24., Some(1)), 1);
        let right = at(29., Some(1));
        assert_eq!(base_name(right), Some("WalkForwardRight"));
        assert_eq!(at(20., Some(right)), right);
        assert_eq!(at(16., Some(right)), 1);
        assert_eq!(base_name(at(-179., Some(FIRST))), Some("WalkBack"));
    }
    #[test]
    fn complete_set_is_atomic_optional_and_slots_never_alias_deferred_dodge() {
        let mut names = vec!["Idle", "Walk", "Run", "Jump", "Death", "SwordIdle"];
        assert_eq!(append_slots(&mut names, |_| false).unwrap(), [None; COUNT]);
        assert_eq!(names.len(), 6);
        assert!(append_slots(&mut names, |name| name == "RunBack").is_err());
        assert_eq!(names.len(), 6);
        let slots = append_slots(&mut names, |_| true).unwrap();
        for (i, slot) in slots.into_iter().enumerate() {
            assert_eq!(names[slot.unwrap()], NAMES[i]);
        }
        assert_eq!(names.len(), 20);
    }
    #[test]
    fn backward_run_is_run_only_for_the_explicit_directional_profile() {
        assert_eq!(gait(4.5, true), Gait::Run);
        assert_eq!(gait(4.5, false), Gait::Walk);
        assert_eq!(gait(2.5, true), Gait::Walk);
        assert_eq!(gait(7., true), Gait::Run);
        assert_eq!(gait(f32::NAN, true), Gait::Stand);
        assert_eq!(gait(f32::INFINITY, true), Gait::Stand);
        assert_eq!(gait(0., true), Gait::Stand);
    }
    #[test]
    fn normalized_phase_preserves_loops_and_rejects_nonfinite_or_empty_clips() {
        assert!((phase(0.35, 0.7, 1.).unwrap() - 0.5).abs() < 1e-6);
        assert!((phase(1.75, 0.7, 1.).unwrap() - 0.5).abs() < 1e-6);
        for (time, old, new) in [
            (f32::NAN, 1., 1.),
            (1., 0., 1.),
            (1., 1., 0.),
            (1., 1., f32::INFINITY),
        ] {
            assert!(phase(time, old, new).is_none());
        }
    }
    #[test]
    fn missing_or_discontinuous_motion_never_invents_a_direction() {
        assert_eq!(choose(Gait::Run, None, Quat::IDENTITY, None), 2);
        let point = GlobalPosition::new([0., 0., 0.]).unwrap();
        assert_eq!(
            choose(Gait::Walk, Some([point; 2]), Quat::IDENTITY, None),
            1
        );
        assert_eq!(
            choose(
                Gait::Walk,
                Some(segment(1., 0., 0.)),
                Quat::from_xyzw(f32::NAN, 0., 0., 0.),
                None
            ),
            1
        );
    }
}
