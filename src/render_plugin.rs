use bevy::{
    prelude::*,
    render::{render_graph::RenderGraph, Render, RenderApp, RenderSet},
};

use crate::render_image_copy::{
    image_copy_extract, receive_image_from_buffer, ImageCopy, ImageCopyNode, MainWorldReceiver,
    RatRenderState, RenderWorldSender,
};

#[derive(Deref)]
pub struct RatRenderPlugin((u32, u32));
impl Plugin for RatRenderPlugin {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        let (width, height) = **self;

        app.insert_resource(MainWorldReceiver(r))
            .insert_resource(RatRenderState::new(width, height));

        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyNode);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(Render, receive_image_from_buffer.after(RenderSet::Render));
    }
}

impl RatRenderPlugin {
    pub fn new(width: u32, height: u32) -> Self {
        Self((width, height))
    }
}
