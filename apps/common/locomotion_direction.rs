//! Eight-way presentation sector shared by the game and editor playtest.
//! Input is character-local angle; neither camera nor targets turn the actor.
pub fn sector(angle: f32, previous: Option<u8>) -> u8 {
    if let Some(old) = previous.filter(|i| *i < 8) {
        let centre = f32::from(old) * std::f32::consts::FRAC_PI_4;
        let error = (angle - centre + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        if error.abs() <= 28.5_f32.to_radians() {
            return old;
        }
    }
    ((angle / std::f32::consts::FRAC_PI_4).round() as i32).rem_euclid(8) as u8
}

/// Preserve normalized locomotion phase when changing direction or gait.
pub fn phase(time: f32, old_duration: f32, new_duration: f32) -> Option<f32> {
    (time.is_finite()
        && old_duration.is_finite()
        && new_duration.is_finite()
        && old_duration > 0.
        && new_duration > 0.)
        .then(|| time.rem_euclid(old_duration) / old_duration * new_duration)
}
