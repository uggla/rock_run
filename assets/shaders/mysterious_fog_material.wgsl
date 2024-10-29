#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import "shader_utils.wgsl"::NEG_HALF_PI
#import "shader_utils.wgsl"::shader_toy_default
#import "shader_utils.wgsl"::rotate2D
#import "shader_utils.wgsl"::TAU
#import "shader_utils.wgsl"::PI
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

const OCTAVES: f32 = 8.0;

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn rand2(co: vec2<f32>) -> f32 {
    return fract(cos(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// Rough Value noise implementation
fn valueNoiseSimple(vl: vec2<f32>) -> f32 {
    let minStep: f32 = 1.0;

    let grid = floor(vl);
    let gridPnt1 = grid;
    let gridPnt2 = vec2<f32>(grid.x, grid.y + minStep);
    let gridPnt3 = vec2<f32>(grid.x + minStep, grid.y);
    let gridPnt4 = vec2<f32>(gridPnt3.x, gridPnt2.y);

    let s = rand2(gridPnt1);
    let t = rand2(gridPnt3);
    let u = rand2(gridPnt2);
    let v = rand2(gridPnt4);

    let x1 = smoothstep(0.0, 1.0, fract(vl.x));
    let interpX1 = mix(s, t, x1);
    let interpX2 = mix(u, v, x1);

    let y = smoothstep(0.0, 1.0, fract(vl.y));
    let interpY = mix(interpX1, interpX2, y);

    return interpY;
}

fn fractalNoise(vl: vec2<f32>) -> f32 {
    var persistance: f32 = 2.0;
    var amplitude: f32 = 0.5;
    var rez: f32 = 0.0;
    var p = vl;

    for (var i: f32 = 0.0; i < OCTAVES; i = i + 1.0) {
        rez = rez + amplitude * valueNoiseSimple(p);
        amplitude = amplitude / persistance;
        p = p * persistance;
    }
    return rez;
}

fn complexFBM(p: vec2<f32>, time: f32) -> f32 {
    let slow = time / 4.5;
    let fast = time / 3.5;
    let offset1 = vec2<f32>(slow, 0.0);
    let offset2 = vec2<f32>(sin(fast) * 0.1, 0.0);

    return fractalNoise(
        p + offset1 + fractalNoise(
            p + fractalNoise(
                p + 2.0 * fractalNoise(p - offset2)
            )
        )
    );
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time: f32 = globals.time;
    var uv: vec2<f32> = in.uv;
    // uv = uv * 2.0 - 1.0; // center the uv coordinates
    var resolution: vec2<f32> = view.viewport.zw; // z is the width, w is the height

    // Define colors
    let color1 = vec3<f32>(0.00, 0.00, 0.00);
    let color2 = vec3<f32>(0.32, 0.0, 0.45);

    let weight = complexFBM(uv, time) * 1.8; // Adjust multiplier to bias towards color1
    var rez = mix(color2, color1, weight);
    rez = clamp(rez, vec3<f32>(0.017, 0.017, 0.018), vec3<f32>(1.0));

    return vec4<f32>(rez, 1.0);
}
