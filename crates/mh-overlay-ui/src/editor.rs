//! Editor de ajustes: estado, dibujo y zonas de clic, sin X de por medio.
//!
//! La ventana solo reenvía clics y teclas; todo lo que decide qué cambia vive aquí, así que se
//! puede probar sin servidor gráfico. Cada cambio se escribe en el TOML, que el overlay relee
//! mientras corre.

use mh_overlay_config::{Color, Config, Monsters, Position};

use crate::canvas::{Canvas, Rect};
use crate::text::Text;

/// Teclas que el editor entiende.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Toggle,
}

/// Ajustes editables, en el orden en que se muestran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Position,
    MarginX,
    MarginY,
    Scale,
    Monsters,
    MaxMonsters,
    Interval,
    ShowParts,
    ShowAilments,
    ClickThrough,
    HideOnEmpty,
}

impl Field {
    pub const ALL: [Self; 11] = [
        Self::Position,
        Self::MarginX,
        Self::MarginY,
        Self::Scale,
        Self::Monsters,
        Self::MaxMonsters,
        Self::Interval,
        Self::ShowParts,
        Self::ShowAilments,
        Self::ClickThrough,
        Self::HideOnEmpty,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Position => "posición",
            Self::MarginX => "margen horizontal",
            Self::MarginY => "margen vertical",
            Self::Scale => "escala",
            Self::Monsters => "monstruos",
            Self::MaxMonsters => "monstruos a la vez",
            Self::Interval => "refresco",
            Self::ShowParts => "mostrar partes",
            Self::ShowAilments => "mostrar estados",
            Self::ClickThrough => "clics atraviesan",
            Self::HideOnEmpty => "ocultar sin monstruos",
        }
    }

    fn is_toggle(self) -> bool {
        matches!(
            self,
            Self::ShowParts | Self::ShowAilments | Self::ClickThrough | Self::HideOnEmpty
        )
    }
}

/// Ancho fijo de la ventana de ajustes; el alto sale del número de filas.
pub const WIDTH: u32 = 470;
const PADDING: f32 = 18.0;
const HEADER: f32 = 74.0;
const ROW: f32 = 34.0;
const FOOTER: f32 = 46.0;

pub struct Editor {
    config: Config,
    selected: usize,
    hover: Option<usize>,
}

