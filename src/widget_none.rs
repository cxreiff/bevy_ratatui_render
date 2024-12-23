use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::RatatuiCameraEdgeDetection;

pub struct RatatuiCameraWidgetNone<'a> {
    camera_image: &'a DynamicImage,
    sobel_image: &'a Option<DynamicImage>,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetNone<'a> {
    pub fn new(
        camera_image: &'a DynamicImage,
        sobel_image: &'a Option<DynamicImage>,
        edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
    ) -> Self {
        Self {
            camera_image,
            sobel_image,
            edge_detection,
        }
    }
}

impl WidgetRef for RatatuiCameraWidgetNone<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let Self {
            camera_image,
            sobel_image,
            edge_detection,
        } = self;

        let (Some(sobel_image), Some(edge_detection)) = (sobel_image, edge_detection) else {
            return;
        };

        let camera_image = camera_image.resize(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Nearest,
        );

        let render_area = Rect {
            x: area.x + area.width.saturating_sub(camera_image.width() as u16) / 2,
            y: area.y + (area.height).saturating_sub(camera_image.height() as u16 / 2) / 2,
            width: camera_image.width() as u16,
            height: camera_image.height() as u16 / 2,
        };

        let mut color_characters = convert_image_to_colors(&camera_image);

        let sobel_image = sobel_image.resize(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Nearest,
        );

        for (index, color) in color_characters.iter_mut().enumerate() {
            let mut character = ' ';
            let x = index as u16 % camera_image.width() as u16;
            let y = index as u16 / camera_image.width() as u16;
            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            if !sobel_image.in_bounds(x as u32, y as u32 * 2) {
                continue;
            }

            let sobel_value = sobel_image.get_pixel(x as u32, y as u32 * 2);

            match edge_detection.edge_characters {
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
                        *color = edge_detection.edge_color.unwrap_or(*color);
                    } else if is_max_sobel(sobel_value[1]) {
                        character = horizontal;
                        *color = edge_detection.edge_color.unwrap_or(*color);
                    } else if is_max_sobel(sobel_value[2]) {
                        character = forward_diagonal;
                        *color = edge_detection.edge_color.unwrap_or(*color);
                    } else if is_max_sobel(sobel_value[3]) {
                        character = backward_diagonal;
                        *color = edge_detection.edge_color.unwrap_or(*color);
                    }
                }
                crate::EdgeCharacters::Single(edge_character) => {
                    if sobel_value.0.iter().any(|val| *val > 0) {
                        character = edge_character;
                        *color = edge_detection.edge_color.unwrap_or(*color);
                    }
                }
            }

            if let Some(cell) = buf.cell_mut((render_area.x + x, render_area.y + y)) {
                cell.set_fg(*color).set_char(character);
            }
        }
    }
}

fn convert_image_to_colors(camera_image: &DynamicImage) -> Vec<Color> {
    let rgb_triplets = convert_image_to_rgb_triplets(camera_image);
    let colors = rgb_triplets
        .iter()
        .map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2]));

    colors.collect()
}

fn convert_image_to_rgb_triplets(camera_image: &DynamicImage) -> Vec<[u8; 3]> {
    let mut rgb_triplets =
        vec![[0; 3]; (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for (y, row) in camera_image.to_rgb8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (camera_image.width() as usize) * (y / 2);
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
