use std::path::Path;

use bevy::{
    asset::{io::AssetSourceId, AssetPath},
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::ExtractComponent,
        render_graph::{RenderGraphApp, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::Buffer,
        RenderApp,
    },
};
use crossbeam_channel::Sender;

use crate::headless::ImageCopier;

pub struct SobelPlugin;

impl Plugin for SobelPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_render_graph_node::<ViewNodeRunner<SobelNode>>(Core3d, SobelLabel)
            .add_render_graph_edges(
                Core3d,
                (Node3d::EndMainPass, SobelLabel, Node3d::Tonemapping),
            );
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct ImageCopierSobel {
    image: Handle<Image>,
    buffer: Buffer,
    sender: Sender<Vec<u8>>,
}

impl ImageCopierSobel {
    pub fn new(image: Handle<Image>, buffer: Buffer, sender: Sender<Vec<u8>>) -> Self {
        Self {
            image,
            buffer,
            sender,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SobelLabel;

#[derive(Default)]
pub struct SobelNode;

impl ViewNode for SobelNode {
    type ViewQuery = (&'static ImageCopier, &'static ImageCopierSobel);

    fn run<'w>(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (image_copier, image_copier_sobel): bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let render_target = RenderTarget::Image(image_copier.image.clone());

        let shader_path = Path::new("bevy_ratatui_render").join("shaders/sobel.wgsl");
        let shader_source = AssetSourceId::from("embedded");
        let shader_asset_path = AssetPath::from_path(&shader_path).with_source(shader_source);

        // TODO: use bevy_mod_edge_detection EdgeDetectionNode as example to configure a sobel
        // filter pass over

        unimplemented!()
    }
}
