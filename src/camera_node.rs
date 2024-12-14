use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            Buffer, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, ImageDataLayout,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        RenderApp,
    },
};

use crate::camera_readback::{RatatuiCameraSender, RatatuiSobelSender};

pub(super) fn plugin(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);

    render_app
        .add_render_graph_node::<ViewNodeRunner<RatatuiCameraNode>>(Core3d, RatatuiCameraLabel);
    render_app.add_render_graph_edge(Core3d, Node3d::Upscaling, RatatuiCameraLabel);
}

#[derive(Default)]
pub struct RatatuiCameraNode;

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
pub struct RatatuiCameraLabel;

impl ViewNode for RatatuiCameraNode {
    type ViewQuery = (&'static RatatuiCameraSender, &'static RatatuiSobelSender);

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (camera_sender, sobel_sender): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();

        let src_image = gpu_images.get(&camera_sender.sender_image).unwrap();
        let src_image_sobel = gpu_images.get(&camera_sender.sender_image).unwrap();

        copy_to_buffer(render_context, world, src_image, &camera_sender.buffer);
        copy_to_buffer(render_context, world, src_image_sobel, &sobel_sender.buffer);

        Ok(())
    }
}

fn copy_to_buffer(
    render_context: &mut RenderContext,
    world: &World,
    src_image: &GpuImage,
    buffer: &Buffer,
) {
    let mut encoder = render_context
        .render_device()
        .create_command_encoder(&CommandEncoderDescriptor::default());

    let block_dimensions = src_image.texture_format.block_dimensions();
    let block_size = src_image.texture_format.block_copy_size(None).unwrap();

    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
        (src_image.size.x as usize / block_dimensions.0 as usize) * block_size as usize,
    );

    let texture_extent = Extent3d {
        width: src_image.size.x,
        height: src_image.size.y,
        depth_or_array_layers: 1,
    };

    encoder.copy_texture_to_buffer(
        src_image.texture.as_image_copy(),
        ImageCopyBuffer {
            buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    std::num::NonZeroU32::new(padded_bytes_per_row as u32)
                        .unwrap()
                        .into(),
                ),
                rows_per_image: None,
            },
        },
        texture_extent,
    );

    let render_queue = world.get_resource::<RenderQueue>().unwrap();
    render_queue.submit(std::iter::once(encoder.finish()));
}
