use bevy::prelude::Component;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::camera_image_pipe::ImageReceiver;
use crate::{
    RatatuiCameraEdgeDetection, RatatuiCameraStrategy, RatatuiRenderWidgetHalfblocks,
    RatatuiRenderWidgetLuminance,
};

#[derive(Component)]
pub struct RatatuiCameraWidget {
    pub camera_receiver: ImageReceiver,
    pub sobel_receiver: Option<ImageReceiver>,
    pub strategy: RatatuiCameraStrategy,
    pub edge_detection: Option<RatatuiCameraEdgeDetection>,
}

impl Widget for &RatatuiCameraWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image = match self
            .camera_receiver
            .receiver_image
            .clone()
            .try_into_dynamic()
        {
            Ok(image) => image,
            Err(e) => panic!("failed to create image buffer {e:?}"),
        };

        let image_sobel = self.sobel_receiver.as_ref().map(|image_sobel| {
            match image_sobel.receiver_image.clone().try_into_dynamic() {
                Ok(image_sobel) => image_sobel,
                Err(e) => panic!("failed to create sobel buffer {e:?}"),
            }
        });

        match self.strategy {
            RatatuiCameraStrategy::HalfBlocks => {
                RatatuiRenderWidgetHalfblocks::new(image).render_ref(area, buf)
            }
            RatatuiCameraStrategy::Luminance(ref config) => {
                RatatuiRenderWidgetLuminance::new(
                    image,
                    image_sobel,
                    config.clone(),
                    self.edge_detection,
                )
                .render_ref(area, buf);
            }
        }
    }
}
