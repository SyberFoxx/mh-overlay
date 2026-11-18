//! Composición del panel: del estado de los monstruos a una imagen.
//!
//! Una sola tarjeta oscura con secciones por monstruo: nombre y porcentaje, una barra de vida
//! dominante con la marca del umbral de captura y, debajo, barras finas de partes y estados.
//! El texto dice el número; las barras solo lo hacen comparable de un vistazo.

use mh_overlay_config::{Color, Config, StyleSection};
use mh_overlay_games::{
    Hunt, Monster, Part, Player, SharpnessLevel, Snapshot, Weapon, WeaponGauge,
};

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
/// Ancho reservado a la derecha de cada barra para su valor ("62 %", "2/3", "18 s").
const VALUE_WIDTH: f32 = 46.0;
/// Alto de la barra de filo, en unidades lógicas.
const SHARPNESS_BAR: f32 = 9.0;
/// Lado de los rombos que marcan el nivel de espíritu.
const PIP_RADIUS: f32 = 3.4;
/// Ancho máximo de la etiqueta en las filas de detalle (buffs y medidores): sus nombres son
/// más largos que los de una parte.
const DETAIL_LABEL_WIDTH: f32 = 118.0;

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
        let session = session_rows(snapshot) as f32 * self.row;

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

/// Cuántas filas consume la cabecera de jugador y cacería.
fn session_rows(snapshot: &Snapshot) -> usize {
    usize::from(snapshot.hunt.is_some()) + snapshot.player.as_ref().map_or(0, player_rows)
}

/// Buffs que se dibujan como máximo: con más, el panel tapa al monstruo.
const MAX_BUFF_ROWS: usize = 6;

