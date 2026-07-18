struct FrameUniform {
    resolution: vec2<f32>,
    frame_index: u32,
    mode: u32,
    camera_real_xyzw: vec4<f32>,
    camera_dual_xyzw: vec4<f32>,
    emitter_position_radius: vec4<f32>,
    emitter_color_intensity: vec4<f32>,
    trace_settings: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct SceneSample {
    distance: f32,
    material: u32,
}

struct TraceHit {
    position: vec3<f32>,
    distance: f32,
    material: u32,
    steps: u32,
    state: u32,
}

@group(0) @binding(0) var<uniform> frame: FrameUniform;
@group(0) @binding(1) var previous_history: texture_2d<f32>;

const MATERIAL_FLOOR: u32 = 0u;
const MATERIAL_BOX: u32 = 1u;
const MATERIAL_SPHERE: u32 = 2u;
const MATERIAL_EMITTER: u32 = 3u;
const MATERIAL_WALL: u32 = 4u;

fn quat_mul(left: vec4<f32>, right: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        left.w * right.xyz + right.w * left.xyz + cross(left.xyz, right.xyz),
        left.w * right.w - dot(left.xyz, right.xyz),
    );
}

fn quat_conjugate(value: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(-value.xyz, value.w);
}

fn quat_rotate(rotation: vec4<f32>, direction: vec3<f32>) -> vec3<f32> {
    return quat_mul(
        quat_mul(rotation, vec4<f32>(direction, 0.0)),
        quat_conjugate(rotation),
    ).xyz;
}

fn dual_quat_translation(real: vec4<f32>, dual: vec4<f32>) -> vec3<f32> {
    return 2.0 * quat_mul(dual, quat_conjugate(real)).xyz;
}

fn rigid_inverse_point(
    world_point: vec3<f32>,
    real: vec4<f32>,
    dual: vec4<f32>,
) -> vec3<f32> {
    let translation = dual_quat_translation(real, dual);
    return quat_rotate(quat_conjugate(real), world_point - translation);
}

fn rigid_transform_point(
    local_point: vec3<f32>,
    real: vec4<f32>,
    dual: vec4<f32>,
) -> vec3<f32> {
    return quat_rotate(real, local_point) + dual_quat_translation(real, dual);
}

fn translated_dual_quat(translation: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(translation * 0.5, 0.0);
}

fn sdf_sphere(local_point: vec3<f32>, radius: f32) -> f32 {
    return length(local_point) - radius;
}

fn sdf_box(local_point: vec3<f32>, half_extent: vec3<f32>) -> f32 {
    let offset = abs(local_point) - half_extent;
    return length(max(offset, vec3<f32>(0.0))) + min(max(offset.x, max(offset.y, offset.z)), 0.0);
}

fn closer(left: SceneSample, right: SceneSample) -> SceneSample {
    if right.distance < left.distance {
        return right;
    }
    return left;
}

fn scene_distance(world_point: vec3<f32>) -> SceneSample {
    var closest = SceneSample(world_point.y + 1.0, MATERIAL_FLOOR);
    closest = closer(closest, SceneSample(world_point.z + 7.0, MATERIAL_WALL));

    let identity = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let box_local = rigid_inverse_point(
        world_point,
        identity,
        translated_dual_quat(vec3<f32>(-1.2, -0.2, -3.6)),
    );
    closest = closer(
        closest,
        SceneSample(sdf_box(box_local, vec3<f32>(0.8)), MATERIAL_BOX),
    );

    let sphere_local = rigid_inverse_point(
        world_point,
        identity,
        translated_dual_quat(vec3<f32>(1.25, 0.0, -4.6)),
    );
    closest = closer(closest, SceneSample(sdf_sphere(sphere_local, 1.0), MATERIAL_SPHERE));

    let emitter_local = rigid_inverse_point(
        world_point,
        identity,
        translated_dual_quat(frame.emitter_position_radius.xyz),
    );
    closest = closer(
        closest,
        SceneSample(
            sdf_sphere(emitter_local, frame.emitter_position_radius.w),
            MATERIAL_EMITTER,
        ),
    );
    return closest;
}

