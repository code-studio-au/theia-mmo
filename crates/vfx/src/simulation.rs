//! Conservative preparation checks; these are not a complete GPU memory policy.
use bevy::{
    math::Vec3A,
    prelude::{Quat, Vec2, Vec3, Vec4},
    reflect::{PartialReflect, ReflectRef},
};
use bevy_sprinkles::asset::{Curve, Gradient, ParticlesAsset, ParticlesDimension};

const MAX_FLOAT: f64 = 1_000_000.0;
const MAX_NODES: usize = 100_000;
const MAX_COLLECTION: usize = 1024;
const MAX_POINTS: usize = 64;

/// Reject unbounded/non-finite reflected values and unsafe timing before a
/// package is considered for runtime. This deliberately accepts a conservative
/// subset and is not complete semantic or GPU resource admission.
pub fn validate(asset: &ParticlesAsset) -> Result<(), String> {
    if asset.dimension != ParticlesDimension::D3
        || asset.emitters.is_empty()
        || asset.emitters.len() > 16
        || asset.colliders.len() > 8
    {
        return Err("Particle effect exceeds dimension, emitter or collider limits".into());
    }
    for emitter in &asset.emitters {
        let time = &emitter.time;
        if !(0.01..=120.).contains(&time.lifetime)
            || !(0. ..=1.).contains(&time.lifetime_randomness)
            || !(0. ..=120.).contains(&time.delay)
            || !(0. ..=1.).contains(&time.explosiveness)
            || !(0. ..=1.).contains(&time.spawn_time_randomness)
            || time.fixed_fps > 120
        {
            return Err("Particle effect has unsupported emitter timing".into());
        }
    }
    // Positive infinity is Sprinkles/Bevy's documented default for a material
    // attenuation distance. Check all other values on a normalized clone.
    let mut checked = asset.clone();
    for emitter in &mut checked.emitters {
        if let bevy_sprinkles::asset::DrawPassMaterial::Standard(material) =
            &mut emitter.draw_pass.material
            && material.attenuation_distance == f32::INFINITY
        {
            material.attenuation_distance = MAX_FLOAT as f32;
        }
    }
    let mut remaining = MAX_NODES;
    inspect(&checked, &mut remaining, 0)
}
fn inspect(value: &dyn PartialReflect, remaining: &mut usize, depth: usize) -> Result<(), String> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or("Particle reflection node budget exceeded")?;
    if depth > 64 {
        return Err("Particle reflection depth exceeded".into());
    }
    if let Some(n) = value.try_downcast_ref::<f32>() {
        return finite(f64::from(*n));
    }
    if let Some(n) = value.try_downcast_ref::<f64>() {
        return finite(*n);
    }
    if let Some(v) = value.try_downcast_ref::<Vec2>() {
        return v
            .to_array()
            .into_iter()
            .try_for_each(|n| finite(f64::from(n)));
    }
    if let Some(v) = value.try_downcast_ref::<Vec3>() {
        return v
            .to_array()
            .into_iter()
            .try_for_each(|n| finite(f64::from(n)));
    }
    if let Some(v) = value.try_downcast_ref::<Vec3A>() {
        return v
            .to_array()
            .into_iter()
            .try_for_each(|n| finite(f64::from(n)));
    }
    if let Some(v) = value.try_downcast_ref::<Vec4>() {
        return v
            .to_array()
            .into_iter()
            .try_for_each(|n| finite(f64::from(n)));
    }
    if let Some(v) = value.try_downcast_ref::<Quat>() {
        return v
            .to_array()
            .into_iter()
            .try_for_each(|n| finite(f64::from(n)));
    }
    if let Some(curve) = value.try_downcast_ref::<Curve>() {
        ordered(curve.points.iter().map(|p| p.position), curve.points.len())?;
    }
    if let Some(gradient) = value.try_downcast_ref::<Gradient>() {
        ordered(
            gradient.stops.iter().map(|p| p.position),
            gradient.stops.len(),
        )?;
    }
    match value.reflect_ref() {
        ReflectRef::Struct(v) => fields(v.field_len(), |i| v.field_at(i), remaining, depth),
        ReflectRef::TupleStruct(v) => fields(v.field_len(), |i| v.field(i), remaining, depth),
        ReflectRef::Tuple(v) => fields(v.field_len(), |i| v.field(i), remaining, depth),
        ReflectRef::List(v) => fields(v.len(), |i| v.get(i), remaining, depth),
        ReflectRef::Array(v) => fields(v.len(), |i| v.get(i), remaining, depth),
        ReflectRef::Enum(v) => fields(v.field_len(), |i| v.field_at(i), remaining, depth),
        ReflectRef::Opaque(v)
            if v.try_downcast_ref::<String>().is_some()
                || v.try_downcast_ref::<bool>().is_some()
                || v.try_downcast_ref::<u8>().is_some()
                || v.try_downcast_ref::<u32>().is_some()
                || v.try_downcast_ref::<u64>().is_some()
                || v.try_downcast_ref::<usize>().is_some() =>
        {
            Ok(())
        }
        _ => Err(format!(
            "Unsupported reflected particle value: {}",
            value.reflect_type_path()
        )),
    }
}
fn fields<'a>(
    n: usize,
    get: impl Fn(usize) -> Option<&'a dyn PartialReflect>,
    remaining: &mut usize,
    depth: usize,
) -> Result<(), String> {
    if n > MAX_COLLECTION {
        return Err("Particle collection exceeds limit".into());
    }
    for i in 0..n {
        inspect(
            get(i).ok_or("Missing reflected particle field")?,
            remaining,
            depth + 1,
        )?;
    }
    Ok(())
}
fn finite(n: f64) -> Result<(), String> {
    if n.is_finite() && n.abs() <= MAX_FLOAT {
        Ok(())
    } else {
        Err("Particle numeric value is non-finite or exceeds limits".into())
    }
}
fn ordered(values: impl Iterator<Item = f32>, len: usize) -> Result<(), String> {
    if !(1..=MAX_POINTS).contains(&len) {
        return Err("Particle curve or gradient exceeds point limit".into());
    }
    let mut previous = -1.;
    for value in values {
        if !(0. ..=1.).contains(&value) || value < previous {
            return Err("Particle curve or gradient positions are invalid".into());
        }
        previous = value;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::asset::{GradientStop, versions};

    fn effect() -> ParticlesAsset {
        versions::migrate(include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles/tests/fixtures/valid_particle_system.ron")).unwrap().asset
    }

    #[test]
    fn validates_nested_values_timing_and_gradient_limits() {
        let asset = effect();
        validate(&asset).unwrap();
        let mut invalid = asset.clone();
        invalid.emitters[0].velocities.initial_velocity.min = f32::NAN;
        assert!(validate(&invalid).is_err());
        invalid = asset.clone();
        invalid.emitters[0].time.lifetime = 0.;
        assert!(validate(&invalid).is_err());
        invalid = asset.clone();
        invalid.emitters[0].time.fixed_fps = 121;
        assert!(validate(&invalid).is_err());
        invalid = asset.clone();
        invalid.emitters[0].colors.color_over_lifetime.stops = vec![
            GradientStop {
                position: 0.5,
                color: [1.; 4]
            };
            65
        ];
        assert!(validate(&invalid).is_err());
        invalid = asset.clone();
        invalid.emitters[0].colors.color_over_lifetime.stops[0].position = 0.9;
        invalid.emitters[0].colors.color_over_lifetime.stops[1].position = 0.1;
        assert!(validate(&invalid).is_err());
        invalid = asset.clone();
        invalid.emitters[0].emission.offset.x = f32::INFINITY;
        assert!(validate(&invalid).is_err());
        invalid = asset;
        invalid.emitters[0].velocities.spread = 1_000_001.;
        assert!(validate(&invalid).is_err());
    }

    #[test]
    fn supports_upstream_examples_within_the_numeric_subset() {
        for (name, bytes) in [
            ("3d-explosion", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/3d-explosion.ron").as_slice()),
            ("acid-pool", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/acid-pool.ron").as_slice()),
            ("loot", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/loot.ron").as_slice()),
            ("magic-puff", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/magic-puff.ron").as_slice()),
            ("rain", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/rain.ron").as_slice()),
            ("tube-trails", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/tube-trails.ron").as_slice()),
            ("windy-snow", include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/windy-snow.ron").as_slice()),
        ] {
            let asset = versions::migrate(bytes).unwrap().asset;
            validate(&asset).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
    }
}
