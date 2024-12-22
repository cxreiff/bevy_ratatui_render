use bevy::{prelude::*, render::extract_component::ExtractComponent};
use ratatui::style::Color;

/// Spawn this component with your bevy camera in order to send each frame's rendered image to
/// a RatatuiCameraWidget that will be inserted into the same camera entity.
///
#[derive(Component, Clone)]
pub struct RatatuiCamera {
    /// Dimensions (width, height) of the image the camera will render to.
    pub dimensions: (u32, u32),

    /// If true, the rendered image dimensions will be resized to match the size and aspect ratio
    /// of the terminal window (at startup and whenever a terminal resize event is received).
    pub autoresize: bool,

    /// When autoresize is true, this function will be used to transform the new terminal
    /// dimensions into the rendered image dimensions. For example, use `|(w, h)| (w*4, h*3)` to
    /// maintain a 4:3 aspect ratio.
    pub autoresize_function: fn((u32, u32)) -> (u32, u32),

    /// Specify the strategy used for converting the rendered image to unicode characters.
    pub strategy: RatatuiCameraStrategy,
}

impl Default for RatatuiCamera {
    fn default() -> Self {
        Self {
            dimensions: (256, 256),
            autoresize: false,
            autoresize_function: |(w, h)| (w * 2, h * 2),
            strategy: RatatuiCameraStrategy::default(),
        }
    }
}

/// Available strategies that can be used for converting the rendered image to unicode characters
/// in the terminal buffer.
///
#[derive(Default, Clone)]
pub enum RatatuiCameraStrategy {
    /// Print to the terminal using unicode halfblock characters. By using both the halfblock
    /// (foreground) color and the background color, we can draw two pixels per buffer cell.
    #[default]
    HalfBlocks,

    /// Given a range of unicode characters sorted in increasing order of opacity, use each pixel's
    /// luminance to select a character from the range.
    Luminance(LuminanceConfig),
}

/// Configuration for the RatatuiCameraStrategy::Luminance terminal rendering strategy.
///
/// # Example:
///
/// The following would configure the plugin to use ' ' and '.' for dimmer areas, use '+' and '#'
/// for brighter areas, and multiply each pixel's luminance value by 5.0:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_render::{RatatuiCamera, RatatuiCameraStrategy, LuminanceConfig};
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// #   commands.spawn(
/// RatatuiCamera {
///     strategy: RatatuiCameraStrategy::Luminance(LuminanceConfig {
///         luminance_characters: vec![' ', '.', '+', '#'],
///         luminance_scale: 5.0,
///     }),
///     ..default()
/// }
/// #   );
/// # };
/// ```
///
#[derive(Clone)]
pub struct LuminanceConfig {
    /// The list of characters, in increasing order of opacity, to use for printing. For example,
    /// put an '@' symbol after a '+' symbol because it is more "opaque", taking up more space in
    /// the cell it is printed in, and so when printed in bright text on a dark background, it
    /// appears to be "brighter".
    pub luminance_characters: Vec<char>,

    /// The number that each luminance value is multiplied by before being used to select
    /// a character. Because most scenes do not occupy the full range of luminance between 0.0 and
    /// 1.0, each luminance value is multiplied by a scaling value first.
    pub luminance_scale: f32,
}

impl LuminanceConfig {
    /// A range of braille unicode characters in increasing order of opacity.
    pub const LUMINANCE_CHARACTERS_BRAILLE: &'static [char] =
        &[' ', '⠂', '⠒', '⠖', '⠶', '⠷', '⠿', '⡿', '⣿'];

    /// A range of miscellaneous characters in increasing order of opacity.
    pub const LUMINANCE_CHARACTERS_MISC: &'static [char] =
        &[' ', '.', ':', '+', '=', '!', '*', '?', '#', '%', '&', '@'];

    /// A range of block characters in increasing order of opacity.
    pub const LUMINANCE_CHARACTERS_SHADING: &'static [char] = &[' ', '░', '▒', '▓', '█'];

    /// The default scaling value to multiply pixel luminance by.
    const LUMINANCE_SCALE_DEFAULT: f32 = 9.;
}

impl Default for LuminanceConfig {
    fn default() -> Self {
        Self {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_BRAILLE.into(),
            luminance_scale: LuminanceConfig::LUMINANCE_SCALE_DEFAULT,
        }
    }
}

/// When spawned with a RatatuiCamera, an edge detection step will run in the render pipeline, and
/// detected edges will be handled differently by each image to unicode character conversion
/// strategy. The edge detection is performed via a sobel filter convolved over the depth, normal,
/// and color textures generated during rendering, resulting in a new texture of detected edges
/// and their directions (horizontal, vertical, both diagonals). Where edges are detected, special
/// characters and optionally an override color can be used.
///
#[derive(Component, ExtractComponent, Clone, Copy)]
pub struct RatatuiCameraEdgeDetection {
    /// Width of the range used for detecting edges. Higher thickness value means a wider edge.
    pub thickness: f32,

    /// Enable using the color texture to detect edges.
    pub color_enabled: bool,
    /// Threshold for edge severity required for an edge to be detected in the color texture.
    pub color_threshold: f32,

    /// Enable using the depth texture to detect edges.
    pub depth_enabled: bool,
    /// Threshold for edge severity required for an edge to be detected in the depth texture.
    pub depth_threshold: f32,

    /// Enable using the normal texture to detect edges.
    pub normal_enabled: bool,
    /// Threshold for edge severity required for an edge to be detected in the normal texture.
    pub normal_threshold: f32,

    /// The unicode characters used for rendering edges in the terminal buffer.
    pub edge_characters: EdgeCharacters,
    /// An override color that replaces the rendered color when an edge is detected.
    pub edge_color: Option<Color>,
}

impl Default for RatatuiCameraEdgeDetection {
    fn default() -> Self {
        Self {
            thickness: 1.4,

            color_enabled: true,
            color_threshold: 0.2,

            depth_enabled: true,
            depth_threshold: 0.05,

            normal_enabled: true,
            normal_threshold: 0.2,

            edge_characters: EdgeCharacters::default(),
            edge_color: None,
        }
    }
}

/// Specify how to handle rendering detected edges as unicode characters.
///
#[derive(Clone, Copy)]
pub enum EdgeCharacters {
    /// Each character in a detected edge will be shown as a specified character.
    Single(char),

    /// Each character in a detected edge will be shown as one of four specified characters based
    /// on the dominant direction of the detected edge.
    Directional {
        vertical: char,
        horizontal: char,
        forward_diagonal: char,
        backward_diagonal: char,
    },
}

impl Default for EdgeCharacters {
    fn default() -> Self {
        Self::Single('+')
    }
}

impl EdgeCharacters {
    pub fn lines() -> Self {
        Self::Directional {
            vertical: '|',
            horizontal: '―',
            forward_diagonal: '⟋',
            backward_diagonal: '⟍',
        }
    }
}
