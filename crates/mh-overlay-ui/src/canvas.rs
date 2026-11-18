//! Lienzo de píxeles en memoria: aquí se dibuja todo, sin X de por medio.
//!
//! Los píxeles son ARGB32 opacos. La translucidez del panel se compone sobre negro antes de
//! subir la imagen, así que el resultado no depende de que haya un compositor: el redondeo de
//! las esquinas lo aporta la forma (SHAPE) de la ventana, no el alfa.

use mh_overlay_config::Color;

/// Rectángulo en píxeles, con origen en la esquina superior izquierda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    pub fn inset(self, horizontal: i32, vertical: i32) -> Self {
        Self {
            x: self.x + horizontal,
            y: self.y + vertical,
            width: self.width.saturating_sub(2 * horizontal.max(0) as u32),
            height: self.height.saturating_sub(2 * vertical.max(0) as u32),
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }
}

/// Imagen ARGB32 lista para subir a la ventana.
pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Canvas {
    /// Lienzo negro opaco: el panel se compone encima, así el alfa del TOML solo oscurece.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0xff00_0000; (width * height) as usize],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn pixel(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        Some(self.pixels[(y as u32 * self.width + x as u32) as usize])
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                self.blend(x, y, color, 1.0);
            }
        }
    }

    /// Rectángulo con esquinas redondeadas y bordes suavizados por muestreo de cobertura.
    pub fn fill_rounded(&mut self, rect: Rect, radius: f32, color: Color) {
        let radius = radius
            .max(0.0)
            .min(rect.width.min(rect.height) as f32 / 2.0);

        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                let coverage = rounded_rect_coverage(x, y, rect, radius);
                self.blend(x, y, color, coverage);
            }
        }
    }

    /// Rectángulo redondeado con degradado vertical, para que las barras tengan volumen.
    pub fn fill_rounded_gradient(&mut self, rect: Rect, radius: f32, top: Color, bottom: Color) {
        let radius = radius
            .max(0.0)
            .min(rect.width.min(rect.height) as f32 / 2.0);
        let height = rect.height.max(1) as f32;

        for y in rect.y..rect.bottom() {
            let t = ((y - rect.y) as f32 / height).clamp(0.0, 1.0);
            let color = Color {
                r: lerp(top.r, bottom.r, t),
                g: lerp(top.g, bottom.g, t),
                b: lerp(top.b, bottom.b, t),
                a: lerp(top.a, bottom.a, t),
            };

            for x in rect.x..rect.right() {
                let coverage = rounded_rect_coverage(x, y, rect, radius);
                self.blend(x, y, color, coverage);
            }
        }
    }

    /// Trazo entre dos puntos con extremos redondeados, para iconos y marcas finas.
    pub fn fill_line(&mut self, from: (f32, f32), to: (f32, f32), thickness: f32, color: Color) {
        let radius = (thickness / 2.0).max(0.5);
        let left = from.0.min(to.0) - radius - 1.0;
        let right = from.0.max(to.0) + radius + 1.0;
        let top = from.1.min(to.1) - radius - 1.0;
        let bottom = from.1.max(to.1) + radius + 1.0;

        for y in top.floor() as i32..=bottom.ceil() as i32 {
            for x in left.floor() as i32..=right.ceil() as i32 {
                let center = (x as f32 + 0.5, y as f32 + 0.5);
                let distance = distance_to_segment(center, from, to);
                self.blend(x, y, color, (radius + 0.5 - distance).clamp(0.0, 1.0));
            }
        }
    }

    /// Rombo relleno centrado en `center`; marca los niveles del medidor de espíritu.
    pub fn fill_diamond(&mut self, center: (f32, f32), radius: f32, color: Color) {
        let radius = radius.max(0.5);
        let reach = radius.ceil() as i32 + 1;
        let (cx, cy) = (center.0.floor() as i32, center.1.floor() as i32);

        for y in cy - reach..=cy + reach {
            for x in cx - reach..=cx + reach {
                let dx = (x as f32 + 0.5 - center.0).abs();
                let dy = (y as f32 + 0.5 - center.1).abs();
                self.blend(x, y, color, (radius + 0.5 - (dx + dy)).clamp(0.0, 1.0));
            }
        }
    }

    /// Mezcla `color` sobre el píxel con una cobertura de 0 a 1 (bordes de texto y esquinas).
    pub fn blend(&mut self, x: i32, y: i32, color: Color, coverage: f32) {
        if coverage <= 0.0 || x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let alpha = (f32::from(color.a) / 255.0) * coverage.clamp(0.0, 1.0);
        if alpha >= 1.0 {
            self.pixels[(y as u32 * self.width + x as u32) as usize] =
                color.to_argb() | 0xff00_0000;
            return;
        }

        let index = (y as u32 * self.width + x as u32) as usize;
        let destination = self.pixels[index];
        let mix = |source: u8, target: u8| -> u32 {
            (f32::from(source) * alpha + f32::from(target) * (1.0 - alpha)).round() as u32
        };

        let r = mix(color.r, ((destination >> 16) & 0xff) as u8);
        let g = mix(color.g, ((destination >> 8) & 0xff) as u8);
        let b = mix(color.b, (destination & 0xff) as u8);

        self.pixels[index] = 0xff00_0000 | (r << 16) | (g << 8) | b;
    }

    /// Bytes en el orden que espera `PutImage` con un visual TrueColor de 24 bits en x86.
    pub fn to_pixmap_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);

        for pixel in &self.pixels {
            bytes.extend_from_slice(&(pixel & 0x00ff_ffff).to_le_bytes());
        }

        bytes
    }
}

