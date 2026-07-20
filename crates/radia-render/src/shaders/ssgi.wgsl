@group(0) @binding(2) var gather_albedo: texture_2d<f32>;
@group(0) @binding(3) var gather_normal_material: texture_2d<f32>;
@group(0) @binding(4) var gather_trace: texture_2d<f32>;
@group(0) @binding(5) var gather_depth: texture_depth_2d;
@group(0) @binding(6) var direct_radiance: texture_2d<f32>;

struct IndirectGather {
    radiance: vec3<f32>,
    accessibility: f32,
}

@vertex
fn vs_ssgi(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    return fullscreen_position(vertex_index);
}

fn valid_surface(pixel: vec2<i32>) -> bool {
    let depth = textureLoad(gather_depth, pixel, 0);
    let state = u32(round(textureLoad(gather_trace, pixel, 0).z));
    return depth > 0.0 && state == TRACE_HIT;
}

fn gather_indirect(pixel: vec2<i32>, framebuffer_position: vec2<f32>) -> IndirectGather {
    let active_mode = frame.mode == MODE_RADIA
        || frame.mode == MODE_GI_ONLY
        || frame.mode == MODE_AMBIENT_OCCLUSION;
    if !active_mode || !valid_surface(pixel) {
        return IndirectGather(vec3<f32>(0.0), 1.0);
    }

    let center_depth = textureLoad(gather_depth, pixel, 0);
    let center_position = reconstruct_world_position(framebuffer_position, center_depth);
    let center_surface = textureLoad(gather_normal_material, pixel, 0);
    if dot(center_surface.xyz, center_surface.xyz) < 0.25 {
        return IndirectGather(vec3<f32>(0.0), 1.0);
    }
    let center_normal = normalize(center_surface.xyz);
    let center_albedo = textureLoad(gather_albedo, pixel, 0).rgb;
    let dimensions = vec2<i32>(textureDimensions(gather_depth));
    let scale = max(1, i32(frame.resolution.y / 360.0));
    let offsets = array<vec2<i32>, 32>(
        vec2<i32>(13, 0),
        vec2<i32>(-13, 0),
        vec2<i32>(-16, 15),
        vec2<i32>(16, -15),
        vec2<i32>(2, -28),
        vec2<i32>(-2, 28),
        vec2<i32>(20, 27),
        vec2<i32>(-20, -27),
        vec2<i32>(-38, -7),
        vec2<i32>(38, 7),
        vec2<i32>(36, -23),
        vec2<i32>(-36, 23),
        vec2<i32>(-12, 44),
        vec2<i32>(12, -44),
        vec2<i32>(-23, -44),
        vec2<i32>(23, 44),
        vec2<i32>(49, 18),
        vec2<i32>(-49, -18),
        vec2<i32>(-51, 21),
        vec2<i32>(51, -21),
        vec2<i32>(25, -53),
        vec2<i32>(-25, 53),
        vec2<i32>(18, 58),
        vec2<i32>(-18, -58),
        vec2<i32>(-55, -32),
        vec2<i32>(55, 32),
        vec2<i32>(65, -14),
        vec2<i32>(-65, 14),
        vec2<i32>(-39, 56),
        vec2<i32>(39, -56),
        vec2<i32>(-9, -70),
        vec2<i32>(9, 70),
    );
    let phase_order = array<u32, 16>(
        0u, 8u, 2u, 10u,
        12u, 4u, 14u, 6u,
        3u, 11u, 1u, 9u,
        15u, 7u, 13u, 5u,
    );
    let phase_cell = u32(pixel.x) % 4u + (u32(pixel.y) % 4u) * 4u;
    let phase = f32(phase_order[phase_cell]) * 0.3926990817;
    let phase_cos = cos(phase);
    let phase_sin = sin(phase);

    var gathered = vec3<f32>(0.0);
    var weight_sum = 0.0;
    var occlusion_sum = 0.0;
    for (var tap_index = 0u; tap_index < 32u; tap_index = tap_index + 1u) {
        let base_offset = vec2<f32>(offsets[tap_index]);
        let phased_offset = vec2<f32>(
            phase_cos * base_offset.x - phase_sin * base_offset.y,
            phase_sin * base_offset.x + phase_cos * base_offset.y,
        );
        let tap = pixel + vec2<i32>(round(phased_offset)) * scale;
        if any(tap < vec2<i32>(0)) || any(tap >= dimensions) {
            continue;
        }
        if !valid_surface(tap) {
            continue;
        }
        let tap_surface = textureLoad(gather_normal_material, tap, 0);
        if dot(tap_surface.xyz, tap_surface.xyz) < 0.25 {
            continue;
        }
        let tap_depth = textureLoad(gather_depth, tap, 0);
        let tap_position = reconstruct_world_position(vec2<f32>(tap) + vec2<f32>(0.5), tap_depth);
        let separation = tap_position - center_position;
        let separation_squared = dot(separation, separation);
        if separation_squared <= 0.0025 || separation_squared >= 16.0 {
            continue;
        }
        let distance = sqrt(separation_squared);
        let direction = separation / distance;
        let receiver_facing = max(dot(center_normal, direction), 0.0);
        let range_weight = 1.0 - smoothstep(0.05, 2.5, distance);
        occlusion_sum = occlusion_sum + receiver_facing * range_weight;

        let tap_material = u32(round(tap_surface.w));
        if is_light(tap_material) || is_dragon(tap_material) {
            continue;
        }
        let source_facing = max(dot(normalize(tap_surface.xyz), -direction), 0.0);
        let weight = receiver_facing * source_facing / (0.2 + separation_squared);
        if weight <= 0.0001 {
            continue;
        }
        gathered = gathered + textureLoad(direct_radiance, tap, 0).rgb * weight;
        weight_sum = weight_sum + weight;
    }

    let accessibility = 1.0 - clamp(occlusion_sum * 0.125, 0.0, 1.0);
    if weight_sum <= 0.0001 {
        return IndirectGather(vec3<f32>(0.0), accessibility);
    }
    return IndirectGather(center_albedo * gathered * (0.55 / weight_sum), accessibility);
}

@fragment
fn fs_ssgi(@builtin(position) framebuffer_position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(framebuffer_position.xy);
    let gathered = gather_indirect(pixel, framebuffer_position.xy);
    return vec4<f32>(gathered.radiance, gathered.accessibility);
}
