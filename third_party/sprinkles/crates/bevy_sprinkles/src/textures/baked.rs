use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
};
use std::collections::{HashMap, HashSet};

use crate::asset::{
    CurveTexture, Gradient, GradientInterpolation, ParticlesAsset, SolidOrGradientColor,
};
use crate::runtime::Particles3d;

const TEXTURE_WIDTH: u32 = 256;

/// Cache for baked gradient textures, avoiding redundant texture creation.
///
/// Each unique gradient (identified by its [`Gradient::cache_key`]) is baked into
/// a 1D RGBA texture once and reused across all emitters that reference it.
#[derive(Resource, Default)]
pub struct GradientTextureCache {
    cache: HashMap<u64, Handle<Image>>,
}

impl GradientTextureCache {
    /// Returns a cached texture handle for the gradient, creating and baking a new
    /// texture if one doesn't already exist.
    pub fn get_or_create(
        &mut self,
        gradient: &Gradient,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        let key = gradient.cache_key();
        if let Some(handle) = self.cache.get(&key) {
            return handle.clone();
        }
        let image = bake_gradient_texture(gradient);
        let handle = images.add(image);
        self.cache.insert(key, handle.clone());
        handle
    }

    /// Returns the cached texture handle for the gradient, if it exists.
    pub fn get(&self, gradient: &Gradient) -> Option<Handle<Image>> {
        self.cache.get(&gradient.cache_key()).cloned()
    }
}

fn bake_gradient_texture(gradient: &Gradient) -> Image {
    let mut data = Vec::with_capacity((TEXTURE_WIDTH * 4) as usize);

    for i in 0..TEXTURE_WIDTH {
        let t = if TEXTURE_WIDTH > 1 {
            i as f32 / (TEXTURE_WIDTH - 1) as f32
        } else {
            0.0
        };
        let color = sample_gradient(gradient, t);
        data.push((color[0] * 255.0).clamp(0.0, 255.0) as u8);
        data.push((color[1] * 255.0).clamp(0.0, 255.0) as u8);
        data.push((color[2] * 255.0).clamp(0.0, 255.0) as u8);
        data.push((color[3] * 255.0).clamp(0.0, 255.0) as u8);
    }

    create_1d_texture(data, TextureFormat::Rgba8UnormSrgb)
}

fn sample_gradient(gradient: &Gradient, t: f32) -> [f32; 4] {
    let stops = &gradient.stops;

    if stops.is_empty() {
        return [1.0, 1.0, 1.0, 1.0];
    }
    if stops.len() == 1 {
        return stops[0].color;
    }

    let t = t.clamp(0.0, 1.0);
    let mut left_idx = 0;
    let mut right_idx = stops.len() - 1;

    for (i, stop) in stops.iter().enumerate() {
        if stop.position <= t {
            left_idx = i;
        }
    }
    for (i, stop) in stops.iter().enumerate() {
        if stop.position >= t {
            right_idx = i;
            break;
        }
    }

    let left = &stops[left_idx];
    let right = &stops[right_idx];

    if left_idx == right_idx {
        return left.color;
    }

    let range = right.position - left.position;
    if range <= 0.0 {
        return left.color;
    }

    let local_t = (t - left.position) / range;

    match gradient.interpolation {
        GradientInterpolation::Steps => left.color,
        GradientInterpolation::Linear => lerp_color(left.color, right.color, local_t),
        GradientInterpolation::Smoothstep => {
            let smooth_t = local_t * local_t * (3.0 - 2.0 * local_t);
            lerp_color(left.color, right.color, smooth_t)
        }
    }
}

fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// A 1x1 white fallback texture used when no gradient texture is available.
#[derive(Resource, Clone, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub struct FallbackGradientTexture {
    /// Handle to the fallback image.
    pub handle: Handle<Image>,
}

