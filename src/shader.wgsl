// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

fn norm_sqr(r: f32, i: f32) -> f32 {
    return r * r + i * i;
}

fn escape_time(cr: f32, ci: f32, limit: f32) -> f32 {
    var zr = 0.0;
    var zi = 0.0;
    var tr: f32;

    for (var i = 0.0; i < limit; i += 1.0) {
        if (norm_sqr(zr, zi) > 4.0) {
            return i;
        } else {
            tr = zr;
            zr = zr * zr - zi * zi + cr;
            zi = 2.0 * tr * zi + ci;
        }
    }

    return limit;
}

// Source: https://www.shadertoy.com/view/XljGzV
fn hsl2rgb(hsl: vec3<f32>) -> vec3<f32> {
    let base = hsl.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0);
    // clamp requires its arguments to be the same type
    let rgb = clamp(abs((base % 6.0) - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    return hsl.z + hsl.y * (rgb - 0.5) * (1.0 - abs(2.0 * hsl.z - 1.0));
}

// Padded for WebGL
struct ViewportUniform {
    scale: f32,
    cx: f32,
    cy: f32,
    xoff: f32,
    yoff: f32,
    _padding_a: f32,
    _padding_b: f32,
    _padding_c: f32,
};
@group(0) @binding(0)
var<uniform> viewport: ViewportUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = viewport.cx + (in.pos.x - viewport.xoff) * viewport.scale;
    let y = viewport.cy + (in.pos.y - viewport.yoff) * viewport.scale;

    let et = escape_time(x, y, 255.0) / 255.0;
    if et == 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let hsl = vec3<f32>(et, 1.0, 0.5);
    let rgb = hsl2rgb(hsl);
    return vec4<f32>(rgb, 1.0);
}
