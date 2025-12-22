//! Styled image rendering for copy-as-image feature
//!
//! This module renders styled text (with syntax highlighting) to an image
//! that can be copied to the clipboard.

use crate::primitives::highlighter::HighlightSpan;
use crate::view::theme::Theme;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};
use ratatui::style::Color;

/// Embedded Fira Mono font for consistent rendering (SIL Open Font License)
const FONT_BYTES: &[u8] = include_bytes!("../assets/FiraMono-Regular.ttf");

/// Configuration for rendering styled text to an image
#[derive(Debug, Clone)]
pub struct StyledImageConfig {
    /// Font size in pixels
    pub font_size: f32,
    /// Horizontal padding in pixels
    pub padding_x: u32,
    /// Vertical padding in pixels
    pub padding_y: u32,
    /// Line height multiplier (1.0 = tight, 1.5 = more spacing)
    pub line_height: f32,
}

impl Default for StyledImageConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            padding_x: 16,
            padding_y: 12,
            line_height: 1.4,
        }
    }
}

/// Result of rendering styled text to an image
pub struct StyledImageResult {
    /// Width of the image in pixels
    pub width: u32,
    /// Height of the image in pixels
    pub height: u32,
    /// RGBA pixel data (4 bytes per pixel)
    pub rgba_bytes: Vec<u8>,
}

/// Convert a ratatui Color to an RGBA color
fn color_to_rgba(color: Color, default: Rgba<u8>) -> Rgba<u8> {
    match color {
        Color::Rgb(r, g, b) => Rgba([r, g, b, 255]),
        Color::Black => Rgba([0, 0, 0, 255]),
        Color::Red => Rgba([205, 49, 49, 255]),
        Color::Green => Rgba([13, 188, 121, 255]),
        Color::Yellow => Rgba([229, 229, 16, 255]),
        Color::Blue => Rgba([36, 114, 200, 255]),
        Color::Magenta => Rgba([188, 63, 188, 255]),
        Color::Cyan => Rgba([17, 168, 205, 255]),
        Color::Gray => Rgba([128, 128, 128, 255]),
        Color::DarkGray => Rgba([80, 80, 80, 255]),
        Color::LightRed => Rgba([241, 76, 76, 255]),
        Color::LightGreen => Rgba([35, 209, 139, 255]),
        Color::LightYellow => Rgba([245, 245, 67, 255]),
        Color::LightBlue => Rgba([59, 142, 234, 255]),
        Color::LightMagenta => Rgba([214, 112, 214, 255]),
        Color::LightCyan => Rgba([41, 184, 219, 255]),
        Color::White => Rgba([229, 229, 229, 255]),
        Color::Reset | Color::Indexed(_) => default,
    }
}

/// Render styled text with syntax highlighting to an RGBA image
///
/// # Arguments
/// * `text` - The text to render
/// * `highlight_spans` - Syntax highlighting spans with byte ranges and colors
/// * `theme` - The theme to use for background and default foreground colors
/// * `config` - Rendering configuration
///
/// # Returns
/// A `StyledImageResult` containing the image dimensions and RGBA bytes
pub fn render_styled_text(
    text: &str,
    highlight_spans: &[HighlightSpan],
    theme: &Theme,
    config: &StyledImageConfig,
) -> StyledImageResult {
    // Load the font
    let font = FontRef::try_from_slice(FONT_BYTES).expect("Failed to load embedded font");
    let scale = PxScale::from(config.font_size);
    let scaled_font = font.as_scaled(scale);

    // Calculate character dimensions (monospace font)
    let char_width = scaled_font.h_advance(scaled_font.glyph_id('M'));
    let line_height = (config.font_size * config.line_height).ceil() as u32;
    let ascent = scaled_font.ascent();

    // Split text into lines and calculate dimensions
    let lines: Vec<&str> = text.lines().collect();
    let max_line_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    // Handle empty text
    if lines.is_empty() || max_line_len == 0 {
        return StyledImageResult {
            width: 1,
            height: 1,
            rgba_bytes: vec![0, 0, 0, 255],
        };
    }

    let width = (max_line_len as f32 * char_width).ceil() as u32 + config.padding_x * 2;
    let height = (lines.len() as u32) * line_height + config.padding_y * 2;

    // Create image with background color
    let bg_color = color_to_rgba(theme.editor_bg, Rgba([30, 30, 30, 255]));
    let fg_color = color_to_rgba(theme.editor_fg, Rgba([212, 212, 212, 255]));
    let mut img = RgbaImage::from_pixel(width, height, bg_color);

    // Build a map of byte offset to color for quick lookup
    let mut color_map: Vec<Option<Color>> = vec![None; text.len()];
    for span in highlight_spans {
        let start = span.range.start.min(text.len());
        let end = span.range.end.min(text.len());
        for i in start..end {
            color_map[i] = Some(span.color);
        }
    }

    // Render each line
    let mut byte_offset = 0;
    for (line_idx, line) in lines.iter().enumerate() {
        let y_baseline = config.padding_y as f32 + (line_idx as u32 * line_height) as f32 + ascent;
        let mut x = config.padding_x as f32;

        for ch in line.chars() {
            let char_byte_len = ch.len_utf8();

            // Get color for this character
            let color = if byte_offset < color_map.len() {
                color_map[byte_offset]
                    .map(|c| color_to_rgba(c, fg_color))
                    .unwrap_or(fg_color)
            } else {
                fg_color
            };

            // Draw the character
            draw_char(&mut img, &scaled_font, ch, x, y_baseline, color);

            x += char_width;
            byte_offset += char_byte_len;
        }

        // Account for newline character
        byte_offset += 1;
    }

    StyledImageResult {
        width,
        height,
        rgba_bytes: img.into_raw(),
    }
}

