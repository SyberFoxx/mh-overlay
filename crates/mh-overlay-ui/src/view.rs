//! Composición del panel: del estado de los monstruos a una imagen.
//!
//! Una sola tarjeta oscura con secciones por monstruo: nombre y porcentaje, una barra de vida
//! dominante con la marca del umbral de captura y, debajo, barras finas de partes y estados.
//! El texto dice el número; las barras solo lo hacen comparable de un vistazo.

use mh_overlay_config::{BarStyle, Color, Config, Direction, Monsters, PanelTheme, StyleSection};
use mh_overlay_games::{
    Buff, Crown, Extracts, HornNote, Hunt, Monster, Part, Player, SharpnessLevel, Snapshot, Weapon,
    WeaponDetail, WeaponGauge, Wirebugs, part_labels,
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
/// Ancho mínimo reservado a la derecha de cada barra para su valor ("62 %", "2/3", "18 s").
const VALUE_WIDTH: f32 = 46.0;
/// Tamaño del texto numérico del título (vida real y aviso de captura).
const TITLE_VALUE_SIZE: f32 = 12.0;
/// Alto de la barra de filo, en unidades lógicas.
const SHARPNESS_BAR: f32 = 8.0;
/// Alto del medidor de espíritu dibujado como espada, en unidades lógicas.
const SPIRIT_SWORD: f32 = 24.0;
/// Anchos de las placas del modo libre, en unidades lógicas.
const FREE_ROW_WIDTH: f32 = 280.0;
const FREE_MONSTER_WIDTH: f32 = 340.0;
const FREE_WEAPON_WIDTH: f32 = 340.0;
const FREE_BUFFS_WIDTH: f32 = 340.0;
const FREE_HUNT_WIDTH: f32 = 240.0;
/// Margen interior de cada placa del modo libre.
const FREE_PADDING: f32 = 9.0;
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
/// Ancho que ocupa la corona en el título, ya con su separación del nombre.
const CROWN_SLOT: f32 = 20.0;
/// Plateada y dorada, los dos metales que usa el juego para las coronas grandes.
const CROWN_SILVER: Color = Color::rgb(0xc9, 0xd3, 0xe0);
const CROWN_GOLD: Color = Color::rgb(0xe6, 0xb4, 0x5e);

/// Dibuja el panel completo; el tamaño depende de la configuración y del contenido.
pub fn render(config: &Config, text: &Text, snapshot: &Snapshot, status: Option<&str>) -> Canvas {
    let geometry = Geometry::new(config.overlay.scale);
    let monsters = visible_monsters(config, snapshot);
    let width = geometry.width(config, text, snapshot, &monsters);
    let mut canvas = Canvas::new(width, geometry.height(config, text, snapshot, &monsters));

    // La superficie cambia de un vistazo: tarjeta oscura, cristal con borde de acento o cartel
    // de bloques rectos. Es lo primero que se pinta y lo que fija el tono del resto.
    paint_surface(
        &mut canvas,
        &config.style,
        RADIUS * geometry.scale,
        geometry.scale,
        true,
        false,
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

        let parts = visible_parts(monster);
        let labels = part_labels(&parts);
        for (part, label) in parts.iter().zip(&labels) {
            let broken = part.is_broken();

            draw_row(
                &mut canvas,
                config,
                text,
                &geometry,
                y,
                Row {
                    label,
                    fraction: part_fraction(config, part),
                    color: if broken {
                        config.style.capture
                    } else if part.severable.is_some() {
                        config.style.severable
                    } else {
                        config.style.part
                    },
                    recoverable: 0.0,
                    segments: part.breaks.map(|breaks| breaks.total),
                    value: if broken {
                        Some("Roto".to_string())
                    } else if config.overlay.show_part_health {
                        part.health.map(|health| health.label())
                    } else {
                        part.breaks.map(|breaks| breaks.label())
                    },
                },
            );

            y += geometry.row;
        }

        for ailment in monster
            .ailments
            .iter()
            .filter(|ailment| ailment.is_active() && !ailment.is_elemental())
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
                    fraction: ailment.fraction(),
                    color: ailment_color(&config.style, ailment.name),
                    recoverable: 0.0,
                    segments: None,
                    value: Some(ailment.value_label()),
                },
            );

            y += geometry.row;
        }
    }

    canvas
}

/// Rectángulo de una placa dentro de la pantalla lógica del editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Placa del modo libre y el grupo al que pertenece (`monster.0` incluye sus partes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedBar {
    pub id: String,
    pub bounds: BarBounds,
}

/// Dibuja el modo libre: una placa por barra sobre un lienzo transparente.
///
/// Las posiciones salen de `[[overlay.bars]]`; lo que no esté configurado se apila en una
/// columna desde `margin_x`/`margin_y`. La vida del monstruo y sus partes forman un solo
/// bloque: mover `monster.N` mueve también las partes, y `direction` decide si cuelgan hacia
/// abajo o hacia la derecha.
pub fn render_free(
    config: &Config,
    text: &Text,
    snapshot: &Snapshot,
    status: Option<&str>,
    screen: (u32, u32),
) -> Canvas {
    let mut canvas = Canvas::transparent(screen.0.max(1), screen.1.max(1));

    free_plates(config, text, snapshot, status, |_, plate, x, y| {
        canvas.composite(plate, x, y);
    });

    canvas
}

/// Rectángulos que ocupan las placas en la pantalla del editor.
///
/// Sirve al editor visual: cada entrada es un grupo arrastrable (`monster.0` abarca la vida
/// y las partes), no una placa suelta. Las coordenadas son las mismas que usa el overlay.
pub fn free_bounds(
    config: &Config,
    text: &Text,
    snapshot: &Snapshot,
    status: Option<&str>,
) -> Vec<PlacedBar> {
    let mut bars: Vec<PlacedBar> = Vec::new();

    free_plates(config, text, snapshot, status, |id, plate, x, y| {
        let right = x + plate.width() as i32;
        let bottom = y + plate.height() as i32;

        if let Some(existing) = bars.iter_mut().find(|bar| bar.id == id) {
            let left = existing.bounds.x.min(x);
            let top = existing.bounds.y.min(y);
            existing.bounds = BarBounds {
                x: left,
                y: top,
                width: (existing.bounds.x + existing.bounds.width as i32).max(right) as u32
                    - left as u32,
                height: (existing.bounds.y + existing.bounds.height as i32).max(bottom) as u32
                    - top as u32,
            };
        } else {
            bars.push(PlacedBar {
                id: id.to_string(),
                bounds: BarBounds {
                    x,
                    y,
                    width: plate.width(),
                    height: plate.height(),
                },
            });
        }
    });

    bars
}