/// Bakes gradient textures for all active particle systems.
pub fn prepare_gradient_textures(
    mut cache: ResMut<GradientTextureCache>,
    mut images: ResMut<Assets<Image>>,
    particle_systems: Query<&Particles3d>,
    assets: Res<Assets<ParticlesAsset>>,
) {
    let mut used = HashSet::new();
    for system in &particle_systems {
        let Some(asset) = assets.get(system) else {
            continue;
        };
        for emitter in &asset.emitters {
            if let SolidOrGradientColor::Gradient { gradient } = &emitter.colors.initial_color {
                used.insert(gradient.cache_key());
                cache.get_or_create(gradient, &mut images);
            }
            used.insert(emitter.colors.color_over_lifetime.cache_key());
            cache.get_or_create(&emitter.colors.color_over_lifetime, &mut images);
        }
    }
    // Drop obsolete strong handles after edits/project changes; Bevy releases the assets.
    cache.cache.retain(|key, _| used.contains(key));
}

/// Cache for baked curve textures, avoiding redundant texture creation.
///
/// Each unique curve (identified by its [`CurveTexture::cache_key`]) is baked into
/// a 1D grayscale texture once and reused across all emitters that reference it.
#[derive(Resource, Default)]
pub struct CurveTextureCache {
    cache: HashMap<u64, Handle<Image>>,
}

impl CurveTextureCache {
    /// Returns a cached texture handle for the curve, creating and baking a new
    /// texture if one doesn't already exist.
    pub fn get_or_create(
        &mut self,
        curve: &CurveTexture,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        let key = curve.cache_key();
        if let Some(handle) = self.cache.get(&key) {
            return handle.clone();
        }
        let image = bake_curve_texture(curve);
        let handle = images.add(image);
        self.cache.insert(key, handle.clone());
        handle
    }

    /// Returns the cached texture handle for the curve, if it exists.
    pub fn get(&self, curve: &CurveTexture) -> Option<Handle<Image>> {
        self.cache.get(&curve.cache_key()).cloned()
    }
}

fn bake_curve_texture(curve: &CurveTexture) -> Image {
    let mut data = Vec::with_capacity((TEXTURE_WIDTH * 4) as usize);

    for i in 0..TEXTURE_WIDTH {
        let t = if TEXTURE_WIDTH > 1 {
            i as f32 / (TEXTURE_WIDTH - 1) as f32
        } else {
            0.0
        };
        let x = curve.sample(t);
        let y = curve.sample_channel(1, t);
        let z = curve.sample_channel(2, t);
        data.push((x.clamp(0.0, 1.0) * 255.0) as u8); // R
        data.push((y.clamp(0.0, 1.0) * 255.0) as u8); // G
        data.push((z.clamp(0.0, 1.0) * 255.0) as u8); // B
        data.push(255); // A
    }

    create_1d_texture(data, TextureFormat::Rgba8Unorm)
}

/// A 1x1 white fallback texture used when no curve texture is available.
#[derive(Resource, Clone, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub struct FallbackCurveTexture {
    /// Handle to the fallback image.
    pub handle: Handle<Image>,
}

impl CurveTextureCache {
    fn prepare_optional(
        &mut self,
        curve: &Option<CurveTexture>,
        images: &mut Assets<Image>,
        used: &mut HashSet<u64>,
    ) {
        if let Some(c) = curve.as_ref().filter(|c| !c.is_constant()) {
            used.insert(c.cache_key());
            self.get_or_create(c, images);
        }
    }
}

/// Bakes curve textures for all active particle systems.
pub fn prepare_curve_textures(
    mut cache: ResMut<CurveTextureCache>,
    mut images: ResMut<Assets<Image>>,
    particle_systems: Query<&Particles3d>,
    assets: Res<Assets<ParticlesAsset>>,
) {
    let mut used = HashSet::new();
    for system in &particle_systems {
        let Some(asset) = assets.get(system) else {
            continue;
        };
        for emitter in &asset.emitters {
            cache.prepare_optional(&emitter.scale.scale_over_lifetime, &mut images, &mut used);
            cache.prepare_optional(&emitter.colors.alpha_over_lifetime, &mut images, &mut used);
            cache.prepare_optional(
                &emitter.colors.emission_over_lifetime,
                &mut images,
                &mut used,
            );
            cache.prepare_optional(
                &emitter.turbulence.influence_over_lifetime,
                &mut images,
                &mut used,
            );
            cache.prepare_optional(&emitter.angle.angle_over_lifetime, &mut images, &mut used);
            cache.prepare_optional(
                &emitter.velocities.radial_velocity.velocity_over_lifetime,
                &mut images,
                &mut used,
            );
            cache.prepare_optional(
                &emitter.velocities.angular_velocity.velocity_over_lifetime,
                &mut images,
                &mut used,
            );
            cache.prepare_optional(
                &emitter.velocities.orbit_velocity.velocity_over_lifetime,
                &mut images,
                &mut used,
            );
            cache.prepare_optional(
                &emitter
                    .velocities
                    .directional_velocity
                    .velocity_over_lifetime,
                &mut images,
                &mut used,
            );
        }
    }
    cache.cache.retain(|key, _| used.contains(key));
}

