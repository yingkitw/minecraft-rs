struct UiUniforms {
    params: vec4<f32>, // x = aspect ratio
};

@group(0) @binding(0) var<uniform> u: UiUniforms;

@vertex
fn vs_ui(@location(0) p: vec2<f32>) -> @builtin(position) vec4<f32> {
    // p is authored in "y-height" units; correct x by aspect so it's square.
    return vec4<f32>(p.x / u.params.x, p.y, 0.0, 1.0);
}

@fragment
fn fs_ui() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.65);
}