/// Vida y aguante, el arma (tipo, filo y medidores) y los buffs activos.
fn player_rows(player: &Player) -> usize {
    let weapon = player.weapon.as_ref().map_or(0, |weapon| {
        1 + usize::from(weapon.sharpness.is_some()) + spirit_rows(weapon) + gauge_rows(weapon).len()
    });

    2 + weapon + player.buffs.len().min(MAX_BUFF_ROWS)
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

/// Vida y aguante del jugador local, con los mismos colores que los monstruos.
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

    // El arma va arriba, como en HunterPie: primero el tipo y después el filo.
    if let Some(weapon) = &player.weapon {
        y = draw_weapon(canvas, config, text, geometry, weapon, y);

        if let Some(WeaponGauge::Spirit { regen, .. }) = &weapon.gauge {
            y = draw_spirit(canvas, config, text, geometry, weapon, y);

            if *regen > 0.0 {
                y = draw_detail(
                    canvas,
                    config,
                    text,
                    geometry,
                    y,
                    Detail {
                        label: "Regen. espíritu",
                        fraction: None,
                        text: format!("{regen:.0}s"),
                        color: timer_color(),
                    },
                );
            }
        }

        for row in gauge_rows(weapon) {
            y = draw_detail(canvas, config, text, geometry, y, row);
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

    // Buffs y estados: nombre, barra de lo que queda y tiempo en segundos.
    for buff in player.buffs.iter().take(MAX_BUFF_ROWS) {
        y = draw_detail(
            canvas,
            config,
            text,
            geometry,
            y,
            Detail {
                label: buff.name,
                fraction: Some(buff.fraction()),
                text: if buff.infinite {
                    "∞".to_string()
                } else {
                    format!("{:.0}s", buff.timer)
                },
                color: if buff.debuff {
                    style.low_health
                } else {
                    style.capture
                },
            },
        );
    }

    y
}

/// Fila de detalle: etiqueta, barra opcional y texto libre a la derecha.
struct Detail<'a> {
    label: &'a str,
    fraction: Option<f32>,
    text: String,
    color: Color,
}

/// Dibuja una fila de detalle y devuelve la `y` de la siguiente.
fn draw_detail(
    canvas: &mut Canvas,
    config: &Config,
    text: &Text,
    geometry: &Geometry,
    y: f32,
    detail: Detail<'_>,
) -> f32 {
    let style = &config.style;
    let size = 12.0 * geometry.scale;
    let label = fit_label(
        text,
        detail.label,
        size,
        DETAIL_LABEL_WIDTH * geometry.scale,
    );

    text.draw(
        canvas,
        &label,
        geometry.padding,
        y + text.ascent(size),
        size,
        style.muted,
    );

    let value_width = text.width(&detail.text, size);
    // Columna derecha fija: todas las barras de detalle terminan a la misma altura y los
    // nombres de buffs más largos no las encogen frame a frame.
    let label_width = DETAIL_LABEL_WIDTH * geometry.scale + 6.0 * geometry.scale;
    let bar_right = canvas.width() as f32 - geometry.padding - VALUE_WIDTH * geometry.scale;

    if let Some(fraction) = detail.fraction {
        let fraction = fraction.clamp(0.0, 1.0);
        let bar_height = geometry.track.max(3.0);
        let bar = Rect::new(
            geometry.padding as i32 + label_width as i32,
            (y + geometry.row * 0.5 - bar_height * 0.5).round() as i32,
            (bar_right - geometry.padding - label_width).max(1.0) as u32,
            bar_height as u32,
        );
        let radius = bar.height as f32 / 2.0;

        canvas.fill_rounded(bar, radius, lighten(style.panel, 0.18));

        let filled = (bar.width as f32 * fraction).round() as u32;
        if filled > 1 {
            canvas.fill_rounded_gradient(
                Rect::new(bar.x, bar.y, filled, bar.height),
                radius,
                lighten(detail.color, 0.3),
                detail.color,
            );
        }
    }

    text.draw(
        canvas,
        &detail.text,
        canvas.width() as f32 - geometry.padding - value_width,
        y + text.ascent(size),
        size,
        style.text,
    );

    y + geometry.row
}

/// Un color para medidores de carga, otro para temporizadores y otro para estados.
fn meter_color() -> Color {
    Color::rgb(0x4f, 0x8d, 0xd6)
}

fn timer_color() -> Color {
    Color::rgb(0xe6, 0xb4, 0x5e)
}

fn state_color() -> Color {
    Color::rgb(0xd6, 0x9a, 0x4f)
}

/// Convierte los medidores del arma en filas del panel. Las armas sin medidor extra no
/// aportan ninguna fila.
fn gauge_rows(weapon: &Weapon) -> Vec<Detail<'static>> {
    let Some(gauge) = &weapon.gauge else {
        return Vec::new();
    };

    match gauge {
        // El espíritu tiene su propio dibujo (rombos de nivel y color por nivel).
        WeaponGauge::Spirit { .. } => Vec::new(),
        WeaponGauge::ChargeBlade {
            phials,
            max_phials,
            charge,
            shield,
            sword,
            axe,
        } => {
            let mut rows = vec![
                Detail {
                    label: "Carga",
                    fraction: Some(charge / 100.0),
                    text: format!("{charge:.0}%"),
                    color: meter_color(),
                },
                Detail {
                    label: "Frascos",
                    fraction: Some(f32::from(*phials) / f32::from(*max_phials).max(1.0)),
                    text: format!("{phials}/{max_phials}"),
                    color: state_color(),
                },
            ];

            for (label, timer) in [("Escudo", shield), ("Espada", sword), ("Hacha", axe)] {
                if *timer > 0.0 {
                    rows.push(Detail {
                        label,
                        fraction: None,
                        text: format!("{timer:.0}s"),
                        color: timer_color(),
                    });
                }
            }

            rows
        }
        WeaponGauge::SwitchAxe {
            buildup,
            charge,
            timer,
        } => {
            let mut rows = vec![
                Detail {
                    label: "Filo",
                    fraction: Some(buildup / 100.0),
                    text: format!("{buildup:.0}%"),
                    color: meter_color(),
                },
                Detail {
                    label: "Carga amped",
                    fraction: Some(charge / 100.0),
                    text: format!("{charge:.0}%"),
                    color: state_color(),
                },
            ];

            if *timer > 0.0 {
                rows.push(Detail {
                    label: "Bono amped",
                    fraction: None,
                    text: format!("{timer:.0}s"),
                    color: timer_color(),
                });
            }

            rows
        }
        WeaponGauge::DualBlades {
            demon,
            arch_demon,
            buildup,
        } => vec![
            Detail {
                label: "Modo",
                fraction: None,
                text: if *arch_demon {
                    "archidemonio"
                } else if *demon {
                    "demonio"
                } else {
                    "—"
                }
                .to_string(),
                color: state_color(),
            },
            Detail {
                label: "Carga demonio",
                fraction: Some(buildup / 100.0),
                text: format!("{buildup:.0}%"),
                color: meter_color(),
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
            let mut rows = vec![Detail {
                label: "Esencias",
                fraction: None,
                text: extracts.label(),
                color: state_color(),
            }];

            for (label, timer) in [
                ("Ataque", attack),
                ("Velocidad", speed),
                ("Defensa", defense),
            ] {
                if *timer > 0.0 {
                    rows.push(Detail {
                        label,
                        fraction: None,
                        text: format!("{timer:.0}s"),
                        color: timer_color(),
                    });
                }
            }

            if *max_stamina > 0.0 {
                rows.push(Detail {
                    label: "Kinsecto",
                    fraction: Some(stamina / max_stamina),
                    text: format!("{stamina:.0}/{max_stamina:.0}"),
                    color: meter_color(),
                });
            }

            rows
        }
    }
}

/// Icono de espada vertical: hoja con punta, guarda y empuñadura. Da contexto al filo sin
/// gastar una palabra y se reconoce a 11 px de alto.
fn draw_blade_icon(canvas: &mut Canvas, x: f32, center_y: f32, scale: f32, color: Color) {
    let center_x = x + 5.0 * scale;
    let thickness = 1.5 * scale;

    canvas.fill_line(
        (center_x, center_y - 5.5 * scale),
        (center_x, center_y + 1.0 * scale),
        thickness * 1.5,
        color,
    );
    canvas.fill_diamond((center_x, center_y - 6.0 * scale), 1.4 * scale, color);
    canvas.fill_line(
        (center_x - 3.5 * scale, center_y + 1.5 * scale),
        (center_x + 3.5 * scale, center_y + 1.5 * scale),
        thickness,
        color,
    );
    canvas.fill_line(
        (center_x, center_y + 1.5 * scale),
        (center_x, center_y + 6.0 * scale),
        thickness * 1.2,
        color,
    );
    canvas.fill_diamond((center_x, center_y + 6.5 * scale), 1.3 * scale, color);
}

/// Tipo de arma y, si gasta filo, su barra de colores con la posición actual marcada.
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
    let size = 12.0 * scale;
    let right = canvas.width() as f32 - geometry.padding;
    let icon = 14.0 * scale;

    draw_blade_icon(
        canvas,
        geometry.padding,
        y + geometry.row * 0.5,
        scale,
        style.muted,
    );

    text.draw(
        canvas,
        weapon.kind.label(),
        geometry.padding + icon,
        y + text.ascent(size),
        size,
        style.text,
    );

    let Some(sharpness) = &weapon.sharpness else {
        return y + geometry.row;
    };

    // El nivel va con el color del filo: el nombre del color se lee solo una vez.
    let level = format!(" · {}", sharpness.level.label());
    let level_width = text.width(&level, size);
    let value = format!("filo {:.0}", sharpness.current);
    let value_width = text.width(&value, size);
    let value_y = y + text.ascent(size);

    text.draw(
        canvas,
        &level,
        right - level_width,
        value_y,
        size,
        sharpness_color(sharpness.level),
    );
    text.draw(
        canvas,
        &value,
        right - level_width - value_width,
        value_y,
        size,
        style.muted,
    );

    let y = y + geometry.row;
    let height = SHARPNESS_BAR * scale;
    let bar = Rect::new(
        geometry.padding as i32,
        (y + geometry.row * 0.5 - height * 0.5).round() as i32,
        geometry.inner_width(canvas).max(1.0) as u32,
        height.max(3.0) as u32,
    );
    let radius = 2.0 * scale;

    canvas.fill_rounded(bar, radius, lighten(style.panel, 0.22));

    let total = sharpness.total();
    if total > 0.0 {
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

    y + geometry.row
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

/// Filas que ocupa el medidor de espíritu (el propio medidor y, si toca, su regeneración).
fn spirit_rows(weapon: &Weapon) -> usize {
    match &weapon.gauge {
        Some(WeaponGauge::Spirit { regen, .. }) => 1 + usize::from(*regen > 0.0),
        _ => 0,
    }
}

/// Espíritu del mandoble: barra con el color del nivel y tres rombos que marcan cuál está
/// cargado, para no tener que leer "nivel 2" en mitad del combate.
fn draw_spirit(
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
        ..
    }) = &weapon.gauge
    else {
        return y;
    };
    let style = &config.style;
    let scale = geometry.scale;
    let size = 12.0 * scale;
    let inner = geometry.inner_width(canvas);
    let color = spirit_color(*level);

    text.draw(
        canvas,
        "Espíritu",
        geometry.padding,
        y + text.ascent(size),
        size,
        style.muted,
    );

    let value = if *max_timer > 0.0 {
        format!("{timer:.0}s")
    } else {
        "—".to_string()
    };
    let value_width = text.width(&value, size);
    text.draw(
        canvas,
        &value,
        canvas.width() as f32 - geometry.padding - value_width,
        y + text.ascent(size),
        size,
        style.text,
    );

    // Tres rombos: el nivel cargado se rellena con el color del propio medidor.
    let pip_gap = 4.0 * scale;
    let pip_space = 3.0 * (PIP_RADIUS * 2.0 + pip_gap) * scale;
    let pips_x = canvas.width() as f32 - geometry.padding - value_width - 6.0 * scale - pip_space;
    let center_y = y + geometry.row * 0.5;

    for pip in 0..3u8 {
        let center = (
            pips_x
                + (PIP_RADIUS + pip_gap * 0.5) * scale
                + f32::from(pip) * (PIP_RADIUS * 2.0 + pip_gap) * scale,
            center_y,
        );
        if pip < *level {
            canvas.fill_diamond(center, PIP_RADIUS * scale, color);
        } else {
            canvas.fill_diamond(center, PIP_RADIUS * scale, lighten(style.panel, 0.22));
        }
    }

    let bar_height = geometry.track.max(3.0);
    let bar = Rect::new(
        geometry.padding as i32 + (LABEL_WIDTH * scale) as i32,
        (center_y - bar_height * 0.5).round() as i32,
        (inner - LABEL_WIDTH * scale - VALUE_WIDTH * scale - pip_space - 12.0 * scale).max(1.0)
            as u32,
        bar_height as u32,
    );
    let radius = bar.height as f32 / 2.0;

    canvas.fill_rounded(bar, radius, lighten(style.panel, 0.18));

    let filled = (bar.width as f32 * (buildup / 100.0).clamp(0.0, 1.0)).round() as u32;
    if filled > 1 {
        canvas.fill_rounded_gradient(
            Rect::new(bar.x, bar.y, filled, bar.height),
            radius,
            lighten(color, 0.3),
            color,
        );
    }

    y + geometry.row
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
