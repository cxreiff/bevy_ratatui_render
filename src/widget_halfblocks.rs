use image::DynamicImage;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};
use ratatui_image::{
    picker::{Picker, ProtocolType},
    FilterType, Resize,
};

pub struct RatatuiRenderWidgetHalfblocks {
    image: DynamicImage,
    picker: Picker,
}

impl RatatuiRenderWidgetHalfblocks {
    pub fn new(image: DynamicImage) -> Self {
        let mut picker = Picker::from_fontsize((1, 2));
        picker.set_protocol_type(ProtocolType::Halfblocks);

        Self { image, picker }
    }
}

impl WidgetRef for RatatuiRenderWidgetHalfblocks {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let Self { image, mut picker } = self;

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