fn free_plates(
    config: &Config,
    text: &Text,
    snapshot: &Snapshot,
    status: Option<&str>,
    mut place: impl FnMut(&str, &Canvas, i32, i32),
) {
    let geometry = Geometry::new(config.overlay.scale);
    let monsters = visible_monsters(config, snapshot);
    let gap = (4.0 * geometry.scale).round() as i32;
    let mut cursor = (config.overlay.margin_x, config.overlay.margin_y);

    if let Some(hunt) = snapshot.hunt {
        let plate = hunt_plate(config, text, &geometry, &hunt);
        let origin = bar_origin(config, "hunt").unwrap_or(cursor);
        place("hunt", &plate, origin.0, origin.1);
        cursor.1 = cursor.1.max(origin.1 + plate.height() as i32 + gap);
    }

    if let Some(player) = &snapshot.player {
        let health = row_plate(
            config,
            text,
            &geometry,
            Row {
                label: "Vida",
                fraction: (player.health.percent() / 100.0).clamp(0.0, 1.0),
                color: if player.health.percent() <= config.style.low_health_percent {
                    config.style.low_health
                } else {
                    config.style.health
                },
                recoverable: player
                    .recoverable
                    .filter(|value| *value > 0.0)
                    .map_or(0.0, |value| (value / player.health.max).clamp(0.0, 1.0)),
                segments: None,
                value: None,
            },
        );
        let origin = bar_origin(config, "player.health").unwrap_or(cursor);
        place("player.health", &health, origin.0, origin.1);
        cursor.1 = cursor.1.max(origin.1 + health.height() as i32 + gap);

        let stamina = row_plate(
            config,
            text,
            &geometry,
            Row {
                label: "Aguante",
                fraction: (player.stamina.percent() / 100.0).clamp(0.0, 1.0),
                color: config.style.part,
                recoverable: 0.0,
                segments: None,
                value: None,
            },
        );
        let origin = bar_origin(config, "player.stamina").unwrap_or(cursor);
        place("player.stamina", &stamina, origin.0, origin.1);
        cursor.1 = cursor.1.max(origin.1 + stamina.height() as i32 + gap);

        if let Some(weapon) = &player.weapon {
            let plate = weapon_plate(config, text, &geometry, weapon);
            let origin = bar_origin(config, "weapon").unwrap_or(cursor);
            place("weapon", &plate, origin.0, origin.1);
            cursor.1 = cursor.1.max(origin.1 + plate.height() as i32 + gap);
        }

        if !player.buffs.is_empty() {
            let plate = buffs_plate(config, text, &geometry, &player.buffs);
            let origin = bar_origin(config, "buffs").unwrap_or(cursor);
            place("buffs", &plate, origin.0, origin.1);
            cursor.1 = cursor.1.max(origin.1 + plate.height() as i32 + gap);
        }
    }

    for (slot, monster) in monsters.iter().enumerate() {
        let health = monster_plate(config, text, &geometry, monster);
        let id = format!("monster.{slot}");
        let direction = config
            .overlay
            .bars
            .iter()
            .find(|bar| bar.id == id)
            .map_or(Direction::Down, |bar| bar.direction);
        let origin = bar_origin(config, &id).unwrap_or(cursor);
        place(&id, &health, origin.0, origin.1);

        let mut part_y = match direction {
            Direction::Down => origin.1 + health.height() as i32 + gap,
            // A la derecha, las partes empiezan a la altura de la vida del monstruo.
            Direction::Right => origin.1,
        };
        let part_x = origin.0 + health.width() as i32 + gap;
        let mut column_height = health.height() as i32;

        let mut place_part = |plate: &Canvas| match direction {
            Direction::Down => {
                place(&id, plate, origin.0, part_y);
                part_y += plate.height() as i32 + gap;
                column_height = part_y - origin.1;
            }
            Direction::Right => {
                place(&id, plate, part_x, part_y);
                part_y += plate.height() as i32 + gap;
                column_height = column_height.max(part_y - origin.1);
            }
        };

        if config.overlay.show_parts {
            let parts = visible_parts(monster);
            let labels = part_labels(&parts);
            for (part, label) in parts.iter().zip(&labels) {
                place_part(&part_plate(config, text, &geometry, part, label));
            }
        }

        if config.overlay.show_ailments {
            for ailment in monster
                .ailments
                .iter()
                .filter(|ailment| ailment.is_active() && !ailment.is_elemental())
                .take(MAX_AILMENTS)
            {
                place_part(&row_plate(
                    config,
                    text,
                    &geometry,
                    Row {
                        label: ailment.name,
                        fraction: ailment.fraction(),
                        color: ailment_color(&config.style, ailment.name),
                        recoverable: 0.0,
                        segments: None,
                        value: Some(ailment.value_label()),
                    },
                ));
            }
        }

        cursor.1 = cursor.1.max(origin.1 + column_height + gap);
    }

    if monsters.is_empty() {
        let plate = status_plate(config, text, &geometry, status);
        let origin = bar_origin(config, "status").unwrap_or(cursor);
        place("status", &plate, origin.0, origin.1);
    }
}

fn bar_origin(config: &Config, id: &str) -> Option<(i32, i32)> {
    config
        .overlay
        .bars
        .iter()
        .find(|bar| bar.id == id)
        .map(|bar| (bar.x, bar.y))
}

/// Color de acento de la interfaz: el borde del cristal y la franja a su izquierda.
fn accent(style: &StyleSection) -> Color {
    style.capture.with_alpha(0xff)
}

/// Fondo del tema `glass`: gris azulado claro y translúcido, no otro bloque oscuro.
fn glass_fill(style: &StyleSection) -> Color {
    lighten(style.panel, 0.3).with_alpha(0xd2)
}

/// Pista de una barra: en cristal apenas un velo; en brutal, un negro sólido.
fn track_fill(style: &StyleSection, theme: PanelTheme, classic: f32) -> Color {
    match theme {
        PanelTheme::Classic => lighten(style.panel, classic),
        PanelTheme::Glass => Color::rgb(0xff, 0xff, 0xff).with_alpha(26),
        PanelTheme::Brutal => Color::rgb(0x00, 0x00, 0x00),
    }
}

/// En `brutal` todo bloque va recto y sin degradado, aunque la barra sea redondeada.
fn is_blocky(theme: PanelTheme, style: BarStyle) -> bool {
    theme == PanelTheme::Brutal || style == BarStyle::Blocks
}

/// Fondo y borde de una superficie: el panel entero o una placa del modo libre.
///
/// `clear_inside` es para las placas: su lienzo es transparente y el fondo se pinta encima de
/// un borde que, sin borrar, duplicaría el alfa y lo volvería casi opaco.
fn paint_surface(
    canvas: &mut Canvas,
    style: &StyleSection,
    radius: f32,
    scale: f32,
    stripe: bool,
    clear_inside: bool,
) {
    let theme = style.panel_theme;
    let full = Rect::new(0, 0, canvas.width(), canvas.height());
    let radius = radius.max(0.0);

    if theme == PanelTheme::Brutal {
        // Esquinas rectas y marco duro: el panel se lee como un cartel, no como una tarjeta.
        let edge = (3.0 * scale).round().max(1.0) as i32;
        canvas.fill_rect(full, lighten(style.panel, 0.55));
        canvas.fill_rect(full.inset(edge, edge), style.panel);
        return;
    }

    let (border, edge) = match theme {
        PanelTheme::Glass => (accent(style), (2.0 * scale).round().max(1.0)),
        _ => (lighten(style.panel, 0.18), 1.0),
    };
    let inner = full.inset(edge as i32, edge as i32);
    let inner_radius = (radius - edge).max(0.0);
    let fill = if theme == PanelTheme::Glass {
        glass_fill(style)
    } else {
        style.panel
    };

    canvas.fill_rounded(full, radius, border);

    if clear_inside {
        canvas.clear_rounded(inner, inner_radius);
    }

    canvas.fill_rounded(inner, inner_radius, fill);

    if stripe {
        let width = (3.0 * scale).round().max(1.0) as u32;
        canvas.fill_rect(
            Rect::new(inner.x, inner.y, width, inner.height),
            accent(style),
        );
    }
}

/// Lienzo transparente con el borde y el fondo de una placa.
fn plate_canvas(width: f32, height: f32, geometry: &Geometry, style: &StyleSection) -> Canvas {
    let width = width.round().max(1.0) as u32;
    let height = height.round().max(1.0) as u32;
    let mut canvas = Canvas::transparent(width, height);
    paint_surface(
        &mut canvas,
        style,
        8.0 * geometry.scale,
        geometry.scale,
        false,
        true,
    );

    canvas
}

/// Geometría de una placa: el mismo dibujo que el panel, con menos margen interior.
fn plate_geometry(geometry: &Geometry) -> Geometry {
    let mut local = Geometry::new(geometry.scale);
    local.padding = FREE_PADDING * geometry.scale;
    local
}

fn row_plate(config: &Config, text: &Text, geometry: &Geometry, row: Row<'_>) -> Canvas {
    let local = plate_geometry(geometry);
    let height = geometry.row + local.padding * 2.0;
    let mut canvas = plate_canvas(
        FREE_ROW_WIDTH * geometry.scale,
        height,
        geometry,
        &config.style,
    );
    draw_row(&mut canvas, config, text, &local, local.padding, row);
    canvas
}

fn part_plate(
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    part: &Part,
    label: &str,
) -> Canvas {
    let broken = part.is_broken();

    row_plate(
        config,
        text,
        geometry,
        Row {
            label,
            fraction: part_fraction(config, part),
            color: if broken {
                config.style.capture
            } else if part.severable.is_some() {
                config.style.severable
            } else {
                config.style.part
            },
            recoverable: 0.0,
            segments: part.breaks.map(|breaks| breaks.total),
            value: if broken {
                Some("Roto".to_string())
            } else if config.overlay.show_part_health {
                part.health.map(|health| health.label())
            } else {
                part.breaks.map(|breaks| breaks.label())
            },
        },
    )
}

fn hunt_plate(config: &Config, text: &Text, geometry: &Geometry, hunt: &Hunt) -> Canvas {
    let local = plate_geometry(geometry);
    let height = geometry.row + local.padding * 2.0;
    let mut canvas = plate_canvas(
        FREE_HUNT_WIDTH * geometry.scale,
        height,
        geometry,
        &config.style,
    );
    draw_hunt(&mut canvas, config, text, &local, hunt, local.padding);
    canvas
}

