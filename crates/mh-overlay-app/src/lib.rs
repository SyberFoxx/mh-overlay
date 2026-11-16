//! Hub de escritorio del overlay: diagnóstico de la conexión y ajustes con vista previa.
//!
//! El panel sobre el juego se sigue dibujando con el lienzo de `mh-overlay-ui`, que no depende
//! de GL ni de un compositor. Esta ventana cubre lo que no cabe en una tira: saber por qué no
//! se lee la memoria y ajustar colores y disposición viendo el resultado antes de aplicarlo.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use mh_overlay_config::{Color, Config, Position};
use mh_overlay_games::{Ailment, GameKind, Health, Hunt, Monster, Part, Player, Snapshot};
use mh_overlay_memory::{AddressMap, MapValidation, Process, maps_directory};
use mh_overlay_ui::OverlayError;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::view;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("no se pudo abrir la ventana del hub: {0}")]
    Window(#[from] eframe::Error),
    #[error(transparent)]
    Overlay(#[from] OverlayError),
    #[error("no se pudo localizar data/maps: {0}")]
    Maps(#[from] mh_overlay_memory::MemoryError),
}

/// Abre el hub con la configuración ya cargada y la guarda en `config_path`.
pub fn run(config_path: PathBuf, config: Config) -> Result<(), AppError> {
    let text = Text::bundled()?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 660.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("mh-overlay · hub"),
        ..Default::default()
    };

    eframe::run_native(
        "mh-overlay · hub",
        options,
        Box::new(move |_context| {
            Ok(Box::new(Hub::new(
                config_path,
                config,
                text,
                maps_directory().ok(),
            )))
        }),
    )
    .map_err(AppError::Window)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Diagnostics,
    Settings,
}

/// Último resultado de intentar leer el juego. Se reemplaza entero en cada refresco para no
/// mezclar datos de una build con el error de otra.
#[derive(Default)]
struct Status {
    pid: Option<i32>,
    base: Option<u64>,
    map: Option<String>,
    validation: Option<MapValidation>,
    snapshot: Snapshot,
    error: Option<String>,
    read_ms: Option<u128>,
    checked_at: Option<Instant>,
}

struct Hub {
    config_path: PathBuf,
    config: Config,
    text: Text,
    maps_directory: Option<PathBuf>,
    tab: Tab,
    status: Status,
    /// `None` fuerza una lectura en el siguiente fotograma (al arrancar o al cambiar de juego).
    last_refresh: Option<Instant>,
    last_saved: Option<Instant>,
    save_error: Option<String>,
    dirty: bool,
    preview: Option<egui::TextureHandle>,
    preview_size: egui::Vec2,
    preview_stale: bool,
}

impl Hub {
    fn new(
        config_path: PathBuf,
        config: Config,
        text: Text,
        maps_directory: Option<PathBuf>,
    ) -> Self {
        Self {
            config_path,
            config,
            text,
            maps_directory,
            tab: Tab::Diagnostics,
            status: Status::default(),
            last_refresh: None,
            last_saved: None,
            save_error: None,
            dirty: false,
            preview: None,
            preview_size: egui::Vec2::ZERO,
            preview_stale: true,
        }
    }

    fn game(&self) -> GameKind {
        self.config.game_kind().unwrap_or(GameKind::Rise)
    }

    fn probe(&self) -> Status {
        let mut status = Status {
            checked_at: Some(Instant::now()),
            ..Default::default()
        };

        let process = match Process::find(&[self.game().module()]) {
            Ok(process) => process,
            Err(error) => {
                status.error = Some(error.to_string());
                return status;
            }
        };

        status.pid = Some(process.pid());
        status.base = Some(process.base());

        let memory = process.memory();
        let directory = match &self.maps_directory {
            Some(directory) => directory,
            None => {
                status.error = Some("no se encontró data/maps".to_string());
                return status;
            }
        };

        let selected = match AddressMap::select_for_game(
            self.game().map_prefix(),
            directory,
            &memory,
            process.base(),
        ) {
            Ok(selected) => selected,
            Err(error) => {
                status.error = Some(error.to_string());
                return status;
            }
        };

        status.map = selected
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        status.validation = Some(selected.validation);

        let started = Instant::now();
        match mh_overlay_games::snapshot(self.game(), memory, &selected.map) {
            Ok(snapshot) => {
                status.snapshot = snapshot;
                status.read_ms = Some(started.elapsed().as_millis());
            }
            Err(error) => status.error = Some(error.to_string()),
        }

        status
    }

    fn refresh(&mut self) {
        self.status = self.probe();
        self.last_refresh = Some(Instant::now());
    }

    fn save(&mut self) {
        self.dirty = false;

        if let Err(error) = self.config.validate() {
            self.save_error = Some(error.to_string());
            return;
        }

        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.save_error = None;
                self.last_saved = Some(Instant::now());
            }
            Err(error) => self.save_error = Some(error.to_string()),
        }
    }

    fn rebuild_preview(&mut self, ctx: &egui::Context) {
        let canvas = view::render(&self.config, &self.text, &sample(), None);
        let width = canvas.width().max(1) as usize;
        let height = canvas.height().max(1) as usize;

        let mut rgba = Vec::with_capacity(width * height * 4);
        for pixel in canvas.pixels() {
            // El lienzo guarda ARGB en little-endian; egui espera RGBA.
            rgba.extend_from_slice(&[
                (pixel >> 16) as u8,
                (pixel >> 8) as u8,
                *pixel as u8,
                (pixel >> 24) as u8,
            ]);
        }

        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        self.preview_size = egui::vec2(width as f32, height as f32);
        self.preview = Some(ctx.load_texture("panel", image, egui::TextureOptions::NEAREST));
        self.preview_stale = false;
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("mh-overlay");
            ui.separator();
            ui.selectable_value(&mut self.tab, Tab::Diagnostics, "Diagnóstico");
            ui.selectable_value(&mut self.tab, Tab::Settings, "Ajustes");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let state = match (&self.status.error, self.status.pid) {
                    (None, Some(_)) => {
                        egui::RichText::new("● leyendo").color(egui::Color32::LIGHT_GREEN)
                    }
                    (Some(_), _) => {
                        egui::RichText::new("● sin juego").color(egui::Color32::LIGHT_RED)
                    }
                    _ => egui::RichText::new("● sin leer").color(egui::Color32::GRAY),
                };
                ui.label(state);
                if ui.button("Actualizar ahora").clicked() {
                    self.refresh();
                }
            });
        });
    }

    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        let status = &self.status;

        egui::Grid::new("diagnostico")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Juego");
                ui.label(format!(
                    "{} ({})",
                    display_game(self.game()),
                    self.game().module()
                ));
                ui.end_row();

                ui.label("Proceso");
                match (status.pid, status.base) {
                    (Some(pid), Some(base)) => {
                        ui.label(format!("pid {pid} · base módulo 0x{base:08X}"))
                    }
                    _ => ui.label("no encontrado").on_hover_text(
                        "es normal si el juego está cerrado; bajo Proton el ejecutable es un proceso normal",
                    ),
                };
                ui.end_row();

                ui.label("Mapa");
                ui.label(status.map.clone().unwrap_or_else(|| "—".to_string()));
                ui.end_row();

                ui.label("Verificación");
                ui.label(match status.validation {
                    Some(MapValidation::Verified) => {
                        "CRC verificado: el mapa es de esta build".to_string()
                    }
                    Some(MapValidation::Mismatch) => {
                        "el mapa NO coincide con esta build: los números serán basura".to_string()
                    }
                    Some(MapValidation::Unavailable) => {
                        "sin CRC: no se puede verificar la build".to_string()
                    }
                    None => "—".to_string(),
                });
                ui.end_row();

                ui.label("Monstruos");
                ui.label(format!("{} en la zona", status.snapshot.monsters.len()));
                ui.end_row();

                ui.label("Jugador");
                ui.label(match &status.snapshot.player {
                    Some(player) => format!(
                        "vida {:.0} % · aguante {:.0} %",
                        player.health.percent(),
                        player.stamina.percent()
                    ),
                    None => "— (fuera de misión)".to_string(),
                });
                ui.end_row();

                ui.label("Cacería");
                ui.label(match &status.snapshot.hunt {
                    Some(hunt) => format!(
                        "{} restante · vidas {}/{}",
                        hunt.time_label(),
                        hunt.deaths,
                        hunt.max_deaths
                    ),
                    None => "— (fuera de misión)".to_string(),
                });
                ui.end_row();

                ui.label("Última lectura");
                ui.label(match (status.checked_at, status.read_ms) {
                    (Some(at), Some(ms)) => {
                        format!("hace {} · {} ms", seconds_ago(at.elapsed()), ms)
                    }
                    (Some(at), None) => format!("hace {}", seconds_ago(at.elapsed())),
                    _ => "—".to_string(),
                });
                ui.end_row();
            });

        if let Some(error) = &status.error {
            ui.add_space(8.0);
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }

        ui.add_space(12.0);
        ui.separator();
        ui.label(
            egui::RichText::new(
                "Si el proceso aparece pero todas las lecturas fallan con «permiso denegado», \
                 el kernel está bloqueando process_vm_readv: usa \
                 `sudo setcap cap_sys_ptrace+ep target/release/mh-overlay` o baja \
                 `kernel.yama.ptrace_scope` a 0.",
            )
            .small()
            .weak(),
        );
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        let before = self.config.clone();

        ui.heading("Juego");
        ui.horizontal(|ui| {
            for kind in [GameKind::Rise, GameKind::World] {
                let selected = self.game() == kind;
                if ui.selectable_label(selected, display_game(kind)).clicked() && !selected {
                    self.config.game.kind = kind.to_string();
                    self.status = Status::default();
                    self.last_refresh = None;
                }
            }

            ui.separator();
            ui.label("Refresco");
            ui.add(
                egui::DragValue::new(&mut self.config.game.interval_ms)
                    .range(100..=5000)
                    .suffix(" ms"),
            );
        });

        ui.add_space(8.0);
        ui.heading("Panel");
        egui::Grid::new("panel")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Anclaje");
                egui::ComboBox::from_id_salt("posicion")
                    .selected_text(display_position(self.config.overlay.position))
                    .show_ui(ui, |ui| {
                        for position in [
                            Position::TopLeft,
                            Position::TopRight,
                            Position::BottomLeft,
                            Position::BottomRight,
                            Position::Center,
                        ] {
                            ui.selectable_value(
                                &mut self.config.overlay.position,
                                position,
                                display_position(position),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Márgenes");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.config.overlay.margin_x)
                            .range(0..=400)
                            .prefix("x "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.config.overlay.margin_y)
                            .range(0..=400)
                            .prefix("y "),
                    );
                });
                ui.end_row();

                ui.label("Escala");
                ui.add(
                    egui::Slider::new(&mut self.config.overlay.scale, 0.5..=3.0)
                        .fixed_decimals(2)
                        .suffix("×"),
                );
                ui.end_row();

                ui.label("Monstruos a la vez");
                ui.add(egui::Slider::new(
                    &mut self.config.overlay.max_monsters,
                    1..=8,
                ));
                ui.end_row();
            });

        ui.add_space(4.0);
        ui.checkbox(&mut self.config.overlay.show_parts, "Mostrar partes");
        ui.checkbox(&mut self.config.overlay.show_ailments, "Mostrar estados");
        ui.checkbox(
            &mut self.config.overlay.click_through,
            "Los clics atraviesan el panel",
        );
        ui.checkbox(
            &mut self.config.overlay.hide_on_empty,
            "Ocultar el panel sin monstruos",
        );

        ui.add_space(8.0);
        ui.heading("Estilo");
        egui::Grid::new("estilo")
            .num_columns(2)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                color_row(ui, "Fondo del panel", &mut self.config.style.panel);
                color_row(ui, "Texto", &mut self.config.style.text);
                color_row(ui, "Texto atenuado", &mut self.config.style.muted);
                color_row(ui, "Vida", &mut self.config.style.health);
                color_row(ui, "Vida baja", &mut self.config.style.low_health);
                color_row(ui, "Umbral de captura", &mut self.config.style.capture);
                color_row(ui, "Parte rompible", &mut self.config.style.part);
                color_row(ui, "Parte cortable", &mut self.config.style.severable);
            });

        ui.horizontal(|ui| {
            ui.label("Vida baja por debajo de");
            ui.add(
                egui::Slider::new(&mut self.config.style.low_health_percent, 0.0..=100.0)
                    .suffix(" %"),
            );
        });

        if self.config != before {
            self.dirty = true;
            self.preview_stale = true;
        }

        ui.add_space(12.0);
        ui.separator();
        ui.horizontal(|ui| match (&self.save_error, self.last_saved) {
            (Some(error), _) => {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
            (None, Some(at)) => {
                ui.label(
                    egui::RichText::new(format!(
                        "guardado hace {} en {}",
                        seconds_ago(at.elapsed()),
                        self.config_path.display()
                    ))
                    .weak(),
                );
            }
            (None, None) => {
                ui.label(egui::RichText::new("los cambios se guardan solos").weak());
            }
        });
    }

    fn preview(&mut self, ui: &mut egui::Ui) {
        ui.heading("Vista previa");
        ui.label(
            egui::RichText::new("Así se verá el panel con monstruos de ejemplo.")
                .small()
                .weak(),
        );
        ui.add_space(8.0);

        let Some(texture) = &self.preview else {
            return;
        };

        let available = ui.available_width();
        let scale = (available / self.preview_size.x).clamp(0.2, 1.0);
        ui.image(egui::load::SizedTexture::new(
            texture.id(),
            self.preview_size * scale,
        ));
    }
}

