use std::io;

use bevy::{prelude::*, render::camera::RenderTarget};
use image::DynamicImage;

use crate::{
    render_headless::{parse_image_data, ImageToSave, MainWorldReceiver},
    render_plugin::RatRenderContext,
    RatContext, RatRenderWidget,
};

pub type RatReceiveOutput = Option<DynamicImage>;
pub type RatCreateOutput = RenderTarget;

pub fn rat_receive(
    images_to_save: Query<&ImageToSave>,
    receiver: Res<MainWorldReceiver>,
    mut images: ResMut<Assets<Image>>,
    mut rat_render_context: ResMut<RatRenderContext>,
) {
    let mut image_data = Vec::new();
    while let Ok(data) = receiver.try_recv() {
        image_data = data;
    }
    if !image_data.is_empty() {
        if let Some(image_to_save) = images_to_save.iter().next() {
            rat_render_context.rendered_image =
                Some(parse_image_data(&mut images, image_to_save, image_data));
        }
    }
}

pub fn rat_print(
    mut rat: ResMut<RatContext>,
    rat_render_context: Res<RatRenderContext>,
) -> io::Result<()> {
    if let Some(image) = rat_render_context.rendered_image.clone() {
        rat.draw(|frame| {
            frame.render_widget(RatRenderWidget::new(image), frame.size());
        })?;
    }

    Ok(())
}
