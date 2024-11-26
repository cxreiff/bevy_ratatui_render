use bevy::render::texture::Image;
use ratatui::prelude::*;
use ratatui::widgets::Widget;
use ratatui_image::{
    picker::{Picker, ProtocolType},
    FilterType, Resize,
};

pub struct RatatuiRenderWidget<'a> {
    image: &'a Image,
    picker: Picker,
}

impl<'a> RatatuiRenderWidget<'a> {
    pub fn new(image: &'a Image) -> Self {
        let mut picker = Picker::from_fontsize((1, 2));
        picker.set_protocol_type(ProtocolType::Halfblocks);

        Self { image, picker }
    }
}

impl<'a> Widget for RatatuiRenderWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Self { image, mut picker } = self;

        let image = match image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create image buffer {e:?}"),
        };

        let image = image.resize(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Nearest,
        );

        let render_area = Rect {
            x: area.x + area.width.saturating_sub(image.width() as u16) / 2,
            y: area.y + (area.height * 2).saturating_sub(image.height() as u16) / 4,
            ..area
        };

        let img_as_halfblocks = picker
            .new_protocol(image, render_area, Resize::Fit(None))
            .unwrap();

        ratatui_image::Image::new(&img_as_halfblocks).render(render_area, buf);
    }
}
