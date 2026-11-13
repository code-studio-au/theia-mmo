//! Opt-in test travel prepared against exact authored navigation and collision.
//! The follower emits ordinary bounded input; it never changes an actor pose.
use super::Step;
use serde::Deserialize;
use std::path::Path;
use theia_core::{coordinates::GlobalPosition, facing::Facing};
use theia_network::{navigation, procedural};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    name: String,
    navigation: String,
    waypoints: Vec<[f64; 3]>,
}
const ARRIVAL: f64 = 0.25;
const STALL_SECONDS: f64 = 8.;
pub(super) struct Travel {
    points: Vec<GlobalPosition>,
    pub index: usize,
    pub failure: Option<&'static str>,
    started: bool,
    elapsed: f64,
    turn_radians: f64,
    stalled: f64,
    unready: f64,
    best_distance: f64,
    best_angle: f64,
}
impl Travel {
    pub fn load(
        path: &Path,
        bytes: &[u8],
        source: &procedural::Source,
    ) -> Result<(String, Self), String> {
        let doc: Document = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if doc.schema != 1
            || doc.name.trim().is_empty()
            || doc.name.len() > 80
            || doc.name.chars().any(char::is_control)
            || doc.navigation.is_empty()
            || doc.navigation.len() > 1024
            || doc.navigation.chars().any(char::is_control)
            || !(2..=navigation::MAX_ROUTE_POINTS).contains(&doc.waypoints.len())
        {
            return Err("Invalid test route schema, name, navigation or waypoints".into());
        }
        let procedural::Source::Authored(world) = source else {
            return Err("Test routes require an authored world".into());
        };
        let prepare = || -> theia_network::Result<Vec<GlobalPosition>> {
            let nav = navigation::Store::open(
                &path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(&doc.navigation),
                world.identity(),
            )?;
            nav.verify()?;
            let tiles = nav
                .manifest()
                .tiles
                .iter()
                .map(|t| nav.tile(t.id))
                .collect::<theia_network::Result<Vec<_>>>()?;
            let query = navigation::query::Query::new(&tiles)?;
            let mut ground = navigation::authored_checks::Grounding::new(world);
            let waypoints = doc
                .waypoints
                .iter()
                .map(|&p| GlobalPosition::new(p).ok_or("Invalid test waypoint".into()))
                .collect::<theia_network::Result<Vec<_>>>()?;
            let mut points = Vec::new();
            let mut samples = 0;
            for pair in waypoints.windows(2) {
                let path = query.route(pair[0], pair[1])?;
                for segment in path.windows(2) {
                    project_segment(
                        &query,
                        &mut ground,
                        segment[0],
                        segment[1],
                        &mut points,
                        &mut samples,
                    )?;
                }
            }
            navigation::authored_checks::clearance(
                &query,
                &mut ground,
                &points.iter().map(|p| p.metres()).collect::<Vec<_>>(),
                &mut 0,
            )?;
            Ok(points)
        };
        let points = prepare().map_err(|e| format!("Test route preparation: {e}"))?;
        Ok((doc.name, Self::new(points)?))
    }
    fn new(points: Vec<GlobalPosition>) -> Result<Self, String> {
        if !(2..=navigation::MAX_ROUTE_POINTS).contains(&points.len()) {
            return Err("Test travel needs a nonempty checked route".into());
        }
        Ok(Self {
            points,
            index: 0,
            failure: None,
            started: false,
            elapsed: 0.,
            turn_radians: 0.,
            stalled: 0.,
            unready: 0.,
            best_distance: f64::INFINITY,
            best_angle: f64::INFINITY,
        })
    }
    pub fn reset(&mut self) {
        self.index = 0;
        self.failure = None;
        self.started = false;
        self.elapsed = 0.;
        self.turn_radians = 0.;
        self.unready = 0.;
        self.reset_progress();
    }
    fn reset_progress(&mut self) {
        self.stalled = 0.;
        self.best_distance = f64::INFINITY;
        self.best_angle = f64::INFINITY;
    }
    pub fn cancel(&mut self, reason: &'static str) {
        self.failure.get_or_insert(reason);
    }
    pub fn finished(&self) -> bool {
        self.index == self.points.len() && self.failure.is_none()
    }
    pub fn description(&self) -> serde_json::Value {
        serde_json::json!({"waypoints":self.points.len(),"waypoint":self.index,"finished":self.finished(),"failure":self.failure,"elapsed_seconds":self.elapsed,"requested_turn_radians":self.turn_radians})
    }
    pub fn wait(&mut self, seconds: f64) {
        if !self.started || self.finished() || self.failure.is_some() {
            return;
        }
        if !seconds.is_finite() || seconds < 0. {
            self.cancel("Invalid controller timing");
            return;
        }
        self.unready += seconds;
        self.elapsed += seconds;
        if self.unready > 10. {
            self.cancel("World readiness timed out");
        }
        if self.elapsed > 600. {
            self.cancel("Travel time limit");
        }
    }
    pub fn sample(
        &mut self,
        seconds: f64,
        feet: GlobalPosition,
        facing: Facing,
        turn_rate: f64,
    ) -> Option<(Step, f64)> {
        if self.failure.is_some() || self.finished() {
            return None;
        }
        if !seconds.is_finite() || seconds < 0. || !turn_rate.is_finite() || turn_rate <= 0. {
            self.cancel("Invalid controller timing");
            return None;
        }
        let dt = seconds.min(0.1);
        self.elapsed += seconds;
        self.unready = 0.;
        if self.elapsed > 600. {
            self.cancel("Travel time limit");
            return None;
        }
        if !self.started {
            if distance(feet, self.points[0]) > 1. {
                self.cancel("Player is not at the checked route start");
                return None;
            }
            self.started = true;
        }
        while self.index < self.points.len() && distance(feet, self.points[self.index]) <= ARRIVAL {
            self.index += 1;
            self.reset_progress();
        }
        if self.finished() {
            return None;
        }
        if self.index > 0
            && segment_distance(feet, self.points[self.index - 1], self.points[self.index]) > 1.
        {
            self.cancel("Player left the checked route corridor");
            return None;
        }
        let target = self.points[self.index];
        let remaining = distance(feet, target);
        let angle = Facing::towards(feet, target).map_or(0., |desired| facing.difference(desired));
        if remaining + 0.05 < self.best_distance || angle.abs() + 0.01 < self.best_angle {
            self.stalled = 0.;
            self.best_distance = self.best_distance.min(remaining);
            self.best_angle = self.best_angle.min(angle.abs());
        } else {
            self.stalled += seconds;
        }
        if self.stalled >= STALL_SECONDS {
            self.cancel("Travel stalled");
            return None;
        }
        let turn = if angle.abs() <= 0.005 {
            0
        } else if angle > 0. {
            1
        } else {
            -1
        };
        let turn_seconds = if turn == 0 {
            dt
        } else {
            dt.min(angle.abs() / turn_rate)
        };
        self.turn_radians += f64::from(turn).abs() * turn_rate * turn_seconds;
        Some((
            Step {
                forward: if angle.abs() <= 0.12 { 1000 } else { 0 },
                turn,
                walk: true,
                ..Default::default()
            },
            turn_seconds,
        ))
    }
}
/// Follow the same floor continuously, rather than interpolating a long segment's
/// endpoint heights across rolling terrain. All original strict layer tolerances
/// and collision checks remain in force. Keep metre-spaced ground points for the
/// follower, then run ordinary clearance again over their connecting segments.
fn project_segment(
    query: &navigation::query::Query,
    ground: &mut navigation::authored_checks::Grounding<'_>,
    start: GlobalPosition,
    end: GlobalPosition,
    points: &mut Vec<GlobalPosition>,
    samples: &mut usize,
) -> theia_network::Result<()> {
    let mut previous = ground.at(start)?;
    let expected_end = ground.at(end)?;
    if points.last() != Some(&previous) {
        points.push(previous);
    }
    let a = start.metres();
    let b = end.metres();
    let steps = ((b[0] - a[0]).hypot(b[2] - a[2]) / 0.1).ceil().max(1.) as usize;
    *samples = samples
        .checked_add(steps)
        .filter(|n| *n <= 8192)
        .ok_or("Test route grounding sample budget exceeded")?;
    for step in 1..=steps {
        let fraction = step as f64 / steps as f64;
        let x = a[0] + (b[0] - a[0]) * fraction;
        let z = a[2] + (b[2] - a[2]) * fraction;
        let candidate = GlobalPosition::new([x, previous.metres()[1], z])
            .ok_or("Invalid projected route point")?;
        let sampled = query.sample(candidate)?;
        previous = ground
            .at(GlobalPosition::new([x, sampled.metres()[1], z]).ok_or("Invalid route height")?)?;
        if step == steps && distance(previous, expected_end) > 0.3 {
            return Err("Test route projection changed floor".into());
        }
        let last = points.last().unwrap().metres();
        if step == steps || (x - last[0]).hypot(z - last[2]) >= 1. {
            if points.last() != Some(&previous) {
                points.push(previous);
            }
            if points.len() > navigation::MAX_ROUTE_POINTS {
                return Err("Test route exceeds waypoint budget".into());
            }
        }
    }
    Ok(())
}