impl Editor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected: 0,
            hover: None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn height() -> u32 {
        (HEADER + Field::ALL.len() as f32 * ROW + FOOTER).round() as u32
    }

    pub fn view(&self, text: &Text, path: &str) -> Canvas {
        let height = Self::height();
        let mut canvas = Canvas::new(WIDTH, height);
        let style = &self.config.style;

        canvas.fill_rounded(
            Rect::new(0, 0, WIDTH, height),
            12.0,
            lighten(style.panel, 0.18),
        );
        canvas.fill_rounded(Rect::new(1, 1, WIDTH - 2, height - 2), 11.0, style.panel);

        text.draw(
            &mut canvas,
            "Ajustes del panel",
            PADDING,
            36.0,
            19.0,
            style.text,
        );

        let subtitle = format!("{path} · los cambios se guardan solos");
        text.draw(&mut canvas, &subtitle, PADDING, 56.0, 11.0, style.muted);

        for (index, field) in Field::ALL.iter().enumerate() {
            let rect = row_rect(index);

            if index == self.selected {
                canvas.fill_rect(rect.inset(2, 0), lighten(style.panel, 0.10));
            } else if self.hover == Some(index) {
                canvas.fill_rect(rect.inset(2, 0), lighten(style.panel, 0.05));
            }

            text.draw(
                &mut canvas,
                field.label(),
                PADDING,
                rect.y as f32 + 22.0,
                14.0,
                if index == self.selected {
                    style.text
                } else {
                    style.muted
                },
            );

            self.draw_value(&mut canvas, text, *field, index);
        }

        let footer = Rect::new(
            PADDING as i32,
            (height as f32 - FOOTER + 4.0).round() as i32,
            WIDTH - 2 * PADDING as u32,
            1,
        );
        canvas.fill_rect(footer, lighten(style.panel, 0.14));

        text.draw(
            &mut canvas,
            "arriba/abajo elegir · izq./der. cambiar · Esc cerrar",
            PADDING,
            height as f32 - 16.0,
            11.0,
            style.muted,
        );

        canvas
    }

    fn draw_value(&self, canvas: &mut Canvas, text: &Text, field: Field, index: usize) {
        let style = &self.config.style;
        let rect = row_rect(index);

        if field.is_toggle() {
            let enabled = self.toggle_value(field);
            let track = Rect::new((WIDTH as f32 - PADDING - 46.0) as i32, rect.y + 6, 46, 22);

            canvas.fill_rounded(
                track,
                11.0,
                if enabled {
                    style.health
                } else {
                    lighten(style.panel, 0.22)
                },
            );

            let knob_x = if enabled {
                track.right() - 20
            } else {
                track.x + 2
            };
            canvas.fill_rounded(Rect::new(knob_x, track.y + 2, 18, 18), 9.0, style.text);

            let state = if enabled { "sí" } else { "no" };
            text.draw(
                canvas,
                state,
                track.x as f32 - 8.0 - text.width(state, 12.0),
                rect.y as f32 + 22.0,
                12.0,
                style.muted,
            );

            return;
        }

        let (minus, plus) = stepper_rects(index);

        for (button, symbol) in [(minus, "-"), (plus, "+")] {
            canvas.fill_rounded(button, 6.0, lighten(style.panel, 0.12));
            text.draw(
                canvas,
                symbol,
                button.x as f32 + 8.0,
                button.y as f32 + 18.0,
                14.0,
                style.text,
            );
        }

        let value = self.value_label(field);
        text.draw(
            canvas,
            &value,
            minus.x as f32 - 8.0 - text.width(&value, 13.0),
            rect.y as f32 + 22.0,
            13.0,
            style.text,
        );
    }

    fn value_label(&self, field: Field) -> String {
        match field {
            Field::Position => match self.config.overlay.position {
                Position::TopLeft => "arriba izq.".to_string(),
                Position::TopRight => "arriba der.".to_string(),
                Position::BottomLeft => "abajo izq.".to_string(),
                Position::BottomRight => "abajo der.".to_string(),
                Position::Center => "centro".to_string(),
            },
            Field::MarginX => format!("{} px", self.config.overlay.margin_x),
            Field::MarginY => format!("{} px", self.config.overlay.margin_y),
            Field::Scale => format!("{:.2}×", self.config.overlay.scale),
            Field::Monsters => self.config.overlay.monsters.label().to_string(),
            Field::MaxMonsters => format!("{}", self.config.overlay.max_monsters),
            Field::Interval => format!("{} ms", self.config.game.interval_ms),
            Field::ShowParts | Field::ShowAilments | Field::ClickThrough | Field::HideOnEmpty => {
                String::new()
            }
        }
    }

    fn toggle_value(&self, field: Field) -> bool {
        match field {
            Field::ShowParts => self.config.overlay.show_parts,
            Field::ShowAilments => self.config.overlay.show_ailments,
            Field::ClickThrough => self.config.overlay.click_through,
            Field::HideOnEmpty => self.config.overlay.hide_on_empty,
            _ => false,
        }
    }

    /// Actualiza la fila bajo el cursor; devuelve si algo cambió.
    pub fn motion(&mut self, x: i32, y: i32) {
        self.hover = row_at(x, y);
    }

    /// Un clic: selecciona la fila y aplica el control que haya debajo.
    pub fn click(&mut self, x: i32, y: i32) -> bool {
        let Some(index) = row_at(x, y) else {
            return false;
        };

        self.selected = index;
        let field = Field::ALL[index];

        if field.is_toggle() {
            return self.adjust(field, 0);
        }

        let (minus, plus) = stepper_rects(index);

        if minus.contains(x, y) {
            self.adjust(field, -1)
        } else if plus.contains(x, y) {
            self.adjust(field, 1)
        } else {
            false
        }
    }

    /// Teclado: mueve la selección o cambia el valor seleccionado.
    pub fn key(&mut self, key: Key) -> bool {
        match key {
            Key::Up => {
                self.selected = self.selected.saturating_sub(1);
                false
            }
            Key::Down => {
                self.selected = (self.selected + 1).min(Field::ALL.len() - 1);
                false
            }
            Key::Left | Key::Right | Key::Toggle => {
                let field = Field::ALL[self.selected];

                if field.is_toggle() {
                    self.adjust(field, 0)
                } else {
                    self.adjust(field, if key == Key::Left { -1 } else { 1 })
                }
            }
        }
    }

    fn adjust(&mut self, field: Field, delta: i32) -> bool {
        // El refresco vive en `[game]`, así que se atiende antes de tomar prestada `[overlay]`.
        if field == Field::Interval {
            let value = self.config.game.interval_ms as i64 + i64::from(delta) * 50;
            self.config.game.interval_ms = value.clamp(100, 2000) as u64;
            return true;
        }

        let overlay = &mut self.config.overlay;

        match field {
            Field::Position => {
                let order = [
                    Position::TopLeft,
                    Position::TopRight,
                    Position::BottomLeft,
                    Position::BottomRight,
                    Position::Center,
                ];
                let current = order
                    .iter()
                    .position(|position| *position == overlay.position)
                    .unwrap_or(0);
                let next = (current as i32 + if delta < 0 { order.len() as i32 - 1 } else { 1 })
                    as usize
                    % order.len();
                overlay.position = order[next];
            }
            Field::MarginX => {
                overlay.margin_x = (overlay.margin_x + delta * 4).clamp(0, 200);
            }
            Field::MarginY => {
                overlay.margin_y = (overlay.margin_y + delta * 4).clamp(0, 200);
            }
            Field::Scale => {
                let steps = (overlay.scale * 20.0).round() as i32 + delta;
                overlay.scale = steps.clamp(10, 60) as f32 / 20.0;
            }
            Field::Monsters => {
                let order = Monsters::ALL;
                let current = order
                    .iter()
                    .position(|mode| *mode == overlay.monsters)
                    .unwrap_or(0);
                let next = (current as i32 + if delta < 0 { order.len() as i32 - 1 } else { 1 })
                    as usize
                    % order.len();
                overlay.monsters = order[next];
            }
            Field::MaxMonsters => {
                let value = overlay.max_monsters as i32 + delta;
                overlay.max_monsters = value.clamp(1, 8) as usize;
            }
            Field::Interval => unreachable!("el refresco se atiende antes"),
            Field::ShowParts => overlay.show_parts = !overlay.show_parts,
            Field::ShowAilments => overlay.show_ailments = !overlay.show_ailments,
            Field::ClickThrough => overlay.click_through = !overlay.click_through,
            Field::HideOnEmpty => overlay.hide_on_empty = !overlay.hide_on_empty,
        }

        true
    }
}

