//! Paleta y estilo del hub.
//!
//! El color vive aquí, por papel —fondo, superficie, texto, acento— y no por pantalla: una
//! pantalla nueva pide tokens y no hexadecimales. Añadir un tema es un constructor de
//! [`Palette`] más una variante en [`mh_overlay_config::Theme`], sin tocar las vistas.

use std::cell::Cell;
use std::sync::Arc;

use eframe::egui;
use mh_overlay_config::Theme;

/// Radio de las tarjetas del hub, en puntos.
pub const RADIUS: u8 = 12;

/// Colores de la interfaz, por papel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Fondo de la ventana.
    pub bg: egui::Color32,
    /// Fondo de las tarjetas y de la barra lateral.
    pub surface: egui::Color32,
    /// Fondo de los controles en reposo.
    pub surface_2: egui::Color32,
    /// Fondo de los controles bajo el puntero.
    pub surface_3: egui::Color32,
    /// Fondo de los campos de texto.
    pub input: egui::Color32,
    /// Bordes y separadores.
    pub border: egui::Color32,
    /// Texto principal.
    pub text: egui::Color32,
    /// Texto secundario y etiquetas.
    pub muted: egui::Color32,
    /// Acento: valores, selección y foco.
    pub accent: egui::Color32,
    /// Segundo foco del fondo ambiental, frío para no competir con el acento.
    pub glow: egui::Color32,
    /// Lectura correcta.
    pub good: egui::Color32,
    /// Error o valor fuera de rango.
    pub bad: egui::Color32,
}

impl Palette {
    /// Tema oscuro, el de por defecto: fondo casi negro con superficies azuladas.
    pub fn dark() -> Self {
        Self {
            bg: egui::Color32::from_rgb(0x0b, 0x0d, 0x12),
            surface: egui::Color32::from_rgb(0x14, 0x18, 0x21),
            surface_2: egui::Color32::from_rgb(0x1b, 0x20, 0x2b),
            surface_3: egui::Color32::from_rgb(0x24, 0x2a, 0x37),
            input: egui::Color32::from_rgb(0x0f, 0x12, 0x19),
            border: egui::Color32::from_rgb(0x2a, 0x31, 0x3f),
            text: egui::Color32::from_rgb(0xe8, 0xec, 0xf4),
            muted: egui::Color32::from_rgb(0x93, 0x9d, 0xb0),
            accent: egui::Color32::from_rgb(0xe6, 0xb4, 0x5e),
            glow: egui::Color32::from_rgb(0x3a, 0x6f, 0xd8),
            good: egui::Color32::from_rgb(0x7f, 0xd4, 0x7f),
            bad: egui::Color32::from_rgb(0xe2, 0x56, 0x5c),
        }
    }

    /// Tema claro: superficies blancas y el mismo ámbar, oscurecido para leerse sobre blanco.
    pub fn light() -> Self {
        Self {
            bg: egui::Color32::from_rgb(0xf4, 0xf6, 0xfa),
            surface: egui::Color32::from_rgb(0xff, 0xff, 0xff),
            surface_2: egui::Color32::from_rgb(0xed, 0xf0, 0xf6),
            surface_3: egui::Color32::from_rgb(0xe1, 0xe6, 0xef),
            input: egui::Color32::from_rgb(0xff, 0xff, 0xff),
            border: egui::Color32::from_rgb(0xd2, 0xd8, 0xe4),
            text: egui::Color32::from_rgb(0x15, 0x1a, 0x23),
            muted: egui::Color32::from_rgb(0x5a, 0x64, 0x76),
            accent: egui::Color32::from_rgb(0x96, 0x67, 0x14),
            glow: egui::Color32::from_rgb(0x6f, 0xa8, 0xe6),
            good: egui::Color32::from_rgb(0x2c, 0x7d, 0x45),
            bad: egui::Color32::from_rgb(0xb3, 0x32, 0x39),
        }
    }

    /// La paleta que se está dibujando ahora mismo.
    ///
    /// Las funciones que no reciben todo el `Hub` —tarjetas, píldoras, la salida del
    /// diagnóstico— la leen de aquí en vez de arrastrar un parámetro por cada llamada. La
    /// ventana es una sola y se dibuja en un hilo, así que el fotograma actual vive en un
    /// `thread_local` que pone [`apply`].
    pub fn current() -> Self {
        CURRENT.with(Cell::get)
    }
}

thread_local! {
    /// Paleta del fotograma en curso; [`apply`] es el único que la escribe.
    static CURRENT: Cell<Palette> = Cell::new(Palette::dark());
}

/// Lo que se aplicó a la ventana: la paleta y si salió oscura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub palette: Palette,
    pub dark: bool,
}

/// Instala la tipografía una sola vez, al abrir la ventana.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "rajdhani".to_owned(),
        Arc::new(egui::FontData::from_static(
            mh_overlay_ui::text::font_bytes(),
        )),
    );
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "rajdhani".to_owned());
    }
    ctx.set_fonts(fonts);
}

