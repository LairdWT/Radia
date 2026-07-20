struct CompositeUniform {
    resolution: vec2<f32>,
    frame_index: u32,
    mode: u32,
}

@group(0) @binding(0) var<uniform> frame: CompositeUniform;
@group(0) @binding(1) var direct_radiance: texture_2d<f32>;
@group(0) @binding(2) var indirect_radiance: texture_2d<f32>;
@group(0) @binding(3) var composite_normal_material: texture_2d<f32>;
@group(0) @binding(4) var composite_depth: texture_depth_2d;

const MODE_RADIA: u32 = 1u;
const MODE_GI_ONLY: u32 = 2u;
const MODE_AMBIENT_OCCLUSION: u32 = 12u;

@vertex
fn vs_composite(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

fn resolved_indirect(pixel: vec2<i32>) -> vec4<f32> {
    let dimensions = vec2<i32>(textureDimensions(composite_depth));
    let center_depth = textureLoad(composite_depth, pixel, 0);
    let center_surface = textureLoad(composite_normal_material, pixel, 0);
    if center_depth <= 0.0 || dot(center_surface.xyz, center_surface.xyz) < 0.25 {
        return textureLoad(indirect_radiance, pixel, 0);
    }

    let center_normal = normalize(center_surface.xyz);
    var value_sum = vec4<f32>(0.0);
    var weight_sum = 0.0;
    for (var y = -2; y <= 2; y = y + 1) {
        for (var x = -2; x <= 2; x = x + 1) {
            let tap = pixel + vec2<i32>(x, y);
            if any(tap < vec2<i32>(0)) || any(tap >= dimensions) {
                continue;
            }
            let tap_depth = textureLoad(composite_depth, tap, 0);
            let tap_surface = textureLoad(composite_normal_material, tap, 0);
            if tap_depth <= 0.0 || dot(tap_surface.xyz, tap_surface.xyz) < 0.25 {
                continue;
            }

            let normal_agreement = smoothstep(
                0.80,
                0.98,
                dot(center_normal, normalize(tap_surface.xyz)),
            );
            let relative_depth = abs(tap_depth - center_depth)
                / max(max(tap_depth, center_depth), 0.00001);
            let depth_agreement = 1.0 - smoothstep(0.02, 0.20, relative_depth);
            let spatial_weight = 1.0 / (1.0 + f32(x * x + y * y));
            let weight = spatial_weight * normal_agreement * depth_agreement;
            value_sum = value_sum + textureLoad(indirect_radiance, tap, 0) * weight;
            weight_sum = weight_sum + weight;
        }
    }

    if weight_sum <= 0.00001 {
        return textureLoad(indirect_radiance, pixel, 0);
    }
    return value_sum / weight_sum;
}

@fragment
fn fs_composite(@builtin(position) framebuffer_position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(framebuffer_position.xy);
    let direct = textureLoad(direct_radiance, pixel, 0).rgb;
    let indirect = resolved_indirect(pixel);
    if frame.mode == MODE_AMBIENT_OCCLUSION {
        return vec4<f32>(vec3<f32>(clamp(indirect.a, 0.0, 1.0)), 1.0);
    }
    if frame.mode == MODE_GI_ONLY {
        return vec4<f32>(indirect.rgb, 1.0);
    }
    if frame.mode == MODE_RADIA {
        return vec4<f32>(direct + indirect.rgb, 1.0);
    }
    return vec4<f32>(direct, 1.0);
}
