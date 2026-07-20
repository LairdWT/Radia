struct FrameUniform {
    resolution: vec2<f32>,
    frame_index: u32,
    mode: u32,
    camera_real_xyzw: vec4<f32>,
    camera_dual_xyzw: vec4<f32>,
    light_position_radius: array<vec4<f32>, 3>,
    light_color_intensity: array<vec4<f32>, 3>,
    trace_settings: vec4<f32>,
    dragon_minimum_error: vec4<f32>,
    dragon_maximum_resolution: vec4<f32>,
    projection: vec4<f32>,
    dragon_real_xyzw: array<vec4<f32>, 3>,
    dragon_dual_xyzw: array<vec4<f32>, 3>,
}

struct DragonField {
    samples: array<f32>,
}

struct SceneSample {
    distance: f32,
    material: u32,
    dragon_index: u32,
}

struct TraceHit {
    position: vec3<f32>,
    field_distance: f32,
    material: u32,
    dragon_index: u32,
    steps: u32,
    state: u32,
}

@group(0) @binding(0) var<uniform> frame: FrameUniform;
@group(0) @binding(1) var<storage, read> dragon_field: DragonField;

const MATERIAL_FLOOR: u32 = 0u;
const MATERIAL_PRIMARY_LIGHT: u32 = 1u;
const MATERIAL_GREEN_LIGHT: u32 = 2u;
const MATERIAL_BLUE_LIGHT: u32 = 3u;
const MATERIAL_WALL: u32 = 4u;
const MATERIAL_DRAGON_CYAN: u32 = 5u;
const MATERIAL_DRAGON_MAGENTA: u32 = 6u;
const MATERIAL_DRAGON_YELLOW: u32 = 7u;
const PI: f32 = 3.14159265358979323846;
const F32_EPSILON: f32 = 0.00000011920928955078125;
const BRDF_DENOMINATOR_GUARD: f32 = 32.0 * F32_EPSILON;
const TRACE_MISS: u32 = 0u;
const TRACE_HIT: u32 = 1u;
const TRACE_INDETERMINATE: u32 = 2u;
const NO_DRAGON: u32 = 0xffffffffu;
const MODE_OFF: u32 = 0u;
const MODE_RADIA: u32 = 1u;
const MODE_GI_ONLY: u32 = 2u;
const MODE_SDF_DISTANCE: u32 = 3u;
const MODE_PRIMITIVE_ID: u32 = 4u;
const MODE_NORMAL: u32 = 5u;
const MODE_STEP_COUNT: u32 = 6u;
const MODE_HIT_STATE: u32 = 7u;
const MODE_TRIANGLE: u32 = 8u;
const MODE_ALBEDO: u32 = 9u;
const MODE_EMISSIVE: u32 = 10u;
const MODE_LINEAR_DEPTH: u32 = 11u;
const MODE_AMBIENT_OCCLUSION: u32 = 12u;

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

fn dragon_real(index: u32) -> vec4<f32> {
    return frame.dragon_real_xyzw[index];
}

fn dragon_dual(index: u32) -> vec4<f32> {
    return frame.dragon_dual_xyzw[index];
}

fn light_position_radius(index: u32) -> vec4<f32> {
    return frame.light_position_radius[index];
}

fn light_color_intensity(index: u32) -> vec4<f32> {
    return frame.light_color_intensity[index];
}

fn light_material(index: u32) -> u32 {
    if index == 0u {
        return MATERIAL_PRIMARY_LIGHT;
    }
    if index == 1u {
        return MATERIAL_GREEN_LIGHT;
    }
    return MATERIAL_BLUE_LIGHT;
}

fn material_light_index(material: u32) -> u32 {
    if material == MATERIAL_PRIMARY_LIGHT {
        return 0u;
    }
    if material == MATERIAL_GREEN_LIGHT {
        return 1u;
    }
    return 2u;
}

fn is_light(material: u32) -> bool {
    return material == MATERIAL_PRIMARY_LIGHT
        || material == MATERIAL_GREEN_LIGHT
        || material == MATERIAL_BLUE_LIGHT;
}

fn dragon_material(index: u32) -> u32 {
    if index == 0u {
        return MATERIAL_DRAGON_CYAN;
    }
    if index == 1u {
        return MATERIAL_DRAGON_MAGENTA;
    }
    return MATERIAL_DRAGON_YELLOW;
}

