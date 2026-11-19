//! Hub de escritorio del overlay: diagnóstico de la conexión y ajustes con vista previa.
//!
//! El panel sobre el juego se sigue dibujando con el lienzo de `mh-overlay-ui`, que no depende
//! de GL ni de un compositor. Esta ventana cubre lo que no cabe en una tira: saber por qué no
//! se lee la memoria, elegir el mapa cuando el CRC no coincide y ajustar colores y disposición
//! viendo el resultado antes de aplicarlo.

mod tray;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui;
use mh_overlay_config::{Color, Config, OverlaySection, Position, StyleSection};
use mh_overlay_games::{
    Ailment, Buff, GameKind, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel,
    Snapshot, Weapon, WeaponDetail, WeaponGauge, WeaponKind,
};
use mh_overlay_memory::{
    AddressMap, MapValidation, Memory, MemoryError, Process, SelectedMap, maps_directory,
};
use mh_overlay_ui::OverlayError;
use mh_overlay_ui::icons;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::view;
use tray::Tray;

/// Paleta del hub. Es la misma familia que el panel sobre el juego: fondo profundo, texto claro
/// y un ámbar de acento, para que la ventana y el overlay se lean como la misma aplicación.
const BG: egui::Color32 = egui::Color32::from_rgb(0x0b, 0x0d, 0x12);
const SURFACE: egui::Color32 = egui::Color32::from_rgb(0x14, 0x18, 0x21);
const SURFACE_2: egui::Color32 = egui::Color32::from_rgb(0x1b, 0x20, 0x2b);
const SURFACE_3: egui::Color32 = egui::Color32::from_rgb(0x24, 0x2a, 0x37);
const INPUT: egui::Color32 = egui::Color32::from_rgb(0x0f, 0x12, 0x19);
const BORDER: egui::Color32 = egui::Color32::from_rgb(0x2a, 0x31, 0x3f);
const TEXT: egui::Color32 = egui::Color32::from_rgb(0xe8, 0xec, 0xf4);
const MUTED: egui::Color32 = egui::Color32::from_rgb(0x93, 0x9d, 0xb0);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xe6, 0xb4, 0x5e);
const GOOD: egui::Color32 = egui::Color32::from_rgb(0x7f, 0xd4, 0x7f);
const BAD: egui::Color32 = egui::Color32::from_rgb(0xe2, 0x56, 0x5c);

/// Versión que enseña el hub, la misma que `--version`.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tipografía, espaciado y colores de los widgets. Se aplica una sola vez, antes de construir
/// la ventana, para que egui no pinte un fotograma con su tema por defecto.
fn apply_theme(ctx: &egui::Context) {
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
    ctx.set_theme(egui::Theme::Dark);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
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

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = INPUT;
    visuals.faint_bg_color = SURFACE_2;
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_color = Some(MUTED);
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.30);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::same(8);
    }

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.weak_bg_fill = SURFACE_2;
    visuals.widgets.inactive.bg_fill = SURFACE_2;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.weak_bg_fill = SURFACE_3;
    visuals.widgets.hovered.bg_fill = SURFACE_3;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.6));
    visuals.widgets.active.weak_bg_fill = ACCENT.gamma_multiply(0.25);
    visuals.widgets.active.bg_fill = ACCENT.gamma_multiply(0.25);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    style.visuals = visuals;
    ctx.set_style_of(egui::Theme::Dark, style);
}