impl eframe::App for Hub {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.last_refresh.is_none_or(|last| {
            last.elapsed() >= Duration::from_millis(self.config.game.interval_ms)
        }) {
            self.refresh();
        }

        // Guardar cuando el usuario suelta el control: arrastrar un deslizador no debe escribir
        // el archivo decenas de veces por segundo.
        if self.dirty && !ctx.input(|input| input.pointer.any_down()) {
            self.save();
        }

        if self.preview_stale {
            self.rebuild_preview(&ctx);
        }

        egui::Panel::top("barra").show(ui, |ui| {
            ui.add_space(4.0);
            self.top_bar(ui);
            ui.add_space(4.0);
        });

        egui::Panel::right("preview")
            .resizable(true)
            .default_size(380.0)
            .show(ui, |ui| {
                ui.add_space(8.0);
                self.preview(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Diagnostics => self.diagnostics(ui),
                Tab::Settings => self.settings(ui),
            });
        });

        ctx.request_repaint_after(Duration::from_millis(250));
    }
}

fn color_row(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    ui.label(label);
    let mut value = egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
    if ui.color_edit_button_srgba(&mut value).changed() {
        *color = Color {
            r: value.r(),
            g: value.g(),
            b: value.b(),
            a: value.a(),
        };
    }
    ui.end_row();
}

