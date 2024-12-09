use bevy::image::Image;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::{RatatuiRenderStrategy, RatatuiRenderWidgetHalfblocks, RatatuiRenderWidgetLuminance};

pub struct RatatuiRenderWidget<'a, 'b, 'c> {
    image: &'a Image,
    sobel: &'b Option<Image>,
    strategy: &'c RatatuiRenderStrategy,
}

impl<'a, 'b, 'c> RatatuiRenderWidget<'a, 'b, 'c> {
    pub fn new(
        image: &'a Image,
        sobel: &'b Option<Image>,
        strategy: &'c RatatuiRenderStrategy,
    ) -> Self {
        Self {
            image,
            sobel,
            strategy,
        }
    }
}

impl Widget for RatatuiRenderWidget<'_, '_, '_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Self {
            image,
            sobel,
            strategy,
        } = self;

        let image = match image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create image buffer {e:?}"),
        };

        let sobel = sobel
            .as_ref()
            .map(|sobel| match sobel.clone().try_into_dynamic() {
                Ok(sobel) => sobel,
                Err(e) => panic!("failed to create sobel buffer {e:?}"),
            });

        match strategy {
            RatatuiRenderStrategy::Halfblocks => {
                RatatuiRenderWidgetHalfblocks::new(image).render_ref(area, buf)
            }
            RatatuiRenderStrategy::Luminance(luminance_config) => {
                RatatuiRenderWidgetLuminance::new(image, sobel, luminance_config.clone())
                    .render_ref(area, buf);
            }
        }
    }
}
