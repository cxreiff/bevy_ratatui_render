use bevy::{prelude::*, render::extract_component::ExtractComponent};

use crate::{camera_node, camera_node_sobel, camera_readback};

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

#[derive(Clone)]
pub struct LuminanceConfig {
    pub luminance_characters: Vec<char>,
    pub luminance_scale: f32,
}

impl LuminanceConfig {
    pub const LUMINANCE_CHARACTERS_DEFAULT: &'static [char] =
        &[' ', '.', ':', '+', '=', '!', '*', '?', '#', '%', '&', '@'];

    pub const LUMINANCE_CHARACTERS_BRAILLE: &'static [char] =
        &[' ', '⠁', '⠉', '⠋', '⠛', '⠟', '⠿', '⡿', '⣿'];

    pub const LUMINANCE_CHARACTERS_SHADING: &'static [char] = &[' ', '░', '▒', '▓', '█'];

    const LUMINANCE_SCALE_DEFAULT: f32 = 9.;
}

impl Default for LuminanceConfig {
    fn default() -> Self {
        Self {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_DEFAULT.into(),
            luminance_scale: LuminanceConfig::LUMINANCE_SCALE_DEFAULT,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, Copy)]
pub struct RatatuiCameraEdgeDetection {
    pub thickness: f32,

    pub color_enabled: bool,
    pub color_threshold: f32,

    pub depth_enabled: bool,
    pub depth_threshold: f32,

    pub normal_enabled: bool,
    pub normal_threshold: f32,

    // TODO: add config for controlling edge characters, but replace ExtractComponentPlugin with
    // custom system that creates ShaderType version of config and inserts that instead.
    pub edge_characters: EdgeCharacters,
}

impl Default for RatatuiCameraEdgeDetection {
    fn default() -> Self {
        Self {
            thickness: 1.4,

            color_enabled: true,
            color_threshold: 0.2,

            depth_enabled: true,
            depth_threshold: 0.2,

            normal_enabled: true,
            normal_threshold: 0.2,

            edge_characters: EdgeCharacters::Directional {
                vertical: '|',
                horizontal: '―',
                forward_diagonal: '⟋',
                backward_diagonal: '⟍',
            },
        }
    }
}

#[derive(Clone, Copy)]
pub enum EdgeCharacters {
    Directional {
        vertical: char,
        horizontal: char,
        forward_diagonal: char,
        backward_diagonal: char,
    },
    Single(char),
}
