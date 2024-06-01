use bevy::render::texture::Image;
use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use ratatui::prelude::*;
use ratatui_image::{
    picker::{Picker, ProtocolType},
    Resize,
};

pub struct RatRenderWidget<'a> {
    image: &'a Image,
    picker: Picker,
}

impl<'a> RatRenderWidget<'a> {
    pub fn new(image: &'a Image) -> Self {
        let mut picker = Picker::new((1, 2));
        picker.protocol_type = ProtocolType::Halfblocks;

        Self { image, picker }
    }
}

impl<'a> Widget for RatRenderWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let RatRenderWidget { image, mut picker } = self;

        //// TODO: commented code would work using the same versions of `image` crate.
        //// These extra steps are necessary until ratatui_image uses images 0.25.
        //
        // let image = match self.rendered_image.clone().try_into_dynamic() {
        //     Ok(image) => image,
        //     Err(e) => panic!("Failed to create image buffer {e:?}"),
        // };

        let buffer = match ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            image.width(),
            image.height(),
            image.data.clone(),
        ) {
            Some(image) => image,
            None => panic!("failed to create image buffer"),
        };

        let image = DynamicImage::ImageRgba8(buffer);

        let image = image.resize(
            area.width as u32,
            area.height as u32 * 2,
            imageops::FilterType::Nearest,
        );

        let render_area = Rect {
            x: area.x + area.width.saturating_sub(image.width() as u16) / 2,
            y: area.y + (area.height * 2).saturating_sub(image.height() as u16) / 4,
            ..area
        };

        let img_as_halfblocks = picker
            .new_protocol(image, render_area, Resize::Fit(None))
            .unwrap();

        ratatui_image::Image::new(img_as_halfblocks.as_ref()).render(render_area, buf);
    }
}
