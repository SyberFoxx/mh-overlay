//! Composición del panel: del estado de los monstruos a una imagen.
//!
//! Una sola tarjeta oscura con secciones por monstruo: nombre y porcentaje, una barra de vida
//! dominante con la marca del umbral de captura y, debajo, barras finas de partes y estados.
//! El texto dice el número; las barras solo lo hacen comparable de un vistazo.

use mh_overlay_config::{Color, Config, StyleSection};
use mh_overlay_games::{Monster, Part};

use crate::canvas::{Canvas, Rect};
use crate::text::Text;

/// Partes visibles por monstruo.
const MAX_PARTS: usize = 8;
/// Estados visibles por monstruo.
const MAX_AILMENTS: usize = 4;
/// Ancho lógico mínimo del panel, antes de escalar.
const BASE_WIDTH: f32 = 320.0;
/// Radio de las esquinas en unidades lógicas.
const RADIUS: f32 = 10.0;
/// Ancho de la columna de etiquetas dentro de una fila.
const LABEL_WIDTH: f32 = 86.0;

/// Dibuja el panel completo; el tamaño depende de la configuración y del contenido.
pub fn render(config: &Config, text: &Text, monsters: &[Monster], status: Option<&str>) -> Canvas {
    let geometry = Geometry::new(config.overlay.scale);
    let monsters = &monsters[..monsters.len().min(config.overlay.max_monsters)];
    let width = geometry.width(text, monsters);
    let mut canvas = Canvas::new(width, geometry.height(config, text, monsters));

    // Borde de un píxel: sobre un juego claro o oscuro, es lo que separa el panel del fondo.
    canvas.fill_rounded(
        Rect::new(0, 0, canvas.width(), canvas.height()),
        RADIUS * geometry.scale,
        lighten(config.style.panel, 0.18),
    );
    canvas.fill_rounded(
        Rect::new(
            1,
            1,
            canvas.width().saturating_sub(2),
            canvas.height().saturating_sub(2),
        ),
        RADIUS * geometry.scale - 1.0,
        config.style.panel,
    );

    let mut y = geometry.padding;

    if monsters.is_empty() {
        let size = 14.0 * geometry.scale;
        let label = status.unwrap_or("sin monstruos grandes en la zona");
        text.draw(
            &mut canvas,
            label,
            geometry.padding,
            y + text.ascent(size),
            size,
            if status.is_some() {
                config.style.low_health
            } else {
                config.style.muted
            },
        );
        return canvas;
    }

    for (index, monster) in monsters.iter().enumerate() {
        if index > 0 {
            y += geometry.gap;
            canvas.fill_rect(
                Rect::new(
                    geometry.padding as i32,
                    y.round() as i32,
                    (canvas.width() as f32 - geometry.padding * 2.0).max(1.0) as u32,
                    geometry.divider.max(1.0) as u32,
                ),
                lighten(config.style.panel, 0.12),
            );
            y += geometry.divider + geometry.gap;
        }

        y = draw_title(&mut canvas, config, text, &geometry, monster, y);
        y = draw_health_bar(&mut canvas, config, &geometry, monster, y);

        for part in visible_parts(monster) {
            let fallback = format!("parte {:02}", part.index);
            draw_row(
                &mut canvas,
                config,
                text,
                &geometry,
                y,
                Row {
                    label: part.name.unwrap_or(&fallback),
                    fraction: part_fraction(part),
                    color: if part.severable.is_some() {
                        config.style.severable
                    } else {
                        config.style.part
                    },
                },
            );

            y += geometry.row;
        }

        for ailment in monster
            .ailments
            .iter()
            .filter(|ailment| ailment.is_active())
            .take(MAX_AILMENTS)
        {
            draw_row(
                &mut canvas,
                config,
                text,
                &geometry,
                y,
                Row {
                    label: ailment.name,
                    fraction: (ailment.buildup_percent() / 100.0).clamp(0.0, 1.0),
                    color: ailment_color(&config.style, ailment.name),
                },
            );

            y += geometry.row;
        }
    }

    canvas
}

/// Medidas en píxeles reales; todo lo que sigue sale de aquí para que el alto calculado y el
/// dibujado no puedan divergir.
struct Geometry {
    scale: f32,
    padding: f32,
    header: f32,
    bar: f32,
    track: f32,
    row: f32,
    gap: f32,
    small: f32,
    divider: f32,
}

impl Geometry {
    fn new(scale: f32) -> Self {
        Self {
            scale,
            padding: 14.0 * scale,
            header: 21.0 * scale,
            bar: 7.0 * scale,
            track: 3.0 * scale,
            row: 15.0 * scale,
            gap: 10.0 * scale,
            small: 5.0 * scale,
            divider: 1.0 * scale,
        }
    }

