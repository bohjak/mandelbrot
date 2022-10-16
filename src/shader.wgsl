// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
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

    return 1.0;
}

struct ViewportUniform {
    window_size: f32,
    abstract_size: f32,
    offset: f32,
};
@group(0) @binding(0)
var<uniform> viewport: ViewportUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.clip_position[0] * viewport.abstract_size / viewport.window_size - viewport.offset;
    let y = in.clip_position[1] * viewport.abstract_size / viewport.window_size - viewport.offset;
    let e = escape_time(x, y, 255.0) / 255.0;
    return vec4<f32>(e, e, e, 1.0);
}
