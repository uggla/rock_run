/// ***************************** ///
/// This is a shadertoy port of 'Tileable Water Caustic' by Dave_Hoskins, who claims to of sound it on glsl sandbox, by 'joltz0r'
/// I have been unable to find the original.
/// ***************************** ///

#import bevy_render::view::View
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import "shaders/shader_utils.wgsl"::TAU


@group(0) @binding(0) var<uniform> view: View;

const MAX_ITER: i32 = 3;
const SPEED:f32 = 1.0;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time: f32 = globals.time * 0.5 + 23.0;
    var uv: vec2<f32> = in.uv;

    // Tiling calculation
    var p: vec2<f32>;
    p = uv * TAU % TAU - 250.0;

    var i: vec2<f32> = vec2<f32>(p); // iterator position
    var c: f32 = 1.0; // color intensity
    let inten: f32 = 0.005; // Intensity factor

    for (var n: i32 = 0; n < MAX_ITER; n = n + 1) {
        let t: f32 = time * (1.0 - (3.5 / f32(n + 1)));
        i = p + vec2<f32>(cos(t - i.x) + sin(t + i.y), sin(t - i.y) + cos(t + i.x));
        c += 1.0 / length(vec2<f32>(p.x / (sin(i.x + t) / inten), p.y / (cos(i.y + t) / inten)));
    }

    // c = color intensity
    c /= f32(MAX_ITER);
    c = 1.17 - pow(c, 1.4);
    var color: vec3<f32> = vec3<f32>(pow(abs(c), 8.0));
    color = clamp(color + vec3<f32>(0.6, 0.00, 0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.8, 0.5, 0.0));

    return vec4<f32>(color, 1.0);
}
