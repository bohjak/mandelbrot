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

    // Correctly it should return the limit, but for the inverted look, pure black looks much better
    return 0.0;
}

struct ViewportUniform {
    scale: f32,
    cx: f32,
    cy: f32,
    xoff: f32,
    yoff: f32,
};
@group(0) @binding(0)
var<uniform> viewport: ViewportUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = viewport.cx + (in.clip_position[0] - viewport.xoff) * viewport.scale;
    let y = viewport.cy + (in.clip_position[1] - viewport.yoff) * viewport.scale;
    // Pure black doesn't escape within the limit; the lighter the shade the more iterations it took to escape
    let e = escape_time(x, y, 255.0) / 255.0;
    return vec4<f32>(e, e, e, 1.0);
}
