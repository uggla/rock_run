#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

@group(2) @binding(0) var<uniform> color: vec4<f32>;

const _SnowflakeAmount: i32 = 200;
const _BlizardFactor: f32 = 0.2;

fn rnd(x: f32) -> f32 {
    return fract(sin(dot(vec2<f32>(x + 47.49, 38.2467 / (x + 2.3)), vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn drawCircle(uv: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return 1.0 - smoothstep(0.0, radius, length(uv - center));
}

fn modulo(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time: f32 = globals.time;
    var uv: vec2<f32> = in.uv;

    uv.y = 1.0 - uv.y;

    var fragColor = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var j: f32;

    for (var i: i32 = 0; i < _SnowflakeAmount; i++) {
        j = f32(i);
        let speed = 0.3 + rnd(cos(j)) * (0.7 + 0.5 * cos(j / (f32(_SnowflakeAmount) * 0.25)));
        let center = vec2<f32>(
            (0.25 - (1.0 - uv.y)) * _BlizardFactor + rnd(j) + 0.1 * cos(time + sin(j)),
            modulo(sin(j) - speed * (time * 1.5 * (0.1 + _BlizardFactor)), 1.0)
        );
        let snowflake = drawCircle(uv, center, 0.0005 + speed * 0.006);
        fragColor += vec4<f32>(snowflake, snowflake, snowflake, snowflake * 0.8);
    }

    return fragColor;
}
