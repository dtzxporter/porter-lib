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
        vec2(1.0, 0.0)
    );

    var out: VertexOutput;

    out.vert_pos = vertices[in_vertex_index];
    out.tex_coord = tex_coords[in_vertex_index];

    return out;
}

struct OptionsUniform {
    input_unorm: u32,
    input_snorm: u32,
    output_unorm: u32,
    output_snorm: u32,
    invert_y: u32,
    scale: f32,
    bias: f32,
}

@group(0) @binding(0)
var<uniform> options: OptionsUniform;

@group(0) @binding(1)
var t_input: texture_2d<f32>;
@group(0) @binding(2)
var s_input: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample: vec4<f32> = textureSample(t_input, s_input, in.tex_coord).xyzw;

    if options.input_unorm == 1u && options.output_snorm == 1u {
        sample = sample * 2.0 - 1.0;
    } else if options.input_snorm == 1u && options.output_unorm == 1u {
        sample = sample * 0.5 + 0.5;
    }

    return sample;
}

@fragment
fn fs_rz_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample: vec2<f32> = textureSample(t_input, s_input, in.tex_coord).xy;

    if options.input_unorm == 1u {
        sample = sample * 2.0 - 1.0;
    }

    let reconstruct = sqrt(1.0 - saturate(dot(sample.xy, sample.xy)));
    var normalized = normalize(vec3<f32>(sample.x, sample.y, reconstruct));

    if options.output_unorm == 1u {
        normalized = normalized * 0.5 + 0.5;
    }

    if options.invert_y == 1u {
        if options.output_unorm == 1u {
            return vec4<f32>(normalized.x, 1.0 - normalized.y, normalized.z, 1.0);
        } else {
            return vec4<f32>(normalized.x, -normalized.y, normalized.z, 1.0);
        }
    } else {
        return vec4<f32>(normalized, 1.0);
    }
}

@fragment
fn fs_sb_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample: vec4<f32> = textureSample(t_input, s_input, in.tex_coord).xyzw;

    return vec4<f32>(fma(sample.xyz, vec3<f32>(options.scale), vec3<f32>(options.bias)), sample.w);
}