fn is_dragon(material: u32) -> bool {
    return material == MATERIAL_DRAGON_CYAN
        || material == MATERIAL_DRAGON_MAGENTA
        || material == MATERIAL_DRAGON_YELLOW;
}

fn sdf_sphere(local_point: vec3<f32>, radius: f32) -> f32 {
    return length(local_point) - radius;
}

fn dragon_sample_index(coordinate: vec3<u32>, resolution: u32) -> u32 {
    return coordinate.x + resolution * (coordinate.y + resolution * coordinate.z);
}

fn dragon_udf_raw(local_point: vec3<f32>) -> f32 {
    let minimum = frame.dragon_minimum_error.xyz;
    let maximum = frame.dragon_maximum_resolution.xyz;
    let resolution = u32(frame.dragon_maximum_resolution.w);
    let clamped_point = clamp(local_point, minimum, maximum);
    let outside_distance = length(local_point - clamped_point);

    let grid = (clamped_point - minimum) / (maximum - minimum) * f32(resolution - 1u);
    let lower = vec3<u32>(floor(grid));
    let upper = min(lower + vec3<u32>(1u), vec3<u32>(resolution - 1u));
    let blend = fract(grid);
    let s000 = dragon_field.samples[dragon_sample_index(lower, resolution)];
    let s100 = dragon_field.samples[dragon_sample_index(vec3<u32>(upper.x, lower.y, lower.z), resolution)];
    let s010 = dragon_field.samples[dragon_sample_index(vec3<u32>(lower.x, upper.y, lower.z), resolution)];
    let s110 = dragon_field.samples[dragon_sample_index(vec3<u32>(upper.x, upper.y, lower.z), resolution)];
    let s001 = dragon_field.samples[dragon_sample_index(vec3<u32>(lower.x, lower.y, upper.z), resolution)];
    let s101 = dragon_field.samples[dragon_sample_index(vec3<u32>(upper.x, lower.y, upper.z), resolution)];
    let s011 = dragon_field.samples[dragon_sample_index(vec3<u32>(lower.x, upper.y, upper.z), resolution)];
    let s111 = dragon_field.samples[dragon_sample_index(upper, resolution)];
    let low_z = mix(mix(s000, s100, blend.x), mix(s010, s110, blend.x), blend.y);
    let high_z = mix(mix(s001, s101, blend.x), mix(s011, s111, blend.x), blend.y);
    let sampled_distance = mix(low_z, high_z, blend.z);
    if outside_distance > 0.0 {
        return max(outside_distance, sampled_distance - outside_distance);
    }
    return sampled_distance;
}

fn dragon_safe_distance(world_point: vec3<f32>, dragon_index: u32) -> f32 {
    let local_point = rigid_inverse_point(
        world_point,
        dragon_real(dragon_index),
        dragon_dual(dragon_index),
    );
    return max(dragon_udf_raw(local_point) - frame.dragon_minimum_error.w, 0.0);
}

fn closer(left: SceneSample, right: SceneSample) -> SceneSample {
    if right.distance < left.distance {
        return right;
    }
    return left;
}

fn scene_distance(world_point: vec3<f32>) -> SceneSample {
    var closest = SceneSample(world_point.y + 1.0, MATERIAL_FLOOR, NO_DRAGON);
    closest = closer(closest, SceneSample(world_point.z + 8.0, MATERIAL_WALL, NO_DRAGON));
    for (var dragon_index = 0u; dragon_index < 3u; dragon_index = dragon_index + 1u) {
        closest = closer(
            closest,
            SceneSample(
                dragon_safe_distance(world_point, dragon_index),
                dragon_material(dragon_index),
                dragon_index,
            ),
        );
    }
    for (var light_index = 0u; light_index < 3u; light_index = light_index + 1u) {
        let light = light_position_radius(light_index);
        closest = closer(
            closest,
            SceneSample(
                sdf_sphere(world_point - light.xyz, light.w),
                light_material(light_index),
                NO_DRAGON,
            ),
        );
    }
    return closest;
}

