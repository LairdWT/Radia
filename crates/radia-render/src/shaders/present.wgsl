struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@group(0) @binding(0) var accumulated_history: texture_2d<f32>;

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
fn fs_present(@builtin(position) framebuffer_position: vec4<f32>) -> @location(0) vec4<f32> {
    let linear_color = max(
        textureLoad(accumulated_history, vec2<i32>(framebuffer_position.xy), 0).rgb,
        vec3<f32>(0.0),
    );
    let tone_mapped = linear_color / (vec3<f32>(1.0) + linear_color);
    return vec4<f32>(tone_mapped, 1.0);
}