/// Icono de la ventana: el mismo dibujo de la bandeja, pasado de ARGB a RGBA.
fn window_icon() -> egui::IconData {
    let size = 64;
    let argb = icons::tray_icon(size);
    let mut rgba = Vec::with_capacity(argb.len());
    for pixel in argb.as_chunks::<4>().0 {
        rgba.extend_from_slice(&[pixel[1], pixel[2], pixel[3], pixel[0]]);
    }

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

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
         para leer sin ventana usa `mh-overlay --print` o `--watch`."
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
/// verifique contra la build en curso. `force_game` fija el juego de esta ejecución (el `--game`
/// de la CLI) y la detección automática no lo pisa.
pub fn run(
    config_path: PathBuf,
    config: Config,
    initial_map: Option<PathBuf>,
    force_game: Option<GameKind>,
) -> Result<(), AppError> {
    let has_display =
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();

    if !has_display {
        return Err(AppError::NoDisplay);
    }

    let text = Text::bundled()?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([860.0, 560.0])
            .with_title("mh-overlay · hub")
            .with_icon(window_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "mh-overlay · hub",
        options,
        Box::new(move |context| {
            apply_theme(&context.egui_ctx);
            Ok(Box::new(Hub::new(
                config_path,
                config,
                text,
                maps_directory().ok(),
                initial_map,
                force_game,
            )))
        }),
    )
    .map_err(AppError::Window)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Diagnostics,
    Weapon,
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
    /// Juego forzado por `--game`: no se guarda y la detección no lo pisa.
    force_game: Option<GameKind>,
    maps: Vec<PathBuf>,
    /// `None` fuerza una lectura en el siguiente fotograma (al arrancar o al cambiar de juego).
    last_refresh: Option<Instant>,
    last_saved: Option<Instant>,
    save_error: Option<String>,
    dirty: bool,
    preview: Option<egui::TextureHandle>,
    preview_size: egui::Vec2,
    preview_stale: bool,
    /// La vista previa sale del juego en vivo, no del ejemplo.
    preview_live: bool,
    /// Panel que el hub lanzó como proceso aparte, si está abierto.
    overlay: Option<Child>,
    /// Bandeja de sistema; `None` si el escritorio no la ofrece.
    tray: Option<Tray>,
    /// Cierre de verdad en curso: no se cancela para minimizar.
    closing: bool,
    /// Actualización corriendo en segundo plano, para no congelar la ventana.
    update: Option<Receiver<(bool, String)>>,
    /// Resultado de la última actualización.
    update_note: Option<(bool, String)>,
    /// Petición de permiso de memoria corriendo (pkexec).
    access: Option<Receiver<String>>,
    /// Resultado de la última petición de permiso.
    access_note: Option<String>,
    /// El usuario abrió la confirmación de «Conceder acceso».
    confirm_access: bool,
}

impl Hub {
    fn new(
        config_path: PathBuf,
        config: Config,
        text: Text,
        maps_directory: Option<PathBuf>,
        map_override: Option<PathBuf>,
        force_game: Option<GameKind>,
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
            force_game,
            maps: Vec::new(),
            last_refresh: None,
            last_saved: None,
            save_error: None,
            dirty: false,
            preview: None,
            preview_size: egui::Vec2::ZERO,
            preview_stale: true,
            preview_live: false,
            overlay: None,
            tray: None,
            closing: false,
            update: None,
            update_note: None,
            access: None,
            access_note: None,
            confirm_access: false,
        };

        hub.tray = Tray::spawn();
        hub.refresh_maps();
        hub
    }

    fn game(&self) -> GameKind {
        self.config.game_kind().unwrap_or(GameKind::Rise)
    }

    fn set_game(&mut self, kind: GameKind) {
        let changed = self.game() != kind;

        // Elegir a mano es una decisión del usuario: la detección automática deja de pisarla
        // hasta que se vuelva a marcar en Ajustes.
        self.config.game.kind = kind.to_string();
        self.config.game.auto_detect = false;
        self.force_game = None;
        self.dirty = true;

        if !changed {
            return;
        }

        self.map_override = None;
        self.attachment = None;
        self.status = Status::default();
        self.refresh_maps();
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
        let configured = self.force_game.unwrap_or_else(|| self.game());
        let auto = self.force_game.is_none() && self.config.game.auto_detect;
        let other = match configured {
            GameKind::Rise => GameKind::World,
            GameKind::World => GameKind::Rise,
        };

        let process = if auto {
            // Una sola pasada por /proc para los dos ejecutables: el proceso que aparezca dice
            // cuál se está jugando, sin que el usuario toque la configuración.
            Process::find(&[configured.module(), other.module()])
        } else {
            Process::find(&[configured.module()])
        };

        let process = match process {
            Ok(process) => process,
            Err(error) => {
                self.attachment = None;
                status.error = Some(error.to_string());
                return status;
            }
        };
        let kind = GameKind::from_module(process.module()).unwrap_or(configured);

        if kind != configured && auto {
            self.config.game.kind = kind.to_string();
            self.map_override = None;
            self.attachment = None;
            self.refresh_maps();
            self.dirty = true;
        }

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
        // La vista previa sigue al juego cuando hay partida; sin juego se queda en el ejemplo.
        self.preview_stale = true;
    }

    /// `true` si el panel lanzado por el hub sigue vivo. Recoge el hijo si ya terminó.
    fn overlay_running(&mut self) -> bool {
        let was_open = self.overlay.is_some();
        let running = self
            .overlay
            .as_mut()
            .is_some_and(|child| matches!(child.try_wait(), Ok(None)));

        if !running {
            self.overlay = None;

            // Si el panel terminó por su cuenta, el menú de la bandeja no debe seguir ofreciendo
            // «cerrar panel».
            if was_open && let Some(tray) = &self.tray {
                tray.set_overlay_running(false);
            }
        }

        running
    }

    /// Abre el panel sobre el juego como proceso aparte, o cierra el que esté abierto.
    fn toggle_overlay(&mut self) {
        if self.overlay_running() {
            if let Some(mut child) = self.overlay.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        } else if let Ok(executable) = std::env::current_exe() {
            let mut command = Command::new(executable);
            command
                .arg("--overlay")
                .arg("--config")
                .arg(&self.config_path);

            // Con detección automática el panel elige solo; si el usuario fijó un juego, se
            // respeta esa elección también en el proceso hijo.
            if self.force_game.is_some() || !self.config.game.auto_detect {
                command.arg("--game").arg(self.game().to_string());
            }

            match command.spawn() {
                Ok(child) => self.overlay = Some(child),
                Err(error) => {
                    eprintln!("aviso: no se pudo abrir el panel: {error}");
                    self.save_error = Some(format!("no se pudo abrir el panel: {error}"));
                }
            }
        }

        if let Some(tray) = &self.tray {
            tray.set_overlay_running(self.overlay.is_some());
        }
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
        let live = self.status.snapshot.player.is_some()
            || self.status.snapshot.hunt.is_some()
            || !self.status.snapshot.monsters.is_empty();
        let snapshot = if live {
            self.status.snapshot.clone()
        } else {
            sample()
        };
        let canvas = view::render(&self.config, &self.text, &snapshot, None);
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
        self.preview_live = live;
    }

    /// Estado de la conexión en una frase corta y su color.
    fn connection_state(&self) -> (&'static str, String, egui::Color32) {
        match (&self.status.error, self.status.pid) {
            (None, Some(pid)) => (
                "Leyendo el juego",
                format!(
                    "pid {pid} · {}",
                    self.status.map.as_deref().unwrap_or("sin mapa")
                ),
                GOOD,
            ),
            (Some(error), Some(_)) => ("Error de lectura", error.clone(), BAD),
            (Some(_), None) => (
                "Sin juego",
                "Abre Monster Hunter y el hub se conecta solo.".to_string(),
                MUTED,
            ),
            _ => (
                "Sin leer todavía",
                "La primera lectura llega en un instante.".to_string(),
                MUTED,
            ),
        }
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, tab: Tab, title: &str, subtitle: &str) {
        let active = self.tab == tab;
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, 44.0), egui::Sense::click());

        let fill = if active {
            ACCENT.gamma_multiply(0.14)
        } else if response.hovered() {
            SURFACE_2
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(9), fill);
        if active {
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(9),
                egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.45)),
                egui::StrokeKind::Inside,
            );
        }

        let x = rect.left() + 12.0;
        ui.painter().text(
            egui::pos2(x, rect.top() + 7.0),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::proportional(15.0),
            if active { ACCENT } else { TEXT },
        );
        ui.painter().text(
            egui::pos2(x, rect.top() + 26.0),
            egui::Align2::LEFT_TOP,
            subtitle,
            egui::FontId::proportional(11.5),
            MUTED,
        );

        if response.clicked() {
            self.tab = tab;
        }
        ui.add_space(2.0);
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(ACCENT)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(9, 4))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("MH").color(BG).strong().size(17.0));
                });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("mh-overlay").strong().size(18.0));
                ui.label(
                    egui::RichText::new("Monster Hunter HUD")
                        .small()
                        .color(MUTED),
                );
            });
        });

        ui.add_space(20.0);
        self.nav_item(ui, Tab::Diagnostics, "Diagnóstico", "Conexión y lectura");
        self.nav_item(ui, Tab::Weapon, "Arma", "Filo, medidores y datos");
        self.nav_item(ui, Tab::Settings, "Ajustes", "Panel y colores");

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            // En un layout de abajo arriba, lo primero que se añade queda más abajo.
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!(
                    "v{VERSION} · {}",
                    if running_as_appimage() {
                        "AppImage"
                    } else {
                        "instalado"
                    }
                ))
                .small()
                .color(MUTED),
            );

            ui.add_space(6.0);
            let updating = self.update.is_some();
            if ui
                .add_enabled(
                    !updating,
                    egui::Button::new(if updating {
                        "Buscando actualización…"
                    } else {
                        "Buscar actualización"
                    })
                    .min_size(egui::vec2(ui.available_width(), 30.0)),
                )
                .on_hover_text(
                    "Comprueba la última release y, si hay una nueva, reemplaza esta instalación \
                     sin desinstalar ni perder la configuración.",
                )
                .clicked()
            {
                self.start_update();
            }

            if let Some((ok, message)) = &self.update_note {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(message)
                        .small()
                        .color(if *ok { GOOD } else { BAD }),
                );
            }

            ui.add_space(10.0);
            let running = self.overlay_running();
            let (label, fill, color) = if running {
                ("Cerrar panel", SURFACE_3, TEXT)
            } else {
                ("Abrir panel", ACCENT, BG)
            };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(label).color(color).strong())
                        .fill(fill)
                        .min_size(egui::vec2(ui.available_width(), 34.0)),
                )
                .on_hover_text("Abre el overlay sobre el juego como proceso aparte")
                .clicked()
            {
                self.toggle_overlay();
            }

            ui.add_space(10.0);
            let (title, detail, color) = self.connection_state();
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, color);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).small().color(color).strong());
                    ui.label(egui::RichText::new(detail).size(10.5).color(MUTED));
                });
            });
        });
    }

    fn page_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.label(egui::RichText::new(title).size(28.0).strong().color(TEXT));
        ui.label(egui::RichText::new(subtitle).size(13.5).color(MUTED));
        ui.add_space(14.0);
    }

    /// Lanza `--update` en otro hilo: la descarga y el reemplazo no deben congelar la ventana.
    fn start_update(&mut self) {
        if self.update.is_some() {
            return;
        }

        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(error) => {
                self.update_note =
                    Some((false, format!("no se pudo localizar el binario: {error}")));
                return;
            }
        };

        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = match Command::new(&executable).arg("--update").output() {
                Ok(output) => {
                    let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();

                    if !error.is_empty() {
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(&error);
                    }

                    (output.status.success(), text)
                }
                Err(error) => (
                    false,
                    format!("no se pudo ejecutar el actualizador: {error}"),
                ),
            };

            let _ = sender.send(result);
        });

        self.update = Some(receiver);
        self.update_note = None;
    }

    fn poll_update(&mut self) {
        let Some(receiver) = &self.update else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.update_note = Some(result);
                self.update = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.update_note =
                    Some((false, "el actualizador terminó sin respuesta".to_string()));
                self.update = None;
            }
        }
    }

    /// `true` cuando el último error es el `EPERM` de `ptrace_scope`.
    fn permission_denied(&self) -> bool {
        self.status.error.as_deref().is_some_and(|error| {
            error.contains("permiso")
                || error.contains("Permission denied")
                || error.contains("denegado")
        })
    }

    /// Baja `kernel.yama.ptrace_scope` a 0 con la ventana de polkit, sin pasar por la consola.
    /// Es el único ajuste del sistema que el overlay necesita y vuelve solo al reiniciar.
    fn grant_memory_access(&mut self) {
        self.confirm_access = false;

        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let message = match Command::new("pkexec")
                .args(["sysctl", "-w", "kernel.yama.ptrace_scope=0"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    "acceso concedido hasta el próximo reinicio".to_string()
                }
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if error.is_empty() {
                        "no se pudo conceder el acceso".to_string()
                    } else {
                        format!("no se pudo conceder el acceso: {error}")
                    }
                }
                Err(error) => format!("no se pudo pedir permisos con pkexec: {error}"),
            };

            let _ = sender.send(message);
        });

        self.access = Some(receiver);
        self.access_note = None;
    }

    fn poll_access(&mut self) {
        let Some(receiver) = &self.access else {
            return;
        };

        match receiver.try_recv() {
            Ok(message) => {
                let granted = message.contains("concedido");
                self.access_note = Some(message);
                self.access = None;

                if granted {
                    self.last_refresh = None;
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.access_note =
                    Some("la petición de permisos terminó sin respuesta".to_string());
                self.access = None;
            }
        }
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
        Self::page_header(
            ui,
            "Diagnóstico",
            "Qué está pasando con la conexión al juego, sin adivinar.",
        );

        let (title, detail, color) = self.connection_state();
        egui::Frame::new()
            .fill(color.gamma_multiply(0.10))
            .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.45)))
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 5.5, color);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(title).size(20.0).strong().color(TEXT));
                        ui.label(egui::RichText::new(detail).size(12.5).color(MUTED));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Leer ahora").clicked() {
                            self.refresh();
                        }
                        if self.status.checked_at.is_none() {
                            ui.spinner();
                        }
                    });
                });
            });
        ui.add_space(10.0);

        card(ui, Some("CONEXIÓN"), |ui| {
            egui::Grid::new("diagnostico")
                .num_columns(2)
                .spacing([24.0, 9.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Juego").color(MUTED));
                    ui.label(format!(
                        "{} · {}",
                        display_game(self.game()),
                        self.game().module()
                    ));
                    ui.end_row();

                    ui.label(egui::RichText::new("Proceso").color(MUTED));
                    match (self.status.pid, self.status.base) {
                        (Some(pid), Some(base)) => {
                            ui.label(format!("pid {pid} · base del módulo 0x{base:08X}"));
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

                    ui.label(egui::RichText::new("Verificación").color(MUTED));
                    let (text, color) = match self.status.validation {
                        Some(MapValidation::Verified) => ("CRC verificado", GOOD),
                        Some(MapValidation::Mismatch) => ("no coincide con esta build", BAD),
                        Some(MapValidation::Unavailable) => ("sin CRC que comparar", MUTED),
                        None => ("—", MUTED),
                    };
                    ui.colored_label(color, text);
                    ui.end_row();
                });
        });

        card(ui, Some("LECTURA"), |ui| {
            let player = match &self.status.snapshot.player {
                Some(player) => format!("{:.0} %", player.health.percent()),
                None => "—".to_string(),
            };
            let hunt = match &self.status.snapshot.hunt {
                Some(hunt) => hunt.time_label(),
                None => "—".to_string(),
            };
            let last = match (self.status.checked_at, self.status.read_ms) {
                (Some(at), Some(ms)) => format!("{} · {} ms", seconds_ago(at.elapsed()), ms),
                (Some(at), None) => seconds_ago(at.elapsed()),
                _ => "—".to_string(),
            };

            ui.horizontal_wrapped(|ui| {
                stat_tile(
                    ui,
                    "Monstruos",
                    &self.status.snapshot.monsters.len().to_string(),
                    ACCENT,
                );
                stat_tile(ui, "Jugador", &player, GOOD);
                stat_tile(ui, "Cacería", &hunt, TEXT);
                stat_tile(ui, "Última lectura", &last, MUTED);
            });

            if let Some(player) = &self.status.snapshot.player {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!(
                        "vida {:.0} % · aguante {:.0} %",
                        player.health.percent(),
                        player.stamina.percent()
                    ))
                    .small()
                    .color(MUTED),
                );
            }
            if let Some(hunt) = &self.status.snapshot.hunt {
                ui.label(
                    egui::RichText::new(format!(
                        "vidas del equipo {}/{}",
                        hunt.deaths, hunt.max_deaths
                    ))
                    .small()
                    .color(MUTED),
                );
            }
        });

        if let Some(error) = &self.status.error {
            alert(ui, BAD, "La lectura falló", error);
        }

        if self.status.validation == Some(MapValidation::Mismatch) {
            alert(
                ui,
                ACCENT,
                "El mapa no es de esta build",
                "Los números que salgan serán basura plausible. Elige a mano el mapa de tu versión \
                 en «Mapa», o pon el archivo correcto en data/maps.",
            );
        }

        if self.permission_denied() {
            card(ui, Some("PERMISO DE MEMORIA"), |ui| {
                ui.label(
                    "El kernel está bloqueando `process_vm_readv`: el proceso se ve, pero ninguna \
                     lectura funciona. El overlay necesita `CAP_SYS_PTRACE` o bajar \
                     `kernel.yama.ptrace_scope`.",
                );
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button("Copiar orden")
                        .on_hover_text("sudo sysctl -w kernel.yama.ptrace_scope=0")
                        .clicked()
                    {
                        ui.ctx()
                            .copy_text("sudo sysctl -w kernel.yama.ptrace_scope=0".to_string());
                    }

                    let can_pkexec = command_exists("pkexec");
                    if ui
                        .add_enabled(
                            can_pkexec && self.access.is_none(),
                            egui::Button::new("Conceder acceso…"),
                        )
                        .on_hover_text(
                            "Pide la contraseña con polkit y aplica el mismo cambio, sin abrir \
                             una terminal",
                        )
                        .clicked()
                    {
                        self.confirm_access = true;
                    }
                });

                if self.confirm_access {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(
                            "Se pedirá tu contraseña y se pondrá kernel.yama.ptrace_scope en 0 \
                             hasta el próximo reinicio. Es el mismo ajuste que la orden de arriba.",
                        )
                        .small()
                        .color(ACCENT),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Conceder").clicked() {
                            self.grant_memory_access();
                        }
                        if ui.button("Cancelar").clicked() {
                            self.confirm_access = false;
                        }
                    });
                }

                if self.access.is_some() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("esperando permisos…")
                                .small()
                                .color(MUTED),
                        );
                    });
                }

                if let Some(note) = &self.access_note {
                    ui.label(egui::RichText::new(note).small().color(
                        if note.contains("concedido") {
                            GOOD
                        } else {
                            BAD
                        },
                    ));
                }
            });
        }
    }

    fn weapon(&mut self, ui: &mut egui::Ui) {
        Self::page_header(
            ui,
            "Arma",
            "La clase equipada, el filo, los medidores y los datos de combate que el juego expone.",
        );

        let Some(weapon) = self
            .status
            .snapshot
            .player
            .as_ref()
            .and_then(|player| player.weapon.as_ref())
        else {
            egui::Frame::new()
                .fill(SURFACE)
                .stroke(egui::Stroke::new(1.0, BORDER))
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        egui::RichText::new("Sin arma detectada")
                            .size(18.0)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Abre el juego y espera a que el hub lea al jugador. Si el diagnóstico \
                             dice «sin juego», el proceso todavía no está corriendo.",
                        )
                        .color(MUTED),
                    );
                });
            return;
        };

        egui::Frame::new()
            .fill(SURFACE)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(weapon.kind.label())
                                .size(24.0)
                                .strong()
                                .color(TEXT),
                        );
                        ui.label(
                            egui::RichText::new(if weapon.kind.has_sharpness() {
                                "Cuerpo a cuerpo"
                            } else {
                                "A distancia"
                            })
                            .size(12.5)
                            .color(MUTED),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(sharpness) = &weapon.sharpness {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Filo").small().color(MUTED));
                                ui.label(
                                    egui::RichText::new(sharpness.level.label())
                                        .size(18.0)
                                        .color(sharpness_color(sharpness.level)),
                                );
                            });
                        }
                    });
                });
            });
        ui.add_space(10.0);

        if let Some(sharpness) = &weapon.sharpness {
            card(ui, Some("FILO"), |ui| {
                sharpness_bar(ui, sharpness);
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for (index, level) in SharpnessLevel::ALL.iter().enumerate() {
                        let length = sharpness.lengths[index];
                        if length == 0 {
                            continue;
                        }
                        ui.label(
                            egui::RichText::new(format!("{} {length}", level.label()))
                                .small()
                                .color(sharpness_color(*level)),
                        );
                    }
                });
                ui.label(
                    egui::RichText::new(format!(
                        "actual {} · máximo {}",
                        sharpness.current.round(),
                        sharpness.max_level.label()
                    ))
                    .small()
                    .color(MUTED),
                );
            });
        }

        if let Some(gauge) = &weapon.gauge {
            card(ui, Some("MEDIDOR DE CLASE"), |ui| {
                egui::Grid::new("arma-medidor")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .show(ui, |ui| {
                        for (label, value) in gauge_summary(gauge) {
                            ui.label(egui::RichText::new(label).color(MUTED));
                            ui.label(value);
                            ui.end_row();
                        }
                    });
            });
        }

        let detail = &weapon.detail;
        if detail.attack.is_some() || detail.affinity.is_some() || detail.element.is_some() {
            card(ui, Some("ESTADÍSTICAS CALCULADAS"), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if let Some(attack) = detail.attack {
                        stat_tile(ui, "Ataque", &format!("{attack:.0}"), ACCENT);
                    }
                    if let Some(affinity) = detail.affinity {
                        stat_tile(ui, "Afinidad", &format!("{affinity:+}%"), GOOD);
                    }
                    if let Some(element) = detail.element {
                        stat_tile(ui, "Elemento", &format!("{element:.0}"), TEXT);
                    }
                });
            });
        }

        if let Some(ranged) = &detail.ranged {
            card(ui, Some("MUNICIÓN"), |ui| {
                egui::Grid::new("arma-municion")
                    .num_columns(4)
                    .spacing([20.0, 5.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Tipo").small().color(MUTED));
                        ui.label(egui::RichText::new("Cargada").small().color(MUTED));
                        ui.label(egui::RichText::new("Reserva").small().color(MUTED));
                        ui.label(egui::RichText::new("Máximo").small().color(MUTED));
                        ui.end_row();

                        for slot in &ranged.ammo {
                            let name = slot
                                .name
                                .map(str::to_string)
                                .unwrap_or_else(|| format!("Munición {}", slot.index));
                            let name = if ranged.equipped == Some(slot.index) {
                                format!("▶ {name}")
                            } else {
                                name
                            };
                            ui.label(name);
                            ui.label(slot.loaded.to_string());
                            ui.label(slot.total.to_string());
                            ui.label(slot.max.to_string());
                            ui.end_row();
                        }
                    });
            });
        }

        if let Some(gunlance) = detail.gunlance {
            card(ui, Some("LANZA PISTOLA"), |ui| {
                egui::Grid::new("arma-lanza")
                    .num_columns(2)
                    .spacing([24.0, 5.0])
                    .show(ui, |ui| {
                        for (label, value) in [
                            (
                                "Normales",
                                format!("{}/{}", gunlance.shells, gunlance.max_shells),
                            ),
                            (
                                "Cargados",
                                format!("{}/{}", gunlance.charged, gunlance.max_charged),
                            ),
                            ("Dragón de fuego", format!("{:.0} s", gunlance.wyvern_fire)),
                            (
                                "Estaca",
                                if gunlance.stake_loaded {
                                    if gunlance.stake_timer > 0.0 {
                                        format!("cargada · {:.0} s", gunlance.stake_timer)
                                    } else {
                                        "cargada".to_string()
                                    }
                                } else {
                                    "—".to_string()
                                },
                            ),
                        ] {
                            ui.label(egui::RichText::new(label).color(MUTED));
                            ui.label(value);
                            ui.end_row();
                        }
                    });
            });
        }

        if let Some(horn) = &detail.horn {
            card(ui, Some("CORNAMUSA"), |ui| {
                ui.label(format!(
                    "Notas en cola: {}",
                    if horn.notes.is_empty() {
                        "—".to_string()
                    } else {
                        horn.notes
                            .iter()
                            .map(|note| note.label())
                            .collect::<Vec<_>>()
                            .join(" · ")
                    }
                ));
                if !horn.songs.is_empty() {
                    ui.label(format!(
                        "Melodías en cola: {}",
                        horn.songs
                            .iter()
                            .map(|song| song.id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !horn.available.is_empty() {
                    ui.collapsing("Melodías disponibles", |ui| {
                        egui::Grid::new("arma-melodias")
                            .num_columns(3)
                            .spacing([18.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("ID").small().color(MUTED));
                                ui.label(egui::RichText::new("Notas").small().color(MUTED));
                                ui.label(egui::RichText::new("Longitud").small().color(MUTED));
                                ui.end_row();
                                for song in &horn.available {
                                    ui.label(song.id.to_string());
                                    ui.label(song.notes_label());
                                    ui.label(song.length.to_string());
                                    ui.end_row();
                                }
                            });
                    });
                }
            });
        }

        if let Some(bow) = detail.bow {
            card(ui, Some("ARCO"), |ui| {
                ui.label(format!(
                    "Carga nivel {}/{} · {:.0} %",
                    bow.level,
                    bow.max_level,
                    bow.charge * 100.0
                ));
            });
        }

        if detail.is_empty() {
            ui.label(
                egui::RichText::new(
                    "Esta build no expone datos de munición, proyectiles o melodías en la \
                     memoria; el detalle disponible es el estado general del arma.",
                )
                .small()
                .color(MUTED),
            );
        }

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "En World, munición, proyectiles y melodías salen de los offsets de HunterPie \
                 legacy. En Rise solo se muestran los campos que HunterPie v2 lee.",
            )
            .small()
            .color(MUTED),
        );
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        let before = self.config.clone();

        Self::page_header(
            ui,
            "Ajustes",
            "Se guarda solo: el panel abierto aplica los cambios en el siguiente refresco.",
        );

        card(ui, Some("JUEGO"), |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Título").color(MUTED));
                for kind in [GameKind::Rise, GameKind::World] {
                    if ui
                        .selectable_label(self.game() == kind, display_game(kind))
                        .clicked()
                    {
                        self.set_game(kind);
                    }
                }
            });

            let auto_before = self.config.game.auto_detect;
            ui.checkbox(
                &mut self.config.game.auto_detect,
                "Detectar Rise/World automáticamente",
            )
            .on_hover_text(
                "Mira si MonsterHunterRise.exe o MonsterHunterWorld.exe están corriendo. \
                 Elegir un juego a mano lo desactiva.",
            );

            if self.config.game.auto_detect && !auto_before {
                self.force_game = None;
            }

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Refresco").color(MUTED));
                ui.add(
                    egui::DragValue::new(&mut self.config.game.interval_ms)
                        .range(100..=5000)
                        .suffix(" ms"),
                );
            });
        });

        card(ui, Some("PANEL"), |ui| {
            egui::Grid::new("panel")
                .num_columns(2)
                .spacing([24.0, 10.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Anclaje").color(MUTED));
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

                    ui.label(egui::RichText::new("Márgenes").color(MUTED));
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

                    ui.label(egui::RichText::new("Escala").color(MUTED));
                    ui.add(
                        egui::Slider::new(&mut self.config.overlay.scale, 0.5..=3.0)
                            .fixed_decimals(2)
                            .suffix("×"),
                    );
                    ui.end_row();

                    ui.label(egui::RichText::new("Monstruos a la vez").color(MUTED));
                    ui.add(egui::Slider::new(
                        &mut self.config.overlay.max_monsters,
                        1..=8,
                    ));
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.columns(2, |columns| {
                columns[0].checkbox(&mut self.config.overlay.show_parts, "Mostrar partes");
                columns[0].checkbox(&mut self.config.overlay.show_ailments, "Mostrar estados");
                columns[1].checkbox(
                    &mut self.config.overlay.click_through,
                    "Los clics atraviesan el panel",
                );
                columns[1].checkbox(
                    &mut self.config.overlay.hide_on_empty,
                    "Ocultar el panel sin monstruos",
                );
            });
        });

        card(ui, Some("ESTILO"), |ui| {
            egui::Grid::new("estilo")
                .num_columns(2)
                .spacing([24.0, 7.0])
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

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Vida baja por debajo de").color(MUTED));
                ui.add(
                    egui::Slider::new(&mut self.config.style.low_health_percent, 0.0..=100.0)
                        .suffix(" %"),
                );
            });

            ui.add_space(6.0);
            if ui.button("Restaurar panel y colores").clicked() {
                self.config.overlay = OverlaySection::default();
                self.config.style = StyleSection::default();
                self.dirty = true;
                self.preview_stale = true;
            }
        });

        if self.config != before {
            self.dirty = true;
            self.preview_stale = true;
        }

        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| match (&self.save_error, self.last_saved) {
            (Some(error), _) => {
                ui.colored_label(BAD, error);
            }
            (None, Some(at)) => {
                ui.label(
                    egui::RichText::new(format!(
                        "guardado hace {} en {}",
                        seconds_ago(at.elapsed()),
                        self.config_path.display()
                    ))
                    .small()
                    .color(MUTED),
                );
            }
            (None, None) => {
                ui.label(
                    egui::RichText::new("los cambios se guardan solos")
                        .small()
                        .color(MUTED),
                );
            }
        });
    }

    fn preview(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Vista previa").size(18.0).strong());
        ui.label(
            egui::RichText::new(if self.preview_live {
                "Datos en vivo del juego: arma, filo, buffs, monstruos y partes."
            } else {
                "Sin partida en curso: así se verá el panel con datos de ejemplo."
            })
            .small()
            .color(MUTED),
        );
        ui.add_space(10.0);

        let Some(texture) = &self.preview else {
            ui.spinner();
            return;
        };

        let available = (ui.available_width() - 24.0).max(80.0);
        let scale = (available / self.preview_size.x).clamp(0.2, 1.0);
        egui::Frame::new()
            .fill(SURFACE_2)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.image(egui::load::SizedTexture::new(
                    texture.id(),
                    self.preview_size * scale,
                ));
            });
    }
}

