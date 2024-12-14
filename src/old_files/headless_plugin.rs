use bevy::{
    asset::embedded_asset,
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::ExtractComponentPlugin,
        render_resource::{Maintain, MapMode},
        renderer::RenderDevice,
        Extract, Render, RenderApp, RenderSet,
    },
};

use crate::{
    headless_node::{self},
    headless_render_pipe::{ImageCopier, ImageCopierSobel},
    RatatuiRenderContext,
};

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "src/", "shaders/sobel.wgsl");

    app.init_resource::<RatatuiRenderContext>()
        .add_plugins((
            ExtractComponentPlugin::<ImageCopier>::default(),
            ExtractComponentPlugin::<ImageCopierSobel>::default(),
        ))
        .add_plugins(headless_node::plugin)
        .add_systems(
            First,
            (receive_rendered_images_system, receive_sobel_images_system),
        )
        .add_systems(PostUpdate, replaced_pipe_cleanup_system)
        .add_event::<ReplacedRenderPipeEvent>();

    let render_app = app.sub_app_mut(RenderApp);
    render_app
        .add_systems(ExtractSchedule, extract_image_copiers_system)
        .add_systems(Render, send_rendered_image_system.after(RenderSet::Render));
}

fn send_rendered_image_system(
    image_copy_sources: Res<ImageCopierList>,
    render_device: Res<RenderDevice>,
) {
    for (image_copier, _image_copier_sobel) in image_copy_sources.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);

        let (s, r) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |r| match r {
            Ok(r) => s.send(r).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        render_device.poll(Maintain::wait()).panic_on_timeout();

        r.recv().expect("Failed to receive the map_async message");

        let _ = image_copier
            .sender
            .send(buffer_slice.get_mapped_range().to_vec());

        image_copier.buffer.unmap();
    }
}

fn receive_rendered_images_system(mut ratatui_render: ResMut<RatatuiRenderContext>) {
    for render_pipe in &mut ratatui_render.values_mut() {
        let mut image_data = Vec::new();
        while let Ok(data) = render_pipe.receiver.try_recv() {
            image_data = data;
        }
        if !image_data.is_empty() {
            let row_bytes = render_pipe.image.width() as usize
                * render_pipe.image.texture_descriptor.format.pixel_size();
            let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);
            if row_bytes == aligned_row_bytes {
                render_pipe.image.data.clone_from(&image_data);
            } else {
                render_pipe.image.data = image_data
                    .chunks(aligned_row_bytes)
                    .take(render_pipe.image.height() as usize)
                    .flat_map(|row| &row[..row_bytes.min(row.len())])
                    .cloned()
                    .collect();
            }
        }
    }
}

fn receive_sobel_images_system(mut ratatui_render: ResMut<RatatuiRenderContext>) {
    for render_pipe in &mut ratatui_render.values_mut() {
        let Some(ref receiver) = render_pipe.receiver_sobel else {
            return;
        };

        let Some(ref mut image) = render_pipe.image_sobel else {
            return;
        };

        let mut image_data = Vec::new();
        while let Ok(data) = receiver.try_recv() {
            image_data = data;
        }
        if !image_data.is_empty() {
            let row_bytes = image.width() as usize * image.texture_descriptor.format.pixel_size();
            let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);
            if row_bytes == aligned_row_bytes {
                image.data.clone_from(&image_data);
            } else {
                image.data = image_data
                    .chunks(aligned_row_bytes)
                    .take(image.height() as usize)
                    .flat_map(|row| &row[..row_bytes.min(row.len())])
                    .cloned()
                    .collect();
            }
        }
    }
}

#[derive(Event)]
pub struct ReplacedRenderPipeEvent {
    pub old_render_target: RenderTarget,
    pub new_render_target: RenderTarget,
}

/// When a new render pipe is created with an existing name, the old pipe is replaced.
/// This system cleans up assets and components from the old pipe.
fn replaced_pipe_cleanup_system(
    mut commands: Commands,
    mut replaced_pipe: EventReader<ReplacedRenderPipeEvent>,
    mut images: ResMut<Assets<Image>>,
    mut camera_query: Query<&mut Camera>,
    mut image_copier_query: Query<(Entity, &mut ImageCopier)>,
) {
    for ReplacedRenderPipeEvent {
        old_render_target,
        new_render_target,
    } in replaced_pipe.read()
    {
        if let Some(old_target_image) = old_render_target.as_image() {
            if let Some(mut camera) = camera_query.iter_mut().find(|camera| {
                if let Some(camera_image) = camera.target.as_image() {
                    return camera_image == old_target_image;
                }

                false
            }) {
                camera.target = new_render_target.clone();
                if let Some(image_handle) = old_render_target.as_image() {
                    images.remove(image_handle);
                }
            }

            if let Some((entity, image_copier)) = image_copier_query
                .iter_mut()
                .find(|(_, image_copier)| image_copier.image == *old_target_image)
            {
                commands.entity(entity).despawn();

                images.remove(&image_copier.image.clone());
            }
        };
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ImageCopierList(Vec<(ImageCopier, Option<ImageCopierSobel>)>);

fn extract_image_copiers_system(
    mut commands: Commands,
    image_copiers: Extract<Query<(&ImageCopier, Option<&ImageCopierSobel>)>>,
) {
    commands.insert_resource(ImageCopierList(
        image_copiers
            .iter()
            .map(|(image_copier, image_copier_sobel)| {
                (image_copier.clone(), image_copier_sobel.cloned())
            })
            .collect(),
    ));
}
