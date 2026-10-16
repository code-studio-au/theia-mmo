//! Temporary offline presentation movement, intentionally not server authority.
use bevy::prelude::*;

pub const START: Vec3 = Vec3::new(0., theia_network::world::PAWN_Y, 0.);

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
    network: Option<Res<crate::network::Network>>,
    mut pawns: Query<&mut Transform, With<LocalPawn>>,
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
    for mut pawn in &mut pawns {
        let xz = theia_network::world::move_player(
            [pawn.translation.x, pawn.translation.z],
            intent,
            time.delta_secs(),
        );
        pawn.translation.x = xz[0];
        pawn.translation.z = xz[1];
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