fn dragon_normal(world_point: vec3<f32>, dragon_index: u32) -> vec3<f32> {
    let rotation = dragon_real(dragon_index);
    let local_point = rigid_inverse_point(world_point, rotation, dragon_dual(dragon_index));
    let cell = (frame.dragon_maximum_resolution.xyz - frame.dragon_minimum_error.xyz)
        / (frame.dragon_maximum_resolution.w - 1.0);
    let delta = max(max(cell.x, cell.y), cell.z);
    let gradient = vec3<f32>(
        dragon_udf_raw(local_point + vec3<f32>(delta, 0.0, 0.0))
            - dragon_udf_raw(local_point - vec3<f32>(delta, 0.0, 0.0)),
        dragon_udf_raw(local_point + vec3<f32>(0.0, delta, 0.0))
            - dragon_udf_raw(local_point - vec3<f32>(0.0, delta, 0.0)),
        dragon_udf_raw(local_point + vec3<f32>(0.0, 0.0, delta))
            - dragon_udf_raw(local_point - vec3<f32>(0.0, 0.0, delta)),
    );
    if dot(gradient, gradient) <= frame.dragon_minimum_error.w * frame.dragon_minimum_error.w {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return normalize(quat_rotate(rotation, normalize(gradient)));
}

fn material_normal(material: u32, dragon_index: u32, world_point: vec3<f32>) -> vec3<f32> {
    if material == MATERIAL_FLOOR {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    if material == MATERIAL_WALL {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    if is_dragon(material) {
        return dragon_normal(world_point, dragon_index);
    }
    let light = light_position_radius(material_light_index(material));
    return normalize(world_point - light.xyz);
}

fn trace_scene(origin: vec3<f32>, direction: vec3<f32>) -> TraceHit {
    var traveled = 0.0;
    let maximum_steps = u32(clamp(frame.trace_settings.z, 1.0, 128.0));
    for (var step = 0u; step < 128u; step = step + 1u) {
        if step >= maximum_steps {
            return TraceHit(
                origin + direction * traveled,
                traveled,
                0u,
                NO_DRAGON,
                step,
                TRACE_INDETERMINATE,
            );
        }
        let position = origin + direction * traveled;
        let sample = scene_distance(position);
        let hit_guard = frame.trace_settings.y * max(1.0, traveled);
        if abs(sample.distance) <= hit_guard {
            return TraceHit(
                position,
                sample.distance,
                sample.material,
                sample.dragon_index,
                step + 1u,
                TRACE_HIT,
            );
        }
        traveled = traveled + max(sample.distance * 0.82, hit_guard * 0.5);
        if traveled > frame.trace_settings.x {
            return TraceHit(
                position,
                traveled,
                sample.material,
                sample.dragon_index,
                step + 1u,
                TRACE_MISS,
            );
        }
    }
    return TraceHit(
        origin + direction * traveled,
        traveled,
        0u,
        NO_DRAGON,
        maximum_steps,
        TRACE_INDETERMINATE,
    );
}

fn light_visible(
    position: vec3<f32>,
    normal: vec3<f32>,
    light_index: u32,
) -> bool {
    let light = light_position_radius(light_index);
    let to_light = light.xyz - position;
    let center_distance = length(to_light);
    if center_distance <= light.w {
        return true;
    }
    let direction = to_light / center_distance;
    let offset = max(
        frame.trace_settings.y * 8.0,
        frame.dragon_minimum_error.w * 2.0,
    );
    var traveled = offset;
    let maximum_distance = center_distance - light.w;
    let origin = position + normal * offset;
    for (var step = 0u; step < 96u; step = step + 1u) {
        if traveled >= maximum_distance {
            return true;
        }
        let sample = scene_distance(origin + direction * traveled);
        let guard = frame.trace_settings.y * max(1.0, traveled);
        if abs(sample.distance) <= guard {
            return is_light(sample.material);
        }
        traveled = traveled + max(sample.distance * 0.82, guard * 0.5);
    }
    return true;
}

fn material_albedo(material: u32) -> vec3<f32> {
    if material == MATERIAL_FLOOR {
        return vec3<f32>(0.52, 0.46, 0.38);
    }
    if material == MATERIAL_WALL {
        return vec3<f32>(0.24, 0.28, 0.34);
    }
    if material == MATERIAL_DRAGON_CYAN {
        return vec3<f32>(0.2925, 0.65, 0.65);
    }
    if material == MATERIAL_DRAGON_MAGENTA {
        return vec3<f32>(0.65, 0.2925, 0.65);
    }
    return vec3<f32>(0.65, 0.65, 0.2925);
}

fn material_roughness(material: u32) -> f32 {
    if material == MATERIAL_DRAGON_CYAN {
        return 0.2;
    }
    if material == MATERIAL_DRAGON_MAGENTA {
        return 0.65;
    }
    if material == MATERIAL_DRAGON_YELLOW {
        return 0.9;
    }
    if material == MATERIAL_FLOOR {
        return 0.85;
    }
    return 0.95;
}

fn material_metallic(material: u32) -> f32 {
    if material == MATERIAL_DRAGON_CYAN {
        return 1.0;
    }
    return 0.0;
}

fn emitted_radiance(material: u32) -> vec3<f32> {
    let light = light_color_intensity(material_light_index(material));
    return light.xyz * light.w;
}

fn fresnel_schlick(cosine: f32, reflectance_at_normal: vec3<f32>) -> vec3<f32> {
    let one_minus_cosine = 1.0 - clamp(cosine, 0.0, 1.0);
    let squared = one_minus_cosine * one_minus_cosine;
    let fifth_power = squared * squared * one_minus_cosine;
    return reflectance_at_normal
        + (vec3<f32>(1.0) - reflectance_at_normal) * fifth_power;
}

fn distribution_ggx(cosine_normal_halfway: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let cosine_squared = cosine_normal_halfway * cosine_normal_halfway;
    let denominator_term = cosine_squared * (alpha_squared - 1.0) + 1.0;
    let denominator = PI * denominator_term * denominator_term;
    return alpha_squared / max(denominator, BRDF_DENOMINATOR_GUARD);
}

fn geometry_schlick_ggx(cosine: f32, roughness: f32) -> f32 {
    let remapped = roughness + 1.0;
    let k = remapped * remapped * 0.125;
    let denominator = cosine * (1.0 - k) + k;
    return cosine / max(denominator, BRDF_DENOMINATOR_GUARD);
}

fn geometry_smith(cosine_normal_view: f32, cosine_normal_light: f32, roughness: f32) -> f32 {
    return geometry_schlick_ggx(cosine_normal_view, roughness)
        * geometry_schlick_ggx(cosine_normal_light, roughness);
}

fn direct_brdf(
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
) -> vec3<f32> {
    let cosine_normal_light = max(dot(normal, light_direction), 0.0);
    let cosine_normal_view = max(dot(normal, view_direction), 0.0);
    if cosine_normal_light <= 0.0 || cosine_normal_view <= 0.0 {
        return vec3<f32>(0.0);
    }

    let reflectance_at_normal = mix(vec3<f32>(0.04), albedo, vec3<f32>(metallic));
    let halfway_sum = light_direction + view_direction;
    let halfway_length_squared = dot(halfway_sum, halfway_sum);
    if halfway_length_squared <= BRDF_DENOMINATOR_GUARD {
        return (vec3<f32>(1.0) - reflectance_at_normal)
            * (1.0 - metallic) * albedo * (cosine_normal_light / PI);
    }
    let halfway = halfway_sum * inverseSqrt(halfway_length_squared);
    let cosine_normal_halfway = max(dot(normal, halfway), 0.0);
    let cosine_view_halfway = max(dot(view_direction, halfway), 0.0);
    let fresnel = fresnel_schlick(cosine_view_halfway, reflectance_at_normal);
    let distribution = distribution_ggx(cosine_normal_halfway, roughness);
    let geometry = geometry_smith(cosine_normal_view, cosine_normal_light, roughness);
    let specular_denominator = max(
        4.0 * cosine_normal_view * cosine_normal_light,
        BRDF_DENOMINATOR_GUARD,
    );
    let specular = fresnel * (distribution * geometry / specular_denominator);
    let diffuse = (vec3<f32>(1.0) - fresnel) * (1.0 - metallic) * albedo / PI;
    return (diffuse + specular) * cosine_normal_light;
}

fn environment_lighting(
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    normal: vec3<f32>,
    view_direction: vec3<f32>,
) -> vec3<f32> {
    let sky_irradiance = vec3<f32>(0.12, 0.17, 0.25);
    let ground_irradiance = vec3<f32>(0.18, 0.10, 0.07);
    let sky_weight = 0.35 + 0.65 * max(normal.y, 0.0);
    let ground_weight = 0.25 + 0.75 * max(-normal.y, 0.0);
    let irradiance = sky_irradiance * sky_weight + ground_irradiance * ground_weight;
    let reflectance_at_normal = mix(vec3<f32>(0.04), albedo, vec3<f32>(metallic));
    let fresnel = fresnel_schlick(max(dot(normal, view_direction), 0.0), reflectance_at_normal);
    let diffuse = (vec3<f32>(1.0) - fresnel) * (1.0 - metallic) * albedo * irradiance / PI;
    let specular_weight = 0.15 + 0.35 * (1.0 - roughness);
    return diffuse + fresnel * irradiance * specular_weight;
}

fn direct_lighting(
    material: u32,
    albedo: vec3<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
    view_direction: vec3<f32>,
) -> vec3<f32> {
    let roughness = material_roughness(material);
    let metallic = material_metallic(material);
    var radiance = environment_lighting(
        albedo,
        metallic,
        roughness,
        normal,
        view_direction,
    );
    for (var light_index = 0u; light_index < 3u; light_index = light_index + 1u) {
        let light = light_position_radius(light_index);
        let to_light = light.xyz - position;
        let distance_squared = max(dot(to_light, to_light), light.w * light.w);
        let light_direction = normalize(to_light);
        let cosine_normal_light = max(dot(normal, light_direction), 0.0);
        if cosine_normal_light > 0.0 && light_visible(position, normal, light_index) {
            let color = light_color_intensity(light_index);
            let irradiance = color.xyz * color.w / (1.0 + distance_squared);
            radiance = radiance + irradiance * direct_brdf(
                albedo,
                metallic,
                roughness,
                normal,
                view_direction,
                light_direction,
            );
        }
    }
    return radiance;
}

fn camera_direction(framebuffer_position: vec2<f32>) -> vec3<f32> {
    let ndc_x = framebuffer_position.x * (2.0 / frame.resolution.x) - 1.0;
    let ndc_y = 1.0 - framebuffer_position.y * (2.0 / frame.resolution.y);
    let aspect = frame.resolution.x / frame.resolution.y;
    return normalize(vec3<f32>(
        ndc_x * frame.projection.x * aspect,
        ndc_y * frame.projection.x,
        -1.0,
    ));
}

fn camera_origin() -> vec3<f32> {
    return dual_quat_translation(frame.camera_real_xyzw, frame.camera_dual_xyzw);
}

fn world_ray_direction(framebuffer_position: vec2<f32>) -> vec3<f32> {
    return normalize(quat_rotate(frame.camera_real_xyzw, camera_direction(framebuffer_position)));
}

fn reverse_z_depth(world_point: vec3<f32>) -> f32 {
    let view_point = rigid_inverse_point(
        world_point,
        frame.camera_real_xyzw,
        frame.camera_dual_xyzw,
    );
    if view_point.z >= -frame.projection.y {
        return 1.0;
    }
    return clamp(frame.projection.y / -view_point.z, 0.0, 1.0);
}

fn reconstruct_world_position(framebuffer_position: vec2<f32>, depth: f32) -> vec3<f32> {
    let camera_ray = camera_direction(framebuffer_position);
    let camera_distance = (-frame.projection.y / depth) / camera_ray.z;
    return camera_origin() + world_ray_direction(framebuffer_position) * camera_distance;
}

fn edge_function(first: vec2<f32>, second: vec2<f32>, point: vec2<f32>) -> f32 {
    return (point.x - first.x) * (second.y - first.y)
        - (point.y - first.y) * (second.x - first.x);
}

fn project_world_point(world_point: vec3<f32>) -> vec2<f32> {
    let view_point = rigid_inverse_point(world_point, frame.camera_real_xyzw, frame.camera_dual_xyzw);
    let aspect = frame.resolution.x / frame.resolution.y;
    let inverse_depth = 1.0 / -view_point.z;
    let ndc = vec2<f32>(
        view_point.x * inverse_depth / (frame.projection.x * aspect),
        view_point.y * inverse_depth / frame.projection.x,
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
        return vec3<f32>(0.004, 0.007, 0.014);
    }
    let first_weight = edge_function(second, third, framebuffer_position) / area;
    let second_weight = edge_function(third, first, framebuffer_position) / area;
    let third_weight = edge_function(first, second, framebuffer_position) / area;
    if first_weight >= 0.0 && second_weight >= 0.0 && third_weight >= 0.0 {
        return first_weight * vec3<f32>(1.0, 0.15, 0.08)
            + second_weight * vec3<f32>(0.08, 0.85, 0.2)
            + third_weight * vec3<f32>(0.1, 0.25, 1.0);
    }
    return vec3<f32>(0.004, 0.007, 0.014);
}

fn fullscreen_position(vertex_index: u32) -> vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}
