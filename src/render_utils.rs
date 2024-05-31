use std::io;

use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_resource::Extent3d, renderer::RenderDevice},
};
use image::DynamicImage;

use crate::{
    render_headless::{
        create_render_textures, parse_image_data, ImageCopier, ImageToSave, MainWorldReceiver,
        RatRenderState,
    },
    RatContext, RatRenderWidget,
};

pub type RatReceiveOutput = Option<DynamicImage>;
pub type RatCreateOutput = RenderTarget;

pub fn rat_create(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    mut rat_state: ResMut<RatRenderState>,
) -> RatCreateOutput {
    let size = Extent3d {
        width: rat_state.width,
        height: rat_state.height,
        ..Default::default()
    };

    let (render_texture, cpu_texture) = create_render_textures(size);
    let render_handle = images.add(render_texture);
    let cpu_handle = images.add(cpu_texture);

    commands.spawn(ImageCopier::new(
        render_handle.clone(),
        size,
        &render_device,
    ));
    commands.spawn(ImageToSave(cpu_handle));

    rat_state.built = true;

    RenderTarget::Image(render_handle)
}

pub fn rat_receive(
    images_to_save: Query<&ImageToSave>,
    receiver: Res<MainWorldReceiver>,
    mut images: ResMut<Assets<Image>>,
    rat_state: ResMut<RatRenderState>,
) -> Option<DynamicImage> {
    if rat_state.built {
        let mut image_data = Vec::new();
        while let Ok(data) = receiver.try_recv() {
            image_data = data;
        }
        if !image_data.is_empty() {
            if let Some(image_to_save) = images_to_save.iter().next() {
                return Some(parse_image_data(&mut images, image_to_save, image_data));
            }
        }
    }

    None
}

pub fn rat_print(In(image): In<RatReceiveOutput>, mut rat: ResMut<RatContext>) -> io::Result<()> {
    if let Some(image) = image {
        rat.draw(|frame| {
            frame.render_widget(RatRenderWidget::new(image), frame.size());
        })?;
    }

    Ok(())
}