/// Draw a single character onto the image
fn draw_char<F: Font>(
    img: &mut RgbaImage,
    font: &ab_glyph::PxScaleFont<&F>,
    ch: char,
    x: f32,
    y_baseline: f32,
    color: Rgba<u8>,
) {
    let glyph_id = font.glyph_id(ch);
    let glyph = glyph_id.with_scale_and_position(font.scale(), ab_glyph::point(x, y_baseline));

    if let Some(outlined) = font.outline_glyph(glyph) {
        let bounds = outlined.px_bounds();
        outlined.draw(|px, py, coverage| {
            let img_x = (bounds.min.x as i32 + px as i32) as u32;
            let img_y = (bounds.min.y as i32 + py as i32) as u32;

            if img_x < img.width() && img_y < img.height() {
                let alpha = (coverage * 255.0) as u8;
                if alpha > 0 {
                    let bg = img.get_pixel(img_x, img_y);
                    let blended = blend_pixel(*bg, color, alpha);
                    img.put_pixel(img_x, img_y, blended);
                }
            }
        });
    }
}

/// Blend a foreground color onto a background with alpha
fn blend_pixel(bg: Rgba<u8>, fg: Rgba<u8>, alpha: u8) -> Rgba<u8> {
    let a = alpha as f32 / 255.0;
    let inv_a = 1.0 - a;

    Rgba([
        (fg[0] as f32 * a + bg[0] as f32 * inv_a) as u8,
        (fg[1] as f32 * a + bg[1] as f32 * inv_a) as u8,
        (fg[2] as f32 * a + bg[2] as f32 * inv_a) as u8,
        255,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_text() {
        let text = "Hello, World!";
        let spans = vec![];
        let theme = Theme::dark();
        let config = StyledImageConfig::default();

        let result = render_styled_text(text, &spans, &theme, &config);

        assert!(result.width > 0);
        assert!(result.height > 0);
        assert!(!result.rgba_bytes.is_empty());
        assert_eq!(
            result.rgba_bytes.len(),
            (result.width * result.height * 4) as usize
        );
    }

    #[test]
    fn test_render_multiline_text() {
        let text = "line 1\nline 2\nline 3";
        let spans = vec![];
        let theme = Theme::dark();
        let config = StyledImageConfig::default();

        let result = render_styled_text(text, &spans, &theme, &config);

        assert!(result.width > 0);
        assert!(result.height > 0);
    }

    #[test]
    fn test_render_empty_text() {
        let text = "";
        let spans = vec![];
        let theme = Theme::dark();
        let config = StyledImageConfig::default();

        let result = render_styled_text(text, &spans, &theme, &config);

        // Should return a minimal 1x1 image
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 1);
    }

    #[test]
    fn test_color_conversion() {
        let default = Rgba([0, 0, 0, 255]);

        assert_eq!(color_to_rgba(Color::Black, default), Rgba([0, 0, 0, 255]));
        assert_eq!(
            color_to_rgba(Color::Rgb(100, 150, 200), default),
            Rgba([100, 150, 200, 255])
        );
    }
}
