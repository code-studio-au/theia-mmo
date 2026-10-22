// Source contract: docs/FOREST_TRIPLANAR.md. Standard Bevy PBR extension.
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, calculate_tbn_mikktspace},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};

struct Parameters { values: vec4<f32>, phase: vec4<f32> }
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> parameters: Parameters;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var top: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var top_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var side: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var side_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var top_normal: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(106) var top_normal_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(107) var side_normal: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(108) var side_normal_sampler: sampler;

fn png_uv(uv: vec2<f32>) -> vec2<f32> { return vec2(uv.x, 1.0 - uv.y); }

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in, is_front);
    // Translation is reduced to a bounded periodic phase in f64 on the CPU.
    // Preserve the source's mirrored X, signed projections and PNG orientation.
    let position = in.world_position.xyz * vec3(-1.0, 1.0, 1.0) * parameters.values.x
        + parameters.phase.xyz;
    let normal = in.world_normal * vec3(-1.0, 1.0, 1.0);
    let powers = pow(abs(normal), vec3(parameters.values.y));
    let blend = powers / (powers.x + powers.y + powers.z + 0.00001);
    let signs = sign(normal);
    let bottom = max(0.0, -blend.y * signs.y);
    let upper = max(0.0, blend.y * signs.y);
    let x = png_uv(position.zy * vec2(signs.x, 1.0));
    let y = png_uv(position.xz * vec2(signs.y, 1.0));
    let z = png_uv(position.xy * vec2(-signs.z, 1.0));
    let colour = textureSample(side, side_sampler, x) * blend.x
        + textureSample(top, top_sampler, y) * upper
        + textureSample(side, side_sampler, y) * bottom
        + textureSample(side, side_sampler, z) * blend.z;
    let packed = textureSample(side_normal, side_normal_sampler, x) * blend.x
        + textureSample(top_normal, top_normal_sampler, y) * upper
        + textureSample(side_normal, side_normal_sampler, y) * bottom
        + textureSample(side_normal, side_normal_sampler, z) * blend.z;
    let xy = vec2(packed.r * packed.a, packed.g) * 2.0 - 1.0;
    let tangent_normal = vec3(xy, sqrt(1.0 - clamp(dot(xy, xy), 0.0, 1.0)));
    pbr.material.base_color = vec4(colour.rgb, 1.0);
    if parameters.values.z == 2.0 { pbr.material.base_color = vec4(tangent_normal * 0.5 + 0.5, 1.0); }
    @if(VERTEX_TANGENTS) {
        let tbn = calculate_tbn_mikktspace(pbr.world_normal, in.world_tangent);
        pbr.N = normalize(tbn * tangent_normal);
    }
    var out: FragmentOutput;
    if (pbr.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr);
    } else {
        out.color = pbr.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
}
