use bevy::color::Luminance;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::camera::LuminanceConfig;
use crate::RatatuiCameraEdgeDetection;

pub struct RatatuiRenderWidgetLuminance {
    image: DynamicImage,
    image_sobel: Option<DynamicImage>,
    config: LuminanceConfig,
    edge_detection: Option<RatatuiCameraEdgeDetection>,
}

impl RatatuiRenderWidgetLuminance {
    pub fn new(
        image: DynamicImage,
        image_sobel: Option<DynamicImage>,
        config: LuminanceConfig,
        edge_detection: Option<RatatuiCameraEdgeDetection>,
    ) -> Self {
        Self {
            image,
            image_sobel,
            config,
            edge_detection,
        }
    }
}

impl WidgetRef for RatatuiRenderWidgetLuminance {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let Self {
            image,
            image_sobel,
            config,
            edge_detection,
        } = self;

        let image = image.resize(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Nearest,
        );

        let render_area = Rect {
            x: area.x + area.width.saturating_sub(image.width() as u16) / 2,
            y: area.y + (area.height).saturating_sub(image.height() as u16 / 2) / 2,
            width: image.width() as u16,
            height: image.height() as u16 / 2,
        };

        let color_characters = convert_image_to_color_characters(
            &image,
            &config.luminance_characters,
            config.luminance_scale,
        );

        let image_sobel = image_sobel.as_ref().map(|image_sobel| {
            image_sobel.resize(
                area.width as u32,
                area.height as u32 * 2,
                FilterType::Nearest,
            )
        });

        for (index, (mut character, color)) in color_characters.iter().enumerate() {
            let x = index as u16 % image.width() as u16;
            let y = index as u16 / image.width() as u16;
            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            if let Some(ref image_sobel) = image_sobel {
                let Some(edge_config) = edge_detection else {
                    return;
                };

                let sobel_value = image_sobel.get_pixel(x as u32, y as u32 * 2);

                match edge_config.edge_characters {
                    crate::EdgeCharacters::Directional {
                        vertical,
                        horizontal,
                        forward_diagonal,
                        backward_diagonal,
                    } => {
                        let is_max_sobel = |current: u8| {
                            sobel_value
                                .0
                                .iter()
                                .all(|val| (current > 0) && (current >= *val))
                        };

                        if is_max_sobel(sobel_value[0]) {
                            character = vertical;
                        } else if is_max_sobel(sobel_value[1]) {
                            character = horizontal;
                        } else if is_max_sobel(sobel_value[2]) {
                            character = forward_diagonal;
                        } else if is_max_sobel(sobel_value[3]) {
                            character = backward_diagonal;
                        }
                    }
                    crate::EdgeCharacters::Single(edge_character) => {
                        if sobel_value.0.iter().any(|val| *val > 0) {
                            character = edge_character;
                        }
                    }
                }
            };

            if let Some(cell) = buf.cell_mut((render_area.x + x, render_area.y + y)) {
                cell.set_fg(*color).set_char(character);
            }
        }
    }
}

fn convert_image_to_color_characters(
    image: &DynamicImage,
    luminance_characters: &[char],
    luminance_scale: f32,
) -> Vec<(char, Color)> {
    let rgb_triplets = convert_image_to_rgb_triplets(image);
    let characters = rgb_triplets
        .iter()
        .map(|rgb| convert_rgb_triplet_to_character(rgb, luminance_characters, luminance_scale));
    let colors = rgb_triplets
        .iter()
        .map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2]));

    characters.zip(colors).collect()
}

fn convert_image_to_rgb_triplets(image: &DynamicImage) -> Vec<[u8; 3]> {
    let mut rgb_triplets = vec![[0; 3]; (image.width() * image.height().div_ceil(2)) as usize];

    for (y, row) in image.to_rgb8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgb_triplets[position] = pixel.0;
            } else {
                rgb_triplets[position][0] =
                    (rgb_triplets[position][0].saturating_add(pixel[0])) / 2;
                rgb_triplets[position][1] =
                    (rgb_triplets[position][1].saturating_add(pixel[1])) / 2;
                rgb_triplets[position][2] =
                    (rgb_triplets[position][2].saturating_add(pixel[2])) / 2;
            }
        }
    }

    rgb_triplets
}

fn convert_rgb_triplet_to_character(
    rgb_triplet: &[u8; 3],
    luminance_characters: &[char],
    luminance_scale: f32,
) -> char {
    let luminance =
        bevy::color::Color::srgb_u8(rgb_triplet[0], rgb_triplet[1], rgb_triplet[2]).luminance();
    let scaled_luminance = (luminance * luminance_scale).min(1.0);
    let character_index = ((scaled_luminance * luminance_characters.len() as f32) as usize)
        .min(luminance_characters.len() - 1);

    let Some(character) = luminance_characters.get(character_index) else {
        return ' ';
    };

    *character
}
