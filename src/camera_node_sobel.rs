use std::path::Path;

use bevy::{
    asset::{embedded_asset, io::AssetSourceId, AssetPath},
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
        prepass::ViewPrepassTextures,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_depth_2d, uniform_buffer, uniform_buffer_sized,
            },
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedPipelineState,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
            UniformBuffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        Render, RenderApp, RenderSet,
    },
};

use crate::camera_readback::RatatuiSobelSender;

pub struct RatatuiCameraNodeSobelPlugin;

impl Plugin for RatatuiCameraNodeSobelPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "src/", "shaders/sobel.wgsl");

        app.add_plugins(ExtractResourcePlugin::<RatatuiCameraNodeSobelConfig>::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(
            Render,
            prepare_config_buffer_system.in_set(RenderSet::Prepare),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<RatatuiCameraNodeSobel>>(
                Core3d,
                RatatuiCameraNodeSobelLabel,
            )
            .add_render_graph_edge(Core3d, Node3d::EndMainPass, RatatuiCameraNodeSobelLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RatatuiCameraNodeSobelConfigBuffer>()
            .init_resource::<RatatuiCameraNodeSobelPipeline>();
    }
}

#[derive(Default)]
pub struct RatatuiCameraNodeSobel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct RatatuiCameraNodeSobelLabel;

impl ViewNode for RatatuiCameraNodeSobel {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewPrepassTextures,
        &'static ViewUniformOffset,
        &'static RatatuiSobelSender,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, view_prepass_textures, view_uniform_offset, sobel_sender): QueryItem<
            'w,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();
        let sobel_pipeline = world.resource::<RatatuiCameraNodeSobelPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        if let CachedPipelineState::Err(pipeline_error) =
            pipeline_cache.get_render_pipeline_state(sobel_pipeline.pipeline_id)
        {
            log::error!("{pipeline_error:?}");
        };

        let Some(pipeline) = pipeline_cache.get_render_pipeline(sobel_pipeline.pipeline_id) else {
            return Ok(());
        };

        let source = view_target.main_texture_view();
        let destination = gpu_images.get(&sobel_sender.sender_image).unwrap();
        let view_uniforms = world.resource::<ViewUniforms>();

        // TODO: pull this data from the LuminanceConfig.
        let config_buffer = world.resource::<RatatuiCameraNodeSobelConfigBuffer>();

        let (Some(depth_prepass), Some(normal_prepass)) = (
            view_prepass_textures.depth_view(),
            view_prepass_textures.normal_view(),
        ) else {
            return Ok(());
        };

        let Some(view_uniforms) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            "ratatui_camera_node_sobel_bind_group",
            &sobel_pipeline.layout,
            &BindGroupEntries::sequential((
                source,
                &sobel_pipeline.sampler,
                depth_prepass,
                normal_prepass,
                view_uniforms,
                &config_buffer.buffer,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ratatui_camera_node_sobel_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &destination.texture_view,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[view_uniform_offset.offset]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource, ShaderType, ExtractResource, Clone, Copy)]
pub struct RatatuiCameraNodeSobelConfig {
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub color_threshold: f32,
}

impl Default for RatatuiCameraNodeSobelConfig {
    fn default() -> Self {
        Self {
            depth_threshold: 0.01,
            normal_threshold: 0.01,
            color_threshold: 0.01,
        }
    }
}

#[derive(Resource)]
pub struct RatatuiCameraNodeSobelConfigBuffer {
    buffer: UniformBuffer<RatatuiCameraNodeSobelConfig>,
}

impl FromWorld for RatatuiCameraNodeSobelConfigBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let config = RatatuiCameraNodeSobelConfig::default();
        let mut buffer = UniformBuffer::default();
        buffer.set(config);
        buffer.write_buffer(render_device, render_queue);

        Self { buffer }
    }
}

fn prepare_config_buffer_system(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut config_buffer: ResMut<RatatuiCameraNodeSobelConfigBuffer>,
    config: Res<RatatuiCameraNodeSobelConfig>,
) {
    let buffer = config_buffer.buffer.get_mut();
    *buffer = *config;
    config_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}

#[derive(Resource)]
struct RatatuiCameraNodeSobelPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for RatatuiCameraNodeSobelPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "ratatui_camera_node_sobel_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // rendered texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    // depth prepass
                    texture_depth_2d(),
                    // normal prepass
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // view
                    uniform_buffer::<ViewUniform>(true),
                    // config
                    uniform_buffer_sized(false, None),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let path = Path::new("bevy_ratatui_render").join("shaders/sobel.wgsl");
        let source = AssetSourceId::from("embedded");
        let asset_path = AssetPath::from_path(&path).with_source(source);
        let shader_handle: Handle<Shader> = world.load_asset(asset_path);

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("ratatui_camera_node_sobel_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader_handle,
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
