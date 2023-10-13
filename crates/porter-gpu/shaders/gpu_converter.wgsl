struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var vertices: array<vec4<f32>, 6> = array<vec4<f32>, 6>(
        vec4(-1.0, -1.0, 0.0, 1.0),
        vec4(-1.0, 1.0, 0.0, 1.0),
        vec4(1.0, -1.0, 0.0, 1.0),
        vec4(1.0, -1.0, 0.0, 1.0),
        vec4(-1.0, 1.0, 0.0, 1.0),
        vec4(1.0, 1.0, 0.0, 1.0)
    );
    var tex_coords: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 1.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
    );

    var out: VertexOutput;

    out.vert_pos = vertices[in_vertex_index];
    out.tex_coord = tex_coords[in_vertex_index];

    return out;
}

@group(0) @binding(0)
var t_input: texture_2d<f32>;
@group(0) @binding(1)
var s_input: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_input, s_input, in.tex_coord);
}

@fragment
fn fs_reconstructz_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample: vec4<f32> = textureSample(t_input, s_input, in.tex_coord);
    let reconstruct = sqrt(1.0 - saturate(dot(sample.xy, sample.xy)));
    let normalized = normalize(vec3<f32>(sample.x, sample.y, reconstruct));

    return vec4<f32>(normalized, 1.0);
}

@fragment
fn fs_reconstructzinverty_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample: vec4<f32> = textureSample(t_input, s_input, in.tex_coord);
    let reconstruct = sqrt(1.0 - saturate(dot(sample.xy, sample.xy)));
    let normalized = normalize(vec3<f32>(sample.x, sample.y, reconstruct));

    return vec4<f32>(normalized.x, 1.0 - normalized.y, normalized.z, 1.0);
}