fn monster_plate(config: &Config, text: &Text, geometry: &Geometry, monster: &Monster) -> Canvas {
    let local = plate_geometry(geometry);
    let width = monster_plate_width(config, text, geometry, monster);
    let height =
        local.padding * 2.0 + geometry.header + geometry.small * 2.0 + geometry.bar.max(4.0);
    let mut canvas = plate_canvas(width, height, geometry, &config.style);
    let mut y = local.padding;
    y = draw_title(&mut canvas, config, text, &local, monster, y);
    draw_health_bar(&mut canvas, config, &local, monster, y);
    canvas
}

/// Ancho justo para que el nombre, el porcentaje y los números no se pisen.
fn monster_plate_width(
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    monster: &Monster,
) -> f32 {
    let scale = geometry.scale;
    let name = text.width(
        &title_label(config.style.panel_theme, monster),
        17.0 * scale,
    );
    let percent = text.width(&percent_label(monster), 14.0 * scale);
    let crown = if monster.crown == Crown::None {
        0.0
    } else {
        CROWN_SLOT * scale
    };
    let numbers = if config.overlay.show_health_numbers {
        text.width(&monster.health.label(), TITLE_VALUE_SIZE * scale) + 8.0 * scale
    } else {
        0.0
    };
    let capture = if monster.is_capturable() {
        text.width("captura", TITLE_VALUE_SIZE * scale) + 8.0 * scale
    } else {
        0.0
    };

    (FREE_MONSTER_WIDTH * scale).max(name + percent + crown + numbers + capture + 40.0 * scale)
}

fn weapon_plate(config: &Config, text: &Text, geometry: &Geometry, weapon: &Weapon) -> Canvas {
    let local = plate_geometry(geometry);
    let chips = weapon_chips(weapon, &config.style);
    let rows = if chips.is_empty() {
        0
    } else {
        chip_rows(
            text,
            &chips,
            &local,
            FREE_WEAPON_WIDTH * geometry.scale - local.padding * 2.0,
        )
    };
    let mut height = local.padding * 2.0 + geometry.weapon;

    if weapon.sharpness.is_some() {
        height += SHARPNESS_BAR * geometry.scale + geometry.small;
    }

    if matches!(weapon.gauge, Some(WeaponGauge::Spirit { .. })) {
        height += geometry.spirit + geometry.small;
    }

    if matches!(weapon.gauge, Some(WeaponGauge::InsectGlaive { .. })) {
        height += insect_glaive_height(text, &local) + geometry.small;
    }

    if rows > 0 {
        height += rows as f32 * geometry.chip
            + rows.saturating_sub(1) as f32 * geometry.chip_gap
            + geometry.small;
    }

    let mut canvas = plate_canvas(
        FREE_WEAPON_WIDTH * geometry.scale,
        height,
        geometry,
        &config.style,
    );
    let mut y = draw_weapon(&mut canvas, config, text, &local, weapon, local.padding);

    if matches!(weapon.gauge, Some(WeaponGauge::Spirit { .. })) {
        y = draw_spirit_sword(&mut canvas, config, text, &local, weapon, y);
        y += geometry.small;
    }

    if matches!(weapon.gauge, Some(WeaponGauge::InsectGlaive { .. })) {
        y = draw_insect_glaive(&mut canvas, config, text, &local, weapon, y);
        y += geometry.small;
    }

    if !chips.is_empty() {
        draw_chips(&mut canvas, config, text, &local, y, &chips);
    }

    canvas
}

fn buffs_plate(config: &Config, text: &Text, geometry: &Geometry, buffs: &[Buff]) -> Canvas {
    let local = plate_geometry(geometry);
    let rows = buff_rows(buffs.len());
    let height = local.padding * 2.0
        + rows as f32 * geometry.buff
        + rows.saturating_sub(1) as f32 * geometry.buff_gap;
    let mut canvas = plate_canvas(
        FREE_BUFFS_WIDTH * geometry.scale,
        height,
        geometry,
        &config.style,
    );
    draw_buffs(&mut canvas, config, text, &local, local.padding, buffs);
    canvas
}

fn status_plate(config: &Config, text: &Text, geometry: &Geometry, status: Option<&str>) -> Canvas {
    let size = 14.0 * geometry.scale;
    let label = status.unwrap_or("sin monstruos grandes en la zona");
    let height = text.height(size) + geometry.padding * 2.0;
    let width = (text.width(label, size) + geometry.padding * 2.0 + 12.0 * geometry.scale)
        .max(FREE_ROW_WIDTH * geometry.scale);
    let mut canvas = plate_canvas(width, height, geometry, &config.style);

    text.draw(
        &mut canvas,
        label,
        geometry.padding,
        geometry.padding + text.ascent(size),
        size,
        if status.is_some() {
            config.style.low_health
        } else {
            config.style.muted
        },
    );

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
    spirit: f32,
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
            spirit: SPIRIT_SWORD * scale,
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

    fn width(
        &self,
        config: &Config,
        text: &Text,
        snapshot: &Snapshot,
        monsters: &[&Monster],
    ) -> u32 {
        let name_size = 17.0 * self.scale;
        let percent_size = 14.0 * self.scale;
        let mut logical = BASE_WIDTH;

        for monster in monsters {
            let name = text.width(&title_label(config.style.panel_theme, monster), name_size);
            let percent = text.width(&percent_label(monster), percent_size);
            let crown = if monster.crown == Crown::None {
                0.0
            } else {
                CROWN_SLOT
            };
            let numbers = if config.overlay.show_health_numbers {
                text.width(&monster.health.label(), TITLE_VALUE_SIZE * self.scale) + 8.0
            } else {
                0.0
            };
            let capture = if monster.is_capturable() {
                text.width("captura", TITLE_VALUE_SIZE * self.scale) + 8.0
            } else {
                0.0
            };
            logical = logical.max(name + percent + crown + numbers + capture + 40.0);
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
        monsters: &[&Monster],
    ) -> u32 {
        let inner = self.width(config, text, snapshot, monsters) as f32 - self.padding * 2.0;
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

/// Monstruos que entran al panel según el modo de selección.
///
/// En modo objetivo, si el juego (o un mapa viejo) no marca ninguno, se cae a la zona entera:
/// un panel vacío a mitad de cacería es peor que uno lleno.
fn visible_monsters<'a>(config: &Config, snapshot: &'a Snapshot) -> Vec<&'a Monster> {
    let all = || snapshot.monsters.iter();
    let selected = match config.overlay.monsters {
        Monsters::Zone => all().collect::<Vec<_>>(),
        Monsters::Target => {
            let targeted: Vec<_> = all().filter(|monster| monster.target).collect();
            if targeted.is_empty() {
                all().collect()
            } else {
                targeted
            }
        }
    };

    selected
        .into_iter()
        .take(config.overlay.max_monsters)
        .collect()
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
            .filter(|ailment| ailment.is_active() && !ailment.is_elemental())
            .take(MAX_AILMENTS)
            .count();
    }

    rows
}

/// Parte visible: la cola rompible y la cortable del mismo nombre se funden en una fila.
///
/// World guarda en la tabla dos entradas para algunas colas (una por mecánica). Sin fundirlas,
/// el panel muestra dos "Cola" y una de ellas parece un rompimiento duplicado.
fn visible_parts(monster: &Monster) -> Vec<Part> {
    let mut parts: Vec<Part> = Vec::new();

    for part in monster.parts.iter().filter(|part| {
        // Una parte rota conserva el componente aunque su valor ya no se lea: sigue en el
        // panel para enseñar "Roto" en vez de desaparecer.
        part.breakable.is_some()
            || part.severable.is_some()
            || part.breakable_present
            || part.severable_present
    }) {
        if let Some(index) = parts.iter().position(|existing| same_tail(existing, part)) {
            merge_parts(&mut parts[index], part.clone());
        } else {
            parts.push(part.clone());
        }
    }

    parts.sort_by_key(|part| part.index);
    parts.truncate(MAX_PARTS);
    parts
}

/// Solo se funden dos entradas con el mismo nombre cuando una corta y la otra rompe.
fn same_tail(left: &Part, right: &Part) -> bool {
    left.name.is_some()
        && left.name == right.name
        && ((left.severable.is_some() && right.severable.is_none() && right.breakable.is_some())
            || (right.severable.is_some() && left.severable.is_none() && left.breakable.is_some()))
}

/// La fila conserva la barra de corte y el contador de la entrada rompible.
fn merge_parts(target: &mut Part, other: Part) {
    if other.severable.is_some() {
        target.severable = other.severable;
        target.index = other.index;
        target.name = other.name;
        target.health = other.health;
    }

    if target.breaks.is_none() {
        target.breaks = other.breaks;
    }

    if target.breakable.is_none() {
        target.breakable = other.breakable;
    }

    target.breakable_present |= other.breakable_present;
    target.severable_present |= other.severable_present;
}

