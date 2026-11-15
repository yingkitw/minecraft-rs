struct Uniforms {
    mvp: mat4x4<f32>,
    cam_pos: vec4<f32>,
    fog_color: vec4<f32>,
    fog: vec4<f32>, // x = start distance, y = end distance
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) shade: f32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) shade: f32,
    @location(2) dist: f32,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = u.mvp * vec4<f32>(in.pos, 1.0);
    out.uv = in.uv;
    out.shade = in.shade;
    out.dist = distance(in.pos, u.cam_pos.xyz);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tex = textureSample(atlas, atlas_samp, in.uv).rgb;
    let lit = tex * in.shade;
    let f = clamp((in.dist - u.fog.x) / (u.fog.y - u.fog.x), 0.0, 1.0);
    return vec4<f32>(mix(lit, u.fog_color.rgb, f), 1.0);
}