impl eframe::App for Hub {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(tray) = &self.tray {
            tray.signals().attach(ctx);
            let signals = tray.signals().clone();

            if signals.take_show() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }

            if signals.take_toggle_overlay() {
                self.toggle_overlay();
            }

            if signals.take_quit() {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // Con bandeja, la X minimiza; sin bandeja no hay forma de recuperar la ventana, así que
        // cerrar termina el proceso como en cualquier aplicación.
        if !self.closing
            && self.tray.is_some()
            && ctx.input(|input| input.viewport().close_requested())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.poll_update();
        self.poll_access();

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

        egui::Panel::left("navegacion")
            .exact_size(216.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(SURFACE)
                    .inner_margin(egui::Margin::symmetric(14, 14)),
            )
            .show(ui, |ui| self.sidebar(ui));

        if self.tab == Tab::Settings {
            egui::Panel::right("preview")
                .resizable(true)
                .default_size(400.0)
                .min_size(280.0)
                .show_separator_line(false)
                .frame(
                    egui::Frame::new()
                        .fill(BG)
                        .inner_margin(egui::Margin::same(20)),
                )
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.preview(ui));
                });
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(26)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.tab {
                        Tab::Diagnostics => self.diagnostics(ui),
                        Tab::Weapon => self.weapon(ui),
                        Tab::Settings => self.settings(ui),
                    });
            });

        ctx.request_repaint_after(Duration::from_millis(250));
    }
}

