#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

struct Config {
    thickness: f32,

    color_enabled: u32,
    color_threshold: f32,

    depth_enabled: u32,
    depth_threshold: f32,

    normal_enabled: u32,
    normal_threshold: f32,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var depth_prepass_texture: texture_depth_2d;
@group(0) @binding(3) var normal_prepass_texture: texture_2d<f32>;
@group(0) @binding(4) var<uniform> view: View;
@group(0) @binding(5) var<uniform> config: Config;

var<private> thickness: f32 = 1.4;

var<private> sobel_horizontal: array<f32, 9> = array<f32, 9>(
    1.0, 0.0, -1.0,
    2.0, 0.0, -2.0,
    1.0, 0.0, -1.0,
);

var<private> sobel_vertical: array<f32, 9> = array<f32, 9>(
     1.0,  2.0,  1.0,
     0.0,  0.0,  0.0,
    -1.0, -2.0, -1.0,
);

var<private> sobel_forward: array<f32, 9> = array<f32, 9>(
     0.0,  1.0,  2.0,
    -1.0,  0.0,  1.0,
    -2.0, -1.0,  0.0,
);

var<private> sobel_backward: array<f32, 9> = array<f32, 9>(
     2.0,  1.0,  0.0,
     1.0,  0.0, -1.0,
     0.0, -1.0, -2.0,
);

var<private> neighbors: array<vec2f, 9> = array<vec2f, 9>(
    vec2f(-1.0,  1.0), vec2f(0.0,  1.0), vec2f(1.0,  1.0),
    vec2f(-1.0,  0.0), vec2f(0.0,  0.0), vec2f(1.0,  0.0),
    vec2f(-1.0, -1.0), vec2f(0.0, -1.0), vec2f(1.0, -1.0),
);

fn detect_edge_f32(samples: ptr<function, array<f32, 9>>) -> vec4f {
    var horizontal = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += (*samples)[i] * sobel_horizontal[i];
    }
    var vertical = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += (*samples)[i] * sobel_vertical[i];
    }
    var forward = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        forward += (*samples)[i] * sobel_forward[i];
    }
    var backward = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        backward += (*samples)[i] * sobel_backward[i];
    }

    let edge = vec4f(
        sqrt(dot(horizontal, horizontal)),
        sqrt(dot(vertical, vertical)),
        sqrt(dot(forward, forward)),
        sqrt(dot(backward, backward)),
    );

    return edge;
}

fn detect_edge_vec3(samples: ptr<function, array<vec3f, 9>>) -> vec4f {
    var horizontal = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += (*samples)[i].xyz * sobel_horizontal[i];
    }
    var vertical = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += (*samples)[i].xyz * sobel_vertical[i];
    }
    var forward = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        forward += (*samples)[i].xyz * sobel_forward[i];
    }
    var backward = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        backward += (*samples)[i].xyz * sobel_backward[i];
    }

    let edge = vec4f(
        sqrt(dot(horizontal, horizontal)),
        sqrt(dot(vertical, vertical)),
        sqrt(dot(forward, forward)),
        sqrt(dot(backward, backward)),
    );

    return edge;
}

fn prepass_depth(frag_coord: vec2f) -> f32 {
    return textureLoad(depth_prepass_texture, vec2i(frag_coord), 0);
}

fn prepass_normal(frag_coord: vec2f) -> vec3f {
    return textureLoad(normal_prepass_texture, vec2i(frag_coord), 0).xyz;
}

fn detect_edge_depth(frag_coord: vec2f) -> vec4f {
    if config.depth_enabled == 0u {
        return vec4f(0.0);
    }

    var samples = array<f32, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] =  prepass_depth(frag_coord + neighbors[i] * config.thickness);
    }

    var edge = detect_edge_f32(&samples);
    if edge.x < config.depth_threshold {
        edge.x = 0.0;
    }
    if edge.y < config.depth_threshold {
        edge.y = 0.0;
    }
    if edge.z < config.depth_threshold {
        edge.z = 0.0;
    }
    if edge.w < config.depth_threshold {
        edge.w = 0.0;
    }

    return edge;
}

fn detect_edge_normal(frag_coord: vec2f) -> vec4f {
    if config.normal_enabled == 0u {
        return vec4f(0.0);
    }

    var samples = array<vec3f, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = prepass_normal(frag_coord + neighbors[i] * config.thickness);
    }

    var edge = detect_edge_vec3(&samples);
    if edge.x < config.normal_threshold {
        edge.x = 0.0;
    }
    if edge.y < config.normal_threshold {
        edge.y = 0.0;
    }
    if edge.z < config.normal_threshold {
        edge.z = 0.0;
    }
    if edge.w < config.normal_threshold {
        edge.w = 0.0;
    }

    return edge;
}

fn detect_edge_color(frag_coord: vec2f) -> vec4f {
    if config.color_enabled == 0u {
        return vec4f(0.0);
    }

    var samples = array<vec3f, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = textureLoad(
            screen_texture,
            vec2i(frag_coord + neighbors[i] * config.thickness),
            0,
        ).rgb;
    }

    var edge = detect_edge_vec3(&samples);
    if edge.x < config.color_threshold {
        edge.x = 0.0;
    }
    if edge.y < config.color_threshold {
        edge.y = 0.0;
    }
    if edge.z < config.color_threshold {
        edge.z = 0.0;
    }
    if edge.w < config.color_threshold {
        edge.w = 0.0;
    }

    return edge;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    let frag_coord = in.position.xy;
    let edge_color = detect_edge_color(frag_coord) * 2.;
    let edge_normal = detect_edge_normal(frag_coord) * 0.5;
    let edge_depth = detect_edge_depth(frag_coord) * 2.;

    var edge = vec4f(0.0);
    edge = max(edge, edge_color);
    edge = max(edge, edge_normal);
    edge = max(edge, edge_depth);

    return vec4f(edge.x, edge.y, edge.z, edge.w);
}