    /// Alto de la sección de un monstruo, sin los separadores.
    fn section(config: &Config, monster: &Monster, geometry: &Geometry) -> f32 {
        let rows = rows(config, monster);
        geometry.header
            + geometry.small
            + geometry.bar.max(2.0)
            + geometry.small
            + rows as f32 * geometry.row
    }

    fn width(&self, text: &Text, monsters: &[Monster]) -> u32 {
        let name_size = 17.0 * self.scale;
        let percent_size = 14.0 * self.scale;
        let mut logical = BASE_WIDTH;

        for monster in monsters {
            let name = text.width(&monster_label(monster), name_size);
            let percent = text.width(&percent_label(monster), percent_size);
            logical = logical.max(name + percent + 40.0);
        }

        (logical * self.scale).round() as u32
    }

    fn height(&self, config: &Config, text: &Text, monsters: &[Monster]) -> u32 {
        if monsters.is_empty() {
            let size = 14.0 * self.scale;
            return (self.padding * 2.0 + text.height(size)).round() as u32;
        }

        let sections = monsters
            .iter()
            .map(|monster| Self::section(config, monster, self))
            .sum::<f32>();
        let separators = (monsters.len() - 1) as f32 * (self.gap * 2.0 + self.divider);

        (self.padding * 2.0 + sections + separators).round() as u32
    }

    fn inner_width(&self, canvas: &Canvas) -> f32 {
        canvas.width() as f32 - self.padding * 2.0
    }
}

fn rows(config: &Config, monster: &Monster) -> usize {
    let mut rows = 0;

    if config.overlay.show_parts {
        rows += visible_parts(monster).len();
    }

    if config.overlay.show_ailments {
        rows += monster
            .ailments
            .iter()
            .filter(|ailment| ailment.is_active())
            .take(MAX_AILMENTS)
            .count();
    }

    rows
}

fn visible_parts(monster: &Monster) -> Vec<&Part> {
    monster
        .parts
        .iter()
        .filter(|part| part.breakable.is_some() || part.severable.is_some())
        .take(MAX_PARTS)
        .collect()
}

fn monster_label(monster: &Monster) -> String {
    match monster.name.as_deref() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => format!("monstruo {}", monster.id),
    }
}

fn percent_label(monster: &Monster) -> String {
    format!("{:.1}%", monster.health.percent())
}

/// Nombre a la izquierda, porcentaje (y aviso de captura) a la derecha.
fn draw_title(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    monster: &Monster,
    y: f32,
) -> f32 {
    let style = &config.style;
    let name_size = 17.0 * geometry.scale;
    let percent_size = 14.0 * geometry.scale;

    text.draw(
        canvas,
        &monster_label(monster),
        geometry.padding,
        y + text.ascent(name_size),
        name_size,
        style.text,
    );

    let percent = percent_label(monster);
    let percent_width = text.width(&percent, percent_size);
    let right = canvas.width() as f32 - geometry.padding;

    text.draw(
        canvas,
        &percent,
        right - percent_width,
        y + text.ascent(percent_size),
        percent_size,
        if monster.health.percent() <= style.low_health_percent {
            style.low_health
        } else {
            style.health
        },
    );

    if monster.is_capturable() {
        let label = "captura";
        let label_size = 12.0 * geometry.scale;
        text.draw(
            canvas,
            label,
            right - percent_width - 8.0 * geometry.scale - text.width(label, label_size),
            y + text.ascent(label_size),
            label_size,
            style.capture,
        );
    }

    y + geometry.header + geometry.small
}

/// Pista oscura, relleno degradado y marca vertical del umbral de captura.
fn draw_health_bar(
    canvas: &mut Canvas,
    config: &Config,
    geometry: &Geometry,
    monster: &Monster,
    y: f32,
) -> f32 {
    let style = &config.style;
    let bar = Rect::new(
        geometry.padding as i32,
        y.round() as i32,
        geometry.inner_width(canvas).max(1.0) as u32,
        geometry.bar.max(2.0) as u32,
    );

    canvas.fill_rounded(bar, bar.height as f32 / 2.0, lighten(style.panel, 0.14));

    let fraction = (monster.health.percent() / 100.0).clamp(0.0, 1.0);
    let filled = (bar.width as f32 * fraction).round() as u32;
    let base = if monster.health.percent() <= style.low_health_percent {
        style.low_health
    } else {
        style.health
    };

    if filled > 1 {
        let fill = Rect::new(bar.x, bar.y, filled, bar.height);
        canvas.fill_rounded_gradient(fill, fill.height as f32 / 2.0, lighten(base, 0.18), base);
    }

    if let Some(threshold) = monster.capture_percent
        && threshold > 0.0
        && threshold < 100.0
    {
        let x = bar.x as f32 + bar.width as f32 * (threshold / 100.0);
        canvas.fill_rect(
            Rect::new(
                x.round() as i32,
                bar.y,
                (geometry.scale * 2.0).max(1.0) as u32,
                bar.height,
            ),
            style.capture,
        );
    }

    y + bar.height as f32 + geometry.small
}

