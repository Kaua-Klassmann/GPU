struct VOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
}

const POSITION = array<vec2<f32>, 3> (
    vec2<f32>(0.0, 0.7),
    vec2<f32>(-0.7, -0.7),
    vec2<f32>(0.7, -0.7),
);
const COLOR = array<vec3<f32>, 3>(
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VOutput {
    var out: VOutput;
    out.position = vec4<f32>(POSITION[in_vertex_index], 0.0, 1.0);
    out.v_color = vec4<f32>(COLOR[in_vertex_index], 1.0);

    return out;
}

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4<f32> {
    return in.v_color;
}