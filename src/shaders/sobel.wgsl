#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

struct Config {
    depth_threshold: f32,
    normal_threshold: f32,
    color_threshold: f32,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var depth_prepass_texture: texture_depth_2d;
@group(0) @binding(3) var normal_prepass_texture: texture_2d<f32>;
@group(0) @binding(4) var<uniform> view: View;
@group(0) @binding(5) var<uniform> config: Config;

var<private> thickness: f32 = 0.8;

var<private> sobel_x: array<f32, 9> = array<f32, 9>(
    1.0, 0.0, -1.0,
    2.0, 0.0, -2.0,
    1.0, 0.0, -1.0,
);

var<private> sobel_y: array<f32, 9> = array<f32, 9>(
     1.0,  2.0,  1.0,
     0.0,  0.0,  0.0,
    -1.0, -2.0, -1.0,
);

var<private> neighbors: array<vec2f, 9> = array<vec2f, 9>(
    vec2f(-1.0,  1.0), vec2f(0.0,  1.0), vec2f(1.0,  1.0),
    vec2f(-1.0,  0.0), vec2f(0.0,  0.0), vec2f(1.0,  0.0),
    vec2f(-1.0, -1.0), vec2f(0.0, -1.0), vec2f(1.0, -1.0),
);

fn detect_edge_vec3(samples: ptr<function, array<vec3f, 9>>) -> vec2f {
    var horizontal = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += (*samples)[i].xyz * sobel_x[i];
    }
    var vertical = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += (*samples)[i].xyz * sobel_y[i];
    }
    return vec2f(horizontal.x, vertical.x);
}

fn detect_edge_color(frag_coord: vec2f) -> vec2f {
    if config.color_threshold == 0.0 {
        return 0.0;
    }

    var samples = array<vec3f, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = textureLoad(
            screen_texture,
            vec2i(frag_coord + neighbors[i] * thickness),
            0
        ).rgb;
    }

    var edge = detect_edge_vec3(&samples);
    if edge.x < config.color_threshold {
        edge.x = 0.0;
    }
    if edge.y < config.color_threshold {
        edge.y = 0.0;
    }

    return edge;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    let frag_coord = in.position.xy;
    let edge_color = detect_edge_color(frag_coord);

    return vec4f(edge_color, 0.0, edge_color, 1.0);
    // return vec4f(1.0, 0.0, 0.0, 1.0);
}
