use image::DynamicImage;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};
use ratatui_image::{
    picker::{Picker, ProtocolType},
    FilterType, Resize,
};

pub struct RatatuiCameraWidgetHalfblocks<'a> {
    camera_image: &'a DynamicImage,
}

impl<'a> RatatuiCameraWidgetHalfblocks<'a> {
    pub fn new(camera_image: &'a DynamicImage) -> Self {
        Self { camera_image }
    }
}

impl WidgetRef for RatatuiCameraWidgetHalfblocks<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut picker = Picker::from_fontsize((1, 2));
        picker.set_protocol_type(ProtocolType::Halfblocks);

        let camera_image = self.camera_image.resize(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Nearest,
        );

        let render_area = Rect {
            x: area.x + area.width.saturating_sub(camera_image.width() as u16) / 2,
            y: area.y + (area.height * 2).saturating_sub(camera_image.height() as u16) / 4,
            ..area
        };

        let image_as_halfblocks = picker
            .new_protocol(camera_image, render_area, Resize::Fit(None))
            .unwrap();

        ratatui_image::Image::new(&image_as_halfblocks).render(render_area, buf);
    }
}
