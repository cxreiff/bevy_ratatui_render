use bevy::prelude::*;

use crate::{camera_node, camera_node_sobel, camera_readback, LuminanceConfig};

pub struct RatatuiCameraPlugin;

impl Plugin for RatatuiCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera_readback::plugin,
            camera_node::plugin,
            camera_node_sobel::RatatuiCameraNodeSobelPlugin,
        ));
    }
}

#[derive(Component)]
pub struct RatatuiCamera {
    pub dimensions: (u32, u32),
    pub autoresize: bool,
    pub autoresize_function: fn((u32, u32)) -> (u32, u32),
    pub strategy: RatatuiCameraStrategy,
}

impl Default for RatatuiCamera {
    fn default() -> Self {
        Self {
            dimensions: (256, 256),
            autoresize: false,
            autoresize_function: |(width, height)| (width * 2, height * 2),
            strategy: RatatuiCameraStrategy::default(),
        }
    }
}

#[derive(Default, Clone)]
pub enum RatatuiCameraStrategy {
    #[default]
    HalfBlocks,
    Luminance(LuminanceConfig),
}