impl Drop for Hub {
    fn drop(&mut self) {
        if let Some(mut child) = self.overlay.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Tarjeta con el fondo y el borde del hub; el título es opcional y va en versalitas.
fn card(ui: &mut egui::Ui, title: Option<&str>, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(title) = title {
                ui.label(egui::RichText::new(title).size(11.0).strong().color(MUTED));
                ui.add_space(8.0);
            }
            add(ui);
        });
    ui.add_space(10.0);
}

/// Cifra grande con su etiqueta debajo, para los resúmenes de la pestaña de diagnóstico.
fn stat_tile(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(SURFACE_2)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(120.0);
            ui.label(egui::RichText::new(value).size(19.0).strong().color(color));
            ui.label(egui::RichText::new(label).size(11.0).color(MUTED));
        });
}

/// Aviso con color propio: error de lectura, mapa desajustado, permiso pendiente.
fn alert(ui: &mut egui::Ui, color: egui::Color32, title: &str, body: &str) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.08))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.45)))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(title).strong().color(color));
            ui.label(egui::RichText::new(body).size(12.5).color(MUTED));
        });
    ui.add_space(10.0);
}

fn sharpness_color(level: SharpnessLevel) -> egui::Color32 {
    let rgb = sharpness_rgb(level);
    egui::Color32::from_rgb(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    )
}

