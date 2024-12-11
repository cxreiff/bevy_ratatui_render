use bevy::image::Image;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::{RatatuiRenderStrategy, RatatuiRenderWidgetHalfblocks, RatatuiRenderWidgetLuminance};

pub struct RatatuiRenderWidget<'a, 'b, 'c> {
    image: &'a Image,
    image_sobel: &'b Option<Image>,
    strategy: &'c RatatuiRenderStrategy,
}

impl<'a, 'b, 'c> RatatuiRenderWidget<'a, 'b, 'c> {
    pub fn new(
        image: &'a Image,
        image_sobel: &'b Option<Image>,
        strategy: &'c RatatuiRenderStrategy,
    ) -> Self {
        Self {
            image,
            image_sobel,
            strategy,
        }
    }
}

impl Widget for RatatuiRenderWidget<'_, '_, '_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Self {
            image,
            image_sobel,
            strategy,
        } = self;

        let image = match image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create image buffer {e:?}"),
        };

        let image_sobel =
            image_sobel
                .as_ref()
                .map(|image_sobel| match image_sobel.clone().try_into_dynamic() {
                    Ok(image_sobel) => image_sobel,
                    Err(e) => panic!("failed to create sobel buffer {e:?}"),
                });

        match strategy {
            RatatuiRenderStrategy::Halfblocks => {
                RatatuiRenderWidgetHalfblocks::new(image).render_ref(area, buf)
            }
            RatatuiRenderStrategy::Luminance(luminance_config) => {
                // // TODO: REMOVE
                // RatatuiRenderWidgetLuminance::new(
                //     image_sobel.clone().unwrap(),
                //     image_sobel,
                //     luminance_config.clone(),
                // )
                // .render_ref(area, buf);

                RatatuiRenderWidgetLuminance::new(image, image_sobel, luminance_config.clone())
                    .render_ref(area, buf);
            }
        }
    }
}
