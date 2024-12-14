use std::path::Path;

use bevy::{
    asset::{io::AssetSourceId, AssetPath},
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::RenderAssets,
        render_graph::{Node, RenderGraphApp, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_depth_2d, uniform_buffer, uniform_buffer_sized,
            },
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType, TextureView, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
};

use crate::headless_plugin::ImageCopierList;

pub const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(410592619790336);

pub struct HeadlessNodeSobelPlugin;

impl Plugin for HeadlessNodeSobelPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(ExtractSchedule, extract_headless_node_sobel_config_system)
            .add_systems(
                Render,
                prepare_headless_node_sobel_config_buffer.in_set(RenderSet::Prepare),
            );

        render_app
            .add_render_graph_node::<HeadlessNodeSobel>(Core3d, HeadlessNodeSobelLabel)
            .add_render_graph_edge(Core3d, Node3d::EndMainPass, HeadlessNodeSobelLabel)
            .add_render_graph_node::<HeadlessNodeSobel>(Core2d, HeadlessNodeSobelLabel)
            .add_render_graph_edge(Core2d, Node2d::EndMainPass, HeadlessNodeSobelLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<HeadlessNodeSobelPipeline>()
            .init_resource::<HeadlessNodeSobelConfigBuffer>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct HeadlessNodeSobelLabel;

#[derive(Default)]
pub struct HeadlessNodeSobel;

impl Node for HeadlessNodeSobel {
    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        _render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let _gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();

        let edge_detection_pipeline = world.resource::<HeadlessNodeSobelPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(_pipeline) =
            pipeline_cache.get_render_pipeline(edge_detection_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let view_uniforms = world.resource::<ViewUniforms>();
        let Some(_view_uniforms) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let _config_buffer = world.resource::<HeadlessNodeSobelConfigBuffer>();

        let image_copier_list = world.get_resource::<ImageCopierList>().unwrap();

        for (image_copier, _image_copier_sobel) in image_copier_list.iter() {
            let _render_target = RenderTarget::Image(image_copier.image.clone());

            // let source = unimplemented!();
            // let destination = unimplemented!();

            // let bind_group = render_context.render_device().create_bind_group(
            //     "edge_detection_bind_group",
            //     &edge_detection_pipeline.layout,
            //     &BindGroupEntries::sequential((
            //         source,
            //         &edge_detection_pipeline.sampler,
            //         // &depth_texture.texture.default_view,
            //         // &normal_texture.texture.default_view,
            //         view_uniforms,
            //         &config_buffer.buffer,
            //     )),
            // );

            // let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            //     label: Some("edge_detection_pass"),
            //     color_attachments: &[Some(RenderPassColorAttachment {
            //         view: destination,
            //         resolve_target: None,
            //         ops: Operations::default(),
            //     })],
            //     depth_stencil_attachment: None,
            //     timestamp_writes: None,
            //     occlusion_query_set: None,
            // });

            // render_pass.set_render_pipeline(pipeline);
            // render_pass.set_bind_group(0, &bind_group, &[view_uniform.offset]);
            // render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}

#[derive(Resource)]
struct HeadlessNodeSobelPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for HeadlessNodeSobelPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "edge_detection_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // screen_texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    // // depth prepass
                    // texture_depth_2d(),
                    // // normal prepass
                    // texture_2d(TextureSampleType::Float { filterable: true }),
                    // view
                    uniform_buffer::<ViewUniform>(true),
                    // config
                    uniform_buffer_sized(false, None),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let shader_path = Path::new("bevy_ratatui_render").join("shaders/sobel.wgsl");
        let shader_source = AssetSourceId::from("embedded");
        let _shader_asset_path = AssetPath::from_path(&shader_path).with_source(shader_source);

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("edge_detection_pipeline".into()),
                    layout: vec![layout.clone()],
                    // This will setup a fullscreen triangle for the vertex state
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: SHADER_HANDLE,
                        shader_defs: vec!["VIEW_PROJECTION_PERSPECTIVE".into()], // TODO detect projection
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: true,
                });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

#[derive(Resource, ShaderType, Clone, Copy)]
pub struct HeadlessNodeSobelConfig {
    pub color_threshold: f32,
}

impl Default for HeadlessNodeSobelConfig {
    fn default() -> Self {
        Self {
            color_threshold: 1.0,
        }
    }
}

#[derive(Resource)]
struct HeadlessNodeSobelConfigBuffer {
    buffer: UniformBuffer<HeadlessNodeSobelConfig>,
}

impl FromWorld for HeadlessNodeSobelConfigBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let config = HeadlessNodeSobelConfig::default();
        let mut buffer = UniformBuffer::default();
        buffer.set(config);
        buffer.write_buffer(render_device, render_queue);

        HeadlessNodeSobelConfigBuffer { buffer }
    }
}

fn extract_headless_node_sobel_config_system(
    mut commands: Commands,
    config: Extract<Res<HeadlessNodeSobelConfig>>,
) {
    commands.insert_resource(**config);
}

fn prepare_headless_node_sobel_config_buffer(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut config_buffer: ResMut<HeadlessNodeSobelConfigBuffer>,
    config: Res<HeadlessNodeSobelConfig>,
) {
    let buffer = config_buffer.buffer.get_mut();
    *buffer = *config;
    config_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}
