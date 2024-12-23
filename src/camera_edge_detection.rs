use bevy::{prelude::*, render::extract_component::ExtractComponent};

/// When spawned with a RatatuiCamera, an edge detection step will run in the render pipeline, and
/// detected edges will be handled differently by each image to unicode character conversion
/// strategy. The edge detection is performed via a sobel filter convolved over the depth, normal,
/// and color textures generated during rendering, resulting in a new texture of detected edges
/// and their directions (horizontal, vertical, both diagonals). Where edges are detected, special
/// characters and optionally an override color can be used.
///
/// Currently just works with `RatatuiCameraStrategy::Luminance` and 3d cameras.
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
    pub edge_color: Option<ratatui::style::Color>,
}

impl Default for RatatuiCameraEdgeDetection {
    fn default() -> Self {
        Self {
            thickness: 2.0,

            color_enabled: true,
            color_threshold: 0.4,

            depth_enabled: true,
            depth_threshold: 0.1,

            normal_enabled: true,
            normal_threshold: 2.5,

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
        Self::Directional {
            vertical: '|',
            horizontal: '―',
            forward_diagonal: '⟋',
            backward_diagonal: '⟍',
        }
    }
}
