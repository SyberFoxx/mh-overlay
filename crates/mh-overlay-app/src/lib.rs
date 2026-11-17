//! Hub de escritorio del overlay: diagnóstico de la conexión y ajustes con vista previa.
//!
//! El panel sobre el juego se sigue dibujando con el lienzo de `mh-overlay-ui`, que no depende
//! de GL ni de un compositor. Esta ventana cubre lo que no cabe en una tira: saber por qué no
//! se lee la memoria, elegir el mapa cuando el CRC no coincide y ajustar colores y disposición
//! viendo el resultado antes de aplicarlo.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui;
use mh_overlay_config::{Color, Config, OverlaySection, Position, StyleSection};
use mh_overlay_games::{
    Ailment, GameKind, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel, Snapshot,
    Weapon, WeaponKind,
};
use mh_overlay_memory::{
    AddressMap, MapValidation, Memory, MemoryError, Process, SelectedMap, maps_directory,
};
use mh_overlay_ui::OverlayError;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::view;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(
        "no se pudo abrir la ventana del hub: {0}\n\n\
         pista: el hub necesita OpenGL (Mesa o el driver de tu tarjeta) y una sesión X11 o \
         Wayland. El panel sobre el juego no lo necesita: `mh-overlay --overlay` sigue \
         funcionando sin GL."
    )]
    Window(#[from] eframe::Error),
    #[error(
        "no hay pantalla: define DISPLAY o WAYLAND_DISPLAY. El hub es una ventana normal; \
         para leer sin ventana usa `mh-overlay --watch`."
    )]
    NoDisplay,
    #[error(transparent)]
    Overlay(#[from] OverlayError),
    #[error("no se pudo localizar data/maps: {0}")]
    Maps(#[from] MemoryError),
}

/// Abre el hub con la configuración ya cargada y la guarda en `config_path`.
///
/// `initial_map` fuerza un `.map` concreto (el `--map` de la CLI); `None` elige el más nuevo que
/// verifique contra la build en curso.
pub fn run(
    config_path: PathBuf,
    config: Config,
    initial_map: Option<PathBuf>,
) -> Result<(), AppError> {
    let has_display =
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();

    if !has_display {
        return Err(AppError::NoDisplay);
    }

    let text = Text::bundled()?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([680.0, 500.0])
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
                initial_map,
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

/// Proceso y mapa ya resueltos, para no volver a parsear y validar cada `.map` en cada refresco:
/// con 100 ms de intervalo eso sería releer todos los archivos cinco veces por segundo.
struct Attachment {
    pid: i32,
    base: u64,
    path: PathBuf,
    from_auto: bool,
    map: AddressMap,
    validation: MapValidation,
}

struct Hub {
    config_path: PathBuf,
    config: Config,
    text: Text,
    maps_directory: Option<PathBuf>,
    tab: Tab,
    status: Status,
    attachment: Option<Attachment>,
    /// `.map` elegido a mano; `None` es “el más nuevo que verifique”.
    map_override: Option<PathBuf>,
    maps: Vec<PathBuf>,
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
        map_override: Option<PathBuf>,
    ) -> Self {
        let mut hub = Self {
            config_path,
            config,
            text,
            maps_directory,
            tab: Tab::Diagnostics,
            status: Status::default(),
            attachment: None,
            map_override,
            maps: Vec::new(),
            last_refresh: None,
            last_saved: None,
            save_error: None,
            dirty: false,
            preview: None,
            preview_size: egui::Vec2::ZERO,
            preview_stale: true,
        };

        hub.refresh_maps();
        hub
    }

    fn game(&self) -> GameKind {
        self.config.game_kind().unwrap_or(GameKind::Rise)
    }

    fn set_game(&mut self, kind: GameKind) {
        if self.game() == kind {
            return;
        }

        self.config.game.kind = kind.to_string();
        self.map_override = None;
        self.attachment = None;
        self.status = Status::default();
        self.refresh_maps();
        self.dirty = true;
        self.last_refresh = None;
    }

    fn refresh_maps(&mut self) {
        self.maps = self
            .maps_directory
            .as_deref()
            .and_then(|directory| {
                AddressMap::candidates_for_game(self.game().map_prefix(), directory).ok()
            })
            .unwrap_or_default();

        // Un mapa de World no sirve para Rise: al cambiar de juego se descarta el elegido.
        if let Some(path) = self.map_override.clone()
            && !self.maps.contains(&path)
        {
            self.map_override = None;
            self.attachment = None;
        }
    }

    fn select_map(&self, memory: &Memory, base: u64) -> Result<SelectedMap, MemoryError> {
        if let Some(path) = &self.map_override {
            let mut map = AddressMap::load(path)?;
            map.set_base(base);
            let validation = map.validate(memory);

            return Ok(SelectedMap {
                path: path.clone(),
                map,
                validation,
            });
        }

        let directory =
            self.maps_directory
                .as_deref()
                .ok_or_else(|| MemoryError::NoMapsForGame {
                    game: self.game().map_prefix().to_string(),
                    directory: "data/maps".to_string(),
                })?;

        AddressMap::select_for_game(self.game().map_prefix(), directory, memory, base)
    }

    fn probe(&mut self) -> Status {
        let mut status = Status {
            checked_at: Some(Instant::now()),
            ..Default::default()
        };
        let kind = self.game();

        let process = match Process::find(&[kind.module()]) {
            Ok(process) => process,
            Err(error) => {
                self.attachment = None;
                status.error = Some(error.to_string());
                return status;
            }
        };

        status.pid = Some(process.pid());
        status.base = Some(process.base());

        let memory = process.memory();
        let base = process.base();

        let reusable = self.attachment.as_ref().is_some_and(|attachment| {
            attachment.pid == process.pid()
                && attachment.base == base
                && match &self.map_override {
                    Some(path) => !attachment.from_auto && &attachment.path == path,
                    None => attachment.from_auto,
                }
        });

        if !reusable {
            match self.select_map(&memory, base) {
                Ok(selected) => {
                    self.attachment = Some(Attachment {
                        pid: process.pid(),
                        base,
                        path: selected.path,
                        from_auto: self.map_override.is_none(),
                        map: selected.map,
                        validation: selected.validation,
                    });
                }
                Err(error) => {
                    self.attachment = None;
                    status.error = Some(error.to_string());
                    return status;
                }
            }
        }

        let Some(attachment) = &self.attachment else {
            status.error = Some("no hay mapa cargado".to_string());
            return status;
        };

        status.map = attachment
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        status.validation = Some(attachment.validation);

        let started = Instant::now();
        match mh_overlay_games::snapshot(kind, memory, &attachment.map) {
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

            let mut choice = self.game();
            egui::ComboBox::from_id_salt("juego-barra")
                .width(190.0)
                .selected_text(display_game(choice))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut choice, GameKind::Rise, display_game(GameKind::Rise));
                    ui.selectable_value(
                        &mut choice,
                        GameKind::World,
                        display_game(GameKind::World),
                    );
                });
            if choice != self.game() {
                self.set_game(choice);
            }

            ui.separator();
            ui.selectable_value(&mut self.tab, Tab::Diagnostics, "Diagnóstico");
            ui.selectable_value(&mut self.tab, Tab::Settings, "Ajustes");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (state, color) = match (&self.status.error, self.status.pid) {
                    (None, Some(_)) => ("● leyendo", egui::Color32::LIGHT_GREEN),
                    (Some(_), Some(_)) => (
                        "● error de lectura",
                        egui::Color32::from_rgb(0xe6, 0xb4, 0x5e),
                    ),
                    (Some(_), None) => ("● sin juego", egui::Color32::GRAY),
                    _ => ("● sin leer", egui::Color32::GRAY),
                };
                ui.label(egui::RichText::new(state).color(color));
                if ui.button("Actualizar ahora").clicked() {
                    self.refresh();
                }
            });
        });
    }

    fn map_row(&mut self, ui: &mut egui::Ui) {
        ui.label("Mapa");

        let mut choice = self.map_override.clone();
        let maps = self.maps.clone();
        let auto_label = "Automático (el más nuevo que verifique)";

        egui::ComboBox::from_id_salt("mapa")
            .width(320.0)
            .selected_text(match &choice {
                Some(path) => file_name(path),
                None => auto_label.to_string(),
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut choice, None, auto_label);

                for path in &maps {
                    ui.selectable_value(&mut choice, Some(path.clone()), file_name(path));
                }
            });

        if choice != self.map_override {
            self.map_override = choice;
            self.attachment = None;
            self.last_refresh = None;
        }
    }

    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        if self.status.checked_at.is_none() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("leyendo…");
            });
            ui.add_space(8.0);
        }

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
                match (self.status.pid, self.status.base) {
                    (Some(pid), Some(base)) => {
                        ui.label(format!("pid {pid} · base módulo 0x{base:08X}"));
                    }
                    _ => {
                        ui.label("no encontrado").on_hover_text(
                            "es normal si el juego está cerrado; bajo Proton el juego es un \
                             proceso normal de Linux",
                        );
                    }
                }
                ui.end_row();

                self.map_row(ui);
                ui.end_row();

                ui.label("Verificación");
                ui.label(validation_text(self.status.validation));
                ui.end_row();

                ui.label("Monstruos");
                ui.label(format!(
                    "{} en la zona",
                    self.status.snapshot.monsters.len()
                ));
                ui.end_row();

                ui.label("Jugador");
                ui.label(match &self.status.snapshot.player {
                    Some(player) => format!(
                        "vida {:.0} % · aguante {:.0} %",
                        player.health.percent(),
                        player.stamina.percent()
                    ),
                    None => "— (fuera de misión)".to_string(),
                });
                ui.end_row();

                ui.label("Cacería");
                ui.label(match &self.status.snapshot.hunt {
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
                ui.label(match (self.status.checked_at, self.status.read_ms) {
                    (Some(at), Some(ms)) => {
                        format!("hace {} · {} ms", seconds_ago(at.elapsed()), ms)
                    }
                    (Some(at), None) => format!("hace {}", seconds_ago(at.elapsed())),
                    _ => "—".to_string(),
                });
                ui.end_row();
            });

        if let Some(error) = &self.status.error {
            ui.add_space(8.0);
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }

        if self.status.validation == Some(MapValidation::Mismatch) {
            ui.add_space(8.0);
            ui.colored_label(
                egui::Color32::from_rgb(0xe6, 0xb4, 0x5e),
                "El mapa elegido no coincide con la build en curso: los números que salgan \
                 serán basura plausible. Elige a mano el mapa de tu versión en «Mapa» o pon el \
                 archivo correcto en data/maps.",
            );
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
                if ui
                    .selectable_label(self.game() == kind, display_game(kind))
                    .clicked()
                {
                    self.set_game(kind);
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
                    .width(220.0)
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

        ui.add_space(8.0);
        if ui.button("Restaurar panel y colores por defecto").clicked() {
            self.config.overlay = OverlaySection::default();
            self.config.style = StyleSection::default();
            self.dirty = true;
            self.preview_stale = true;
        }

        if self.config != before {
            self.dirty = true;
            self.preview_stale = true;
        }

        ui.add_space(12.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| match (&self.save_error, self.last_saved) {
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
            ui.spinner();
            return;
        };

        let available = (ui.available_width() - 8.0).max(80.0);
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

        if self.tab == Tab::Settings {
            egui::Panel::right("preview")
                .resizable(true)
                .default_size(380.0)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical().show(ui, |ui| self.preview(ui));
                });
        }

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

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn validation_text(validation: Option<MapValidation>) -> &'static str {
    match validation {
        Some(MapValidation::Verified) => "CRC verificado: el mapa es de esta build",
        Some(MapValidation::Mismatch) => "el mapa NO coincide con esta build",
        Some(MapValidation::Unavailable) => "sin CRC: no se puede verificar la build",
        None => "—",
    }
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
            recoverable: Some(90.0),
            weapon: Some(Weapon {
                kind: WeaponKind::Longsword,
                sharpness: Some(Sharpness {
                    lengths: [40, 30, 30, 30, 30, 20, 0],
                    level: SharpnessLevel::White,
                    max_level: SharpnessLevel::White,
                    current: 50.0,
                }),
            }),
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

    #[test]
    fn both_games_have_a_readable_name_and_a_module() {
        for kind in [GameKind::Rise, GameKind::World] {
            assert!(display_game(kind).starts_with("Monster Hunter"));
            assert!(kind.module().ends_with(".exe"));
        }
    }
}
