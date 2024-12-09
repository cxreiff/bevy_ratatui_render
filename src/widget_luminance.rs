use bevy::color::Luminance;
use image::imageops::FilterType;
use image::DynamicImage;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

pub struct RatatuiRenderWidgetLuminance {
    image: DynamicImage,
    sobel: Option<DynamicImage>,
    config: LuminanceConfig,
}

impl RatatuiRenderWidgetLuminance {
    pub fn new(image: DynamicImage, sobel: Option<DynamicImage>, config: LuminanceConfig) -> Self {
        Self {
            image,
            sobel,
            config,
        }
    }
}

impl WidgetRef for RatatuiRenderWidgetLuminance {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let Self {
            image,
            sobel,
            config,
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

        if let Some(_sobel) = sobel {
            // TODO: handle replacing characters with line characters based on sobel filter.
        }

        for (index, (character, color)) in color_characters.iter().enumerate() {
            let x = index as u16 % image.width() as u16;
            let y = index as u16 / image.width() as u16;
            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            buf.cell_mut((render_area.x + x, render_area.y + y))
                .map(|cell| cell.set_fg(*color).set_char(*character));
        }
    }
}

#[derive(Clone)]
pub struct LuminanceConfig {
    pub luminance_characters: Vec<char>,
    pub luminance_scale: f32,
    pub edge_detection: bool,
}

impl Default for LuminanceConfig {
    fn default() -> Self {
        Self {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_DEFAULT.into(),
            luminance_scale: LuminanceConfig::LUMINANCE_SCALE_DEFAULT,
            edge_detection: false,
        }
    }
}

impl LuminanceConfig {
    pub const LUMINANCE_CHARACTERS_DEFAULT: &'static [char] =
        &[' ', '.', ':', '+', '=', '!', '*', '?', '#', '%', '&', '@'];

    pub const LUMINANCE_CHARACTERS_BRAILLE: &'static [char] =
        &['⠁', '⠉', '⠋', '⠛', '⠟', '⠿', '⡿', '⣿'];

    const LUMINANCE_SCALE_DEFAULT: f32 = 9.;
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
                rgb_triplets[position][0] = (rgb_triplets[position][0] + pixel[0]) / 2;
                rgb_triplets[position][1] = (rgb_triplets[position][1] + pixel[1]) / 2;
                rgb_triplets[position][2] = (rgb_triplets[position][2] + pixel[2]) / 2;
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
