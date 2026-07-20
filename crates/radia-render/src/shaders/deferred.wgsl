@group(0) @binding(2) var gbuffer_albedo: texture_2d<f32>;
@group(0) @binding(3) var gbuffer_normal_material: texture_2d<f32>;
@group(0) @binding(4) var gbuffer_emissive: texture_2d<f32>;
@group(0) @binding(5) var gbuffer_trace: texture_2d<f32>;
@group(0) @binding(6) var gbuffer_depth: texture_depth_2d;

@vertex
fn vs_deferred(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    return fullscreen_position(vertex_index);
}

fn trace_state_color(state: u32) -> vec3<f32> {
    if state == TRACE_HIT {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    if state == TRACE_INDETERMINATE {
        return vec3<f32>(0.8, 0.0, 0.8);
    }
    return vec3<f32>(0.0, 0.25, 1.0);
}

fn shade_deferred(
    position: vec3<f32>,
    ray_direction: vec3<f32>,
    albedo: vec3<f32>,
    normal_material: vec4<f32>,
    emissive: vec3<f32>,
    trace: vec4<f32>,
) -> vec3<f32> {
    let material = u32(round(normal_material.w));
    if frame.mode == MODE_SDF_DISTANCE {
        return vec3<f32>(exp(-abs(trace.x) * 160.0));
    }
    if frame.mode == MODE_PRIMITIVE_ID {
        let palette = array<vec3<f32>, 8>(
            vec3<f32>(0.75, 0.72, 0.68),
            vec3<f32>(1.0, 0.04, 0.02),
            vec3<f32>(0.02, 1.0, 0.08),
            vec3<f32>(0.03, 0.12, 1.0),
            vec3<f32>(0.22, 0.28, 0.38),
            vec3<f32>(0.15, 0.85, 0.85),
            vec3<f32>(0.85, 0.15, 0.85),
            vec3<f32>(0.85, 0.85, 0.15),
        );
        return palette[min(material, 7u)];
    }
    if frame.mode == MODE_NORMAL {
        return normalize(normal_material.xyz) * 0.5 + 0.5;
    }
    if frame.mode == MODE_ALBEDO {
        return clamp(albedo, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    if frame.mode == MODE_EMISSIVE {
        let nonnegative = max(emissive, vec3<f32>(0.0));
        return nonnegative / (vec3<f32>(1.0) + nonnegative);
    }
    if is_light(material) {
        return emissive;
    }

    let normal = normalize(normal_material.xyz);
    let view_direction = -ray_direction;
    return direct_lighting(material, albedo, position, normal, view_direction);
}

@fragment
fn fs_deferred(@builtin(position) framebuffer_position: vec4<f32>) -> @location(0) vec4<f32> {
    if frame.mode == MODE_TRIANGLE {
        return vec4<f32>(triangle_baseline(framebuffer_position.xy), 1.0);
    }

    let pixel = vec2<i32>(framebuffer_position.xy);
    let trace = textureLoad(gbuffer_trace, pixel, 0);
    let state = u32(round(trace.z));
    if frame.mode == MODE_HIT_STATE {
        return vec4<f32>(trace_state_color(state), 1.0);
    }
    if frame.mode == MODE_STEP_COUNT {
        let normalized_steps = clamp(trace.y / frame.trace_settings.z, 0.0, 1.0);
        return vec4<f32>(vec3<f32>(normalized_steps), 1.0);
    }
    if frame.mode == MODE_LINEAR_DEPTH {
        let normalized_depth = select(
            1.0,
            clamp(trace.w / frame.trace_settings.x, 0.0, 1.0),
            state == TRACE_HIT,
        );
        return vec4<f32>(vec3<f32>(normalized_depth), 1.0);
    }

    let depth = textureLoad(gbuffer_depth, pixel, 0);
    if depth <= 0.0 {
        let physical_mode = frame.mode == MODE_OFF
            || frame.mode == MODE_RADIA
            || frame.mode == MODE_GI_ONLY;
        let background = select(vec3<f32>(0.0), vec3<f32>(0.004, 0.007, 0.014), physical_mode);
        return vec4<f32>(background, 1.0);
    }

    let position = reconstruct_world_position(framebuffer_position.xy, depth);
    let ray_direction = normalize(position - camera_origin());
    let albedo = textureLoad(gbuffer_albedo, pixel, 0).rgb;
    let normal_material = textureLoad(gbuffer_normal_material, pixel, 0);
    let emissive = textureLoad(gbuffer_emissive, pixel, 0).rgb;
    let color = shade_deferred(
        position,
        ray_direction,
        albedo,
        normal_material,
        emissive,
        trace,
    );
    return vec4<f32>(color, 1.0);
}