fn display_game(kind: GameKind) -> &'static str {
    match kind {
        GameKind::Rise => "Monster Hunter Rise",
        GameKind::World => "Monster Hunter World",
    }
}

fn display_position(position: Position) -> &'static str {
    match position {
        Position::TopLeft => "arriba izquierda",
        Position::TopRight => "arriba derecha",
        Position::BottomLeft => "abajo izquierda",
        Position::BottomRight => "abajo derecha",
        Position::Center => "centro",
    }
}

fn seconds_ago(elapsed: Duration) -> String {
    if elapsed.as_secs() < 1 {
        "menos de 1 s".to_string()
    } else {
        format!("{} s", elapsed.as_secs())
    }
}

/// Datos de ejemplo para la vista previa: no dependen de que el juego esté abierto.
fn sample() -> Snapshot {
    let mut rathalos = Monster::new(12, Health::new(1600.0, 2350.0));
    rathalos.name = Some("Rathalos".to_string());
    rathalos.capture_percent = Some(20.0);
    rathalos.parts = vec![
        part(0, "Cabeza", 0.0, true, true),
        part(1, "Cuello", 0.0, false, false),
        part(2, "Cuerpo", 0.0, false, false),
        part(3, "Ala izq.", 0.0, true, true),
        part(4, "Ala der.", 0.0, true, false),
        part(5, "Cola", 0.0, true, true),
    ];
    rathalos.ailments = vec![
        ailment("Veneno", 120.0, 265.0, 0.0, 0.0),
        ailment("Aturdimiento", 40.0, 150.0, 3.0, 5.0),
    ];

    let mut anjanath = Monster::new(22, Health::new(340.0, 2750.0));
    anjanath.name = Some("Anjanath".to_string());
    anjanath.capture_percent = Some(20.0);
    anjanath.parts = vec![
        part(0, "Cabeza", 0.0, true, true),
        part(1, "Cuerpo", 0.0, false, false),
        part(2, "Pata izq.", 0.0, true, false),
    ];

    Snapshot {
        player: Some(Player {
            health: Health::new(712.0, 1000.0),
            stamina: Health::new(126.0, 150.0),
        }),
        hunt: Some(Hunt {
            time_left: 12.0 * 60.0 + 34.0,
            deaths: 1,
            max_deaths: 3,
        }),
        monsters: vec![rathalos, anjanath],
    }
}

fn part(index: usize, name: &'static str, damaged: f32, breakable: bool, severable: bool) -> Part {
    Part {
        index,
        name: Some(name),
        flinch: Some(Health::new(100.0 - damaged, 100.0)),
        breakable: breakable.then(|| Health::new(100.0 - damaged, 100.0)),
        severable: severable.then(|| Health::new(100.0 - damaged, 100.0)),
    }
}

fn ailment(name: &'static str, buildup: f32, max: f32, timer: f32, max_timer: f32) -> Ailment {
    Ailment {
        name,
        buildup,
        max_buildup: max,
        timer,
        max_timer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sample_fills_every_section_of_the_panel() {
        let snapshot = sample();

        assert!(snapshot.player.is_some());
        assert!(snapshot.hunt.is_some());
        assert_eq!(snapshot.monsters.len(), 2);
        assert!(snapshot.monsters[0].capture_percent.is_some());
    }
}