/// Contenido de una fila secundaria.
struct Row<'a> {
    label: &'a str,
    fraction: f32,
    color: Color,
}

/// Fila secundaria: etiqueta, barra fina y porcentaje.
fn draw_row(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    y: f32,
    row: Row<'_>,
) {
    let style = &config.style;
    let fraction = row.fraction;
    let color = row.color;
    let size = 12.0 * geometry.scale;
    let inner = geometry.inner_width(canvas);
    let label = fit_label(text, row.label, size, LABEL_WIDTH * geometry.scale);

    text.draw(
        canvas,
        &label,
        geometry.padding,
        y + text.ascent(size),
        size,
        style.muted,
    );

    let bar_height = geometry.track.max(2.0);
    let bar = Rect::new(
        geometry.padding as i32 + (LABEL_WIDTH * geometry.scale) as i32,
        (y + geometry.row * 0.5 - bar_height * 0.5).round() as i32,
        (inner - LABEL_WIDTH * geometry.scale - 46.0 * geometry.scale).max(1.0) as u32,
        bar_height as u32,
    );

    canvas.fill_rounded(bar, bar.height as f32 / 2.0, lighten(style.panel, 0.14));

    let filled = (bar.width as f32 * fraction.clamp(0.0, 1.0)).round() as u32;
    if filled > 0 {
        canvas.fill_rounded(
            Rect::new(bar.x, bar.y, filled, bar.height),
            bar.height as f32 / 2.0,
            color,
        );
    }

    let percent = format!("{:.0}%", fraction * 100.0);
    text.draw(
        canvas,
        &percent,
        canvas.width() as f32 - geometry.padding - text.width(&percent, size),
        y + text.ascent(size),
        size,
        style.muted,
    );
}

fn part_fraction(part: &Part) -> f32 {
    let health = part.severable.or(part.breakable);
    health.map_or(0.0, |health| (health.percent() / 100.0).clamp(0.0, 1.0))
}

/// Recorta la etiqueta con puntos suspensivos si no cabe en la columna.
///
/// Los nombres de la localización son más largos que `parte NN` y, sin esto, el texto se
/// montaría encima de la barra.
fn fit_label(text: &Text, label: &str, size: f32, max_width: f32) -> String {
    if text.width(label, size) <= max_width {
        return label.to_string();
    }

    let mut characters: Vec<char> = label.chars().collect();
    while !characters.is_empty() {
        characters.pop();
        let candidate = format!("{}..", characters.iter().collect::<String>());
        if text.width(&candidate, size) <= max_width {
            return candidate;
        }
    }

    "..".to_string()
}

/// Un color por estado, para no leer la etiqueta cuando solo se mira el borde del panel.
fn ailment_color(style: &StyleSection, name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "fuego" => Color::rgb(0xf0, 0x6e, 0x3c),
        "agua" | "plaga de agua" => Color::rgb(0x4f, 0xa8, 0xe6),
        "rayo" | "plaga de rayo" => Color::rgb(0xf0, 0xd0, 0x4a),
        "hielo" | "plaga de hielo" => Color::rgb(0x7f, 0xd8, 0xe6),
        "dragón" | "dragon" | "plaga de dragón" => Color::rgb(0xa9, 0x7a, 0xe6),
        "veneno" => Color::rgb(0xb0, 0x7a, 0xd6),
        "sueño" | "sueno" => Color::rgb(0x8f, 0x9f, 0xe6),
        "parálisis" | "paralisis" => Color::rgb(0xd8, 0xc0, 0x6a),
        "explosión" | "explosion" | "nitro" => Color::rgb(0xe0, 0x6a, 0x4a),
        "aturdimiento" | "aturdir" | "destello" | "flash" => Color::rgb(0xe6, 0xc0, 0x7a),
        "montura" | "montar" | "derribo" => Color::rgb(0x6f, 0xd0, 0xb0),
        "agotamiento" | "fatiga" => Color::rgb(0x9a, 0xa8, 0x6a),
        "trampa eléctrica" | "trampa de escollos" | "trampa de foso" => {
            Color::rgb(0x7a, 0xc0, 0xe6)
        }
        "tranquilizante" | "tranquilizar" => Color::rgb(0xc0, 0xa0, 0xe6),
        "estiércol" | "bomba boñiga" => Color::rgb(0xa8, 0x9a, 0x7a),
        _ => style.capture,
    }
}