fn row_rect(index: usize) -> Rect {
    Rect::new(
        0,
        (HEADER + index as f32 * ROW).round() as i32,
        WIDTH,
        ROW as u32,
    )
}

fn row_at(x: i32, y: i32) -> Option<usize> {
    if x < 0 || x >= WIDTH as i32 {
        return None;
    }

    let offset = y as f32 - HEADER;
    if offset < 0.0 || offset >= Field::ALL.len() as f32 * ROW {
        return None;
    }

    Some((offset / ROW) as usize)
}

fn stepper_rects(index: usize) -> (Rect, Rect) {
    let rect = row_rect(index);
    let plus = Rect::new((WIDTH as f32 - PADDING - 24.0) as i32, rect.y + 6, 24, 22);
    let minus = Rect::new(plus.x - 28, rect.y + 6, 24, 22);

    (minus, plus)
}

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
        a: mix(color.a).max(200),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> Editor {
        Editor::new(Config::default())
    }

    fn click_plus(editor: &mut Editor, index: usize) -> bool {
        let (_, plus) = stepper_rects(index);
        editor.click(plus.x + 4, plus.y + 4)
    }

    #[test]
    fn the_window_tall_fits_every_row() {
        let height = Editor::height() as f32;

        assert!(height > HEADER + Field::ALL.len() as f32 * ROW);
        assert!(height >= HEADER + Field::ALL.len() as f32 * ROW + FOOTER);
    }

    #[test]
    fn the_monsters_row_cycles_the_selection_mode() {
        let mut editor = editor();
        let index = Field::ALL
            .iter()
            .position(|field| *field == Field::Monsters)
            .unwrap();

        assert!(click_plus(&mut editor, index));
        assert_eq!(editor.config().overlay.monsters, Monsters::Target);

        assert!(click_plus(&mut editor, index));
        assert_eq!(editor.config().overlay.monsters, Monsters::Zone);
    }

    #[test]
    fn the_plus_button_raises_a_value_and_stops_at_the_limit() {
        let mut editor = editor();
        let index = Field::ALL
            .iter()
            .position(|field| *field == Field::MaxMonsters)
            .unwrap();

        assert!(click_plus(&mut editor, index));
        assert_eq!(editor.config().overlay.max_monsters, 5);

        for _ in 0..10 {
            click_plus(&mut editor, index);
        }

        assert_eq!(editor.config().overlay.max_monsters, 8);
    }

    #[test]
    fn the_minus_button_lowers_the_scale_below_one() {
        let mut editor = editor();
        let index = Field::ALL
            .iter()
            .position(|field| *field == Field::Scale)
            .unwrap();
        let (minus, _) = stepper_rects(index);

        editor.click(minus.x + 4, minus.y + 4);

        assert!((editor.config().overlay.scale - 0.95).abs() < 0.001);
    }

    #[test]
    fn clicking_a_toggle_flips_it() {
        let mut editor = editor();
        let index = Field::ALL
            .iter()
            .position(|field| *field == Field::ShowParts)
            .unwrap();
        let rect = row_rect(index);

        assert!(editor.click(rect.x + 300, rect.y + 10));
        assert!(!editor.config().overlay.show_parts);
    }

    #[test]
    fn the_position_cycles_through_every_anchor() {
        let mut editor = editor();
        let index = Field::ALL
            .iter()
            .position(|field| *field == Field::Position)
            .unwrap();

        for expected in [
            Position::TopRight,
            Position::BottomLeft,
            Position::BottomRight,
            Position::Center,
            Position::TopLeft,
        ] {
            click_plus(&mut editor, index);
            assert_eq!(editor.config().overlay.position, expected);
        }
    }

    #[test]
    fn clicking_outside_the_rows_changes_nothing() {
        let mut editor = editor();
        let before = editor.config().clone();

        assert!(!editor.click(10, 5));
        assert_eq!(editor.config(), &before);
    }

    #[test]
    fn the_keyboard_moves_the_selection_and_changes_values() {
        let mut editor = editor();

        assert!(!editor.key(Key::Down));
        assert!(editor.key(Key::Right));
        assert_eq!(editor.config().overlay.margin_x, 28);
    }

    #[test]
    fn a_rendered_editor_paints_the_rows() {
        let text = Text::bundled().unwrap();
        let editor = editor();
        let canvas = editor.view(&text, "~/.config/mh-overlay/config.toml");

        assert_eq!(canvas.width(), WIDTH);
        assert_eq!(canvas.height(), Editor::height());
        assert!(
            canvas.pixels().iter().any(|pixel| *pixel != 0xff00_0000),
            "el editor no dibujó nada"
        );
    }
}
