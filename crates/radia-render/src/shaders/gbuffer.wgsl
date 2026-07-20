struct GBufferOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) normal_material: vec4<f32>,
    @location(2) emissive: vec4<f32>,
    @location(3) trace: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_gbuffer(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    return fullscreen_position(vertex_index);
}

@fragment
fn fs_gbuffer(@builtin(position) framebuffer_position: vec4<f32>) -> GBufferOutput {
    let origin = camera_origin();
    let direction = world_ray_direction(framebuffer_position.xy);
    let hit = trace_scene(origin, direction);

    var output: GBufferOutput;
    output.albedo = vec4<f32>(0.0);
    output.normal_material = vec4<f32>(0.0);
    output.emissive = vec4<f32>(0.0);
    output.trace = vec4<f32>(
        hit.field_distance,
        f32(hit.steps),
        f32(hit.state),
        length(hit.position - origin),
    );
    output.depth = 0.0;

    if hit.state != TRACE_HIT {
        let diagnostic_trace_mode = frame.mode == MODE_STEP_COUNT
            || frame.mode == MODE_HIT_STATE;
        if diagnostic_trace_mode {
            output.depth = 1.0;
        }
        return output;
    }

    output.depth = reverse_z_depth(hit.position);
    let normal = material_normal(hit.material, hit.dragon_index, hit.position);
    output.albedo = vec4<f32>(material_albedo(hit.material), 1.0);
    output.normal_material = vec4<f32>(normal, f32(hit.material));
    if is_light(hit.material) {
        output.emissive = vec4<f32>(emitted_radiance(hit.material), 1.0);
    }
    return output;
}
