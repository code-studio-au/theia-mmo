//! Windowless CPU texture-cache lifecycle probe. No renderer or editor UI is started.
use bevy::prelude::*;
use bevy_sprinkles::{
    asset::{CurveTexture, EmitterData, ParticlesAsset, ParticlesDimension},
    runtime::Particles3d,
    textures::{
        CurveTextureCache, GradientTextureCache, prepare_curve_textures, prepare_gradient_textures,
    },
};
use std::time::{Duration, Instant};

fn rss_kib() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|line| line.starts_with("VmRSS:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}
fn main() {
    let seconds: u64 = std::env::var("THEIA_SOAK_SECONDS")
        .unwrap_or_else(|_| "120".into())
        .parse()
        .expect("seconds");
    assert!((1..=3600).contains(&seconds));
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
    let started = Instant::now();
    let mut samples = Vec::new();
    for cycle in 0..1000 {
        let mut emitter = EmitterData::default();
        emitter.colors.color_over_lifetime.stops[0].color[0] = cycle as f32 / 1000.;
        let mut curve = CurveTexture::default();
        curve.x.points[0].value = cycle as f64 / 1000.;
        emitter.scale.scale_over_lifetime = Some(curve);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Soak".into(),
                ParticlesDimension::D3,
                default(),
                vec![emitter],
                vec![],
                false,
                default(),
            ));
        let first = app.world_mut().spawn(Particles3d(handle.clone())).id();
        let second = app.world_mut().spawn(Particles3d(handle.clone())).id();
        for _ in 0..4 {
            app.update();
        }
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 2);
        app.world_mut().despawn(first);
        app.update();
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 2);
        app.world_mut().despawn(second);
        drop(handle);
        for _ in 0..6 {
            app.update();
        }
        assert!(app.world().resource::<Assets<Image>>().is_empty());
        assert!(app.world().resource::<Assets<ParticlesAsset>>().is_empty());
        if cycle % 100 == 99 {
            samples.push(serde_json::json!({"cycle":cycle+1,"rss_kib":rss_kib()}));
        }
    }
    let idle = Instant::now();
    let mut next_sample = Duration::ZERO;
    while idle.elapsed() < Duration::from_secs(seconds) {
        app.update();
        assert!(app.world().resource::<Assets<Image>>().is_empty());
        if idle.elapsed() >= next_sample {
            samples.push(serde_json::json!({"idle_seconds":idle.elapsed().as_secs_f64(),"rss_kib":rss_kib()}));
            next_sample += Duration::from_secs(10);
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    println!(
        "{}",
        serde_json::json!({"passed":true,"cycles":1000,"idle_seconds":seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),"images_after_close":0,"effect_assets_after_close":0,"samples":samples,"scope":"CPU baked gradient/curve cache only; no window, editor UI, GPU allocation or full-session leak qualification"})
    );
}
