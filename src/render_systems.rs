use std::io;

use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_resource::Extent3d, renderer::RenderDevice},
};

use crate::{
    render_headless::{
        create_render_textures, parse_image_data, ImageCopier, ImageToSave, MainWorldReceiver,
    },
    render_plugin::{RatRenderConfig, RatRenderContext},
    RatContext, RatRenderWidget,
};

pub fn rat_create(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    rat_render_config: ResMut<RatRenderConfig>,
    render_device: Res<RenderDevice>,
) {
    let size = Extent3d {
        width: rat_render_config.width,
        height: rat_render_config.height,
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

    commands.insert_resource(RatRenderContext {
        camera_target: RenderTarget::Image(render_handle),
        rendered_image: None,
    });
}

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
    rat.draw(|frame| {
        frame.render_widget(
            RatRenderWidget::new(&rat_render_context.rendered_image),
            frame.size(),
        );
    })?;

    Ok(())
}