/// Lado en unidades lógicas y color de la corona, o `None` si no hay ninguna.
fn crown_mark(crown: Crown) -> Option<(f32, Color)> {
    match crown {
        Crown::None => None,
        Crown::Mini => Some((11.0, CROWN_SILVER)),
        Crown::Silver => Some((14.0, CROWN_SILVER)),
        Crown::Gold => Some((14.0, CROWN_GOLD)),
    }
}

fn monster_label(monster: &Monster) -> String {
    match monster.name.as_deref() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => format!("monstruo {}", monster.id),
    }
}

/// Nombre del monstruo como lo pinta el tema: `brutal` lo pasa a mayúsculas.
fn title_label(theme: PanelTheme, monster: &Monster) -> String {
    let label = monster_label(monster);

    if theme == PanelTheme::Brutal {
        label.to_uppercase()
    } else {
        label
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
    let theme = style.panel_theme;
    let scale = geometry.scale;
    let name_size = 17.0 * geometry.scale;
    let percent_size = 14.0 * geometry.scale;
    let label = title_label(theme, monster);

    // En brutal el título se apoya en una banda sólida con su bloque de acento: se lee como
    // cabecera de cartel y no como texto suelto sobre el fondo.
    if theme == PanelTheme::Brutal {
        let band = Rect::new(
            0,
            y.round() as i32,
            canvas.width(),
            geometry.header.round().max(1.0) as u32,
        );
        canvas.fill_rect(band, lighten(style.panel, 0.26));
        canvas.fill_rect(
            Rect::new(
                0,
                band.y,
                (4.0 * scale).round().max(1.0) as u32,
                band.height,
            ),
            accent(style),
        );
    }

    let name_width = text.width(&label, name_size);
    text.draw(
        canvas,
        &label,
        geometry.padding,
        y + text.ascent(name_size),
        name_size,
        style.text,
    );

    // La corona va pegada al nombre: es un dato de la instancia, no del monstruo en general.
    if let Some((size, color)) = crown_mark(monster.crown) {
        let badge = size * geometry.scale;
        let left = geometry.padding + name_width + 6.0 * geometry.scale;
        icons::crown(
            canvas,
            left,
            y + (geometry.header - badge) / 2.0,
            badge,
            monster.crown == Crown::Mini,
            color,
        );
    }

    let percent = percent_label(monster);
    let percent_width = text.width(&percent, percent_size);
    let right = canvas.width() as f32 - geometry.padding;
    let percent_left = right - percent_width;
    let value_size = TITLE_VALUE_SIZE * geometry.scale;
    let gap = 8.0 * geometry.scale;
    let mut column = percent_left;

    text.draw(
        canvas,
        &percent,
        percent_left,
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
        column -= gap + text.width(label, value_size);
        text.draw(
            canvas,
            label,
            column,
            y + text.ascent(value_size),
            value_size,
            style.capture,
        );
    }

    if config.overlay.show_health_numbers {
        let numbers = monster.health.label();
        column -= gap + text.width(&numbers, value_size);
        text.draw(
            canvas,
            &numbers,
            column,
            y + text.ascent(value_size),
            value_size,
            style.muted,
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
    let theme = style.panel_theme;
    let scale = geometry.scale;
    let bar = Rect::new(
        geometry.padding as i32,
        y.round() as i32,
        geometry.inner_width(canvas).max(1.0) as u32,
        geometry.bar.max(4.0) as u32,
    );
    let radius = if theme == PanelTheme::Brutal {
        0.0
    } else {
        match style.bar_style {
            BarStyle::Rounded => 3.0 * scale,
            BarStyle::Blocks => 0.0,
            BarStyle::Pill => bar.height as f32 / 2.0,
        }
    };

    // El borde separa la barra del panel y la pista queda hundida un píxel. En brutal el
    // marco es claro y la pista negra: bloques duros en vez de una ranura.
    let frame = if theme == PanelTheme::Brutal {
        lighten(style.panel, 0.55)
    } else {
        track_fill(style, theme, 0.26)
    };
    canvas.fill_rounded(bar, radius, frame);
    canvas.fill_rounded(
        bar.inset(1, 1),
        (radius - 1.0).max(0.0),
        track_fill(style, theme, 0.06),
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
        let inner_radius = (radius - 1.0).max(0.0);
        // En tramos el relleno va plano: el degradado suaviza los cortes y aquí se busca lo
        // contrario, que cada tramo se lea como un bloque.
        if is_blocky(theme, style.bar_style) {
            canvas.fill_rounded(fill, inner_radius, base);
        } else {
            canvas.fill_rounded_gradient(fill, inner_radius, lighten(base, 0.34), base);
        }
    }

    // Marcas cada 10 %: la barra se lee como una escala y no como una mancha de color.
    let tick = if theme == PanelTheme::Brutal {
        Color::rgb(0x00, 0x00, 0x00)
    } else if style.bar_style == BarStyle::Blocks {
        style.panel
    } else {
        lighten(style.panel, 0.5).with_alpha(60)
    };
    let tick_width = if is_blocky(theme, style.bar_style) {
        (scale * 2.0).max(2.0)
    } else {
        scale.max(1.0)
    };
    for step in 1..10 {
        let x = bar.x as f32 + bar.width as f32 * (step as f32 / 10.0);
        canvas.fill_rect(
            Rect::new(
                x.round() as i32,
                bar.y + 1,
                tick_width.round() as u32,
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

        if matches!(weapon.gauge, Some(WeaponGauge::Spirit { .. })) {
            height += geometry.spirit + geometry.small;
        }

        if matches!(weapon.gauge, Some(WeaponGauge::InsectGlaive { .. })) {
            height += insect_glaive_height(text, geometry) + geometry.small;
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

    if player.wirebugs.is_some_and(|wirebugs| wirebugs.max > 0) {
        height += geometry.small + geometry.row;
    }

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
    let style = &config.style;
    let (color, height) = match style.panel_theme {
        PanelTheme::Classic => (lighten(style.panel, 0.12), geometry.divider),
        PanelTheme::Glass => (accent(style).with_alpha(96), geometry.divider),
        PanelTheme::Brutal => (lighten(style.panel, 0.35), (2.0 * geometry.scale).max(1.0)),
    };

    canvas.fill_rect(
        Rect::new(
            geometry.padding as i32,
            y.round() as i32,
            (canvas.width() as f32 - geometry.padding * 2.0).max(1.0) as u32,
            height.max(1.0) as u32,
        ),
        color,
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

        if matches!(weapon.gauge, Some(WeaponGauge::Spirit { .. })) {
            y = draw_spirit_sword(canvas, config, text, geometry, weapon, y);
            y += geometry.small;
        }

        if matches!(weapon.gauge, Some(WeaponGauge::InsectGlaive { .. })) {
            y = draw_insect_glaive(canvas, config, text, geometry, weapon, y);
            y += geometry.small;
        }

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

    if let Some(wirebugs) = player.wirebugs.filter(|wirebugs| wirebugs.max > 0) {
        y += geometry.small;
        y = draw_wirebugs(canvas, config, text, geometry, wirebugs, y);
    }

    if !player.buffs.is_empty() {
        y += geometry.small;
        y = draw_buffs(canvas, config, text, geometry, y, &player.buffs);
    }

    y
}

/// Fila de los cordópteros: una barra con un tramo por cordóptero, las cargadas llenas y la
/// que se está recuperando a medio llenar. El bloqueo cambia el color y el valor lo dice.
fn draw_wirebugs(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    wirebugs: Wirebugs,
    y: f32,
) -> f32 {
    let style = &config.style;

    draw_row(
        canvas,
        config,
        text,
        geometry,
        y,
        Row {
            label: "Cordópteros",
            fraction: wirebugs.fraction(),
            segments: Some(u32::from(wirebugs.max)),
            value: Some(wirebugs.label()),
            color: if wirebugs.blocked {
                style.low_health
            } else {
                style.capture
            },
            recoverable: 0.0,
        },
    );

    y + geometry.row
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
        let value = format!("filo {:.0}", sharpness.hits_left());
        let value_width = text.width(&value, 12.0 * scale);
        let next = sharpness
            .next_segment()
            .map(|(level, hits)| format!("· {} {hits}", level.label()));
        let next_width = next
            .as_ref()
            .map_or(0.0, |label| text.width(label, 12.0 * scale) + 6.0 * scale);
        let mut x = right - next_width;

        if let Some(next) = &next {
            text.draw(
                canvas,
                next,
                x + 6.0 * scale,
                baseline,
                12.0 * scale,
                style.muted,
            );
        }
        text.draw(
            canvas,
            level,
            x - level_width,
            baseline,
            12.0 * scale,
            sharpness_color(sharpness.level),
        );
        x -= level_width;
        text.draw(
            canvas,
            &value,
            x - value_width - 6.0 * scale,
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

/// Medidor de espíritu de la espada larga: una espada horizontal cuya hoja se llena con la
/// carga y toma el color del nivel (blanco, amarillo o rojo).
///
/// La forma importa: un número o una barra genérica obliga a pensar; el color de la hoja se lee
/// de un vistazo, como el medidor del juego.
fn draw_spirit_sword(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    weapon: &Weapon,
    y: f32,
) -> f32 {
    let Some(WeaponGauge::Spirit {
        buildup,
        level,
        timer,
        max_timer,
        regen,
    }) = weapon.gauge
    else {
        return y;
    };

    let style = &config.style;
    let scale = geometry.scale;
    let size = 11.0 * scale;
    let color = spirit_color(level);
    let mut info = format!("Nv {level}");

    if max_timer > 0.0 {
        info.push_str(&format!(" · {:.0}s", timer.max(0.0)));
    }

    if regen > 0.0 {
        info.push_str(&format!(" · +{:.0}s", regen));
    }

    let right = canvas.width() as f32 - geometry.padding;
    let info_width = text.width(&info, size);
    let gap = 10.0 * scale;
    let x = geometry.padding;
    let sword_width = (right - info_width - gap - x).clamp(90.0 * scale, 280.0 * scale);
    let center = y + geometry.spirit * 0.5;
    let half = 4.0 * scale;
    let handle_start = x + 3.0 * scale;
    let guard_x = x + 13.0 * scale;
    let blade_start = guard_x + 3.0 * scale;
    let blade_end = x + sword_width;
    let tip = blade_end - 15.0 * scale;

    // Empuñadura y pomo; el guardia separa la mano de la hoja.
    canvas.fill_line(
        (handle_start, center),
        (guard_x - 2.0 * scale, center),
        4.0 * scale,
        style.muted,
    );
    canvas.fill_diamond((x + 2.0 * scale, center), 2.6 * scale, style.muted);
    canvas.fill_line(
        (guard_x, center - 7.0 * scale),
        (guard_x, center + 7.0 * scale),
        2.2 * scale,
        lighten(color, 0.35),
    );

    // Hoja apagada primero: la parte sin cargar también tiene forma de espada.
    canvas.fill_line(
        (blade_start, center),
        (tip, center),
        half * 1.5,
        lighten(style.panel, 0.12),
    );

    let fraction = (buildup / 100.0).clamp(0.0, 1.0);
    let filled_end = blade_start + (blade_end - blade_start) * fraction;
    let body_end = filled_end.min(tip);

    if fraction > 0.0 {
        canvas.fill_line((blade_start, center), (body_end, center), half * 1.5, color);
        canvas.fill_line(
            (blade_start, center - half * 0.45),
            (body_end, center - half * 0.45),
            1.0 * scale,
            lighten(color, 0.5),
        );
    }

    // La punta solo se rellena cuando la carga llega al final de la hoja.
    if filled_end >= tip {
        canvas.fill_line(
            (tip, center - half),
            (blade_end, center),
            half * 0.75,
            color,
        );
        canvas.fill_line(
            (tip, center + half),
            (blade_end, center),
            half * 0.75,
            color,
        );
    }

    // Contorno al final: mantiene legible la silueta sobre cualquier relleno.
    for (from, to) in [
        ((blade_start, center - half), (tip, center - half)),
        ((tip, center - half), (blade_end, center)),
        ((blade_end, center), (tip, center + half)),
        ((tip, center + half), (blade_start, center + half)),
    ] {
        canvas.fill_line(from, to, 1.15 * scale, style.muted);
    }

    text.draw(
        canvas,
        &info,
        right - info_width,
        center - text.height(size) * 0.5 + text.ascent(size),
        size,
        if level == 0 { style.muted } else { color },
    );

    y + geometry.spirit
}

/// Barra de filo: guiones del color de cada nivel y una punta sobre el filo actual.
///
/// Los guiones a la izquierda de la punta son lo que queda; los de la derecha, el filo ya
/// gastado, se apagan. Así el color vigente y cuánto resta se leen sin números, y la punta da
/// el punto exacto sin tapar el color.
fn draw_sharpness(
    canvas: &mut Canvas,
    config: &Config,
    geometry: &Geometry,
    sharpness: &mh_overlay_games::Sharpness,
    y: f32,
) {
    let style = &config.style;
    let theme = style.panel_theme;
    let scale = geometry.scale;
    let height = (SHARPNESS_BAR * scale).max(3.0);
    let bar = Rect::new(
        geometry.padding as i32,
        y.round() as i32,
        geometry.inner_width(canvas).max(1.0) as u32,
        height.round() as u32,
    );
    let radius = if theme == PanelTheme::Brutal {
        0.0
    } else {
        match style.bar_style {
            BarStyle::Rounded => (2.0 * scale).max(0.0),
            BarStyle::Blocks => 0.0,
            BarStyle::Pill => height / 2.0,
        }
    };

    canvas.fill_rounded(bar, radius, track_fill(style, theme, 0.22));

    let total = sharpness.total();
    if total <= 0.0 {
        return;
    }

    // La punta marca el filo actual: a su izquierda queda lo que el arma todavía aguanta.
    let marker_x = (bar.x as f32 + bar.width as f32 * sharpness.fraction())
        .clamp(bar.x as f32 + 1.0, bar.right() as f32 - 1.0);
    let dash = (5.0 * scale).max(2.0);
    let gap = (2.0 * scale).max(1.0);
    let mut x = bar.x as f32;

    for (index, level) in SharpnessLevel::ALL.iter().enumerate() {
        let length = f32::from(sharpness.lengths[index]);

        if length <= 0.0 {
            continue;
        }

        let width = bar.width as f32 * length / total;
        let end = x + width;
        let color = sharpness_color(*level);
        let spent = mix_with(color, style.panel, 0.72);
        let mut start = x;

        while start < end - 0.5 {
            let stop = (start + dash).min(end);
            let cut = marker_x.clamp(start, stop);

            if cut > start {
                draw_dash(canvas, bar, start, cut, radius, color);
            }

            if stop > cut {
                draw_dash(canvas, bar, cut.max(start), stop, radius, spent);
            }

            start += dash + gap;
        }

        x = end;
    }

    // Remate en punta: un chevrón sobre la barra y una línea de un píxel que la atraviesa.
    let tip = bar.y as f32 - scale;
    let base = tip - (6.0 * scale);
    let half = (4.0 * scale).max(2.0);
    let stroke = (1.6 * scale).max(1.0);
    canvas.fill_line((marker_x, tip), (marker_x - half, base), stroke, style.text);
    canvas.fill_line((marker_x, tip), (marker_x + half, base), stroke, style.text);
    canvas.fill_rect(
        Rect::new(
            marker_x.round() as i32,
            bar.y,
            scale.round().max(1.0) as u32,
            bar.height,
        ),
        style.text,
    );
}

/// Un guion de la barra de filo, ya recortado al espacio entre la punta y el nivel.
fn draw_dash(canvas: &mut Canvas, bar: Rect, start: f32, stop: f32, radius: f32, color: Color) {
    let left = start.round() as i32;
    let right = stop.round() as i32;

    if right - left < 1 {
        return;
    }

    canvas.fill_rounded(
        Rect::new(left, bar.y, (right - left) as u32, bar.height),
        radius.min((right - left) as f32 / 2.0),
        color,
    );
}

/// Mezcla `color` con `towards`; apaga el filo gastado sin perder el tono del nivel.
fn mix_with(color: Color, towards: Color, amount: f32) -> Color {
    let mix = |from: u8, to: u8| -> u8 {
        (f32::from(from) + (f32::from(to) - f32::from(from)) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };

    Color {
        r: mix(color.r, towards.r),
        g: mix(color.g, towards.g),
        b: mix(color.b, towards.b),
        a: 0xff,
    }
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
        // La espada se dibuja aparte: no es una ficha, es el medidor entero.
        WeaponGauge::Spirit { .. } => Vec::new(),
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
        // El glaive tiene su propio medidor (tres barras de esencia y el aguante del kinsecto),
        // dibujado aparte: fichas sueltas no dejan comparar los tiempos de un vistazo.
        WeaponGauge::InsectGlaive { .. } => Vec::new(),
    }
}

/// Contador de esencias del glaive, tal como lo enseña la cabecera del medidor.
fn glaive_summary(extracts: &Extracts) -> String {
    let active: Vec<&str> = [
        (extracts.attack, "rojo"),
        (extracts.speed, "blanco"),
        (extracts.defense, "naranja"),
    ]
    .into_iter()
    .filter(|(on, _)| *on)
    .map(|(_, name)| name)
    .collect();

    match active.len() {
        3 => "3/3 · completo".to_string(),
        0 => "sin esencias · 0/3".to_string(),
        count => format!("{} · {count}/3", active.join(" + ")),
    }
}

/// Fracción de la barra de una esencia, o `None` cuando todavía no se conoce su duración
/// máxima: una barra llena al arrancar mentiría sobre el tiempo que le queda.
fn essence_fraction(remaining: f32, max: f32) -> Option<f32> {
    (remaining.is_finite() && max.is_finite() && max > 0.0)
        .then(|| (remaining / max).clamp(0.0, 1.0))
}

/// Alto del medidor del glaive: contador, tres barras de esencia con su tiempo y el kinsecto.
fn insect_glaive_height(text: &Text, geometry: &Geometry) -> f32 {
    let scale = geometry.scale;
    let header = text.height(11.0 * scale);
    let essence = 5.0 * scale + 2.0 * scale + text.height(10.0 * scale);
    let stamina = text.height(11.0 * scale).max(6.0 * scale);

    header + geometry.small + essence + geometry.small + stamina
}

/// Medidor del glaive insecto: una barra por esencia con su color real y el contador, y debajo
/// el aguante del kinsecto. Devuelve la `y` del bloque siguiente.
fn draw_insect_glaive(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    weapon: &Weapon,
    y: f32,
) -> f32 {
    let Some(WeaponGauge::InsectGlaive {
        extracts,
        attack,
        speed,
        defense,
        attack_max,
        speed_max,
        defense_max,
        stamina,
        max_stamina,
    }) = weapon.gauge
    else {
        return y;
    };

    let style = &config.style;
    let scale = geometry.scale;
    let inner = geometry.inner_width(canvas);
    let right = geometry.padding + inner;
    let bar_height = 5.0 * scale;
    let mut y = y;

    // Contador, con el acento que ya tenía la ficha al completar las tres esencias.
    let header_size = 11.0 * scale;
    let summary = glaive_summary(&extracts);
    let complete = extracts.attack && extracts.speed && extracts.defense;
    let baseline = y + text.ascent(header_size);
    text.draw(
        canvas,
        "Esencias",
        geometry.padding,
        baseline,
        header_size,
        style.muted,
    );
    text.draw(
        canvas,
        &summary,
        right - text.width(&summary, header_size),
        baseline,
        header_size,
        if complete { style.capture } else { style.text },
    );
    y += text.height(header_size) + geometry.small;

    // Una barra por esencia, en el orden del juego: rojo, blanco y naranja.
    let essence_size = 10.0 * scale;
    let item_gap = 8.0 * scale;
    let item_width = (inner - item_gap * 2.0).max(3.0 * scale) / 3.0;
    let essences = [
        (attack, attack_max, Color::rgb(0xe8, 0x55, 0x4a)),
        (speed, speed_max, Color::rgb(0xe8, 0xf0, 0xf8)),
        (defense, defense_max, Color::rgb(0xe0, 0x8a, 0x3c)),
    ];

    for (index, (remaining, max, color)) in essences.into_iter().enumerate() {
        let x = geometry.padding + index as f32 * (item_width + item_gap);
        let track = Rect::new(
            x.round() as i32,
            y.round() as i32,
            item_width.round().max(1.0) as u32,
            bar_height.round().max(1.0) as u32,
        );
        let radius = if style.bar_style == BarStyle::Blocks {
            0.0
        } else {
            bar_height / 2.0
        };

        canvas.fill_rounded(track, radius, track_fill(style, style.panel_theme, 0.18));

        if let Some(fraction) = essence_fraction(remaining, max) {
            let filled = (track.width as f32 * fraction).round() as u32;
            if filled > 1 {
                canvas.fill_rounded(
                    Rect::new(track.x, track.y, filled, track.height),
                    radius,
                    color,
                );
            }
        }

        // Sin máximo conocido la barra queda vacía y el tiempo sigue siendo el dato.
        let label = if remaining > 0.0 {
            format!("{remaining:.0}s")
        } else {
            "—".to_string()
        };
        text.draw(
            canvas,
            &label,
            x,
            y + bar_height + 2.0 * scale + text.ascent(essence_size),
            essence_size,
            style.muted,
        );
    }
    y += bar_height + 2.0 * scale + text.height(essence_size) + geometry.small;

    // Aguante del kinsecto, con la fracción que el jugador no ve en el HUD.
    let stamina_size = 11.0 * scale;
    let label = "Kinsecto";
    let value = if max_stamina > 0.0 {
        format!("{stamina:.0}/{max_stamina:.0}")
    } else {
        "—".to_string()
    };
    let value_width = text.width(&value, stamina_size);
    let bar_x = geometry.padding + text.width(label, stamina_size) + 6.0 * scale;
    let bar_right = right - value_width - 6.0 * scale;
    let bar = Rect::new(
        bar_x.round() as i32,
        (y + (text.height(stamina_size) - bar_height) * 0.5).round() as i32,
        (bar_right - bar_x).round().max(1.0) as u32,
        bar_height.round().max(1.0) as u32,
    );
    let radius = if style.bar_style == BarStyle::Blocks {
        0.0
    } else {
        bar.height as f32 / 2.0
    };
    let baseline = y + text.ascent(stamina_size);

    text.draw(
        canvas,
        label,
        geometry.padding,
        baseline,
        stamina_size,
        style.muted,
    );
    canvas.fill_rounded(bar, radius, track_fill(style, style.panel_theme, 0.18));

    if let Some(fraction) = essence_fraction(stamina, max_stamina) {
        let filled = (bar.width as f32 * fraction).round() as u32;
        if filled > 1 {
            canvas.fill_rounded(
                Rect::new(bar.x, bar.y, filled, bar.height),
                radius,
                style.part,
            );
        }
    }

    text.draw(
        canvas,
        &value,
        right - value_width,
        baseline,
        stamina_size,
        style.text,
    );

    y + text.height(stamina_size)
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

/// Color del medidor de espíritu según el nivel cargado: blanco, amarillo y rojo, como el
/// juego. Sin nivel, la hoja queda apagada.
fn spirit_color(level: u8) -> Color {
    match level {
        0 => Color::rgb(0x5c, 0x68, 0x7a),
        1 => Color::rgb(0xe8, 0xf0, 0xf8),
        2 => Color::rgb(0xf0, 0xd0, 0x4a),
        _ => Color::rgb(0xe8, 0x55, 0x4a),
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

    let value = row
        .value
        .unwrap_or_else(|| format!("{:.0}%", fraction * 100.0));
    let value_width = text.width(&value, size).max(VALUE_WIDTH * geometry.scale);
    let bar_height = geometry.track.max(3.0);
    let bar = Rect::new(
        geometry.padding as i32 + (LABEL_WIDTH * geometry.scale) as i32,
        (y + geometry.row * 0.5 - bar_height * 0.5).round() as i32,
        (inner - LABEL_WIDTH * geometry.scale - value_width).max(1.0) as u32,
        bar_height as u32,
    );
    let theme = style.panel_theme;
    let radius = if theme == PanelTheme::Brutal {
        0.0
    } else {
        match style.bar_style {
            BarStyle::Rounded | BarStyle::Pill => bar.height as f32 / 2.0,
            BarStyle::Blocks => 0.0,
        }
    };

    canvas.fill_rounded(bar, radius, track_fill(style, theme, 0.18));

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
        let filled_bar = Rect::new(bar.x, bar.y, filled, bar.height);

        // Brutal pinta bloques planos: el degradado es justo lo que ablanda el cartel.
        if theme == PanelTheme::Brutal {
            canvas.fill_rounded(filled_bar, radius, color);
        } else {
            canvas.fill_rounded_gradient(filled_bar, radius, lighten(color, 0.3), color);
        }
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
                track_fill(style, theme, 0.04),
            );
        }
    }

    text.draw(
        canvas,
        &value,
        canvas.width() as f32 - geometry.padding - text.width(&value, size),
        y + text.ascent(size),
        size,
        style.muted,
    );
}

fn part_fraction(config: &Config, part: &Part) -> f32 {
    // Con la vida numérica activada, la barra tiene que medir lo mismo que el número; si no,
    // una parte al 75 % de vida podría pintar un 50 % de progreso de rompimiento.
    if config.overlay.show_part_health
        && let Some(health) = part.health
    {
        return (health.percent() / 100.0).clamp(0.0, 1.0);
    }

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
    use mh_overlay_config::Layout;
    use mh_overlay_games::{
        Ailment, Breaks, Buff, Extracts, Health, Sharpness, WeaponGauge, WeaponKind,
    };

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
    fn the_target_mode_only_draws_the_monster_the_game_marks() {
        let (mut config, text) = config();
        config.overlay.monsters = Monsters::Target;

        let mut marked = monster(60.0);
        marked.target = true;
        let selected = render(
            &config,
            &text,
            &Snapshot::of(vec![monster(90.0), marked]),
            None,
        );
        let alone = render(&config, &text, &Snapshot::of(vec![monster(90.0)]), None);

        assert_eq!(selected.height(), alone.height());
    }

    #[test]
    fn the_target_mode_falls_back_to_the_zone_without_a_target() {
        let (mut config, text) = config();
        config.overlay.monsters = Monsters::Target;
        let both = vec![monster(80.0), monster(40.0)];

        let fallback = render(&config, &text, &Snapshot::of(both), None);
        let one = render(&config, &text, &Snapshot::of(vec![monster(80.0)]), None);

        assert!(fallback.height() > one.height());
    }

    #[test]
    fn a_crown_draws_a_badge_next_to_the_name() {
        let (config, text) = config();
        let plain = render(&config, &text, &Snapshot::of(vec![monster(80.0)]), None);
        let mut crowned = monster(80.0);
        crowned.crown = Crown::Gold;
        let gold = render(&config, &text, &Snapshot::of(vec![crowned]), None);

        assert_eq!(plain.height(), gold.height());
        assert_eq!(plain.width(), gold.width());
        let changed = plain
            .pixels()
            .iter()
            .zip(gold.pixels())
            .filter(|(left, right)| left != right)
            .count();
        assert!(changed > 0, "la corona no cambió ningún píxel");
    }

    #[test]
    fn the_title_grows_to_fit_the_raw_health() {
        let (mut config, text) = config();
        let mut named = Monster::new(101, Health::new(12_345.0, 23_456.0));
        named.name = Some("Un nombre de monstruo bastante largo para forzar el ancho".to_string());
        let snapshot = Snapshot::of(vec![named]);
        let plain = render(&config, &text, &snapshot, None);

        config.overlay.show_health_numbers = true;
        let numbered = render(&config, &text, &snapshot, None);

        assert!(numbered.width() > plain.width());
    }

    #[test]
    fn a_cut_and_a_break_entry_of_the_same_tail_share_a_row() {
        let mut hunted = Monster::new(34, Health::new(100.0, 100.0));
        hunted.parts = vec![
            Part {
                index: 0,
                name: Some("Cola"),
                flinch: None,
                breakable: None,
                severable: Some(Health::new(80.0, 100.0)),
                health: Some(Health::new(80.0, 100.0)),
                breaks: None,
                ..Part::default()
            },
            Part {
                index: 7,
                name: Some("Cola"),
                flinch: None,
                breakable: Some(Health::new(50.0, 100.0)),
                severable: None,
                health: Some(Health::new(50.0, 100.0)),
                breaks: Some(Breaks::new(1, Some(3), 0, 1)),
                ..Part::default()
            },
        ];

        let rows = visible_parts(&hunted);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].index, 0);
        assert_eq!(rows[0].health, Some(Health::new(80.0, 100.0)));
        assert!(rows[0].breaks.is_some());
    }

    #[test]
    fn raw_part_health_replaces_the_break_counter() {
        let (mut config, text) = config();
        let mut detailed = monster(60.0);
        detailed.parts.push(Part {
            index: 1,
            name: Some("cola"),
            flinch: None,
            breakable: Some(Health::new(75.0, 100.0)),
            severable: None,
            health: Some(Health::new(1_500.0, 2_000.0)),
            breaks: Some(Breaks::new(1, Some(2), 1, 2)),
            ..Part::default()
        });
        let snapshot = Snapshot::of(vec![detailed]);
        let counter = render(&config, &text, &snapshot, None);

        config.overlay.show_part_health = true;
        let numbers = render(&config, &text, &snapshot, None);

        assert_eq!(counter.width(), numbers.width());
        assert!((part_fraction(&config, &snapshot.monsters[0].parts[0]) - 0.75).abs() < 0.001);
        let changed = counter
            .pixels()
            .iter()
            .zip(numbers.pixels())
            .filter(|(left, right)| left != right)
            .count();
        assert!(changed > 0, "la vida de la parte no cambió ningún píxel");
    }

    fn alpha(canvas: &Canvas, x: i32, y: i32) -> u8 {
        canvas.pixel(x, y).map_or(0, |pixel| (pixel >> 24) as u8)
    }

    #[test]
    fn free_mode_leaves_the_background_transparent() {
        let (mut config, text) = config();
        config.overlay.layout = Layout::Free;
        let canvas = render_free(&config, &text, &Snapshot::default(), None, (640, 480));

        assert_eq!(canvas.pixel(0, 0), Some(0));
        assert!(
            alpha(&canvas, 40, 36) > 0,
            "la placa de estado no se dibujó"
        );
    }

    #[test]
    fn a_monster_group_moves_its_parts_to_the_right() {
        let (mut config, text) = config();
        let mut hunted = monster(50.0);
        hunted.parts.push(Part {
            index: 0,
            name: Some("Cola"),
            flinch: None,
            breakable: Some(Health::new(40.0, 100.0)),
            severable: None,
            health: Some(Health::new(40.0, 100.0)),
            breaks: None,
            ..Part::default()
        });
        config.overlay.bars = vec![mh_overlay_config::BarPlacement {
            id: "monster.0".to_string(),
            x: 100,
            y: 100,
            direction: Direction::Right,
        }];
        let canvas = render_free(
            &config,
            &text,
            &Snapshot::of(vec![hunted]),
            None,
            (800, 400),
        );

        assert!(alpha(&canvas, 110, 110) > 0, "falta la vida del monstruo");
        assert!(alpha(&canvas, 460, 120) > 0, "la parte no siguió a la vida");
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
                health: Some(Health::new(40.0, 100.0)),
                breaks: Some(Breaks::new(1, Some(2), 1, 2)),
                ..Part::default()
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
            wirebugs: None,
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
            wirebugs: None,
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
            wirebugs: None,
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
    fn the_spirit_gauge_draws_the_blade_in_the_level_color() {
        let (config, text) = config();
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            wirebugs: None,
            weapon: Some(Weapon {
                kind: WeaponKind::Longsword,
                sharpness: None,
                gauge: Some(WeaponGauge::Spirit {
                    buildup: 80.0,
                    level: 3,
                    timer: 98.0,
                    max_timer: 140.0,
                    regen: 0.0,
                }),
                detail: WeaponDetail::default(),
            }),
            buffs: Vec::new(),
        };
        let canvas = render(&config, &text, &preview(&player), None);
        let red = canvas
            .pixels()
            .iter()
            .filter(|pixel| {
                let r = (**pixel >> 16) & 0xff;
                let g = (**pixel >> 8) & 0xff;
                let b = **pixel & 0xff;
                r > 180 && g < 120 && b < 120
            })
            .count();

        assert!(red > 20, "la espada de nivel 3 no pintó la hoja de rojo");
    }

    #[test]
    fn spirit_levels_use_white_yellow_and_red() {
        assert_eq!(spirit_color(1), Color::rgb(0xe8, 0xf0, 0xf8));
        assert_eq!(spirit_color(2), Color::rgb(0xf0, 0xd0, 0x4a));
        assert_eq!(spirit_color(3), Color::rgb(0xe8, 0x55, 0x4a));
    }

    #[test]
    fn buffs_are_capped_so_they_do_not_cover_the_monsters() {
        let (config, text) = config();
        let mut player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            wirebugs: None,
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
            wirebugs: None,
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
            wirebugs: Some(Wirebugs::new(2, 1, Some(0.5), false)),
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
    fn wirebugs_add_a_row_and_the_blocked_state_changes_it() {
        let (config, text) = config();
        let mut player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: None,
            wirebugs: None,
            buffs: Vec::new(),
        };
        let without = render(&config, &text, &preview(&player), None);

        player.wirebugs = Some(Wirebugs::new(2, 1, Some(0.5), false));
        let with = render(&config, &text, &preview(&player), None);
        assert!(
            with.height() > without.height(),
            "la fila de cordópteros no añadió alto"
        );

        player.wirebugs = Some(Wirebugs::new(2, 1, Some(0.5), true));
        let blocked = render(&config, &text, &preview(&player), None);
        let changed = with
            .pixels()
            .iter()
            .zip(blocked.pixels())
            .filter(|(left, right)| left != right)
            .count();
        assert!(changed > 0, "el bloqueo no cambió ningún píxel");
    }

    #[test]
    fn the_sharpness_colors_are_drawn_in_the_bar() {
        let (config, text) = config();
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            wirebugs: None,
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
            health: Some(Health::new(40.0, 100.0)),
            breaks: Some(Breaks::new(1, Some(2), 1, 2)),
            ..Part::default()
        });
        detailed.ailments.push(Ailment {
            name: "veneno",
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

    #[test]
    fn the_panel_theme_changes_the_surface_at_a_glance() {
        let (mut config, text) = config();
        let snapshot = Snapshot::of(vec![monster(50.0)]);
        let classic = render(&config, &text, &snapshot, None);

        assert_eq!(config.style.panel_theme, PanelTheme::Classic);
        assert_eq!(
            classic.pixel(0, 0),
            Some(0xff00_0000),
            "el clásico conserva las esquinas recortadas"
        );

        config.style.panel_theme = PanelTheme::Glass;
        let glass = render(&config, &text, &snapshot, None);
        config.style.panel_theme = PanelTheme::Brutal;
        let brutal = render(&config, &text, &snapshot, None);

        let changed = |other: &Canvas| {
            classic
                .pixels()
                .iter()
                .zip(other.pixels())
                .filter(|(left, right)| left != right)
                .count()
        };

        assert!(
            changed(&glass) > 2000,
            "cristal cambia {} px",
            changed(&glass)
        );
        assert!(
            changed(&brutal) > 2000,
            "brutal cambia {} px",
            changed(&brutal)
        );
        assert_eq!(
            glass.pixel(0, 0),
            Some(0xff00_0000),
            "cristal mantiene la esquina redondeada"
        );
        assert_ne!(
            brutal.pixel(0, 0),
            Some(0xff00_0000),
            "brutal deja la esquina recta y pintada"
        );
    }

    #[test]
    fn the_free_plates_follow_the_panel_theme() {
        let (mut config, text) = config();
        let snapshot = Snapshot::of(vec![monster(50.0)]);
        let classic = render_free(&config, &text, &snapshot, None, (640, 480));

        config.style.panel_theme = PanelTheme::Brutal;
        let brutal = render_free(&config, &text, &snapshot, None, (640, 480));

        // La placa del monstruo arranca en el margen: el clásico le corta la esquina y brutal
        // la deja recta, igual que el panel.
        assert_eq!(alpha(&classic, 24, 24), 0);
        assert!(alpha(&brutal, 24, 24) > 0);
    }

    #[test]
    fn the_sharpness_bar_keeps_its_dashes_with_every_bar_style() {
        let sharpness = Sharpness {
            lengths: [40, 30, 30, 30, 30, 20, 0],
            level: SharpnessLevel::White,
            max_level: SharpnessLevel::White,
            current: 180.0,
        };
        let geometry = Geometry::new(1.0);
        let mut reference = Canvas::new(1, 1);
        reference.fill_rect(
            Rect::new(0, 0, 1, 1),
            track_fill(&Config::default().style, PanelTheme::Classic, 0.22),
        );
        let track = reference.pixel(0, 0).unwrap();

        for bar_style in BarStyle::ALL {
            let mut config = Config::default();
            config.style.bar_style = bar_style;
            let mut canvas = Canvas::new(320, 40);
            draw_sharpness(&mut canvas, &config, &geometry, &sharpness, 4.0);

            let row: Vec<u32> = (14..306).filter_map(|x| canvas.pixel(x, 8)).collect();
            let gaps = row.iter().filter(|pixel| **pixel == track).count();
            let white = sharpness_color(SharpnessLevel::White).to_argb() | 0xff00_0000;

            assert!(
                gaps > 40,
                "{bar_style:?}: solo {gaps} píxeles de hueco entre guiones"
            );
            assert!(
                row.contains(&white),
                "{bar_style:?}: falta el color del filo actual"
            );
        }
    }

    fn glaive(
        extracts: Extracts,
        attack: (f32, f32),
        speed: (f32, f32),
        defense: (f32, f32),
    ) -> Weapon {
        Weapon {
            kind: WeaponKind::InsectGlaive,
            sharpness: None,
            gauge: Some(WeaponGauge::InsectGlaive {
                extracts,
                attack: attack.0,
                attack_max: attack.1,
                speed: speed.0,
                speed_max: speed.1,
                defense: defense.0,
                defense_max: defense.1,
                stamina: 80.0,
                max_stamina: 100.0,
            }),
            detail: WeaponDetail::default(),
        }
    }

    #[test]
    fn the_glaive_counter_says_what_is_active() {
        assert_eq!(
            glaive_summary(&Extracts {
                attack: true,
                speed: true,
                defense: false,
                heal: false,
            }),
            "rojo + blanco · 2/3"
        );
    }

    #[test]
    fn a_full_glaive_reads_as_complete_and_an_empty_one_as_none() {
        assert_eq!(
            glaive_summary(&Extracts {
                attack: true,
                speed: true,
                defense: true,
                heal: false,
            }),
            "3/3 · completo"
        );
        assert_eq!(glaive_summary(&Extracts::default()), "sin esencias · 0/3");
    }

    #[test]
    fn an_essence_bar_is_only_filled_when_its_maximum_is_known() {
        assert_eq!(essence_fraction(60.0, 120.0), Some(0.5));
        assert_eq!(essence_fraction(10.0, 0.0), None);
        assert_eq!(essence_fraction(150.0, 120.0), Some(1.0));
    }

    #[test]
    fn the_glaive_meter_paints_an_essence_only_with_a_known_maximum() {
        let (config, text) = config();
        let geometry = Geometry::new(1.0);
        let known = glaive(
            Extracts {
                attack: true,
                speed: true,
                defense: true,
                heal: false,
            },
            (60.0, 120.0),
            (60.0, 90.0),
            (30.0, 60.0),
        );
        let unknown = glaive(
            Extracts {
                attack: true,
                ..Extracts::default()
            },
            (30.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
        );
        let red = Color::rgb(0xe8, 0x55, 0x4a).to_argb() | 0xff00_0000;

        let mut with_max = Canvas::new(400, 120);
        draw_insect_glaive(&mut with_max, &config, &text, &geometry, &known, 0.0);
        let mut without_max = Canvas::new(400, 120);
        draw_insect_glaive(&mut without_max, &config, &text, &geometry, &unknown, 0.0);

        assert!(
            with_max.pixels().contains(&red),
            "la esencia roja con máximo conocido no se dibuja"
        );
        assert!(
            !without_max.pixels().contains(&red),
            "sin máximo la barra no debe arrancar rellena"
        );
    }

    #[test]
    fn the_glaive_meter_height_matches_what_is_drawn() {
        let (config, text) = config();
        let geometry = Geometry::new(1.0);
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            wirebugs: None,
            weapon: Some(glaive(
                Extracts {
                    attack: true,
                    speed: true,
                    defense: false,
                    heal: false,
                },
                (30.0, 60.0),
                (45.0, 90.0),
                (0.0, 0.0),
            )),
            buffs: Vec::new(),
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
    fn the_other_gauges_keep_their_chips() {
        // El medidor propio del glaive no debe cambiar las fichas de las demás clases.
        let style = StyleSection::default();
        let weapon = Weapon {
            kind: WeaponKind::ChargeBlade,
            sharpness: None,
            gauge: Some(WeaponGauge::ChargeBlade {
                phials: 3,
                max_phials: 5,
                charge: 62.0,
                shield: 30.0,
                sword: 12.0,
                axe: 0.0,
            }),
            detail: WeaponDetail::default(),
        };
        let chips = weapon_gauge_chips(&weapon, &style);

        assert_eq!(chips[0].label, "Carga");
        assert_eq!(chips[1].label, "Frascos");
        assert_eq!(chips[1].value, "3/5");
    }

    #[test]
    fn a_part_with_only_flinch_still_has_no_row() {
        // El componente de flinch no alcanza para enseñar una parte: solo romper o cortar.
        let mut monster = monster(50.0);
        monster.parts.push(Part {
            index: 2,
            name: Some("Cuello"),
            flinch: Some(Health::new(90.0, 100.0)),
            ..Part::default()
        });

        assert!(visible_parts(&monster).is_empty());
    }
}
