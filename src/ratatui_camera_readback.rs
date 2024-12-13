use bevy::{
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_resource::{Maintain, MapMode},
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
};

use crate::{
    image_pipe::ImageReceiver,
    ratatui_camera::{
        RatatuiCameraReceiver, RatatuiCameraSender, RatatuiSobelReceiver, RatatuiSobelSender,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ExtractComponentPlugin::<RatatuiCameraSender>::default(),
        ExtractComponentPlugin::<RatatuiSobelSender>::default(),
    ))
    .add_systems(
        First,
        (receive_camera_images_system, receive_sobel_images_system),
    );

    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(Render, send_camera_images_system.after(RenderSet::Render));
}

fn send_camera_images_system(
    ratatui_camera_senders: Query<&RatatuiCameraSender>,
    render_device: Res<RenderDevice>,
) {
    for ratatui_camera_sender in &ratatui_camera_senders {
        let buffer_slice = ratatui_camera_sender.buffer.slice(..);

        let (s, r) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |r| match r {
            Ok(r) => s.send(r).expect("failed to send map update"),
            Err(err) => panic!("failed to map buffer: {err}"),
        });

        render_device.poll(Maintain::wait()).panic_on_timeout();

        r.recv().expect("failed to receive the map_async message");

        let _ = ratatui_camera_sender
            .sender
            .send(buffer_slice.get_mapped_range().to_vec());

        ratatui_camera_sender.buffer.unmap();
    }
}

fn receive_camera_images_system(mut ratatui_camera_receivers: Query<&mut RatatuiCameraReceiver>) {
    for mut ratatui_camera_receiver in &mut ratatui_camera_receivers {
        receive_image(&mut ratatui_camera_receiver);
    }
}

fn receive_sobel_images_system(mut ratatui_sobel_receivers: Query<&mut RatatuiSobelReceiver>) {
    for mut ratatui_sobel_receiver in &mut ratatui_sobel_receivers {
        receive_image(&mut ratatui_sobel_receiver);
    }
}

fn receive_image(image_receiver: &mut ImageReceiver) {
    let mut image_data = Vec::new();
    while let Ok(data) = image_receiver.receiver.try_recv() {
        image_data = data;
    }

    if !image_data.is_empty() {
        let row_bytes = image_receiver.receiver_image.width() as usize
            * image_receiver
                .receiver_image
                .texture_descriptor
                .format
                .pixel_size();

        let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);

        if row_bytes == aligned_row_bytes {
            image_receiver.receiver_image.data.clone_from(&image_data);
        } else {
            image_receiver.receiver_image.data = image_data
                .chunks(aligned_row_bytes)
                .take(image_receiver.receiver_image.height() as usize)
                .flat_map(|row| &row[..row_bytes.min(row.len())])
                .cloned()
                .collect();
        }
    }
}
