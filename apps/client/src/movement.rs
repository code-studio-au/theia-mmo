//! Temporary offline presentation movement, intentionally not server authority.
use bevy::prelude::*;

pub const START: Vec3 = Vec3::new(0., 0.9, 0.);
const WALK_SPEED: f32 = 5.;
const RUN_SPEED: f32 = 8.;
const WORLD_LIMIT: f32 = 19.;

#[derive(Component)]
pub struct LocalPawn;

#[derive(Resource, Default)]
pub struct MoveIntent {
    pub direction: Vec2,
    pub sprint: bool,
}

pub fn move_local_pawn(
    time: Res<Time<Fixed>>,
    intent: Res<MoveIntent>,
    mut pawns: Query<&mut Transform, With<LocalPawn>>,
) {
    let direction = intent.direction.clamp_length_max(1.);
    let speed = if intent.sprint { RUN_SPEED } else { WALK_SPEED };
    for mut pawn in &mut pawns {
        pawn.translation += Vec3::new(direction.x, 0., direction.y) * speed * time.delta_secs();
        pawn.translation.x = pawn.translation.x.clamp(-WORLD_LIMIT, WORLD_LIMIT);
        pawn.translation.z = pawn.translation.z.clamp(-WORLD_LIMIT, WORLD_LIMIT);
        if direction != Vec2::ZERO {
            pawn.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