fn create_1d_texture(data: Vec<u8>, format: TextureFormat) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: TEXTURE_WIDTH,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC;
    image
}

fn create_fallback_texture(format: TextureFormat) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255, 255, 255, 255],
        format,
        default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC;
    image
}

/// Creates and inserts the [`FallbackGradientTexture`] resource.
pub fn create_fallback_gradient_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let handle = images.add(create_fallback_texture(TextureFormat::Rgba8UnormSrgb));
    commands.insert_resource(FallbackGradientTexture { handle });
}

/// Creates and inserts the [`FallbackCurveTexture`] resource.
pub fn create_fallback_curve_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let handle = images.add(create_fallback_texture(TextureFormat::Rgba8Unorm));
    commands.insert_resource(FallbackCurveTexture { handle });
}

#[cfg(test)]
mod theia_cache_tests {
    use super::*;
    use crate::asset::{EmitterData, ParticlesDimension};

    #[test]
    fn editing_and_closing_projects_releases_obsolete_baked_textures() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<ParticlesAsset>()
            .init_resource::<GradientTextureCache>()
            .init_resource::<CurveTextureCache>()
            .add_systems(
                PostUpdate,
                (prepare_gradient_textures, prepare_curve_textures),
            );
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Cache test".into(),
                ParticlesDimension::D3,
                default(),
                vec![EmitterData::default()],
                vec![],
                false,
                default(),
            ));
        let first = app.world_mut().spawn(Particles3d(handle.clone())).id();
        let second = app.world_mut().spawn(Particles3d(handle.clone())).id();
        let mut previous = None;
        for edit in 0..128 {
            {
                let mut assets = app.world_mut().resource_mut::<Assets<ParticlesAsset>>();
                let emitter = &mut assets.get_mut(&handle).unwrap().emitters[0];
                emitter.colors.color_over_lifetime.stops[0].color[0] = edit as f32 / 128.;
                let mut curve = CurveTexture::default();
                curve.x.points[0].value = edit as f64 / 128.;
                curve.x.points[1].value = 1.;
                emitter.scale.scale_over_lifetime = Some(curve);
            }
            app.update();
            assert_eq!(
                app.world().resource::<GradientTextureCache>().cache.len(),
                1
            );
            assert_eq!(app.world().resource::<CurveTextureCache>().cache.len(), 1);
            let current = app
                .world()
                .resource::<GradientTextureCache>()
                .cache
                .values()
                .next()
                .unwrap()
                .id();
            if let Some(old) = previous {
                // Give Bevy's normal asset lifecycle time to process dropped handles.
                app.update();
                app.update();
                assert!(!app.world().resource::<Assets<Image>>().contains(old));
            }
            previous = Some(current);
            assert!(app.world().resource::<Assets<Image>>().len() <= 4);
        }
        app.world_mut().despawn(first);
        app.update();
        assert_eq!(
            app.world().resource::<GradientTextureCache>().cache.len(),
            1
        );
        app.world_mut().despawn(second);
        for _ in 0..4 {
            app.update();
        }
        assert!(
            app.world()
                .resource::<GradientTextureCache>()
                .cache
                .is_empty()
        );
        assert!(app.world().resource::<CurveTextureCache>().cache.is_empty());
        assert!(app.world().resource::<Assets<Image>>().is_empty());
    }
}