/// Barra de filo por tramos: cada color ocupa su longitud y lo ya gastado se ve apagado, así
/// que de un vistazo se sabe cuánto queda antes del siguiente color.
fn sharpness_bar(ui: &mut egui::Ui, sharpness: &Sharpness) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(5), INPUT);

    let total: f32 = sharpness
        .lengths
        .iter()
        .map(|length| f32::from(*length))
        .sum();
    if total <= 0.0 {
        return;
    }

    let inner = rect.shrink(1.0);
    let mut left = inner.left();
    let mut consumed = (total - sharpness.current).max(0.0);

    for (index, level) in SharpnessLevel::ALL.iter().enumerate() {
        let length = f32::from(sharpness.lengths[index]);
        if length <= 0.0 {
            continue;
        }

        let segment = length / total * inner.width();
        let used = consumed.min(length);
        let lit = segment * (1.0 - used / length);
        let dim = segment - lit;
        let color = sharpness_color(*level);

        if dim > 0.0 {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(left, inner.top()),
                    egui::vec2(dim, inner.height()),
                ),
                egui::CornerRadius::ZERO,
                color.gamma_multiply(0.22),
            );
        }
        if lit > 0.0 {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(left + dim, inner.top()),
                    egui::vec2(lit, inner.height()),
                ),
                egui::CornerRadius::ZERO,
                color,
            );
        }

        left += segment;
        consumed -= used;
    }

    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(5),
        egui::Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|directory| directory.join(name).is_file())
    })
}

