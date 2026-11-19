//! Composición del panel: del estado de los monstruos a una imagen.
//!
//! Una sola tarjeta oscura con secciones por monstruo: nombre y porcentaje, una barra de vida
//! dominante con la marca del umbral de captura y, debajo, barras finas de partes y estados.
//! El texto dice el número; las barras solo lo hacen comparable de un vistazo.

use mh_overlay_config::{Color, Config, StyleSection};
use mh_overlay_games::{
    Buff, HornNote, Hunt, Monster, Part, Player, SharpnessLevel, Snapshot, Weapon, WeaponDetail,
    WeaponGauge,
};

use crate::canvas::{Canvas, Rect};
use crate::icons::{self, BuffGlyph};
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
/// Ancho reservado a la derecha de cada barra para su valor ("62 %", "2/3", "18 s").
const VALUE_WIDTH: f32 = 46.0;
/// Alto de la barra de filo, en unidades lógicas.
const SHARPNESS_BAR: f32 = 8.0;
/// Lado del icono del arma en la cabecera del jugador.
const WEAPON_ICON: f32 = 22.0;
/// Fichas de datos del arma por fila; se reparten en varias filas si no caben.
const CHIP_HEIGHT: f32 = 21.0;
/// Iconos de buff por fila: la cuadrícula sustituye a una lista de barras.
const BUFF_COLUMNS: usize = 8;
/// Lado de cada baldosa de buff, en unidades lógicas.
const BUFF_TILE: f32 = 32.0;
/// Baldosas de buff como máximo; el resto se resume en una baldosa "+N".
const MAX_BUFFS: usize = BUFF_COLUMNS * 2;