/// Aclara un color hacia blanco; mantiene el alfa para no alterar la translucidez del panel.
fn lighten(color: Color, amount: f32) -> Color {
    let mix = |component: u8| -> u8 {
        (f32::from(component) + (255.0 - f32::from(component)) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };

    Color {
        r: mix(color.r),
        g: mix(color.g),
        b: mix(color.b),
        a: color.a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mh_overlay_games::{Ailment, Health};

    fn monster(percent: f32) -> Monster {
        Monster::new(7, Health::new(percent, 100.0))
    }

    fn config() -> (Config, Text) {
        (Config::default(), Text::bundled().unwrap())
    }

    #[test]
    fn a_panel_without_monsters_says_so_and_stays_small() {
        let (config, text) = config();
        let canvas = render(&config, &text, &[], None);

        assert_eq!(canvas.width(), BASE_WIDTH as u32);
        assert!(canvas.height() < 80, "alto {}", canvas.height());
    }

    #[test]
    fn two_monsters_make_a_taller_panel_than_one() {
        let (config, text) = config();

        let one = render(&config, &text, &[monster(50.0)], None);
        let two = render(&config, &text, &[monster(50.0), monster(20.0)], None);

        assert!(two.height() > one.height());
    }

    #[test]
    fn the_content_fits_inside_the_panel() {
        let (config, text) = config();
        let mut detailed = monster(60.0);

        for index in 0..6 {
            detailed.parts.push(Part {
                index,
                name: None,
                flinch: None,
                breakable: Some(Health::new(40.0, 100.0)),
                severable: None,
            });
        }
        let canvas = render(&config, &text, &[detailed], None);

        let last_row_bottom = 14.0 + 21.0 + 5.0 + 7.0 + 5.0 + 6.0 * 15.0;
        assert!(
            canvas.height() as f32 >= last_row_bottom,
            "el panel mide {} y la última fila acaba en {last_row_bottom}",
            canvas.height()
        );
    }

    #[test]
    fn scale_multiplies_the_panel() {
        let (mut config, text) = config();
        let normal = render(&config, &text, &[monster(50.0)], None);

        config.overlay.scale = 2.0;
        let large = render(&config, &text, &[monster(50.0)], None);

        assert!(large.height() > normal.height());
        assert!(large.width() >= normal.width() * 19 / 10);
    }

    #[test]
    fn the_panel_corners_are_cut_and_the_center_is_painted() {
        let (config, text) = config();
        let canvas = render(&config, &text, &[monster(50.0)], None);

        assert_eq!(canvas.pixel(0, 0), Some(0xff00_0000));
        assert_ne!(
            canvas.pixel(canvas.width() as i32 / 2, canvas.height() as i32 / 2),
            Some(0xff00_0000)
        );
    }

    #[test]
    fn a_low_health_bar_uses_the_low_health_color() {
        let (config, text) = config();
        let canvas = render(&config, &text, &[monster(10.0)], None);
        let wanted = config.style.low_health.to_argb() | 0xff00_0000;

        assert!(
            canvas.pixels().contains(&wanted),
            "el color de vida baja no aparece en el panel"
        );
    }

    #[test]
    fn parts_and_ailments_add_rows() {
        let (config, text) = config();
        let plain = render(&config, &text, &[monster(60.0)], None);

        let mut detailed = monster(60.0);
        detailed.parts.push(Part {
            index: 1,
            name: Some("cola"),
            flinch: None,
            breakable: Some(Health::new(40.0, 100.0)),
            severable: None,
        });
        detailed.ailments.push(Ailment {
            name: "fuego",
            buildup: 30.0,
            max_buildup: 100.0,
            timer: 0.0,
            max_timer: 60.0,
        });
        let with_rows = render(&config, &text, &[detailed], None);

        assert!(with_rows.height() > plain.height());
    }

    #[test]
    fn a_long_name_widens_the_panel() {
        let (config, text) = config();

        let mut named = monster(50.0);
        named.name = Some("Rathalos violeta de las Tierras Altas".to_string());
        let wide = render(&config, &text, &[named], None);

        assert!(wide.width() > BASE_WIDTH as u32);
    }

    #[test]
    fn the_capture_threshold_is_marked_in_its_own_color() {
        let (config, text) = config();
        let mut target = monster(50.0);
        target.capture_percent = Some(20.0);
        let canvas = render(&config, &text, &[target], None);
        let wanted = config.style.capture.to_argb() | 0xff00_0000;

        assert!(canvas.pixels().contains(&wanted));
    }
}