fn running_as_appimage() -> bool {
    std::env::var_os("APPIMAGE").is_some()
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

fn display_game(kind: GameKind) -> &'static str {
    match kind {
        GameKind::Rise => "Monster Hunter Rise",
        GameKind::World => "Monster Hunter World",
    }
}

fn sharpness_rgb(level: SharpnessLevel) -> u32 {
    match level {
        SharpnessLevel::Red => 0xd6_4a_4a,
        SharpnessLevel::Orange => 0xe0_8a_3c,
        SharpnessLevel::Yellow => 0xe0_d0_4a,
        SharpnessLevel::Green => 0x7f_d4_7f,
        SharpnessLevel::Blue => 0x4f_9f_e6,
        SharpnessLevel::White => 0xe8_f0_f8,
        SharpnessLevel::Purple => 0xa9_7a_e6,
    }
}

fn gauge_summary(gauge: &WeaponGauge) -> Vec<(&'static str, String)> {
    match gauge {
        WeaponGauge::Spirit {
            buildup,
            level,
            timer,
            max_timer,
            regen,
        } => vec![
            ("Espíritu", format!("{buildup:.0} %")),
            ("Nivel", format!("{level}/3")),
            ("Duración", format!("{timer:.0} / {max_timer:.0} s")),
            ("Regeneración", format!("{regen:.0} s")),
        ],
        WeaponGauge::ChargeBlade {
            phials,
            max_phials,
            charge,
            shield,
            sword,
            axe,
        } => vec![
            ("Frascos", format!("{phials}/{max_phials}")),
            ("Carga", format!("{charge:.0} %")),
            ("Escudo", format!("{shield:.0} s")),
            ("Espada", format!("{sword:.0} s")),
            ("Hacha", format!("{axe:.0} s")),
        ],
        WeaponGauge::SwitchAxe {
            buildup,
            charge,
            timer,
        } => vec![
            ("Filo", format!("{buildup:.0} %")),
            ("Amped", format!("{charge:.0} %")),
            ("Bono", format!("{timer:.0} s")),
        ],
        WeaponGauge::DualBlades {
            demon,
            arch_demon,
            buildup,
        } => vec![
            (
                "Modo",
                if *arch_demon {
                    "archidemonio".to_string()
                } else if *demon {
                    "demonio".to_string()
                } else {
                    "—".to_string()
                },
            ),
            ("Carga", format!("{buildup:.0} %")),
        ],
        WeaponGauge::InsectGlaive {
            extracts,
            attack,
            speed,
            defense,
            stamina,
            max_stamina,
        } => vec![
            ("Esencias", extracts.label().to_string()),
            ("Ataque", format!("{attack:.0} s")),
            ("Velocidad", format!("{speed:.0} s")),
            ("Defensa", format!("{defense:.0} s")),
            ("Kinsecto", format!("{stamina:.0}/{max_stamina:.0}")),
        ],
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
                gauge: Some(WeaponGauge::Spirit {
                    buildup: 62.0,
                    level: 2,
                    timer: 98.0,
                    max_timer: 140.0,
                    regen: 0.0,
                }),
                detail: WeaponDetail::default(),
            }),
            buffs: vec![
                Buff {
                    name: "Semilla del poder",
                    timer: 84.0,
                    max_timer: 180.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Protección divina",
                    timer: 37.0,
                    max_timer: 120.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Zumo de energía",
                    timer: 240.0,
                    max_timer: 240.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Aguante infinito (furia)",
                    timer: 45.0,
                    max_timer: 60.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Regeneración de filo",
                    timer: 25.0,
                    max_timer: 60.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Veneno",
                    timer: 12.0,
                    max_timer: 20.0,
                    debuff: true,
                    infinite: false,
                },
                Buff {
                    name: "Fuego",
                    timer: 8.0,
                    max_timer: 15.0,
                    debuff: true,
                    infinite: false,
                },
            ],
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
        breaks: None,
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