/// Dibuja el panel completo; el tamaño depende de la configuración y del contenido.
pub fn render(config: &Config, text: &Text, snapshot: &Snapshot, status: Option<&str>) -> Canvas {
    let geometry = Geometry::new(config.overlay.scale);
    let monsters = &snapshot.monsters[..snapshot.monsters.len().min(config.overlay.max_monsters)];
    let width = geometry.width(text, snapshot, monsters);
    let mut canvas = Canvas::new(width, geometry.height(config, text, snapshot, monsters));

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
    let has_session = snapshot.hunt.is_some() || snapshot.player.is_some();

    if let Some(hunt) = snapshot.hunt {
        y = draw_hunt(&mut canvas, config, text, &geometry, &hunt, y);
    }

    if let Some(player) = &snapshot.player {
        y = draw_player(&mut canvas, config, text, &geometry, player, y);
    }

    if monsters.is_empty() {
        if has_session {
            y += geometry.small;
        }

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

    if has_session {
        y += geometry.gap;
        draw_divider(&mut canvas, config, &geometry, y);
        y += geometry.divider + geometry.gap;
    }

    for (index, monster) in monsters.iter().enumerate() {
        if index > 0 {
            y += geometry.gap;
            draw_divider(&mut canvas, config, &geometry, y);
            y += geometry.divider + geometry.gap;
        }

        y = draw_title(&mut canvas, config, text, &geometry, monster, y);
        y = draw_health_bar(&mut canvas, config, &geometry, monster, y);

        for part in visible_parts(monster) {
            let fallback = format!("parte {:02}", part.index);
            let broken = part
                .breaks
                .is_some_and(|(done, total)| total > 0 && done >= total);

            draw_row(
                &mut canvas,
                config,
                text,
                &geometry,
                y,
                Row {
                    label: part.name.unwrap_or(&fallback),
                    fraction: part_fraction(part),
                    color: if broken {
                        config.style.capture
                    } else if part.severable.is_some() {
                        config.style.severable
                    } else {
                        config.style.part
                    },
                    recoverable: 0.0,
                    segments: part.breaks.map(|(_, total)| total),
                    value: part.breaks.map(|(done, total)| format!("{done}/{total}")),
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
                    recoverable: 0.0,
                    segments: None,
                    value: None,
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
    weapon: f32,
    icon: f32,
    chip: f32,
    chip_gap: f32,
    buff: f32,
    buff_gap: f32,
}

impl Geometry {
    fn new(scale: f32) -> Self {
        Self {
            scale,
            padding: 14.0 * scale,
            header: 22.0 * scale,
            bar: 12.0 * scale,
            track: 4.0 * scale,
            row: 15.0 * scale,
            gap: 10.0 * scale,
            small: 5.0 * scale,
            divider: 1.0 * scale,
            weapon: 27.0 * scale,
            icon: WEAPON_ICON * scale,
            chip: CHIP_HEIGHT * scale,
            chip_gap: 4.0 * scale,
            buff: BUFF_TILE * scale,
            buff_gap: 4.0 * scale,
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

    fn width(&self, text: &Text, snapshot: &Snapshot, monsters: &[Monster]) -> u32 {
        let name_size = 17.0 * self.scale;
        let percent_size = 14.0 * self.scale;
        let mut logical = BASE_WIDTH;

        for monster in monsters {
            let name = text.width(&monster_label(monster), name_size);
            let percent = text.width(&percent_label(monster), percent_size);
            logical = logical.max(name + percent + 40.0);
        }

        if let Some(hunt) = snapshot.hunt {
            let size = 12.0 * self.scale;
            let label = text.width("Cacería", size);
            let value = text.width(&hunt_label(&hunt), size);
            logical = logical.max((label + value + 60.0) / self.scale);
        }

        (logical * self.scale).round() as u32
    }

    fn height(
        &self,
        config: &Config,
        text: &Text,
        snapshot: &Snapshot,
        monsters: &[Monster],
    ) -> u32 {
        let inner = self.width(text, snapshot, monsters) as f32 - self.padding * 2.0;
        let session = session_height(text, snapshot, self, inner, &config.style);

        if monsters.is_empty() {
            let size = 14.0 * self.scale;
            let extra = if session > 0.0 { self.small } else { 0.0 };
            return (self.padding * 2.0 + session + extra + text.height(size)).round() as u32;
        }

        let sections = monsters
            .iter()
            .map(|monster| Self::section(config, monster, self))
            .sum::<f32>();
        let separators = (monsters.len() - 1) as f32 * (self.gap * 2.0 + self.divider);
        let session_separator = if session > 0.0 {
            self.gap * 2.0 + self.divider
        } else {
            0.0
        };

        (self.padding * 2.0 + session + session_separator + sections + separators).round() as u32
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

/// Bisel, pista oscura, relleno degradado, escala cada 10 % y marca del umbral de captura.
fn draw_health_bar(
    canvas: &mut Canvas,
    config: &Config,
    geometry: &Geometry,
    monster: &Monster,
    y: f32,
) -> f32 {
    let style = &config.style;
    let scale = geometry.scale;
    let bar = Rect::new(
        geometry.padding as i32,
        y.round() as i32,
        geometry.inner_width(canvas).max(1.0) as u32,
        geometry.bar.max(4.0) as u32,
    );
    let radius = 3.0 * scale;

    // El borde claro separa la barra del panel y la pista queda hundida un píxel.
    canvas.fill_rounded(bar, radius, lighten(style.panel, 0.26));
    canvas.fill_rounded(
        bar.inset(1, 1),
        (radius - 1.0).max(0.0),
        lighten(style.panel, 0.06),
    );

    let fraction = (monster.health.percent() / 100.0).clamp(0.0, 1.0);
    let filled = (bar.width as f32 * fraction).round() as u32;
    let base = if monster.health.percent() <= style.low_health_percent {
        style.low_health
    } else {
        style.health
    };

    if filled > 3 {
        let fill = Rect::new(
            bar.x + 1,
            bar.y + 1,
            filled - 2,
            bar.height.saturating_sub(2),
        );
        canvas.fill_rounded_gradient(fill, (radius - 1.0).max(0.0), lighten(base, 0.34), base);
    }

    // Marcas cada 10 %: la barra se lee como una escala y no como una mancha de color.
    let tick = lighten(style.panel, 0.5).with_alpha(60);
    for step in 1..10 {
        let x = bar.x as f32 + bar.width as f32 * (step as f32 / 10.0);
        canvas.fill_rect(
            Rect::new(
                x.round() as i32,
                bar.y + 1,
                scale.round().max(1.0) as u32,
                bar.height.saturating_sub(2),
            ),
            tick,
        );
    }

    if let Some(threshold) = monster.capture_percent
        && threshold > 0.0
        && threshold < 100.0
    {
        let x = bar.x as f32 + bar.width as f32 * (threshold / 100.0);
        canvas.fill_rect(
            Rect::new(
                x.round() as i32,
                bar.y - (scale).round() as i32,
                (scale * 2.0).max(2.0) as u32,
                bar.height + (scale * 2.0).round() as u32,
            ),
            style.capture,
        );
    }

    y + bar.height as f32 + geometry.small
}

/// Alto de la cabecera de jugador y cacería, en píxeles.
fn session_height(
    text: &Text,
    snapshot: &Snapshot,
    geometry: &Geometry,
    inner: f32,
    style: &StyleSection,
) -> f32 {
    let hunt = snapshot.hunt.map_or(0.0, |_| geometry.row);

    hunt + snapshot.player.as_ref().map_or(0.0, |player| {
        player_height(text, player, geometry, inner, style)
    })
}

/// Alto del bloque del jugador: arma, filo, fichas de la clase y cuadrícula de buffs.
fn player_height(
    text: &Text,
    player: &Player,
    geometry: &Geometry,
    inner: f32,
    style: &StyleSection,
) -> f32 {
    let mut height = 0.0;

    if let Some(weapon) = &player.weapon {
        height += geometry.weapon;

        if weapon.sharpness.is_some() {
            height += SHARPNESS_BAR * geometry.scale + geometry.small;
        }

        let chips = weapon_chips(weapon, style);

        if !chips.is_empty() {
            let rows = chip_rows(text, &chips, geometry, inner);
            height += rows as f32 * geometry.chip
                + rows.saturating_sub(1) as f32 * geometry.chip_gap
                + geometry.small;
        }
    }

    height += geometry.row * 2.0;

    if !player.buffs.is_empty() {
        let rows = buff_rows(player.buffs.len());
        height += geometry.small
            + rows as f32 * geometry.buff
            + rows.saturating_sub(1) as f32 * geometry.buff_gap;
    }

    height
}

/// Filas que ocupa la cuadrícula de buffs, con el resumen `+N` incluido.
fn buff_rows(buffs: usize) -> usize {
    buffs.min(MAX_BUFFS).div_ceil(BUFF_COLUMNS)
}

/// Separador de una sección, del ancho interior del panel.
fn draw_divider(canvas: &mut Canvas, config: &Config, geometry: &Geometry, y: f32) {
    canvas.fill_rect(
        Rect::new(
            geometry.padding as i32,
            y.round() as i32,
            (canvas.width() as f32 - geometry.padding * 2.0).max(1.0) as u32,
            geometry.divider.max(1.0) as u32,
        ),
        lighten(config.style.panel, 0.12),
    );
}

fn hunt_label(hunt: &Hunt) -> String {
    format!(
        "{}  ·  {}/{}",
        hunt.time_label(),
        hunt.deaths,
        hunt.max_deaths
    )
}

/// Reloj restante a la derecha y vidas del equipo, destacadas si se agotan.
fn draw_hunt(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    hunt: &Hunt,
    y: f32,
) -> f32 {
    let style = &config.style;
    let size = 12.0 * geometry.scale;
    let value = hunt_label(hunt);

    text.draw(
        canvas,
        "Cacería",
        geometry.padding,
        y + text.ascent(size),
        size,
        style.muted,
    );

    text.draw(
        canvas,
        &value,
        canvas.width() as f32 - geometry.padding - text.width(&value, size),
        y + text.ascent(size),
        size,
        if hunt.deaths >= hunt.max_deaths {
            style.low_health
        } else {
            style.text
        },
    );

    y + geometry.row
}

/// Vida y aguante, el arma con su icono y sus fichas, y los buffs como iconos con temporizador.
fn draw_player(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    player: &Player,
    y: f32,
) -> f32 {
    let style = &config.style;
    let health = player.health.percent();
    let mut y = y;

    if let Some(weapon) = &player.weapon {
        y = draw_weapon(canvas, config, text, geometry, weapon, y);

        let chips = weapon_chips(weapon, style);

        if !chips.is_empty() {
            y = draw_chips(canvas, config, text, geometry, y, &chips);
            y += geometry.small;
        }
    }

    let recoverable = player
        .recoverable
        .filter(|value| *value > 0.0)
        .map_or(0.0, |value| (value / player.health.max).clamp(0.0, 1.0));

    draw_row(
        canvas,
        config,
        text,
        geometry,
        y,
        Row {
            label: "Vida",
            fraction: (health / 100.0).clamp(0.0, 1.0),
            segments: None,
            value: None,
            color: if health <= style.low_health_percent {
                style.low_health
            } else {
                style.health
            },
            recoverable,
        },
    );

    draw_row(
        canvas,
        config,
        text,
        geometry,
        y + geometry.row,
        Row {
            label: "Aguante",
            fraction: (player.stamina.percent() / 100.0).clamp(0.0, 1.0),
            segments: None,
            value: None,
            color: style.part,
            recoverable: 0.0,
        },
    );

    y += geometry.row * 2.0;

    if !player.buffs.is_empty() {
        y += geometry.small;
        y = draw_buffs(canvas, config, text, geometry, y, &player.buffs);
    }

    y
}

/// Cabecera del arma: icono de la clase, nombre y, a la derecha, el filo actual.
fn draw_weapon(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    weapon: &Weapon,
    y: f32,
) -> f32 {
    let style = &config.style;
    let scale = geometry.scale;
    let size = 13.0 * scale;
    let right = canvas.width() as f32 - geometry.padding;
    let baseline = y + (geometry.weapon - text.height(size)) * 0.5 + text.ascent(size);

    icons::weapon(
        canvas,
        weapon.kind,
        geometry.padding,
        y + (geometry.weapon - geometry.icon) * 0.5,
        geometry.icon,
        if weapon.kind.has_sharpness() {
            style.text
        } else {
            style.muted
        },
    );

    text.draw(
        canvas,
        weapon.kind.label(),
        geometry.padding + geometry.icon + 6.0 * scale,
        baseline,
        size,
        style.text,
    );

    if let Some(sharpness) = &weapon.sharpness {
        let level = sharpness.level.label();
        let level_width = text.width(level, 12.0 * scale);
        let value = format!("filo {:.0}", sharpness.current);
        let value_width = text.width(&value, 12.0 * scale);

        text.draw(
            canvas,
            level,
            right - level_width,
            baseline,
            12.0 * scale,
            sharpness_color(sharpness.level),
        );
        text.draw(
            canvas,
            &value,
            right - level_width - value_width - 6.0 * scale,
            baseline,
            12.0 * scale,
            style.muted,
        );
    } else {
        let label = "a distancia";
        text.draw(
            canvas,
            label,
            right - text.width(label, 12.0 * scale),
            baseline,
            12.0 * scale,
            style.muted,
        );
    }

    let mut y = y + geometry.weapon;

    if let Some(sharpness) = &weapon.sharpness {
        draw_sharpness(canvas, config, geometry, sharpness, y);
        y += SHARPNESS_BAR * scale + geometry.small;
    }

    y
}

/// Barra de filo con un hueco entre colores y la posición actual marcada.
fn draw_sharpness(
    canvas: &mut Canvas,
    config: &Config,
    geometry: &Geometry,
    sharpness: &mh_overlay_games::Sharpness,
    y: f32,
) {
    let style = &config.style;
    let scale = geometry.scale;
    let height = SHARPNESS_BAR * scale;
    let bar = Rect::new(
        geometry.padding as i32,
        y.round() as i32,
        geometry.inner_width(canvas).max(1.0) as u32,
        height.max(3.0) as u32,
    );
    let radius = 2.0 * scale;

    canvas.fill_rounded(bar, radius, lighten(style.panel, 0.22));

    let total = sharpness.total();
    if total <= 0.0 {
        return;
    }

    let mut x = bar.x as f32;

    for (index, level) in SharpnessLevel::ALL.iter().enumerate() {
        let length = f32::from(sharpness.lengths[index]);
        let width = bar.width as f32 * length / total;

        if width >= 1.5 {
            let color = sharpness_color(*level);
            let segment = Rect::new(
                (x + scale).round() as i32,
                bar.y + scale.round() as i32,
                (width - scale * 2.0).max(1.0) as u32,
                bar.height.saturating_sub((2.0 * scale).round() as u32),
            );

            canvas.fill_rounded_gradient(segment, radius, lighten(color, 0.32), color);
        }

        x += width;
    }

    let marker_x = bar.x as f32 + bar.width as f32 * sharpness.fraction();
    canvas.fill_rect(
        Rect::new(
            marker_x.round() as i32 - 1,
            bar.y - scale.round() as i32,
            (scale * 2.0).max(2.0) as u32,
            bar.height + (scale * 2.0).round() as u32,
        ),
        style.text,
    );
    canvas.fill_diamond(
        (marker_x, bar.y as f32 - 3.0 * scale),
        2.0 * scale,
        style.text,
    );
}

/// Ficha compacta de un dato del arma.
struct Chip {
    glyph: BuffGlyph,
    label: &'static str,
    value: String,
    /// Relleno de la línea inferior; `None` para los temporizadores.
    fraction: Option<f32>,
    color: Color,
}

/// Fichas del arma: el medidor de clase, los datos de combate de la clase y el filo máximo.
fn weapon_chips(weapon: &Weapon, style: &StyleSection) -> Vec<Chip> {
    let mut chips = weapon_gauge_chips(weapon, style);
    chips.extend(weapon_detail_chips(&weapon.detail, style));

    if let Some(sharpness) = &weapon.sharpness {
        chips.push(Chip {
            glyph: BuffGlyph::Sharpness,
            label: "Filo máx.",
            value: sharpness.max_level.label().to_string(),
            fraction: None,
            color: sharpness_color(sharpness.max_level),
        });
    }

    chips
}

/// Datos de combate que el juego expone además del medidor: munición, proyectiles, notas y
/// estadísticas calculadas del arma.
fn weapon_detail_chips(detail: &WeaponDetail, style: &StyleSection) -> Vec<Chip> {
    let mut chips = Vec::new();
    let meter = style.part;
    let state = style.capture;

    if let Some(attack) = detail.attack {
        chips.push(Chip {
            glyph: BuffGlyph::Attack,
            label: "Ataque",
            value: format!("{attack:.0}"),
            fraction: None,
            color: meter,
        });
    }

    if let Some(affinity) = detail.affinity {
        chips.push(Chip {
            glyph: BuffGlyph::Misc,
            label: "Afinidad",
            value: format!("{affinity:+}%"),
            fraction: None,
            color: if affinity >= 0 {
                state
            } else {
                style.low_health
            },
        });
    }

    if let Some(element) = detail.element.filter(|value| *value > 0.0) {
        chips.push(Chip {
            glyph: BuffGlyph::Flame,
            label: "Elemento",
            value: format!("{element:.0}"),
            fraction: None,
            color: meter,
        });
    }

    if let Some(ranged) = &detail.ranged {
        if let Some(selected) = ranged.selected() {
            chips.push(Chip {
                glyph: BuffGlyph::Bolt,
                label: "Cargada",
                value: selected.loaded.to_string(),
                fraction: None,
                color: meter,
            });
        }

        let reserve: u32 = ranged.ammo.iter().map(|slot| slot.total).sum();
        if reserve > 0 {
            chips.push(Chip {
                glyph: BuffGlyph::Misc,
                label: "Reserva",
                value: reserve.to_string(),
                fraction: None,
                color: state,
            });
        }
    }

    if let Some(gunlance) = detail.gunlance {
        chips.push(Chip {
            glyph: BuffGlyph::Bolt,
            label: "Proyectiles",
            value: format!("{}/{}", gunlance.shells, gunlance.max_shells),
            fraction: Some(if gunlance.max_shells > 0 {
                gunlance.shells as f32 / gunlance.max_shells as f32
            } else {
                0.0
            }),
            color: meter,
        });

        if gunlance.max_charged > 0 {
            chips.push(Chip {
                glyph: BuffGlyph::Flame,
                label: "Cargados",
                value: format!("{}/{}", gunlance.charged, gunlance.max_charged),
                fraction: Some(gunlance.charged as f32 / gunlance.max_charged as f32),
                color: state,
            });
        }

        if gunlance.wyvern_fire > 0.0 {
            chips.push(Chip {
                glyph: BuffGlyph::Flame,
                label: "Dragón",
                value: format!("{:.0}s", gunlance.wyvern_fire),
                fraction: None,
                color: state,
            });
        }

        if gunlance.stake_loaded {
            chips.push(Chip {
                glyph: BuffGlyph::Misc,
                label: "Estaca",
                value: if gunlance.stake_timer > 0.0 {
                    format!("{:.0}s", gunlance.stake_timer)
                } else {
                    "cargada".to_string()
                },
                fraction: None,
                color: state,
            });
        }
    }

    if let Some(horn) = &detail.horn {
        if !horn.notes.is_empty() {
            chips.push(Chip {
                glyph: BuffGlyph::Song,
                label: "Notas",
                value: horn
                    .notes
                    .iter()
                    .map(|note| horn_note_short(*note))
                    .collect::<Vec<_>>()
                    .join("·"),
                fraction: None,
                color: state,
            });
        }

        if !horn.songs.is_empty() {
            chips.push(Chip {
                glyph: BuffGlyph::Song,
                label: "Melodías",
                value: horn.songs.len().to_string(),
                fraction: None,
                color: meter,
            });
        }
    }

    if let Some(bow) = detail.bow {
        chips.push(Chip {
            glyph: BuffGlyph::Bolt,
            label: "Carga",
            value: format!("{}/{}", bow.level, bow.max_level),
            fraction: Some(bow.charge),
            color: meter,
        });
    }

    chips
}

/// Convierte el medidor de clase en fichas cortas, que ocupan menos alto que una fila por dato.
fn weapon_gauge_chips(weapon: &Weapon, style: &StyleSection) -> Vec<Chip> {
    let Some(gauge) = &weapon.gauge else {
        return Vec::new();
    };

    let meter = style.part;
    let state = style.capture;
    let timer = style.capture;

    match gauge {
        WeaponGauge::Spirit {
            buildup,
            level,
            timer: time,
            max_timer,
            regen,
        } => {
            let mut chips = vec![
                Chip {
                    glyph: BuffGlyph::Attack,
                    label: "Espíritu",
                    value: format!("{buildup:.0}%"),
                    fraction: Some(buildup / 100.0),
                    color: spirit_color(*level),
                },
                Chip {
                    glyph: BuffGlyph::Stun,
                    label: "Nivel",
                    value: format!("{level}/3"),
                    fraction: None,
                    color: spirit_color(*level),
                },
            ];

            if *max_timer > 0.0 {
                chips.push(Chip {
                    glyph: BuffGlyph::Misc,
                    label: "Duración",
                    value: format!("{time:.0}s"),
                    fraction: None,
                    color: timer,
                });
            }

            if *regen > 0.0 {
                chips.push(Chip {
                    glyph: BuffGlyph::Health,
                    label: "Regen",
                    value: format!("{regen:.0}s"),
                    fraction: None,
                    color: timer,
                });
            }

            chips
        }
        WeaponGauge::ChargeBlade {
            phials,
            max_phials,
            charge,
            shield,
            sword,
            axe,
        } => {
            let mut chips = vec![
                Chip {
                    glyph: BuffGlyph::Bolt,
                    label: "Carga",
                    value: format!("{charge:.0}%"),
                    fraction: Some(charge / 100.0),
                    color: meter,
                },
                Chip {
                    glyph: BuffGlyph::Bubble,
                    label: "Frascos",
                    value: format!("{phials}/{max_phials}"),
                    fraction: Some(f32::from(*phials) / f32::from(*max_phials).max(1.0)),
                    color: state,
                },
            ];

            for (label, glyph, time) in [
                ("Escudo", BuffGlyph::Defense, shield),
                ("Espada", BuffGlyph::Attack, sword),
                ("Hacha", BuffGlyph::Sharpness, axe),
            ] {
                if *time > 0.0 {
                    chips.push(Chip {
                        glyph,
                        label,
                        value: format!("{time:.0}s"),
                        fraction: None,
                        color: timer,
                    });
                }
            }

            chips
        }
        WeaponGauge::SwitchAxe {
            buildup,
            charge,
            timer: time,
        } => {
            let mut chips = vec![
                Chip {
                    glyph: BuffGlyph::Sharpness,
                    label: "Filo",
                    value: format!("{buildup:.0}%"),
                    fraction: Some(buildup / 100.0),
                    color: meter,
                },
                Chip {
                    glyph: BuffGlyph::Bolt,
                    label: "Amped",
                    value: format!("{charge:.0}%"),
                    fraction: Some(charge / 100.0),
                    color: state,
                },
            ];

            if *time > 0.0 {
                chips.push(Chip {
                    glyph: BuffGlyph::Misc,
                    label: "Bono",
                    value: format!("{time:.0}s"),
                    fraction: None,
                    color: timer,
                });
            }

            chips
        }
        WeaponGauge::DualBlades {
            demon,
            arch_demon,
            buildup,
        } => vec![
            Chip {
                glyph: BuffGlyph::Flame,
                label: "Modo",
                value: if *arch_demon {
                    "archidemonio"
                } else if *demon {
                    "demonio"
                } else {
                    "—"
                }
                .to_string(),
                fraction: None,
                color: if *demon || *arch_demon {
                    spirit_color(3)
                } else {
                    style.muted
                },
            },
            Chip {
                glyph: BuffGlyph::Bolt,
                label: "Carga",
                value: format!("{buildup:.0}%"),
                fraction: Some(buildup / 100.0),
                color: meter,
            },
        ],
        WeaponGauge::InsectGlaive {
            extracts,
            attack,
            speed,
            defense,
            stamina,
            max_stamina,
        } => {
            let mut chips = vec![Chip {
                glyph: BuffGlyph::Stamina,
                label: "Esencias",
                value: extracts.label(),
                fraction: None,
                color: state,
            }];

            for (label, time) in [
                ("Ataque", attack),
                ("Velocidad", speed),
                ("Defensa", defense),
            ] {
                if *time > 0.0 {
                    chips.push(Chip {
                        glyph: BuffGlyph::Misc,
                        label,
                        value: format!("{time:.0}s"),
                        fraction: None,
                        color: timer,
                    });
                }
            }

            if *max_stamina > 0.0 {
                chips.push(Chip {
                    glyph: BuffGlyph::Evasion,
                    label: "Kinsecto",
                    value: format!("{stamina:.0}/{max_stamina:.0}"),
                    fraction: Some(stamina / max_stamina),
                    color: meter,
                });
            }

            chips
        }
    }
}

fn horn_note_short(note: HornNote) -> &'static str {
    match note {
        HornNote::Purple => "P",
        HornNote::Red => "R",
        HornNote::Orange => "N",
        HornNote::Yellow => "A",
        HornNote::Green => "V",
        HornNote::Blue => "Z",
        HornNote::LightBlue => "L",
        HornNote::White => "B",
    }
}

fn chip_width(text: &Text, chip: &Chip, geometry: &Geometry) -> f32 {
    let size = 10.5 * geometry.scale;
    let padding = 6.0 * geometry.scale;

    padding * 2.0
        + 12.0 * geometry.scale
        + 4.0 * geometry.scale
        + text.width(chip.label, size)
        + 4.0 * geometry.scale
        + text.width(&chip.value, size)
}

fn chip_rows(text: &Text, chips: &[Chip], geometry: &Geometry, inner: f32) -> usize {
    let mut rows = 1;
    let mut x = 0.0;

    for chip in chips {
        let width = chip_width(text, chip, geometry);

        if x > 0.0 && x + width > inner {
            rows += 1;
            x = 0.0;
        }

        x += width + geometry.chip_gap;
    }

    rows
}

/// Dibuja las fichas del arma y devuelve la `y` de la siguiente sección.
fn draw_chips(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    y: f32,
    chips: &[Chip],
) -> f32 {
    let style = &config.style;
    let scale = geometry.scale;
    let size = 10.5 * scale;
    let padding = 6.0 * scale;
    let icon = 12.0 * scale;
    let right = canvas.width() as f32 - geometry.padding;
    let mut x = geometry.padding;
    let mut row_y = y;

    for chip in chips {
        let width = chip_width(text, chip, geometry);

        if x > geometry.padding && x + width > right {
            x = geometry.padding;
            row_y += geometry.chip + geometry.chip_gap;
        }

        let rect = Rect::new(
            x.round() as i32,
            row_y.round() as i32,
            width.round().max(1.0) as u32,
            geometry.chip.round().max(1.0) as u32,
        );
        canvas.fill_rounded(rect, 4.0 * scale, lighten(style.panel, 0.10));

        if let Some(fraction) = chip.fraction {
            let filled = (rect.width as f32 * fraction.clamp(0.0, 1.0)).round() as u32;
            if filled > 1 {
                canvas.fill_rect(
                    Rect::new(
                        rect.x + 1,
                        rect.bottom() - (2.0 * scale).round().max(1.0) as i32,
                        filled.saturating_sub(2),
                        (2.0 * scale).round().max(1.0) as u32,
                    ),
                    chip.color,
                );
            }
        }

        icons::buff(
            canvas,
            chip.glyph,
            x + padding,
            row_y + (geometry.chip - icon) * 0.5,
            icon,
            chip.color,
        );

        let baseline = row_y + (geometry.chip - text.height(size)) * 0.5 + text.ascent(size);
        let label_x = x + padding + icon + 4.0 * scale;

        text.draw(canvas, chip.label, label_x, baseline, size, style.muted);
        text.draw(
            canvas,
            &chip.value,
            label_x + text.width(chip.label, size) + 4.0 * scale,
            baseline,
            size,
            style.text,
        );

        x += width + geometry.chip_gap;
    }

    row_y + geometry.chip
}

/// Cuadrícula de buffs: una baldosa por buff, con icono y tiempo; los estados negativos llevan
/// borde rojo. Si hay más de los que caben, la última baldosa resume el resto.
fn draw_buffs(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    y: f32,
    buffs: &[Buff],
) -> f32 {
    let shown = buffs.len().min(MAX_BUFFS);
    let hidden = buffs.len().saturating_sub(MAX_BUFFS);
    let mut x = geometry.padding;
    let mut row_y = y;
    let mut column = 0;

    for (index, buff) in buffs.iter().take(shown).enumerate() {
        if index > 0 && column == BUFF_COLUMNS {
            column = 0;
            x = geometry.padding;
            row_y += geometry.buff + geometry.buff_gap;
        }

        if hidden > 0 && index == MAX_BUFFS - 1 {
            draw_more_tile(canvas, config, text, geometry, x, row_y, hidden);
        } else {
            draw_buff_tile(canvas, config, text, geometry, x, row_y, buff);
        }

        x += geometry.buff + geometry.buff_gap;
        column += 1;
    }

    row_y + geometry.buff
}

fn draw_buff_tile(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    x: f32,
    y: f32,
    buff: &Buff,
) {
    let style = &config.style;
    let scale = geometry.scale;
    let glyph = icons::buff_glyph(buff.name);
    let tile = Rect::new(
        x.round() as i32,
        y.round() as i32,
        geometry.buff.round().max(1.0) as u32,
        geometry.buff.round().max(1.0) as u32,
    );
    let radius = 5.0 * scale;

    if buff.debuff {
        canvas.fill_rounded(tile, radius, style.low_health);
        canvas.fill_rounded(
            tile.inset(1, 1),
            (radius - 1.0).max(0.0),
            lighten(style.panel, 0.13),
        );
    } else {
        canvas.fill_rounded(tile, radius, lighten(style.panel, 0.13));
    }

    let icon = 17.0 * scale;
    icons::buff(
        canvas,
        glyph,
        x + (geometry.buff - icon) * 0.5,
        y + 4.0 * scale,
        icon,
        if buff.debuff {
            style.low_health
        } else {
            buff_color(style, glyph)
        },
    );

    let label = if buff.infinite {
        "∞".to_string()
    } else {
        buff_time_label(buff.timer)
    };
    let size = 9.0 * scale;
    let baseline = y + geometry.buff - 5.0 * scale;
    text.draw(
        canvas,
        &label,
        x + (geometry.buff - text.width(&label, size)) * 0.5,
        baseline,
        size,
        if buff.debuff {
            style.low_health
        } else {
            style.text
        },
    );
}

fn draw_more_tile(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    x: f32,
    y: f32,
    hidden: usize,
) {
    let style = &config.style;
    let tile = Rect::new(
        x.round() as i32,
        y.round() as i32,
        geometry.buff.round().max(1.0) as u32,
        geometry.buff.round().max(1.0) as u32,
    );

    canvas.fill_rounded(tile, 5.0 * geometry.scale, lighten(style.panel, 0.10));

    let label = format!("+{hidden}");
    let size = 11.0 * geometry.scale;
    text.draw(
        canvas,
        &label,
        x + (geometry.buff - text.width(&label, size)) * 0.5,
        y + (geometry.buff + text.ascent(size)) * 0.5,
        size,
        style.muted,
    );
}

/// Temporizador corto para una baldosa de 32 px: segundos por debajo de un minuto y minutos
/// por encima, para que nunca se salga del icono.
fn buff_time_label(timer: f32) -> String {
    if !timer.is_finite() || timer <= 0.0 {
        "—".to_string()
    } else if timer < 60.0 {
        format!("{timer:.0}s")
    } else {
        format!("{:.0}m", timer / 60.0)
    }
}

/// Color del medidor de espíritu según el nivel cargado.
fn spirit_color(level: u8) -> Color {
    match level {
        0 => Color::rgb(0x5c, 0x68, 0x7a),
        1 => Color::rgb(0xe8, 0xf0, 0xf8),
        2 => Color::rgb(0x63, 0xb4, 0xf0),
        _ => Color::rgb(0xf0, 0xc4, 0x5a),
    }
}

/// Un color por familia, para distinguir el buff por el icono sin leer el nombre.
fn buff_color(style: &StyleSection, glyph: BuffGlyph) -> Color {
    match glyph {
        BuffGlyph::Attack | BuffGlyph::Food | BuffGlyph::Song => style.capture,
        BuffGlyph::Defense | BuffGlyph::Water => style.part,
        BuffGlyph::Health | BuffGlyph::Stamina => style.health,
        BuffGlyph::Evasion | BuffGlyph::Ice | BuffGlyph::Bubble | BuffGlyph::Wind => {
            Color::rgb(0x7f, 0xd8, 0xe6)
        }
        BuffGlyph::Sharpness => style.text,
        BuffGlyph::Flame => Color::rgb(0xf0, 0x6e, 0x3c),
        BuffGlyph::Bolt | BuffGlyph::Stun => Color::rgb(0xf0, 0xd0, 0x4a),
        BuffGlyph::Dragon | BuffGlyph::Poison => Color::rgb(0xb0, 0x7a, 0xd6),
        BuffGlyph::Bleed => style.low_health,
        BuffGlyph::Sleep => Color::rgb(0x8f, 0x9f, 0xe6),
        BuffGlyph::Misc => style.muted,
    }
}

/// Un color por nivel de filo, del rojo al púrpura.
fn sharpness_color(level: SharpnessLevel) -> Color {
    match level {
        SharpnessLevel::Red => Color::rgb(0xd6, 0x4a, 0x4a),
        SharpnessLevel::Orange => Color::rgb(0xe0, 0x8a, 0x3c),
        SharpnessLevel::Yellow => Color::rgb(0xe0, 0xd0, 0x4a),
        SharpnessLevel::Green => Color::rgb(0x7f, 0xd4, 0x7f),
        SharpnessLevel::Blue => Color::rgb(0x4f, 0x9f, 0xe6),
        SharpnessLevel::White => Color::rgb(0xe8, 0xf0, 0xf8),
        SharpnessLevel::Purple => Color::rgb(0xa9, 0x7a, 0xe6),
    }
}

/// Contenido de una fila secundaria.
struct Row<'a> {
    label: &'a str,
    fraction: f32,
    color: Color,
    /// Parte de la barra que se puede recuperar (vida roja), como fracción del máximo.
    recoverable: f32,
    /// Tramos en que se divide la barra (los rompimientos de una parte).
    segments: Option<u32>,
    /// Texto de la columna derecha; si falta, el porcentaje.
    value: Option<String>,
}

/// Fila secundaria: etiqueta, barra fina (opcionalmente a tramos) y su valor.
fn draw_row(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    y: f32,
    row: Row<'_>,
) {
    let style = &config.style;
    let fraction = row.fraction.clamp(0.0, 1.0);
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

    let bar_height = geometry.track.max(3.0);
    let bar = Rect::new(
        geometry.padding as i32 + (LABEL_WIDTH * geometry.scale) as i32,
        (y + geometry.row * 0.5 - bar_height * 0.5).round() as i32,
        (inner - LABEL_WIDTH * geometry.scale - VALUE_WIDTH * geometry.scale).max(1.0) as u32,
        bar_height as u32,
    );
    let radius = bar.height as f32 / 2.0;

    canvas.fill_rounded(bar, radius, lighten(style.panel, 0.18));

    // La vida recuperable se dibuja detrás del relleno: es la parte que aún se puede curar.
    if row.recoverable > 0.0 {
        let width =
            (bar.width as f32 * (fraction + row.recoverable).clamp(0.0, 1.0)).round() as u32;
        if width > 1 {
            canvas.fill_rounded(
                Rect::new(bar.x, bar.y, width, bar.height),
                radius,
                style.low_health,
            );
        }
    }

    let filled = (bar.width as f32 * fraction).round() as u32;
    if filled > 1 {
        canvas.fill_rounded_gradient(
            Rect::new(bar.x, bar.y, filled, bar.height),
            radius,
            lighten(color, 0.3),
            color,
        );
    }

    // Un hueco por cada rompimiento: se ve cuántos aguanta la parte y cuál es el siguiente.
    if let Some(segments) = row.segments.filter(|segments| *segments > 1) {
        for step in 1..segments {
            let x = bar.x as f32 + bar.width as f32 * (step as f32 / segments as f32);
            canvas.fill_rect(
                Rect::new(
                    x.round() as i32,
                    bar.y,
                    geometry.scale.round().max(1.0) as u32,
                    bar.height,
                ),
                lighten(style.panel, 0.04),
            );
        }
    }

    let value = row
        .value
        .unwrap_or_else(|| format!("{:.0}%", fraction * 100.0));
    text.draw(
        canvas,
        &value,
        canvas.width() as f32 - geometry.padding - text.width(&value, size),
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
    use mh_overlay_games::{Ailment, Buff, Health, Sharpness, WeaponGauge, WeaponKind};

    fn monster(percent: f32) -> Monster {
        Monster::new(7, Health::new(percent, 100.0))
    }

    fn config() -> (Config, Text) {
        (Config::default(), Text::bundled().unwrap())
    }

    #[test]
    fn a_panel_without_monsters_says_so_and_stays_small() {
        let (config, text) = config();
        let canvas = render(&config, &text, &Snapshot::of(Vec::new()), None);

        assert_eq!(canvas.width(), BASE_WIDTH as u32);
        assert!(canvas.height() < 80, "alto {}", canvas.height());
    }

    #[test]
    fn two_monsters_make_a_taller_panel_than_one() {
        let (config, text) = config();

        let one = render(&config, &text, &Snapshot::of(vec![monster(50.0)]), None);
        let two = render(
            &config,
            &text,
            &Snapshot::of(vec![monster(50.0), monster(20.0)]),
            None,
        );

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
                breaks: Some((1, 2)),
            });
        }
        let canvas = render(&config, &text, &Snapshot::of(vec![detailed]), None);

        let last_row_bottom = 14.0 + 22.0 + 5.0 + 12.0 + 5.0 + 6.0 * 15.0;
        assert!(
            canvas.height() as f32 >= last_row_bottom,
            "el panel mide {} y la última fila acaba en {last_row_bottom}",
            canvas.height()
        );
    }

    #[test]
    fn the_player_and_hunt_add_rows_to_the_panel() {
        let (config, text) = config();
        let plain = render(&config, &text, &Snapshot::of(vec![monster(60.0)]), None);

        let mut snapshot = Snapshot::of(vec![monster(60.0)]);
        snapshot.player = Some(Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: None,
            buffs: Vec::new(),
        });
        snapshot.hunt = Some(Hunt {
            time_left: 754.0,
            deaths: 0,
            max_deaths: 3,
        });
        let session = render(&config, &text, &snapshot, None);

        assert!(session.height() > plain.height());
    }

    #[test]
    fn the_weapon_and_its_sharpness_add_rows_to_the_player() {
        let (config, text) = config();
        let mut player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: None,
            buffs: Vec::new(),
        };
        let plain = render(&config, &text, &preview(&player), None);

        player.weapon = Some(Weapon {
            kind: WeaponKind::Longsword,
            sharpness: Some(Sharpness {
                lengths: [40, 30, 30, 30, 30, 20, 0],
                level: SharpnessLevel::White,
                max_level: SharpnessLevel::White,
                current: 10.0,
            }),
            gauge: None,
            detail: WeaponDetail::default(),
        });
        let with_weapon = render(&config, &text, &preview(&player), None);

        assert!(with_weapon.height() > plain.height());
    }

    #[test]
    fn the_weapon_gauge_adds_its_rows_below_the_sharpness() {
        let (config, text) = config();
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: Some(Weapon {
                kind: WeaponKind::Longsword,
                sharpness: None,
                gauge: None,
                detail: WeaponDetail::default(),
            }),
            buffs: Vec::new(),
        };
        let plain = render(&config, &text, &preview(&player), None);

        let with_gauge = Player {
            weapon: Some(Weapon {
                kind: WeaponKind::Longsword,
                sharpness: None,
                gauge: Some(WeaponGauge::Spirit {
                    buildup: 62.0,
                    level: 2,
                    timer: 98.0,
                    max_timer: 140.0,
                    regen: 0.0,
                }),
                detail: WeaponDetail::default(),
            }),
            ..player
        };
        let gauge = render(&config, &text, &preview(&with_gauge), None);

        assert!(gauge.height() > plain.height());
    }

    #[test]
    fn buffs_are_capped_so_they_do_not_cover_the_monsters() {
        let (config, text) = config();
        let mut player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: None,
            buffs: Vec::new(),
        };
        let plain = render(&config, &text, &preview(&player), None);

        player.buffs = (0..12)
            .map(|index| Buff {
                name: "Buff",
                timer: 60.0 + index as f32,
                max_timer: 120.0,
                debuff: false,
                infinite: false,
            })
            .collect();
        let many = render(&config, &text, &preview(&player), None);

        // Seis filas de buffs como máximo, aunque haya doce activos.
        assert!(many.height() > plain.height());
        assert!(many.height() < plain.height() + 7 * 24);
    }

    #[test]
    fn the_buff_icons_fit_in_a_grid_instead_of_one_row_each() {
        let (config, text) = config();
        let plain = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: None,
            buffs: Vec::new(),
        };
        let without = render(&config, &text, &preview(&plain), None);

        let mut with_buffs = plain.clone();
        with_buffs.buffs = (0..8)
            .map(|index| Buff {
                name: "Semilla del poder",
                timer: 60.0 + index as f32,
                max_timer: 180.0,
                debuff: false,
                infinite: false,
            })
            .collect();
        let with = render(&config, &text, &preview(&with_buffs), None);

        // Ocho buffs ocupan una sola fila de 32 px, no ocho filas de texto.
        assert!(
            with.height() - without.height() < 60,
            "la cuadrícula creció {} px",
            with.height() - without.height()
        );
    }

    #[test]
    fn the_measured_player_height_matches_what_is_drawn() {
        let (config, text) = config();
        let geometry = Geometry::new(1.0);
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: Some(40.0),
            weapon: Some(Weapon {
                kind: WeaponKind::ChargeBlade,
                sharpness: Some(Sharpness {
                    lengths: [40, 30, 30, 30, 30, 20, 10],
                    level: SharpnessLevel::White,
                    max_level: SharpnessLevel::Purple,
                    current: 15.0,
                }),
                gauge: Some(WeaponGauge::ChargeBlade {
                    phials: 3,
                    max_phials: 5,
                    charge: 62.0,
                    shield: 30.0,
                    sword: 12.0,
                    axe: 0.0,
                }),
                detail: WeaponDetail::default(),
            }),
            buffs: (0..10)
                .map(|index| Buff {
                    name: "Ataque +",
                    timer: 30.0 + index as f32,
                    max_timer: 120.0,
                    debuff: index == 9,
                    infinite: false,
                })
                .collect(),
        };

        let inner = BASE_WIDTH - geometry.padding * 2.0;
        let measured = player_height(&text, &player, &geometry, inner, &config.style);
        let mut canvas = Canvas::new(BASE_WIDTH as u32, 800);
        let drawn = draw_player(
            &mut canvas,
            &config,
            &text,
            &geometry,
            &player,
            geometry.padding,
        );

        assert_eq!((drawn - geometry.padding).round(), measured.round());
    }

    #[test]
    fn the_sharpness_colors_are_drawn_in_the_bar() {
        let (config, text) = config();
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: Some(Weapon {
                kind: WeaponKind::Longsword,
                sharpness: Some(Sharpness {
                    lengths: [40, 30, 30, 30, 30, 20, 0],
                    level: SharpnessLevel::White,
                    max_level: SharpnessLevel::White,
                    current: 10.0,
                }),
                gauge: None,
                detail: WeaponDetail::default(),
            }),
            buffs: Vec::new(),
        };
        let canvas = render(&config, &text, &preview(&player), None);
        let purple = sharpness_color(SharpnessLevel::Purple).to_argb() | 0xff00_0000;
        let white = sharpness_color(SharpnessLevel::White).to_argb() | 0xff00_0000;

        assert!(canvas.pixels().contains(&purple) || canvas.pixels().contains(&white));
    }

    fn preview(player: &Player) -> Snapshot {
        let mut snapshot = Snapshot::of(vec![monster(50.0)]);
        snapshot.player = Some(player.clone());
        snapshot
    }

    #[test]
    fn scale_multiplies_the_panel() {
        let (mut config, text) = config();
        let normal = render(&config, &text, &Snapshot::of(vec![monster(50.0)]), None);

        config.overlay.scale = 2.0;
        let large = render(&config, &text, &Snapshot::of(vec![monster(50.0)]), None);

        assert!(large.height() > normal.height());
        assert!(large.width() >= normal.width() * 19 / 10);
    }

    #[test]
    fn the_panel_corners_are_cut_and_the_center_is_painted() {
        let (config, text) = config();
        let canvas = render(&config, &text, &Snapshot::of(vec![monster(50.0)]), None);

        assert_eq!(canvas.pixel(0, 0), Some(0xff00_0000));
        assert_ne!(
            canvas.pixel(canvas.width() as i32 / 2, canvas.height() as i32 / 2),
            Some(0xff00_0000)
        );
    }

    #[test]
    fn a_low_health_bar_uses_the_low_health_color() {
        let (config, text) = config();
        let canvas = render(&config, &text, &Snapshot::of(vec![monster(10.0)]), None);
        let wanted = config.style.low_health.to_argb() | 0xff00_0000;

        assert!(
            canvas.pixels().contains(&wanted),
            "el color de vida baja no aparece en el panel"
        );
    }

    #[test]
    fn parts_and_ailments_add_rows() {
        let (config, text) = config();
        let plain = render(&config, &text, &Snapshot::of(vec![monster(60.0)]), None);

        let mut detailed = monster(60.0);
        detailed.parts.push(Part {
            index: 1,
            name: Some("cola"),
            flinch: None,
            breakable: Some(Health::new(40.0, 100.0)),
            severable: None,
            breaks: Some((1, 2)),
        });
        detailed.ailments.push(Ailment {
            name: "fuego",
            buildup: 30.0,
            max_buildup: 100.0,
            timer: 0.0,
            max_timer: 60.0,
        });
        let with_rows = render(&config, &text, &Snapshot::of(vec![detailed]), None);

        assert!(with_rows.height() > plain.height());
    }

    #[test]
    fn a_long_name_widens_the_panel() {
        let (config, text) = config();

        let mut named = monster(50.0);
        named.name = Some("Rathalos violeta de las Tierras Altas".to_string());
        let wide = render(&config, &text, &Snapshot::of(vec![named]), None);

        assert!(wide.width() > BASE_WIDTH as u32);
    }

    #[test]
    fn the_capture_threshold_is_marked_in_its_own_color() {
        let (config, text) = config();
        let mut target = monster(50.0);
        target.capture_percent = Some(20.0);
        let canvas = render(&config, &text, &Snapshot::of(vec![target]), None);
        let wanted = config.style.capture.to_argb() | 0xff00_0000;

        assert!(canvas.pixels().contains(&wanted));
    }
}
