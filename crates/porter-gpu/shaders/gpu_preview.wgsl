struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct GridInput {
    @builtin(vertex_index) in_vertex_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct GridOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct BoneInput {
    @location(0) position: vec3<f32>,
}

struct BoneOutput {
    @builtin(position) position: vec4<f32>,
}

struct ImageInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct ImageOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CameraUniform {
    target_c: vec3<f32>,
    view_matrix: mat4x4<f32>,
    inverse_view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    inverse_model_matrix: mat4x4<f32>,
    default_shaded: u32,
    srgb: u32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_albedo: texture_2d<f32>;
@group(1) @binding(1)
var s_albedo: sampler;

fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let less = linear * 12.92;
    let more = pow(max(linear, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) * 1.055 - vec3<f32>(0.055);
    return select(less, more, linear > vec3<f32>(0.0031308));
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let mvp: mat4x4<f32> = camera.projection_matrix * camera.view_matrix * camera.model_matrix;

    var out: VertexOutput;

    out.position = mvp * vec4<f32>(in.position, 1.0);
    out.frag_position = vec3<f32>((camera.model_matrix * vec4<f32>(in.position, 1.0)).xyz);
    out.normal = (transpose(camera.inverse_model_matrix) * vec4<f32>(in.normal, 0.0)).xyz;
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return fs_main_full(in, true);
}

@fragment
fn fs_main_nocull(in: VertexOutput) -> @location(0) vec4<f32> {
    return fs_main_full(in, false);
}

fn fs_main_full(in: VertexOutput, culling: bool) -> vec4<f32> {
    let ambient_strength: f32 = 0.1;
    let ambient: vec3<f32> = ambient_strength * vec3<f32>(1.0, 1.0, 1.0);

    let normal: vec3<f32> = normalize(in.normal);
    let light_dir: vec3<f32> = normalize(camera.inverse_view_matrix[3].xyz - in.frag_position);

    var diff: f32;

    if culling {
        diff = max(dot(normal, light_dir), 0.0);
    } else {
        diff = max(abs(dot(normal, light_dir)), 0.0);
    }

    let diffuse: vec3<f32> = diff * vec3<f32>(1.0, 1.0, 1.0);

    if camera.default_shaded == 1u {
        return vec4<f32>((ambient + diffuse) * vec3<f32>(0.603, 0.603, 0.603), 1.0);
    } else if camera.srgb == 1u {
        return vec4<f32>((ambient + diffuse) * linear_to_srgb(textureSample(t_albedo, s_albedo, in.uv).xyz), 1.0);
    } else {
        return vec4<f32>((ambient + diffuse) * textureSample(t_albedo, s_albedo, in.uv).xyz, 1.0);
    }
}

@vertex
fn vs_grid_main(in: GridInput) -> GridOutput {
    let vp: mat4x4<f32> = camera.projection_matrix * camera.view_matrix;

    var out: GridOutput;

    out.position = vp * vec4<f32>(in.position, 1.0);
    out.color = vec4<f32>(in.color, 1.0);

    return out;
}

@fragment
fn fs_grid_main(in: GridOutput) -> @location(0) vec4<f32> {
    return in.color;
}

@vertex
fn vs_bone_main(in: BoneInput) -> BoneOutput {
    let mvp: mat4x4<f32> = camera.projection_matrix * camera.view_matrix * camera.model_matrix;

    var out: BoneOutput;

    out.position = mvp * vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn fs_bone_main(in: BoneOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.153, 0.608, 0.831, 1.0);
}

@vertex
fn vs_image_main(in: ImageInput) -> ImageOutput {
    let vp: mat4x4<f32> = camera.projection_matrix * camera.view_matrix;

    var out: ImageOutput;

    out.position = vp * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_image_main(in: ImageOutput) -> @location(0) vec4<f32> {
    let sample: vec4<f32> = textureSample(t_albedo, s_albedo, in.uv);

    if camera.srgb == 1u {
        return vec4<f32>(linear_to_srgb(sample.xyz), sample.w);
    } else {
        return sample;
    }
}

@fragment
fn fs_image_grayscale(in: ImageOutput) -> @location(0) vec4<f32> {
    let sample: vec4<f32> = textureSample(t_albedo, s_albedo, in.uv);

    if camera.srgb == 1u {
        return vec4<f32>(linear_to_srgb(sample.xxx), 1.0);
    } else {
        return vec4<f32>(sample.xxx, 1.0);
    }
}