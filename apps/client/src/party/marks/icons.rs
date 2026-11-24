//! Original geometric symbols, rasterized once into ordinary Bevy images.
//! No external font glyphs or licensed game artwork are required.
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use theia_core::party::marks::Symbol;

fn polygon(point: Vec2, vertices: &[Vec2]) -> bool {
    let mut inside = false;
    for (a, b) in vertices.iter().zip(vertices.iter().cycle().skip(1)) {
        if (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
    }
    inside
}
fn shape(symbol: Symbol, p: Vec2) -> bool {
    let Vec2 { x, y } = p;
    match symbol {
        Symbol::Skull => {
            ((x * x / 0.48 + (y + 0.15).powi(2) / 0.39 < 1.)
                || (x.abs() < 0.42 && (0.1..0.68).contains(&y)))
                && ((x.abs() - 0.25).powi(2) + (y + 0.13).powi(2) >= 0.025)
                && !(x.abs() < 0.08 && (0.1..0.27).contains(&y))
                && !((x.abs() - 0.14).abs() < 0.035 && y > 0.44)
        }
        Symbol::Cross => {
            x.abs() < 0.68 && y.abs() < 0.68 && ((x - y).abs() < 0.21 || (x + y).abs() < 0.21)
        }
        Symbol::Square => x.abs() < 0.67 && y.abs() < 0.67,
        Symbol::Moon => {
            p.length_squared() < 0.53 && (p - Vec2::new(0.34, -0.24)).length_squared() > 0.4
        }
        Symbol::Triangle => polygon(
            p,
            &[
                Vec2::new(0., -0.78),
                Vec2::new(0.75, 0.65),
                Vec2::new(-0.75, 0.65),
            ],
        ),
        Symbol::Diamond => x.abs() / 0.65 + y.abs() / 0.85 < 1.,
        Symbol::Circle => p.length_squared() < 0.49,
        Symbol::Star => {
            let vertices: [Vec2; 10] = std::array::from_fn(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 10. - std::f32::consts::FRAC_PI_2;
                Vec2::new(angle.cos(), angle.sin()) * if i % 2 == 0 { 0.85 } else { 0.37 }
            });
            polygon(p, &vertices)
        }
    }
}
fn tint(symbol: Symbol) -> [u8; 3] {
    match symbol {
        Symbol::Skull => [248, 242, 220],
        Symbol::Cross => [248, 65, 56],
        Symbol::Square => [50, 170, 255],
        Symbol::Moon => [195, 226, 255],
        Symbol::Triangle => [65, 235, 100],
        Symbol::Diamond => [208, 100, 255],
        Symbol::Circle => [255, 152, 42],
        Symbol::Star => [255, 222, 65],
    }
}
pub(super) fn image(symbol: Symbol) -> Image {
    const SIZE: usize = 64;
    let mut pixels = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut sums = [0u32; 4];
            for sy in 0..3 {
                for sx in 0..3 {
                    let p = Vec2::new(
                        x as f32 + (sx as f32 + 0.5) / 3.,
                        y as f32 + (sy as f32 + 0.5) / 3.,
                    ) * (2. / SIZE as f32)
                        - Vec2::ONE;
                    let inside = shape(symbol, p);
                    let outline = !inside
                        && [
                            Vec2::X,
                            Vec2::Y,
                            -Vec2::X,
                            -Vec2::Y,
                            Vec2::new(0.7, 0.7),
                            Vec2::new(-0.7, 0.7),
                            Vec2::new(0.7, -0.7),
                            Vec2::new(-0.7, -0.7),
                        ]
                        .iter()
                        .any(|d| shape(symbol, p + *d * 0.075));
                    if inside || outline {
                        let color = if inside { tint(symbol) } else { [12, 15, 22] };
                        for i in 0..3 {
                            sums[i] += u32::from(color[i]);
                        }
                        sums[3] += 255;
                    }
                }
            }
            let offset = (y * SIZE + x) * 4;
            for i in 0..3 {
                pixels[offset + i] = (sums[i] * 255).checked_div(sums[3]).unwrap_or(0) as u8;
            }
            pixels[offset + 3] = (sums[3] / 9) as u8;
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
