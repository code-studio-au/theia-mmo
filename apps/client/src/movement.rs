//! Temporary offline presentation movement, intentionally not server authority.
use bevy::prelude::*;

pub const START: Vec3 = Vec3::new(0., theia_network::world::PAWN_Y, 0.);

#[derive(Component)]
pub struct LocalPawn;

/// Presentation speed sampled from actual movement, never a gameplay authority.
#[derive(Component, Default)]
pub struct Motion(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gait {
    Stand,
    Walk,
    Run,
}
impl Gait {
    pub fn from_speed(speed: f32) -> Self {
        if speed > 6.5 {
            Self::Run
        } else if speed > 0.1 {
            Self::Walk
        } else {
            Self::Stand
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Stand => 0,
            Self::Walk => 1,
            Self::Run => 2,
        }
    }
}

#[derive(Resource, Default)]
pub struct MoveIntent {
    pub direction: Vec2,
    pub sprint: bool,
}

pub fn move_local_pawn(
    time: Res<Time<Fixed>>,
    intent: Res<MoveIntent>,
    network: Option<Res<crate::network::Network>>,
    mut pawns: Query<(&mut Transform, Option<&mut Motion>), With<LocalPawn>>,
) {
    if network.is_some() {
        return;
    }
    let direction = intent.direction.clamp_length_max(1.);
    let intent = theia_network::wire::Intent {
        axes: [
            (intent.direction.x.clamp(-1., 1.) * 1000.) as i16,
            (intent.direction.y.clamp(-1., 1.) * 1000.) as i16,
        ],
        sprint: intent.sprint,
    };
    for (mut pawn, motion) in &mut pawns {
        let before = pawn.translation;
        let xz = theia_network::world::move_player(
            [pawn.translation.x, pawn.translation.z],
            intent,
            time.delta_secs(),
        );
        pawn.translation.x = xz[0];
        pawn.translation.z = xz[1];
        if let Some(mut motion) = motion {
            motion.0 = pawn.translation.distance(before) / time.delta_secs().max(1e-6);
        }
        if direction != Vec2::ZERO {
            pawn.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use theia_network::wire::WORLD_LIMIT;

    #[test]
    fn gait_distinguishes_rest_walking_and_running() {
        assert_eq!(Gait::from_speed(0.), Gait::Stand);
        assert_eq!(Gait::from_speed(5.), Gait::Walk);
        assert_eq!(Gait::from_speed(8.), Gait::Run);
    }

    #[test]
    fn pressing_into_a_wall_reports_idle_despite_movement_input() {
        let mut app = App::new();
        let mut time = Time::<Fixed>::from_hz(60.);
        time.advance_by(Duration::from_secs_f64(1. / 60.));
        app.insert_resource(time)
            .insert_resource(MoveIntent {
                direction: Vec2::NEG_X,
                sprint: true,
            })
            .add_systems(FixedUpdate, move_local_pawn);
        let pawn = app
            .world_mut()
            .spawn((
                LocalPawn,
                Motion::default(),
                Transform::from_xyz(-2.65, START.y, -7.),
            ))
            .id();
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(
            Gait::from_speed(app.world().get::<Motion>(pawn).unwrap().0),
            Gait::Stand
        );
    }

    fn simulate(direction: Vec2, sprint: bool, hz: u32, ticks: u32) -> Vec3 {
        let mut app = App::new();
        let mut time = Time::<Fixed>::from_hz(f64::from(hz));
        time.advance_by(Duration::from_secs_f64(1. / f64::from(hz)));
        app.insert_resource(time)
            .insert_resource(MoveIntent { direction, sprint })
            .add_systems(FixedUpdate, move_local_pawn);
        let entity = app
            .world_mut()
            .spawn((LocalPawn, Transform::from_translation(START)))
            .id();
        for _ in 0..ticks {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world().get::<Transform>(entity).unwrap().translation
    }

    #[test]
    fn diagonal_input_cannot_increase_speed() {
        let straight = simulate(Vec2::X, false, 60, 60) - START;
        let diagonal = simulate(Vec2::ONE, false, 60, 60) - START;
        assert!((straight.length() - 5.).abs() < 1e-4);
        assert!((diagonal.length() - straight.length()).abs() < 1e-4);
    }

    #[test]
    fn fixed_step_rates_produce_the_same_distance() {
        assert!(
            (simulate(Vec2::NEG_Y, true, 30, 30) - simulate(Vec2::NEG_Y, true, 60, 60)).length()
                < 1e-4
        );
        assert!((simulate(Vec2::NEG_Y, true, 60, 60).z + 8.).abs() < 1e-4);
    }

    #[test]
    fn movement_stays_on_the_ground_inside_the_sandbox() {
        let position = simulate(Vec2::ONE, true, 60, 600);
        assert_eq!(position, Vec3::new(WORLD_LIMIT, START.y, WORLD_LIMIT));
        assert_eq!(simulate(Vec2::ZERO, false, 60, 60), START);
    }
}