/// Aplica el tema configurado —siguiendo al escritorio si es `auto`— y devuelve la paleta.
pub fn apply(ctx: &egui::Context, theme: Theme) -> Look {
    ctx.set_theme(match theme {
        Theme::Auto => egui::ThemePreference::System,
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
    });

    let egui_theme = ctx.theme();
    let dark = matches!(egui_theme, egui::Theme::Dark);
    let palette = if dark {
        Palette::dark()
    } else {
        Palette::light()
    };

    let style = style_for(ctx, egui_theme, dark, palette);
    ctx.set_style_of(egui_theme, style);
    CURRENT.with(|current| current.set(palette));

    Look { palette, dark }
}

/// Estilo de egui para una paleta: métricas, tipografía y estado de cada widget.
fn style_for(ctx: &egui::Context, theme: egui::Theme, dark: bool, palette: Palette) -> egui::Style {
    let mut style = (*ctx.style_of(theme)).clone();
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.input;
    visuals.faint_bg_color = palette.surface_2;
    visuals.override_text_color = Some(palette.text);
    visuals.weak_text_color = Some(palette.muted);
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.30);
    visuals.selection.stroke = egui::Stroke::new(1.0, palette.accent);

    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::same(8);
    }

    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.inactive.weak_bg_fill = palette.surface_2;
    visuals.widgets.inactive.bg_fill = palette.surface_2;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, palette.text);
    visuals.widgets.hovered.weak_bg_fill = palette.surface_3;
    visuals.widgets.hovered.bg_fill = palette.surface_3;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, palette.accent.gamma_multiply(0.6));
    visuals.widgets.active.weak_bg_fill = palette.accent.gamma_multiply(0.25);
    visuals.widgets.active.bg_fill = palette.accent.gamma_multiply(0.25);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.accent);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.interact_size.y = 28.0;
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::proportional(26.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(15.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(15.0)),
        (egui::TextStyle::Small, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(13.0)),
    ]
    .into();

    style
}

/// Pinta el fondo ambiental del hub dentro de `ui`: dos focos suaves sobre el color de fondo.
///
/// Se calcula al dibujar, así que no viaja ninguna imagen ni licencia, sigue a la paleta y
/// escala con la ventana. `opacity` a cero lo apaga sin cambiar de modo.
pub fn paint_backdrop(ui: &mut egui::Ui, palette: Palette, opacity: f32) {
    let opacity = if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        0.0
    };

    if opacity <= 0.0 {
        return;
    }

    // El marco del panel tiene margen: se pinta un poco más allá para no dejar un borde seco.
    let rect = ui.max_rect().expand(26.0);
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::ZERO, palette.bg);

    let short = rect.width().min(rect.height());
    glow(
        painter,
        rect.right_top() + egui::vec2(-short * 0.12, short * 0.10),
        short * 0.85,
        palette.accent,
        0.10 * opacity,
    );
    glow(
        painter,
        rect.left_bottom() + egui::vec2(short * 0.08, -short * 0.18),
        short * 1.05,
        palette.glow,
        0.12 * opacity,
    );
}

/// Halo radial: círculos concéntricos con el alfa repartido para que el centro sume `peak`.
fn glow(painter: &egui::Painter, center: egui::Pos2, radius: f32, color: egui::Color32, peak: f32) {
    const STEPS: usize = 12;

    for step in 0..STEPS {
        let t = (step + 1) as f32 / STEPS as f32;
        let alpha = peak * t * t / STEPS as f32;
        let step_radius = radius * (1.0 - t);

        painter.circle_filled(center, step_radius, with_alpha(color, alpha));
    }
}

fn with_alpha(color: egui::Color32, alpha: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_themes_define_every_role_and_they_differ() {
        let dark = Palette::dark();
        let light = Palette::light();

        assert_ne!(dark.bg, light.bg);
        assert_ne!(dark.text, light.text);
        assert_ne!(dark.muted, light.muted);
        assert_ne!(dark.accent, light.accent);
        assert_ne!(dark.glow, light.glow);
    }

    #[test]
    fn text_is_readable_over_its_own_background() {
        for palette in [Palette::dark(), Palette::light()] {
            for (label, color) in [
                ("texto", palette.text),
                ("secundario", palette.muted),
                ("acento", palette.accent),
            ] {
                let ratio = contrast_ratio(color, palette.bg);
                assert!(ratio >= 3.0, "{label} sobre el fondo da {ratio:.2}:1");
            }
        }
    }

    /// Contraste WCAG entre dos colores opacos.
    fn contrast_ratio(left: egui::Color32, right: egui::Color32) -> f32 {
        let (lighter, darker) = {
            let (a, b) = (luminance(left), luminance(right));
            if a > b { (a, b) } else { (b, a) }
        };

        (lighter + 0.05) / (darker + 0.05)
    }

    fn luminance(color: egui::Color32) -> f32 {
        let channel = |value: u8| {
            let value = f32::from(value) / 255.0;
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }
}
