use bevy::{
    prelude::*,
    render::{
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::{CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, ImageDataLayout},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        RenderApp,
    },
};

use crate::headless_plugin::ImageCopierList;

pub(super) fn plugin(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);

    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
    graph.add_node(ImageCopyLabel, ImageCopyNode);
    graph.add_node_edge(CameraDriverLabel, ImageCopyLabel);
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
pub struct ImageCopyLabel;

#[derive(Default)]
pub struct ImageCopyNode;

impl Node for ImageCopyNode {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();
        let image_copier_list = world.get_resource::<ImageCopierList>().unwrap();

        for (image_copier, image_copier_sobel) in image_copier_list.iter() {
            if !image_copier.enabled() {
                return Ok(());
            }

            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());

            let src_image = gpu_images.get(&image_copier.image).unwrap();

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
                    buffer: &image_copier.buffer,
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

            if let Some(image_copier_sobel) = image_copier_sobel {
                let mut encoder = render_context
                    .render_device()
                    .create_command_encoder(&CommandEncoderDescriptor::default());

                let src_image = gpu_images.get(&image_copier_sobel.image).unwrap();

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
                        buffer: &image_copier.buffer,
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
            }

            let render_queue = world.get_resource::<RenderQueue>().unwrap();
            render_queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }
}
