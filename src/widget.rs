use bevy::prelude::Component;
use image::DynamicImage;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::{
    RatatuiCameraEdgeDetection, RatatuiCameraStrategy, RatatuiRenderWidgetHalfblocks,
    RatatuiRenderWidgetLuminance,
};

#[derive(Component)]
pub struct RatatuiCameraWidget {
    pub camera_image: DynamicImage,
    pub sobel_image: Option<DynamicImage>,
    pub strategy: RatatuiCameraStrategy,
    pub edge_detection: Option<RatatuiCameraEdgeDetection>,
}

impl Widget for &RatatuiCameraWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.strategy {
            RatatuiCameraStrategy::HalfBlocks => {
                RatatuiRenderWidgetHalfblocks::new(&self.camera_image).render_ref(area, buf)
            }
            RatatuiCameraStrategy::Luminance(ref strategy_config) => {
                RatatuiRenderWidgetLuminance::new(
                    &self.camera_image,
                    &self.sobel_image,
                    strategy_config,
                    &self.edge_detection,
                )
                .render_ref(area, buf);
            }
        }
    }
}
