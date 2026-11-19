//! Texto con la tipografía incluida (Rajdhani, OFL, ver `assets/OFL.txt`).
//!
//! Rajdhani es una condensada de trazo recto: números y etiquetas se leen a tamaño pequeño sin
//! competir con la interfaz del juego.

use mh_overlay_config::Color;

use crate::OverlayError;
use crate::canvas::Canvas;

/// Bytes de la fuente, embebida para no depender de las fuentes del sistema.
const FONT: &[u8] = include_bytes!("../assets/Rajdhani-SemiBold.ttf");

/// La misma tipografía del panel, para que el hub no tenga un aspecto distinto.
pub fn font_bytes() -> &'static [u8] {
    FONT
}

pub struct Text {
    font: fontdue::Font,
}

impl Text {
    pub fn bundled() -> Result<Self, OverlayError> {
        let font = fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default())
            .map_err(|error| OverlayError::Font(error.to_string()))?;

        Ok(Self { font })
    }

    /// Ancho de una línea: la suma de los avances de sus glifos.
    pub fn width(&self, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|character| self.font.rasterize(character, size).0.advance_width)
            .sum()
    }

    /// Alto de línea recomendado para este tamaño.
    pub fn height(&self, size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(size)
            .map(|metrics| metrics.new_line_size)
            .unwrap_or(size * 1.2)
    }

    /// Distancia de la línea base al borde superior de una línea.
    pub fn ascent(&self, size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(size)
            .map(|metrics| metrics.ascent)
            .unwrap_or(size * 0.8)
    }

    /// Dibuja `text` con la línea base en `baseline`; `x` es el borde izquierdo del primer glifo.
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: f32,
        baseline: f32,
        size: f32,
        color: Color,
    ) {
        let mut cursor = x;

        for character in text.chars() {
            let (metrics, coverage) = self.font.rasterize(character, size);

            for row in 0..metrics.height {
                for column in 0..metrics.width {
                    let alpha = coverage[row * metrics.width + column];
                    if alpha == 0 {
                        continue;
                    }

                    let pixel_x = cursor.round() as i32 + metrics.xmin + column as i32;
                    let pixel_y =
                        baseline.round() as i32 - metrics.ymin - metrics.height as i32 + row as i32;

                    canvas.blend(pixel_x, pixel_y, color, f32::from(alpha) / 255.0);
                }
            }

            cursor += metrics.advance_width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_font_loads() {
        let text = Text::bundled().unwrap();

        assert!(text.width("Rathalos", 16.0) > 0.0);
    }

    #[test]
    fn a_longer_string_is_wider() {
        let text = Text::bundled().unwrap();

        assert!(text.width("Rathalos", 16.0) > text.width("Ra", 16.0));
    }

    #[test]
    fn a_bigger_size_is_taller() {
        let text = Text::bundled().unwrap();

        assert!(text.height(32.0) > text.height(16.0));
    }

    #[test]
    fn drawing_text_changes_pixels() {
        let text = Text::bundled().unwrap();
        let mut canvas = Canvas::new(120, 40);

        text.draw(
            &mut canvas,
            "75%",
            4.0,
            28.0,
            20.0,
            Color::rgb(255, 255, 255),
        );

        assert!(
            canvas.pixels().iter().any(|pixel| *pixel != 0xff00_0000),
            "el texto no tocó ningún píxel"
        );
    }
}
