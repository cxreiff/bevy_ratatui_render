use image::{imageops, DynamicImage};
use ratatui::prelude::*;
use ratatui_image::{
    picker::{Picker, ProtocolType},
    Image, Resize,
};

pub struct RatRenderWidget {
    image: DynamicImage,
    picker: Picker,
}

impl RatRenderWidget {
    pub fn new(image: DynamicImage) -> Self {
        let mut picker = Picker::new((1, 2));
        picker.protocol_type = ProtocolType::Halfblocks;

        Self { image, picker }
    }
}

impl Widget for RatRenderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let RatRenderWidget { image, mut picker } = self;

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