fn distance(a: GlobalPosition, b: GlobalPosition) -> f64 {
    a.metres()
        .into_iter()
        .zip(b.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
fn segment_distance(p: GlobalPosition, a: GlobalPosition, b: GlobalPosition) -> f64 {
    let p = p.metres();
    let a = a.metres();
    let b = b.metres();
    let length = (0..3).map(|i| (b[i] - a[i]).powi(2)).sum::<f64>();
    let t = if length == 0. {
        0.
    } else {
        ((0..3).map(|i| (p[i] - a[i]) * (b[i] - a[i])).sum::<f64>() / length).clamp(0., 1.)
    };
    (0..3)
        .map(|i| (p[i] - a[i] - t * (b[i] - a[i])).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(x: f64, z: f64) -> GlobalPosition {
        GlobalPosition::new([8192. + x, 0.008, 8192. + z]).unwrap()
    }
    fn route() -> Travel {
        Travel::new(vec![
            point(0., 0.),
            point(0., -12.),
            point(8., -12.),
            point(8., -20.),
        ])
        .unwrap()
    }
    #[test]
    #[ignore = "requires THEIA_ROUTE_WORLD and THEIA_ROUTE_FILE with actual authored navigation"]
    fn actual_rolling_route_is_grounded_continuously_without_floor_snapping() {
        let world = std::path::PathBuf::from(std::env::var_os("THEIA_ROUTE_WORLD").unwrap());
        let path = std::path::PathBuf::from(std::env::var_os("THEIA_ROUTE_FILE").unwrap());
        let source = procedural::Source::Authored(std::sync::Arc::new(
            theia_network::authored_world::Store::open(&world).unwrap(),
        ));
        let bytes = std::fs::read(&path).unwrap();
        let (_, travel) = Travel::load(&path, &bytes, &source).unwrap();
        assert!(travel.points.len() > 4);
        for pair in travel.points.windows(2) {
            let a = pair[0].metres();
            let b = pair[1].metres();
            assert!((a[0] - b[0]).hypot(a[2] - b[2]) <= 1.11);
        }
        let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for point in invalid["waypoints"].as_array_mut().unwrap() {
            point[1] = serde_json::json!(point[1].as_f64().unwrap() + 20.);
        }
        assert!(Travel::load(&path, &serde_json::to_vec(&invalid).unwrap(), &source).is_err());
    }
    #[test]
    fn continuous_walk_with_bounded_corner_turns_arrives_at_all_frame_rates() {
        for hz in [20., 30., 60., 144.] {
            let mut travel = route();
            let mut feet = point(0., 0.);
            let mut facing = Facing::default();
            let rate = std::f64::consts::PI;
            let dt = 1. / hz;
            let mut turning = 0.;
            let mut translating = 0;
            for _ in 0..(60. * hz) as usize {
                let Some((step, seconds)) = travel.sample(dt, feet, facing, rate) else {
                    break;
                };
                assert!(step.walk && !step.jump);
                assert_eq!(step.strafe, 0);
                assert!(seconds <= dt);
                let delta = f64::from(step.turn) * rate * seconds;
                assert!(delta.abs() <= rate * dt + 1e-9);
                turning += delta.abs();
                facing = Facing::from_radians(facing.radians() + delta).unwrap();
                let mut p = feet.metres();
                let forward = facing.forward();
                if step.forward != 0 {
                    translating += 1;
                }
                p[0] += forward[0] * f64::from(step.forward) / 1000. * 2.5 * dt;
                p[2] += forward[1] * f64::from(step.forward) / 1000. * 2.5 * dt;
                feet = GlobalPosition::new(p).unwrap();
            }
            assert!(travel.finished(), "{hz} Hz: {:?}", travel.description());
            assert!(distance(feet, point(8., -20.)) <= ARRIVAL);
            assert!(
                turning < 4.,
                "No camera/full-spin correction loop: {turning}"
            );
            assert!(translating > (10. * hz) as usize);
        }
    }
    #[test]
    fn stall_wrong_start_layer_drift_and_context_cancellation_stop_output() {
        let mut travel = route();
        for _ in 0..100 {
            travel.sample(0.1, point(0., 0.), Facing::default(), 3.);
        }
        assert_eq!(travel.failure, Some("Travel stalled"));
        assert!(
            travel
                .sample(0.1, point(0., -2.), Facing::default(), 3.)
                .is_none()
        );
        travel.reset();
        assert!(
            travel
                .sample(0.1, point(3., 0.), Facing::default(), 3.)
                .is_none()
        );
        assert_eq!(
            travel.failure,
            Some("Player is not at the checked route start")
        );
        travel.reset();
        let high = GlobalPosition::new([8192., 3., 8192.]).unwrap();
        assert!(travel.sample(0.1, high, Facing::default(), 3.).is_none());
        travel.reset();
        travel.sample(0.1, point(0., 0.), Facing::default(), 3.);
        assert!(
            travel
                .sample(0.1, point(2., -3.), Facing::default(), 3.)
                .is_none()
        );
        assert_eq!(
            travel.failure,
            Some("Player left the checked route corridor")
        );
        travel.reset();
        travel.cancel("Manual takeover");
        assert!(
            travel
                .sample(0.1, point(0., 0.), Facing::default(), 3.)
                .is_none()
        );
        assert!(!travel.finished());
    }
    #[test]
    fn lost_readiness_and_long_frame_cannot_extend_travel_indefinitely() {
        let mut travel = route();
        travel.wait(30.); // Loading before the first admitted pose is not progress.
        assert!(travel.failure.is_none());
        travel.sample(0.1, point(0., 0.), Facing::default(), 3.);
        travel.wait(11.);
        assert_eq!(travel.failure, Some("World readiness timed out"));
        travel.reset();
        assert!(
            travel
                .sample(601., point(0., 0.), Facing::default(), 3.)
                .is_none()
        );
        assert_eq!(travel.failure, Some("Travel time limit"));
    }
    #[test]
    fn timing_validation_and_shortest_turn_across_yaw_wrap() {
        let mut travel = Travel::new(vec![point(0., 0.), point(0., -12.)]).unwrap();
        let facing = Facing::from_radians(std::f64::consts::TAU - 0.02).unwrap();
        let (step, seconds) = travel.sample(0.1, point(0., 0.), facing, 3.).unwrap();
        assert_eq!(step.turn, 1);
        assert!(seconds < 0.01);
        assert!(travel.sample(f64::NAN, point(0., 0.), facing, 3.).is_none());
        assert_eq!(travel.failure, Some("Invalid controller timing"));
    }
}