/// Distancia de un punto al segmento `from`-`to`, en píxeles.
fn distance_to_segment(point: (f32, f32), from: (f32, f32), to: (f32, f32)) -> f32 {
    let (px, py) = point;
    let (x0, y0) = from;
    let (x1, y1) = to;
    let (dx, dy) = (x1 - x0, y1 - y0);
    let length = dx * dx + dy * dy;
    let t = if length <= f32::EPSILON {
        0.0
    } else {
        (((px - x0) * dx + (py - y0) * dy) / length).clamp(0.0, 1.0)
    };

    let nearest = (x0 + t * dx, y0 + t * dy);
    ((px - nearest.0).powi(2) + (py - nearest.1).powi(2)).sqrt()
}

fn lerp(from: u8, to: u8, t: f32) -> u8 {
    (f32::from(from) + (f32::from(to) - f32::from(from)) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

/// Cobertura de un píxel dentro de un rectángulo redondeado, calculada desde su distancia.
pub(crate) fn rounded_rect_coverage(x: i32, y: i32, rect: Rect, radius: f32) -> f32 {
    if radius <= 0.5 {
        return 1.0;
    }

    let center_x = x as f32 + 0.5;
    let center_y = y as f32 + 0.5;
    let left = rect.x as f32;
    let top = rect.y as f32;
    let right = rect.right() as f32;
    let bottom = rect.bottom() as f32;

    // Distancia al rectángulo "encogido": negativa dentro, positiva fuera.
    let inner_left = left + radius;
    let inner_right = right - radius;
    let inner_top = top + radius;
    let inner_bottom = bottom - radius;

    let clamped_x = center_x.clamp(inner_left, inner_right);
    let clamped_y = center_y.clamp(inner_top, inner_bottom);
    let distance = ((center_x - clamped_x).powi(2) + (center_y - clamped_y).powi(2)).sqrt();

    // Un píxel de transición en el borde.
    (radius + 0.5 - distance).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_canvas_is_opaque_black() {
        let canvas = Canvas::new(4, 3);

        assert_eq!(canvas.pixel(0, 0), Some(0xff00_0000));
        assert_eq!(canvas.pixel(3, 2), Some(0xff00_0000));
        assert_eq!(canvas.pixel(4, 0), None);
    }

    #[test]
    fn filling_a_rect_sets_its_pixels_and_leaves_the_rest() {
        let mut canvas = Canvas::new(10, 10);
        canvas.fill_rect(Rect::new(2, 2, 3, 3), Color::rgb(255, 0, 0));

        assert_eq!(canvas.pixel(2, 2), Some(0xffff_0000));
        assert_eq!(canvas.pixel(4, 4), Some(0xffff_0000));
        assert_eq!(canvas.pixel(5, 5), Some(0xff00_0000));
    }

    #[test]
    fn a_translucent_fill_darkens_the_background_instead_of_leaving_alpha() {
        let mut canvas = Canvas::new(2, 2);
        canvas.fill_rect(Rect::new(0, 0, 2, 2), Color::rgb(255, 0, 0).with_alpha(128));

        let pixel = canvas.pixel(0, 0).unwrap();
        assert_eq!(pixel >> 24, 0xff);
        assert!((0x60..=0xa0).contains(&((pixel >> 16) & 0xff)));
    }

    #[test]
    fn rounded_corners_are_cut_out() {
        let mut canvas = Canvas::new(40, 40);
        canvas.fill_rounded(Rect::new(0, 0, 40, 40), 12.0, Color::rgb(0, 255, 0));

        // La esquina queda fuera del radio; el centro, dentro.
        assert_eq!(canvas.pixel(0, 0), Some(0xff00_0000));
        assert_eq!(canvas.pixel(20, 20), Some(0xff00_ff00));
    }

    #[test]
    fn a_line_paints_between_its_ends_and_stops_there() {
        let mut canvas = Canvas::new(20, 20);
        canvas.fill_line((2.0, 10.0), (18.0, 10.0), 2.0, Color::rgb(0, 255, 0));

        assert_eq!(canvas.pixel(10, 10), Some(0xff00_ff00));
        assert_eq!(canvas.pixel(2, 10), Some(0xff00_ff00));
        assert_eq!(canvas.pixel(10, 2), Some(0xff00_0000));
    }

    #[test]
    fn a_diamond_is_wider_in_the_middle_than_at_its_corners() {
        let mut canvas = Canvas::new(20, 20);
        canvas.fill_diamond((10.0, 10.0), 5.0, Color::rgb(255, 0, 0));

        assert_eq!(canvas.pixel(10, 10), Some(0xffff_0000));
        assert_eq!(canvas.pixel(10, 13), Some(0xffff_0000));
        assert_eq!(canvas.pixel(6, 6), Some(0xff00_0000));
    }

    #[test]
    fn pixmap_bytes_drop_the_alpha_channel() {
        let mut canvas = Canvas::new(1, 1);
        canvas.fill_rect(Rect::new(0, 0, 1, 1), Color::rgb(0x12, 0x34, 0x56));

        assert_eq!(canvas.to_pixmap_bytes(), [0x56, 0x34, 0x12, 0x00]);
    }
}
