#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

struct Config {
    depth_threshold: f32,
    normal_threshold: f32,
    color_threshold: f32,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
///@group(0) @binding(2) var depth_prepass_texture: texture_depth_2d;
@group(0) @binding(2) var normal_prepass_texture: texture_2d<f32>;
@group(0) @binding(3) var<uniform> view: View;
@group(0) @binding(4) var<uniform> config: Config;

/// Retrieve the perspective camera near clipping plane
///fn perspective_camera_near() -> f32 {
///    return view.projection[3][2];
///}

/// Convert ndc depth to linear view z.
/// Note: Depth values in front of the camera will be negative as -z is forward
/// fn depth_ndc_to_view_z(ndc_depth: f32) -> f32 {
/// #ifdef VIEW_PROJECTION_PERSPECTIVE
///    return perspective_camera_near() / ndc_depth;
/// #else ifdef VIEW_PROJECTION_ORTHOGRAPHIC
///    return -(view.projection[3][2] - ndc_depth) / view.projection[2][2];
///#else
///    let view_pos = view.inverse_projection * vec4(0.0, 0.0, ndc_depth, 1.0);
///    return view_pos.z / view_pos.w;
///#endif
///}

fn prepass_depth(frag_coord: vec2f) -> f32 {
    return textureLoad(depth_prepass_texture, vec2i(frag_coord), 0);
}

fn prepass_normal(frag_coord: vec2f) -> vec3f {
    return textureLoad(normal_prepass_texture, vec2i(frag_coord), 0).xyz;
}

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

var<private> neighbours: array<vec2f, 9> = array<vec2f, 9>(
    vec2f(-1.0,  1.0), vec2f(0.0,  1.0), vec2f(1.0,  1.0),
    vec2f(-1.0,  0.0), vec2f(0.0,  0.0), vec2f(1.0,  0.0),
    vec2f(-1.0, -1.0), vec2f(0.0, -1.0), vec2f(1.0, -1.0),
);

var<private> thickness: f32 = 0.8;

fn detect_edge_f32(samples: ptr<function, array<f32, 9>>) -> f32 {
    var horizontal = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += (*samples)[i] * sobel_x[i];
    }
    var vertical = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += (*samples)[i] * sobel_y[i];
    }
    var edge = sqrt(dot(horizontal, horizontal) + dot(vertical, vertical));
    return edge;
}

fn detect_edge_vec3(samples: ptr<function, array<vec3f, 9>>) -> f32 {
    var horizontal = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += (*samples)[i].xyz * sobel_x[i];
    }
    var vertical = vec3f(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += (*samples)[i].xyz * sobel_y[i];
    }
    var edge = sqrt(dot(horizontal, horizontal) + dot(vertical, vertical));
    return edge;
}

/// returns the (0.0, 0.0) .. (1.0, 1.0) position within the viewport for the current render target
/// [0 .. render target viewport size] eg. [(0.0, 0.0) .. (1280.0, 720.0)] to [(0.0, 0.0) .. (1.0, 1.0)]
fn frag_coord_to_uv(frag_coord: vec2<f32>) -> vec2<f32> {
    return (frag_coord - view.viewport.xy) / view.viewport.zw;
}

/// Convert uv [0.0 .. 1.0] coordinate to ndc space xy [-1.0 .. 1.0]
fn uv_to_ndc(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2(2.0, -2.0) + vec2(-1.0, 1.0);
}

///fn detect_edge_depth(frag_coord: vec2f) -> f32 {
///    if config.depth_threshold == 0.0 {
///        return 0.0;
///    }
///
///    var samples = array<f32, 9>();
///    for (var i = 0; i < 9; i++) {
///        samples[i] =  depth_ndc_to_view_z(prepass_depth(frag_coord + neighbours[i] * thickness));
///    }
///
///    let edge = detect_edge_f32(&samples);
///
///    // let ndc = uv_to_ndc(frag_coord_to_uv(frag_coord));
///    // let pos = position_ndc_to_view(vec3(ndc, -1.0));
///    // let dir = normalize(pos);
///    // let n = prepass_normal(frag_coord);
///    // let t1 = smoothstep(0.8, 1.0, 1.0 - dot(n, dir));
///    // let t2 = mix(0.1, 1000.0, t1);
///
///    // Make the threshold change based on depth
///    let d = depth_ndc_to_view_z(prepass_depth(frag_coord));
///    if edge < config.depth_threshold * d {
///        return 0.0;
///    }
///    return edge;
///}

fn detect_edge_normal(frag_coord: vec2f) -> f32 {
    if config.normal_threshold == 0.0 {
        return 0.0;
    }

    var samples = array<vec3f, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = prepass_normal(frag_coord + neighbours[i] * thickness);
    }

    let edge = detect_edge_vec3(&samples);
    if edge < config.normal_threshold {
        return 0.0;
    }
    return edge;
}

fn detect_edge_color(frag_coord: vec2f) -> f32 {
    if config.color_threshold == 0.0 {
        return 0.0;
    }

    var samples = array<vec3f, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = textureLoad(screen_texture, vec2i(frag_coord + neighbours[i] * thickness), 0).rgb;
    }

    let edge = detect_edge_vec3(&samples);
    if edge < config.color_threshold {
        return 0.0;
    }
    return edge;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    let frag_coord = in.position.xy;
    /// let edge_depth = detect_edge_depth(frag_coord);
    let edge_depth = 0.0;
    let edge_normal = detect_edge_normal(frag_coord);
    let edge_color = detect_edge_color(frag_coord);
    let edge = max(edge_depth, max(edge_normal, edge_color));

    return vec4(200.0, 200.0, 200.0, 200.0);

    ///if config.debug == 1u {
    ///    return vec4(edge_depth, edge_normal, edge_color, 1.0);
    ///}

    ///if edge > 0.01 {
    ///    return vec4(200.0, 0.0, 0.0, 0.0);
    ///}


    ///return color;
}
