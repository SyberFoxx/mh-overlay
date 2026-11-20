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

    /// Lienzo transparente: el fondo no se pinta y el alfa de cada trazo se conserva. Es lo que
    /// necesita el modo de barras sueltas, donde solo deben verse las placas.
    pub fn transparent(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
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

    /// Borra el interior de un rectángulo redondeado, para dejar solo un borde.
    ///
    /// Se usa en las placas del modo libre: pintar el borde y luego el fondo encima
    /// duplicaría el alfa y las volvería casi opacas.
    pub fn clear_rounded(&mut self, rect: Rect, radius: f32) {
        let radius = radius
            .max(0.0)
            .min(rect.width.min(rect.height) as f32 / 2.0);

        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                if rounded_rect_coverage(x, y, rect, radius) >= 0.5 {
                    self.pixels[(y as u32 * self.width + x as u32) as usize] = 0;
                }
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
    ///
    /// Es una composición source-over normal: sobre un lienzo opaco el resultado sigue siendo
    /// opaco, y sobre uno transparente el alfa se acumula en vez de perderse.
    pub fn blend(&mut self, x: i32, y: i32, color: Color, coverage: f32) {
        if coverage <= 0.0 || x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let source = (f32::from(color.a) / 255.0) * coverage.clamp(0.0, 1.0);
        if source <= 0.0 {
            return;
        }

        let index = (y as u32 * self.width + x as u32) as usize;
        let destination = self.pixels[index];
        let destination_alpha = f32::from((destination >> 24) as u8) / 255.0;
        let alpha = source + destination_alpha * (1.0 - source);

        if alpha <= f32::EPSILON {
            return;
        }

        let mix = |source_component: u8, destination_component: u8| -> u32 {
            ((f32::from(source_component) * source
                + f32::from(destination_component) * destination_alpha * (1.0 - source))
                / alpha)
                .round()
                .clamp(0.0, 255.0) as u32
        };

        let r = mix(color.r, ((destination >> 16) & 0xff) as u8);
        let g = mix(color.g, ((destination >> 8) & 0xff) as u8);
        let b = mix(color.b, (destination & 0xff) as u8);
        let a = (alpha * 255.0).round().clamp(0.0, 255.0) as u32;

        self.pixels[index] = (a << 24) | (r << 16) | (g << 8) | b;
    }

    /// Copia otro lienzo encima, respetando el alfa de cada píxel. Las placas del modo libre se
    /// componen así sobre el lienzo de la pantalla.
    pub fn composite(&mut self, other: &Canvas, x: i32, y: i32) {
        for row in 0..other.height as i32 {
            for column in 0..other.width as i32 {
                let pixel = other.pixels[(row as u32 * other.width + column as u32) as usize];
                let a = (pixel >> 24) as u8;

                if a == 0 {
                    continue;
                }

                self.blend(
                    x + column,
                    y + row,
                    Color {
                        r: ((pixel >> 16) & 0xff) as u8,
                        g: ((pixel >> 8) & 0xff) as u8,
                        b: (pixel & 0xff) as u8,
                        a,
                    },
                    1.0,
                );
            }
        }
    }

    /// Caja que rodea los píxeles con alfa; `None` si el lienzo está vacío.
    pub fn content_bounds(&self) -> Option<Rect> {
        let mut left = self.width;
        let mut top = self.height;
        let mut right = 0;
        let mut bottom = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                if (self.pixels[(y * self.width + x) as usize] >> 24) == 0 {
                    continue;
                }

                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }

        (right > left && bottom > top)
            .then(|| Rect::new(left as i32, top as i32, right - left, bottom - top))
    }

    /// Copia un rectángulo a un lienzo transparente; se usa para la vista previa recortada.
    pub fn crop(&self, rect: Rect) -> Canvas {
        let width = rect.width.min(self.width).max(1);
        let height = rect.height.min(self.height).max(1);
        let mut out = Canvas::transparent(width, height);

        for y in 0..height {
            for x in 0..width {
                let source_x = rect.x + x as i32;
                let source_y = rect.y + y as i32;

                if source_x < 0
                    || source_y < 0
                    || source_x >= self.width as i32
                    || source_y >= self.height as i32
                {
                    continue;
                }

                out.pixels[(y * width + x) as usize] =
                    self.pixels[(source_y as u32 * self.width + source_x as u32) as usize];
            }
        }

        out
    }

    /// Rectángulos de una scanline por fila donde el alfa supera `threshold`.
    ///
    /// X11 no compone alfa en un visual de 24 bits, así que el modo libre recorta la ventana a
    /// las placas con SHAPE. Es la misma idea que el recorte redondeado del panel.
    pub fn opaque_rectangles(&self, threshold: u8) -> Vec<Rect> {
        let mut rectangles = Vec::new();

        for y in 0..self.height {
            let mut span: Option<(u32, u32)> = None;

            for x in 0..self.width {
                let alpha = (self.pixels[(y * self.width + x) as usize] >> 24) as u8;
                let inside = alpha >= threshold;

                match (inside, span) {
                    (true, None) => span = Some((x, x + 1)),
                    (true, Some((start, _))) => span = Some((start, x + 1)),
                    (false, Some((start, end))) => {
                        rectangles.push(Rect::new(start as i32, y as i32, end - start, 1));
                        span = None;
                    }
                    (false, None) => {}
                }
            }

            if let Some((start, end)) = span {
                rectangles.push(Rect::new(start as i32, y as i32, end - start, 1));
            }
        }

        rectangles
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
    fn clearing_a_rounded_rect_leaves_only_its_border() {
        let mut canvas = Canvas::transparent(20, 20);
        canvas.fill_rounded(Rect::new(0, 0, 20, 20), 5.0, Color::rgb(255, 0, 0));
        canvas.clear_rounded(Rect::new(1, 1, 18, 18), 4.0);

        assert_eq!(canvas.pixel(0, 0), Some(0));
        assert_eq!(canvas.pixel(1, 10), Some(0));
        assert_eq!(canvas.pixel(0, 10), Some(0xffff_0000));
    }

    #[test]
    fn a_transparent_canvas_keeps_the_alpha_of_a_translucent_fill() {
        let mut canvas = Canvas::transparent(2, 2);
        canvas.fill_rect(Rect::new(0, 0, 2, 2), Color::rgb(255, 0, 0).with_alpha(128));

        let pixel = canvas.pixel(0, 0).unwrap();
        assert_eq!(pixel >> 24, 0x80);
        assert_eq!((pixel >> 16) & 0xff, 0xff);
    }

    #[test]
    fn compositing_a_plate_keeps_the_gap_transparent() {
        let mut screen = Canvas::transparent(8, 8);
        let mut plate = Canvas::transparent(3, 3);
        plate.fill_rect(Rect::new(0, 0, 3, 3), Color::rgb(0, 0, 255));
        screen.composite(&plate, 2, 3);

        assert_eq!(screen.pixel(0, 0), Some(0));
        assert_eq!(screen.pixel(3, 4), Some(0xff00_00ff));
        assert_eq!(screen.opaque_rectangles(1).len(), 3);
    }

    #[test]
    fn content_bounds_and_crop_keep_only_the_plates() {
        let mut screen = Canvas::transparent(10, 10);
        screen.fill_rect(Rect::new(3, 2, 4, 2), Color::rgb(255, 0, 0));

        let bounds = screen.content_bounds().unwrap();
        assert_eq!(bounds, Rect::new(3, 2, 4, 2));

        let cropped = screen.crop(bounds);
        assert_eq!(cropped.width(), 4);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.pixel(0, 0), Some(0xffff_0000));
    }

    #[test]
    fn pixmap_bytes_drop_the_alpha_channel() {
        let mut canvas = Canvas::new(1, 1);
        canvas.fill_rect(Rect::new(0, 0, 1, 1), Color::rgb(0x12, 0x34, 0x56));

        assert_eq!(canvas.to_pixmap_bytes(), [0x56, 0x34, 0x12, 0x00]);
    }
}