fn material_normal(material: u32, world_point: vec3<f32>) -> vec3<f32> {
    if material == MATERIAL_FLOOR {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    if material == MATERIAL_WALL {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    if material == MATERIAL_SPHERE {
        return normalize(world_point - vec3<f32>(1.25, 0.0, -4.6));
    }
    if material == MATERIAL_EMITTER {
        return normalize(world_point - frame.emitter_position_radius.xyz);
    }
    let local_point = world_point - vec3<f32>(-1.2, -0.2, -3.6);
    let boundary = abs(local_point) - vec3<f32>(0.8);
    if boundary.x >= boundary.y && boundary.x >= boundary.z {
        return vec3<f32>(select(-1.0, 1.0, local_point.x >= 0.0), 0.0, 0.0);
    }
    if boundary.y >= boundary.z {
        return vec3<f32>(0.0, select(-1.0, 1.0, local_point.y >= 0.0), 0.0);
    }
    return vec3<f32>(0.0, 0.0, select(-1.0, 1.0, local_point.z >= 0.0));
}

fn trace_scene(origin: vec3<f32>, direction: vec3<f32>) -> TraceHit {
    var traveled = 0.0;
    let maximum_steps = u32(clamp(frame.trace_settings.z, 1.0, 128.0));
    for (var step = 0u; step < 128u; step = step + 1u) {
        if step >= maximum_steps {
            return TraceHit(origin + direction * traveled, traveled, 0u, step, 2u);
        }
        let position = origin + direction * traveled;
        let sample = scene_distance(position);
        let hit_guard = frame.trace_settings.y * max(1.0, traveled);
        if abs(sample.distance) <= hit_guard {
            return TraceHit(position, sample.distance, sample.material, step + 1u, 1u);
        }
        traveled = traveled + max(sample.distance, hit_guard * 0.5);
        if traveled > frame.trace_settings.x {
            return TraceHit(position, traveled, sample.material, step + 1u, 0u);
        }
    }
    return TraceHit(origin + direction * traveled, traveled, 0u, maximum_steps, 2u);
}

fn radical_inverse(index: u32) -> f32 {
    var bits = index;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10;
}

fn hash_u32(value: u32) -> u32 {
    var mixed = value;
    mixed = (mixed ^ (mixed >> 16u)) * 2146121005u;
    mixed = (mixed ^ (mixed >> 15u)) * 2221713035u;
    return mixed ^ (mixed >> 16u);
}

fn cosine_direction(normal: vec3<f32>, sample_index: u32, pixel_key: u32) -> vec3<f32> {
    let sequence_length = 1024.0;
    let sequence_index = sample_index & 1023u;
    let first_phase = radical_inverse(hash_u32(pixel_key));
    let second_phase = radical_inverse(hash_u32(pixel_key ^ 277803737u));
    let first = fract((f32(sequence_index) + 0.5) / sequence_length + first_phase);
    let second = fract(radical_inverse(sequence_index) + second_phase);
    let radius = sqrt(first);
    let angle = 6.283185307179586 * second;
    let local = vec3<f32>(radius * cos(angle), radius * sin(angle), sqrt(max(0.0, 1.0 - first)));
    let helper = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(normal.y) < 0.999);
    let tangent = normalize(cross(helper, normal));
    let bitangent = cross(normal, tangent);
    return normalize(tangent * local.x + bitangent * local.y + normal * local.z);
}

fn material_albedo(material: u32) -> vec3<f32> {
    if material == MATERIAL_FLOOR {
        return vec3<f32>(0.72, 0.68, 0.62);
    }
    if material == MATERIAL_WALL {
        return vec3<f32>(0.52, 0.58, 0.66);
    }
    if material == MATERIAL_BOX {
        return vec3<f32>(0.18, 0.48, 0.82);
    }
    return vec3<f32>(0.68, 0.22, 0.16);
}

fn shade(hit: TraceHit, pixel_key: u32) -> vec3<f32> {
    if hit.state == 0u {
        return vec3<f32>(0.015, 0.025, 0.045);
    }
    if hit.state == 2u {
        return vec3<f32>(0.8, 0.0, 0.8);
    }
    if hit.material == MATERIAL_EMITTER {
        return frame.emitter_color_intensity.xyz * frame.emitter_color_intensity.w;
    }

    let normal = material_normal(hit.material, hit.position);
    let albedo = material_albedo(hit.material);
    let sun_direction = normalize(vec3<f32>(0.35, 0.82, 0.44));
    let direct = albedo * (0.035 + 0.75 * max(dot(normal, sun_direction), 0.0));

    let bounce_direction = cosine_direction(normal, frame.frame_index, pixel_key);
    let bounce_origin = hit.position + normal * frame.trace_settings.y * max(4.0, hit.distance);
    let bounce = trace_scene(bounce_origin, bounce_direction);
    var indirect = vec3<f32>(0.0);
    if bounce.state == 1u && bounce.material == MATERIAL_EMITTER {
        indirect = albedo * frame.emitter_color_intensity.xyz * frame.emitter_color_intensity.w;
    }

    if frame.mode == 2u {
        return indirect;
    }
    if frame.mode == 1u {
        return direct + indirect;
    }
    if frame.mode == 3u {
        return vec3<f32>(exp(-abs(hit.distance) * 200.0));
    }
    if frame.mode == 4u {
        let palette = array<vec3<f32>, 5>(
            vec3<f32>(0.8, 0.8, 0.8),
            vec3<f32>(0.1, 0.4, 1.0),
            vec3<f32>(1.0, 0.2, 0.1),
            vec3<f32>(1.0, 0.8, 0.1),
            vec3<f32>(0.2, 0.8, 0.5),
        );
        return palette[min(hit.material, 4u)];
    }
    if frame.mode == 5u {
        return normal * 0.5 + 0.5;
    }
    if frame.mode == 6u {
        return vec3<f32>(f32(hit.steps) / frame.trace_settings.z);
    }
    if frame.mode == 7u {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return direct;
}

fn edge_function(first: vec2<f32>, second: vec2<f32>, point: vec2<f32>) -> f32 {
    return (point.x - first.x) * (second.y - first.y)
        - (point.y - first.y) * (second.x - first.x);
}

fn project_world_point(world_point: vec3<f32>) -> vec2<f32> {
    let view_point = rigid_inverse_point(
        world_point,
        frame.camera_real_xyzw,
        frame.camera_dual_xyzw,
    );
    let tangent_half_fov = tan(0.5 * 1.0471975511965976);
    let aspect = frame.resolution.x / frame.resolution.y;
    let inverse_depth = 1.0 / -view_point.z;
    let ndc = vec2<f32>(
        view_point.x * inverse_depth / (tangent_half_fov * aspect),
        view_point.y * inverse_depth / tangent_half_fov,
    );
    return vec2<f32>(
        (ndc.x * 0.5 + 0.5) * frame.resolution.x,
        (0.5 - ndc.y * 0.5) * frame.resolution.y,
    );
}

fn triangle_baseline(framebuffer_position: vec2<f32>) -> vec3<f32> {
    let angle = f32(frame.frame_index) * (1.0 / 60.0);
    let model_real = vec4<f32>(0.0, sin(angle * 0.5), 0.0, cos(angle * 0.5));
    let model_dual = 0.5 * quat_mul(vec4<f32>(0.0, 0.0, -2.5, 0.0), model_real);
    let local_positions = array<vec3<f32>, 3>(
        vec3<f32>(-1.25, -0.9, 0.0),
        vec3<f32>(1.25, -0.9, 0.0),
        vec3<f32>(0.0, 1.25, 0.0),
    );
    let first = project_world_point(rigid_transform_point(local_positions[0], model_real, model_dual));
    let second = project_world_point(rigid_transform_point(local_positions[1], model_real, model_dual));
    let third = project_world_point(rigid_transform_point(local_positions[2], model_real, model_dual));
    let area = edge_function(first, second, third);
    if abs(area) <= 0.000001 {
        return vec3<f32>(0.015, 0.025, 0.045);
    }
    let first_weight = edge_function(second, third, framebuffer_position) / area;
    let second_weight = edge_function(third, first, framebuffer_position) / area;
    let third_weight = edge_function(first, second, framebuffer_position) / area;
    if first_weight >= 0.0 && second_weight >= 0.0 && third_weight >= 0.0 {
        return first_weight * vec3<f32>(1.0, 0.15, 0.08)
            + second_weight * vec3<f32>(0.08, 0.85, 0.2)
            + third_weight * vec3<f32>(0.1, 0.25, 1.0);
    }
    return vec3<f32>(0.015, 0.025, 0.045);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_accumulate(@builtin(position) framebuffer_position: vec4<f32>) -> @location(0) vec4<f32> {
    if frame.mode == 8u {
        return vec4<f32>(triangle_baseline(framebuffer_position.xy), 1.0);
    }
    let ndc_x = framebuffer_position.x * (2.0 / frame.resolution.x) - 1.0;
    let ndc_y = 1.0 - framebuffer_position.y * (2.0 / frame.resolution.y);
    let tangent_half_fov = tan(0.5 * 1.0471975511965976);
    let aspect = frame.resolution.x / frame.resolution.y;
    let camera_direction = normalize(vec3<f32>(
        ndc_x * tangent_half_fov * aspect,
        ndc_y * tangent_half_fov,
        -1.0,
    ));
    let ray_origin = dual_quat_translation(frame.camera_real_xyzw, frame.camera_dual_xyzw);
    let ray_direction = normalize(quat_rotate(frame.camera_real_xyzw, camera_direction));
    let hit = trace_scene(ray_origin, ray_direction);
    let pixel_key = u32(framebuffer_position.x) + u32(framebuffer_position.y) * u32(frame.resolution.x);
    let current = shade(hit, pixel_key);
    if frame.frame_index == 0u || frame.mode >= 3u {
        return vec4<f32>(current, 1.0);
    }
    let previous = textureLoad(previous_history, vec2<i32>(framebuffer_position.xy), 0).rgb;
    let weight = 1.0 / f32(frame.frame_index + 1u);
    return vec4<f32>(previous + (current - previous) * weight, 1.0);
}
