use image::{imageops, DynamicImage};
use ratatui::prelude::*;
use ratatui_image::{
    picker::{Picker, ProtocolType},
    Image, Resize,
};

pub struct RatRenderWidget<'a> {
    image: &'a Option<DynamicImage>,
    picker: Picker,
}

impl<'a> RatRenderWidget<'a> {
    pub fn new(image: &'a Option<DynamicImage>) -> Self {
        let mut picker = Picker::new((1, 2));
        picker.protocol_type = ProtocolType::Halfblocks;

        Self { image, picker }
    }
}

impl<'a> Widget for RatRenderWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let RatRenderWidget { image, mut picker } = self;

        if let Some(image) = image {
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

            Image::new(img_as_halfblocks.as_ref()).render(render_area, buf);
        }
    }
}
