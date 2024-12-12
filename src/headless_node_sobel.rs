use std::path::Path;

use bevy::{
    asset::{io::AssetSourceId, AssetPath},
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        core_3d::graph::{Core3d, Node3d},
    },
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::RenderAssets,
        render_graph::{Node, RenderGraphApp, RenderLabel},
        texture::GpuImage,
        RenderApp,
    },
};

use crate::headless_plugin::ImageCopierList;

pub struct HeadlessNodeSobelPlugin;

impl Plugin for HeadlessNodeSobelPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_render_graph_node::<SobelNode>(Core3d, SobelLabel)
            .add_render_graph_edge(Core3d, Node3d::EndMainPass, SobelLabel)
            .add_render_graph_node::<SobelNode>(Core2d, SobelLabel)
            .add_render_graph_edge(Core2d, Node2d::EndMainPass, SobelLabel);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SobelLabel;

#[derive(Default)]
pub struct SobelNode;

impl Node for SobelNode {
    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        _render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let _gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();
        let image_copier_list = world.get_resource::<ImageCopierList>().unwrap();

        for (image_copier, _image_copier_sobel) in image_copier_list.iter() {
            let _render_target = RenderTarget::Image(image_copier.image.clone());

            let shader_path = Path::new("bevy_ratatui_render").join("shaders/sobel.wgsl");
            let shader_source = AssetSourceId::from("embedded");
            let _shader_asset_path = AssetPath::from_path(&shader_path).with_source(shader_source);

            // TODO: use bevy_mod_edge_detection EdgeDetectionNode as example to configure a sobel
            // filter pass over
        }

        Ok(())
    }
}
