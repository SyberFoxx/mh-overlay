//! Hub de escritorio del overlay: diagnóstico de la conexión y ajustes con vista previa.
//!
//! El panel sobre el juego se sigue dibujando con el lienzo de `mh-overlay-ui`, que no depende
//! de GL ni de un compositor. Esta ventana cubre lo que no cabe en una tira: saber por qué no
//! se lee la memoria, elegir el mapa cuando el CRC no coincide y ajustar colores y disposición
//! viendo el resultado antes de aplicarlo.

pub mod log;
mod theme;
mod tray;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui;
use mh_overlay_config::{
    Backdrop, BarPlacement, Color, Config, Direction, Layout, LayoutPreset, Monsters,
    OverlaySection, Position, StylePreset, StyleSection, Theme,
};
use mh_overlay_games::{
    Ailment, Buff, Crown, GameKind, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel,
    Snapshot, Weapon, WeaponDetail, WeaponGauge, WeaponKind,
};
use mh_overlay_memory::{
    AddressMap, Lookalike, MapValidation, Memory, MemoryError, Process, SelectedMap, lookalikes,
    maps_directory,
};
use mh_overlay_ui::OverlayError;
use mh_overlay_ui::icons;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::view;
use theme::{Palette, RADIUS};
use tray::Tray;

/// Versión que enseña el hub, la misma que `--version`.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Alto aproximado del pie de la barra lateral; solo se usa para separarlo de la navegación.
const FOOTER_HEIGHT: f32 = 178.0;

/// Hueco entre la navegación y el pie de la barra lateral. Nunca negativo, para que una ventana
/// baja no empuje el pie fuera de la vista.
fn footer_space(available: f32, footer: f32) -> f32 {
    (available - footer).max(0.0)
}

/// Ancho a partir del cual la vista previa cabe al lado de los ajustes sin apretar el formulario.
const PREVIEW_NEXT_TO_SETTINGS: f32 = 1180.0;

/// Si la ventana no da para las dos columnas, la vista previa pasa debajo en vez de robarle
/// ancho a los controles (que es lo que cortaba los textos a media palabra).
fn packed_preview(window_width: f32) -> bool {
    window_width < PREVIEW_NEXT_TO_SETTINGS
}

/// Icono de la ventana: el mismo dibujo de la bandeja, pasado de ARGB a RGBA.
/// Tamaño recordado, dentro de lo que la ventana admite; un TOML editado a mano no la encoge
/// hasta hacerla inutilizable.
fn window_size(width: f32, height: f32) -> [f32; 2] {
    [width.clamp(860.0, 4096.0), height.clamp(560.0, 2160.0)]
}

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
    let log_path = log::install();
    // Desde aquí, cualquier pánico deja mensaje y pila en el registro en vez de perderse.
    log::install_panic_hook();

    let has_display =
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();

    log::info(&format!(
        "arranque · {VERSION} · {} · {} · tema {} · config {}",
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            "wayland"
        } else {
            "x11"
        },
        std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "escritorio sin declarar".into()),
        config.hub.theme,
        config_path.display()
    ));

    if !has_display {
        log::warn("sin DISPLAY ni WAYLAND_DISPLAY: no se abre la ventana");
        return Err(AppError::NoDisplay);
    }

    if log_path.is_none() {
        // El hub funciona igual, pero conviene que se sepa por qué no hay rastro.
        eprintln!("aviso: no hay dónde escribir el registro (¿HOME sin definir?)");
    }

    let text = Text::bundled()?;
    // El tema se aplica dentro del cierre, antes de mover `config` al `Hub`.
    let theme = config.hub.theme;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(window_size(
                config.hub.window_width,
                config.hub.window_height,
            ))
            .with_min_inner_size([860.0, 560.0])
            .with_title("mh-overlay · hub")
            .with_icon(window_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "mh-overlay · hub",
        options,
        Box::new(move |context| {
            theme::install(&context.egui_ctx);
            // Antes del primer fotograma, para que la ventana no parpadee con el tema de egui.
            theme::apply(&context.egui_ctx, theme);
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
    News,
    Diagnostics,
    Weapon,
    Settings,
}

/// Pestaña pedida con Ctrl+1..4; consume la tecla para que no la vea también otro control.
fn tab_shortcut(input: &mut egui::InputState) -> Option<Tab> {
    [
        (egui::Key::Num1, Tab::News),
        (egui::Key::Num2, Tab::Diagnostics),
        (egui::Key::Num3, Tab::Weapon),
        (egui::Key::Num4, Tab::Settings),
    ]
    .into_iter()
    .find_map(|(key, tab)| input.consume_key(egui::Modifiers::CTRL, key).then_some(tab))
}

fn consume_escape(input: &mut egui::InputState) -> bool {
    input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
}

/// Estado del `.map` en una línea, para el diagnóstico copiable y `--doctor`.
fn validation_label(validation: Option<MapValidation>) -> &'static str {
    match validation {
        Some(MapValidation::Verified) => "CRC verificado",
        Some(MapValidation::Mismatch) => "CRC no coincide con la build",
        Some(MapValidation::Unavailable) => "sin CRC que comparar",
        None => "sin validar",
    }
}

/// Notas de la última release, tal como las imprime `--release-notes`.
#[derive(Debug, Clone, Default)]
struct ReleaseInfo {
    tag: String,
    version: String,
    published: String,
    body: String,
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
    /// Paleta vigente, para el dibujo propio (tarjetas, píldoras, gráficas).
    palette: Palette,
    /// Procesos que mencionan o mapean el ejecutable, pedidos a mano desde Diagnóstico.
    seen: Option<Vec<Lookalike>>,
    /// Cuándo se copió el diagnóstico, para confirmarlo sin un diálogo.
    copied_at: Option<Instant>,
    /// Tema con el que se vistió la ventana; `None` hasta el primer fotograma.
    applied_theme: Option<Theme>,
    /// Si la paleta aplicada es oscura, para notar un cambio del escritorio en `auto`.
    applied_dark: bool,
    maps_directory: Option<PathBuf>,
    tab: Tab,
    status: Status,
    attachment: Option<Attachment>,
    /// `.map` elegido a mano; `None` es “el más nuevo que verifique”.
    map_override: Option<PathBuf>,
    /// Paso de la guía inicial; `None` cuando ya no se muestra.
    onboarding: Option<usize>,
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
    /// Ventana escondida en la bandeja: no hay nada que pintar ni que leer.
    hidden: bool,
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
    /// Comprobación de versión corriendo en segundo plano.
    check: Option<Receiver<Option<String>>>,
    /// La comprobación la pidió el botón: hay que dar feedback aunque no haya novedad.
    check_manual: bool,
    /// Versión nueva publicada, si la hay.
    available: Option<String>,
    /// La actualización ya está instalada y toca reiniciar.
    restart_ready: bool,
    /// Notas de la última release, para la pestaña Novedades.
    notes: Option<ReleaseInfo>,
    notes_error: Option<String>,
    notes_check: Option<Receiver<Result<ReleaseInfo, String>>>,
    /// Nombre que se está escribiendo para guardar la disposición actual.
    preset_name: String,
    /// Preset elegido en el hub; los primeros son los integrados.
    preset_selected: Option<usize>,
    /// Grupo que se está arrastrando en el editor visual.
    drag_bar: Option<String>,
    /// Distancia entre el puntero y la esquina de la placa, en píxeles lógicos.
    drag_offset: egui::Vec2,
    /// Guías del imán que se dibujan mientras se arrastra.
    snap_guide: Option<(i32, i32)>,
    /// Resolución física del monitor del hub, detectada en cada fotograma.
    detected_screen: Option<(u32, u32)>,
    /// Resolución elegida a mano para el editor; `None` usa la detectada.
    editor_screen: Option<(u32, u32)>,
    /// Fuerza la página al principio al cambiar de pestaña.
    scroll_to_top: bool,
    /// Últimos rectángulos del editor, reutilizados mientras se arrastra una placa.
    editor_bounds: Vec<view::PlacedBar>,
    /// Configuración con la que se calcularon esos rectángulos.
    editor_bounds_config: Config,
    /// Lectura con la que se calcularon esos rectángulos.
    editor_bounds_at: Option<Instant>,
    /// Lienzo con el que se calcularon esos rectángulos.
    editor_bounds_screen: Option<(u32, u32)>,
    /// Última vez que se subió una vista previa a la GPU.
    preview_built_at: Option<Instant>,
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
        let onboarding = (!config.hub.onboarding_done).then_some(0);
        let mut hub = Self {
            config_path,
            config,
            text,
            palette: Palette::dark(),
            seen: None,
            copied_at: None,
            applied_theme: None,
            applied_dark: true,
            maps_directory,
            tab: Tab::Diagnostics,
            status: Status::default(),
            attachment: None,
            map_override,
            onboarding,
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
            hidden: false,
            closing: false,
            update: None,
            update_note: None,
            access: None,
            access_note: None,
            confirm_access: false,
            check: None,
            check_manual: false,
            available: None,
            restart_ready: false,
            notes: None,
            notes_error: None,
            notes_check: None,
            preset_name: String::new(),
            preset_selected: None,
            drag_bar: None,
            drag_offset: egui::Vec2::ZERO,
            snap_guide: None,
            detected_screen: None,
            editor_screen: None,
            scroll_to_top: true,
            editor_bounds: Vec::new(),
            editor_bounds_config: Config::default(),
            editor_bounds_at: None,
            editor_bounds_screen: None,
            preview_built_at: None,
        };

        hub.tray = Tray::spawn();
        hub.refresh_maps();
        hub.start_check(false);
        hub.start_notes();
        hub
    }

    /// Viste la ventana con el tema configurado y lo rehace si cambia el del escritorio.
    ///
    /// Se llama una vez por fotograma: cuando no hay nada que cambiar no toca el estilo, que
    /// si no invalidaría la maquetación en cada frame.
    /// Cambia de pestaña y vuelve arriba, para que un atajo no deje media pantalla scrolleada.
    fn set_tab(&mut self, tab: Tab) {
        if self.tab != tab {
            self.tab = tab;
            self.scroll_to_top = true;
        }
    }

    /// Atajos: Ctrl+1..4 cambia de pestaña y Esc cierra la guía o la confirmación.
    fn shortcuts(&mut self, ctx: &egui::Context) {
        let (selected, escape) =
            ctx.input_mut(|input| (tab_shortcut(input), consume_escape(input)));

        if escape {
            if self.onboarding.is_some() {
                self.finish_onboarding();
            } else if self.confirm_access {
                self.confirm_access = false;
            }
        }

        if let Some(tab) = selected {
            self.set_tab(tab);
        }
    }

    fn sync_theme(&mut self, ctx: &egui::Context) {
        let theme = self.config.hub.theme;
        let system_dark = matches!(ctx.theme(), egui::Theme::Dark);
        let changed = self.applied_theme != Some(theme)
            || (theme == Theme::Auto && self.applied_dark != system_dark);

        if changed {
            let look = theme::apply(ctx, theme);
            self.palette = look.palette;
            self.applied_theme = Some(theme);
            self.applied_dark = look.dark;
            self.preview_stale = true;
        }
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
        let previous = self.status.error.clone();
        self.status = self.probe();
        self.last_refresh = Some(Instant::now());

        // Solo las transiciones: el mismo error a 10 Hz llenaría el archivo.
        if self.status.error != previous {
            match &self.status.error {
                Some(error) => log::warn(&format!("lectura: {error}")),
                None => log::info("lectura recuperada"),
            }
        }
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
                    log::warn(&format!("no se pudo abrir el panel: {error}"));
                    eprintln!("aviso: no se pudo abrir el panel: {error}");
                    self.save_error = Some(format!("no se pudo abrir el panel: {error}"));
                }
            }
        }

        if let Some(tray) = &self.tray {
            tray.set_overlay_running(self.overlay.is_some());
        }
    }

    /// Guarda el tamaño actual de la ventana para la próxima vez que se abra.
    fn remember_geometry(&mut self, ctx: &egui::Context) {
        let size = ctx.viewport_rect().size();

        // Escondida o minimizada la ventana no informa de un tamaño real; guardar eso la dejaría
        // diminuta para siempre.
        if !size.x.is_finite() || !size.y.is_finite() || size.x < 400.0 || size.y < 300.0 {
            return;
        }

        if (self.config.hub.window_width - size.x).abs() < 1.0
            && (self.config.hub.window_height - size.y).abs() < 1.0
        {
            return;
        }

        self.config.hub.window_width = size.x;
        self.config.hub.window_height = size.y;
        self.dirty = true;
        self.save();
    }

    fn save(&mut self) {
        self.dirty = false;

        if let Err(error) = self.config.validate() {
            log::warn(&format!("configuración inválida, no se guarda: {error}"));
            self.save_error = Some(error.to_string());
            return;
        }

        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.save_error = None;
                self.last_saved = Some(Instant::now());
            }
            Err(error) => {
                log::warn(&format!("no se guardó la configuración: {error}"));
                self.save_error = Some(error.to_string());
            }
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
        let canvas = if self.config.overlay.layout == Layout::Free {
            // La vista previa usa una pantalla lógica de 1920×1080 y luego recorta el hueco
            // vacío: las placas se ven al mismo tamaño que en el juego, no en miniatura.
            let rendered =
                view::render_free(&self.config, &self.text, &snapshot, None, (1920, 1080));
            let cropped = rendered.content_bounds().map(|bounds| {
                let pad = 12;
                let x = (bounds.x - pad).max(0);
                let y = (bounds.y - pad).max(0);
                let width = (bounds.width as i32 + pad * 2).min(1920 - x) as u32;
                let height = (bounds.height as i32 + pad * 2).min(1080 - y) as u32;
                rendered.crop(mh_overlay_ui::canvas::Rect::new(x, y, width, height))
            });
            cropped.unwrap_or(rendered)
        } else {
            view::render(&self.config, &self.text, &snapshot, None)
        };
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
        self.preview_built_at = Some(Instant::now());
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
                self.palette.good,
            ),
            (Some(error), Some(_)) => ("Error de lectura", error.clone(), self.palette.bad),
            (Some(_), None) => (
                "Sin juego",
                "Abre Monster Hunter y el hub se conecta solo.".to_string(),
                self.palette.muted,
            ),
            _ => (
                "Sin leer todavía",
                "La primera lectura llega en un instante.".to_string(),
                self.palette.muted,
            ),
        }
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, tab: Tab, title: &str, subtitle: &str, hint: &str) {
        let active = self.tab == tab;
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, 50.0), egui::Sense::click());

        let fill = if active {
            self.palette.accent.gamma_multiply(0.14)
        } else if response.hovered() {
            self.palette.surface_2
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(9), fill);
        if active {
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(9),
                egui::Stroke::new(1.0, self.palette.accent.gamma_multiply(0.45)),
                egui::StrokeKind::Inside,
            );
        }

        // El texto se coloca con un layout de verdad, no pintado a mano: así el atajo reserva su
        // hueco y el subtítulo se recorta con puntos suspensivos en vez de pisar el borde.
        let inner = rect.shrink2(egui::vec2(12.0, 7.0));
        let mut face = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        {
            let ui = &mut face;
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(title).size(15.0).color(if active {
                    self.palette.accent
                } else {
                    self.palette.text
                }));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(hint).size(10.5).color(if active {
                        self.palette.accent.gamma_multiply(0.8)
                    } else {
                        self.palette.muted.gamma_multiply(0.7)
                    }));
                });
            });
            ui.add(
                egui::Label::new(
                    egui::RichText::new(subtitle)
                        .size(11.0)
                        .color(self.palette.muted),
                )
                .truncate(),
            );
        }

        if response.clicked() {
            self.set_tab(tab);
        }
        ui.add_space(2.0);
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(self.palette.accent)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(9, 4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("MH")
                            .color(self.palette.bg)
                            .strong()
                            .size(17.0),
                    );
                });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("mh-overlay").strong().size(18.0));
                ui.label(
                    egui::RichText::new("Monster Hunter HUD")
                        .small()
                        .color(self.palette.muted),
                );
            });
        });

        ui.add_space(20.0);
        self.nav_item(
            ui,
            Tab::News,
            "Novedades",
            "Qué trae la última versión",
            "Ctrl+1",
        );
        self.nav_item(
            ui,
            Tab::Diagnostics,
            "Diagnóstico",
            "Conexión y lectura",
            "Ctrl+2",
        );
        self.nav_item(ui, Tab::Weapon, "Arma", "Filo, medidores y datos", "Ctrl+3");
        self.nav_item(ui, Tab::Settings, "Ajustes", "Panel y colores", "Ctrl+4");

        // El pie se ancla al fondo con un hueco calculado y se escribe de arriba abajo: así el
        // orden del código es el orden en pantalla y ningún texto queda pisado por el botón que
        // tiene debajo.
        ui.add_space(footer_space(ui.available_height(), FOOTER_HEIGHT));

        let (title, detail, color) = self.connection_state();
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, color);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).small().color(color).strong());
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(detail)
                            .size(10.5)
                            .color(self.palette.muted),
                    )
                    .wrap(),
                );
            });
        });

        ui.add_space(14.0);
        let running = self.overlay_running();
        let (label, fill, color) = if running {
            ("Cerrar panel", self.palette.surface_3, self.palette.text)
        } else {
            ("Abrir panel", self.palette.accent, self.palette.bg)
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

        // Con el panel abierto y sin monstruos no hay nada que enseñar: el panel se oculta a
        // propósito, y sin decirlo parece que «Abrir panel» no hace nada.
        if running && self.status.snapshot.monsters.is_empty() && self.config.overlay.hide_on_empty
        {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("en espera: el panel aparece al entrar en una cacería")
                    .small()
                    .color(self.palette.muted),
            );
        }

        ui.add_space(10.0);
        let checking = self.check.is_some();
        let updating = self.update.is_some();
        let busy = checking || updating;
        let (label, hint, enabled) = if self.restart_ready {
            (
                "Reiniciar ahora",
                "Cierra y vuelve a abrir mh-overlay con la versión nueva",
                true,
            )
        } else if self.available.is_some() {
            (
                "Instalar actualización",
                "Descarga y reemplaza esta instalación sin desinstalar nada",
                !updating,
            )
        } else if checking {
            (
                "Buscando actualización…",
                "Consultando la última release",
                false,
            )
        } else {
            (
                "Buscar actualización",
                "Comprueba si hay una versión nueva, sin descargar nada",
                !busy,
            )
        };

        if ui
            .add_enabled(
                enabled,
                egui::Button::new(label).min_size(egui::vec2(ui.available_width(), 30.0)),
            )
            .on_hover_text(hint)
            .clicked()
        {
            if self.restart_ready {
                self.restart(ui.ctx());
            } else if self.available.is_some() {
                self.start_update();
            } else {
                self.start_check(true);
            }
        }

        if let Some((ok, message)) = &self.update_note {
            ui.add_space(6.0);
            ui.add(
                egui::Label::new(egui::RichText::new(message).small().color(if *ok {
                    self.palette.good
                } else {
                    self.palette.bad
                }))
                .wrap(),
            );
        }

        if let Some(version) = &self.available {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!("Actualización {version} disponible"))
                    .small()
                    .color(self.palette.accent),
            );
        }

        ui.add_space(8.0);
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
            .color(self.palette.muted),
        );
    }

    fn page_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        let palette = Palette::current();
        ui.label(
            egui::RichText::new(title)
                .size(28.0)
                .strong()
                .color(palette.text),
        );
        ui.label(
            egui::RichText::new(subtitle)
                .size(13.5)
                .color(palette.muted),
        );
        ui.add_space(14.0);
    }

    /// Guía de primera ejecución: juego, permiso de memoria y primer panel.
    fn onboarding(&mut self, ctx: &egui::Context) {
        let Some(step) = self.onboarding else {
            return;
        };

        let mut done = false;
        let mut next = None;
        let mut open_overlay = false;

        egui::Window::new("Primeros pasos")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .min_width(440.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!("PASO {} DE 3", step + 1))
                        .size(11.0)
                        .strong()
                        .color(self.palette.accent),
                );
                ui.add_space(4.0);

                match step {
                    0 => self.onboarding_game(ui),
                    1 => self.onboarding_permission(ui),
                    _ => open_overlay = self.onboarding_ready(ui),
                }

                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if step > 0 && ui.button("Atrás").clicked() {
                        next = Some(step - 1);
                    }

                    if ui.button("Saltar guía").clicked() {
                        done = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if step < 2 {
                            if ui.button("Siguiente").clicked() {
                                next = Some(step + 1);
                            }
                        } else if ui.button("Empezar").clicked() {
                            done = true;
                        }
                    });
                });
            });

        if open_overlay {
            self.toggle_overlay();
            done = true;
        }

        if done {
            self.finish_onboarding();
        } else if let Some(step) = next {
            self.onboarding = Some(step);
        }
    }

    fn onboarding_game(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new(
                "mh-overlay lee la memoria del juego mientras corre bajo \
                                 Proton y dibuja el estado de la cacería encima.",
            )
            .size(15.0),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Puede detectar Rise o World solo, o quedarse con el título que elijas.",
            )
            .color(self.palette.muted),
        );
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(self.config.game.auto_detect, "Detectar automáticamente")
                .on_hover_text(
                    "Mira si MonsterHunterRise.exe o MonsterHunterWorld.exe están corriendo",
                )
                .clicked()
            {
                self.config.game.auto_detect = true;
                self.force_game = None;
                self.dirty = true;
                self.last_refresh = None;
            }

            for kind in [GameKind::Rise, GameKind::World] {
                let selected = !self.config.game.auto_detect && self.game() == kind;

                if ui.selectable_label(selected, display_game(kind)).clicked() {
                    self.set_game(kind);
                }
            }
        });

        ui.add_space(8.0);
        let (title, detail, color) = self.connection_state();
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, color);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).small().color(color).strong());
                ui.label(
                    egui::RichText::new(detail)
                        .size(11.0)
                        .color(self.palette.muted),
                );
            });
        });
    }

    fn onboarding_permission(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Leer la memoria necesita permiso de ptrace")
                .size(17.0)
                .strong(),
        );
        ui.add_space(2.0);
        ui.label(
            "En un Arch normal, kernel.yama.ptrace_scope=1 bloquea process_vm_readv. El overlay \
             necesita CAP_SYS_PTRACE o bajar ese valor. No escribe nada en el juego.",
        );

        if self.permission_denied() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("La última lectura falló por permisos.")
                    .color(self.palette.bad)
                    .strong(),
            );
        }

        ui.add_space(6.0);
        if command_exists("pkexec") {
            if ui
                .add_enabled(
                    self.access.is_none(),
                    egui::Button::new("Conceder acceso con polkit"),
                )
                .clicked()
            {
                self.grant_memory_access();
            }
            ui.label(
                egui::RichText::new(
                    "Se pedirá tu contraseña y se pondrá el valor en 0 hasta el próximo reinicio.",
                )
                .small()
                .color(self.palette.muted),
            );
        } else {
            if ui.button("Copiar orden").clicked() {
                ui.ctx()
                    .copy_text("sudo sysctl -w kernel.yama.ptrace_scope=0".to_string());
            }
            ui.label(
                egui::RichText::new("sudo sysctl -w kernel.yama.ptrace_scope=0")
                    .monospace()
                    .small()
                    .color(self.palette.muted),
            );
        }

        if self.access.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new("esperando permisos…")
                        .small()
                        .color(self.palette.muted),
                );
            });
        }

        if let Some(note) = &self.access_note {
            ui.label(
                egui::RichText::new(note)
                    .small()
                    .color(if note.contains("concedido") {
                        self.palette.good
                    } else {
                        self.palette.bad
                    }),
            );
        }

        if ui.button("Comprobar el juego").clicked() {
            self.attachment = None;
            self.last_refresh = None;
        }
    }

    fn onboarding_ready(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label(
            egui::RichText::new("Ya está todo listo")
                .size(17.0)
                .strong(),
        );
        ui.add_space(2.0);
        ui.label(
            "El panel se abre como una ventana aparte y se puede cerrar desde aquí. La \
             configuración se guarda sola.",
        );
        ui.add_space(6.0);

        egui::Grid::new("onboarding-resumen")
            .num_columns(2)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Juego").color(self.palette.muted));
                ui.label(display_game(self.game()));
                ui.end_row();

                ui.label(egui::RichText::new("Conexión").color(self.palette.muted));
                ui.label(self.connection_state().0);
                ui.end_row();
            });

        ui.add_space(8.0);
        if self.overlay_running() {
            ui.label(
                egui::RichText::new("El panel ya está abierto.")
                    .small()
                    .color(self.palette.good),
            );
        } else if ui.button("Abrir el panel ahora").clicked() {
            return true;
        }

        false
    }

    fn finish_onboarding(&mut self) {
        self.config.hub.onboarding_done = true;
        self.onboarding = None;
        self.dirty = true;
        self.tab = Tab::Diagnostics;
        // Se guarda en el acto: cerrar justo después de la guía no debe volver a mostrarla.
        self.save();
    }

    fn replay_onboarding(&mut self) {
        self.config.hub.onboarding_done = false;
        self.onboarding = Some(0);
        self.dirty = true;
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

    /// Pide las notas de la última release para la pestaña Novedades.
    fn start_notes(&mut self) {
        if self.notes.is_some() || self.notes_check.is_some() {
            return;
        }

        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(error) => {
                self.notes_error = Some(format!("no se pudo localizar el binario: {error}"));
                return;
            }
        };

        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = match Command::new(&executable).arg("--release-notes").output() {
                Ok(output) if output.status.success() => {
                    parse_release_notes(&String::from_utf8_lossy(&output.stdout))
                        .ok_or_else(|| "la release no trae notas legibles".to_string())
                }
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    Err(if error.is_empty() {
                        "no se pudieron leer las notas de la release".to_string()
                    } else {
                        error
                    })
                }
                Err(error) => Err(format!("no se pudo consultar la release: {error}")),
            };
            let _ = sender.send(result);
        });

        self.notes_error = None;
        self.notes_check = Some(receiver);
    }

    fn retry_notes(&mut self) {
        self.notes = None;
        self.notes_error = None;
        self.start_notes();
    }

    fn poll_notes(&mut self) {
        let Some(receiver) = &self.notes_check else {
            return;
        };

        match receiver.try_recv() {
            Ok(Ok(info)) => {
                self.notes = Some(info);
                self.notes_error = None;
                self.notes_check = None;
            }
            Ok(Err(error)) => {
                self.notes_error = Some(error);
                self.notes_check = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.notes_error =
                    Some("la consulta de novedades terminó sin respuesta".to_string());
                self.notes_check = None;
            }
        }
    }

    /// Pregunta a la última release si hay versión nueva. No descarga nada.
    fn start_check(&mut self, manual: bool) {
        if self.check.is_some() {
            return;
        }

        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(error) => {
                if manual {
                    self.update_note =
                        Some((false, format!("no se pudo localizar el binario: {error}")));
                }
                return;
            }
        };

        if manual {
            self.update_note = None;
        }
        self.check_manual = manual;

        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let available = Command::new(&executable)
                .arg("--check-update")
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| parse_available(&String::from_utf8_lossy(&output.stdout)));
            let _ = sender.send(available);
        });

        self.check = Some(receiver);
    }

    fn poll_check(&mut self) {
        let Some(receiver) = &self.check else {
            return;
        };

        match receiver.try_recv() {
            Ok(available) => {
                if let Some(version) = &available {
                    self.available = Some(version.clone());
                }

                if self.check_manual {
                    self.update_note = Some(match &available {
                        Some(version) => (true, format!("versión {version} disponible")),
                        None => (true, "estás al día".to_string()),
                    });
                }

                self.check_manual = false;
                self.check = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                if self.check_manual {
                    self.update_note = Some((false, "no se pudo comprobar la versión".to_string()));
                }
                self.check_manual = false;
                self.check = None;
            }
        }
    }

    /// Relanza la instalación ya actualizada: el AppImage si lo hay, o este binario.
    fn restart(&mut self, ctx: &egui::Context) {
        let executable = std::env::var_os("APPIMAGE")
            .map(PathBuf::from)
            .or_else(|| std::env::current_exe().ok());

        let Some(executable) = executable else {
            self.update_note = Some((
                false,
                "no se pudo localizar la instalación para reiniciar".to_string(),
            ));
            return;
        };

        match Command::new(executable).spawn() {
            Ok(_) => {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(error) => {
                self.update_note = Some((false, format!("no se pudo reiniciar: {error}")));
            }
        }
    }

    fn poll_update(&mut self) {
        let Some(receiver) = &self.update else {
            return;
        };

        match receiver.try_recv() {
            Ok((ok, message)) => {
                if ok && message.contains("actualizado") {
                    self.available = None;
                    self.restart_ready = true;
                    self.update_note = Some((true, "actualización instalada".to_string()));
                } else {
                    self.update_note = Some((ok, message));
                }
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

    fn news(&mut self, ui: &mut egui::Ui) {
        Self::page_header(
            ui,
            "Novedades",
            "Qué cambió en la última release y qué versión tienes instalada.",
        );

        if self.notes.is_none() && self.notes_error.is_none() && self.notes_check.is_none() {
            self.start_notes();
        }

        let installed = VERSION.to_string();
        let available = self.available.clone();
        let notes_version = self.notes.as_ref().map(|notes| notes.version.clone());

        card(ui, Some("VERSIÓN"), |ui| {
            egui::Grid::new("novedades-version")
                .num_columns(2)
                .spacing([24.0, 7.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Instalada").color(self.palette.muted));
                    ui.label(egui::RichText::new(format!("v{installed}")).strong());
                    ui.end_row();

                    ui.label(egui::RichText::new("Última publicada").color(self.palette.muted));
                    ui.label(
                        egui::RichText::new(match &notes_version {
                            Some(version) => format!("v{version}"),
                            None => "consultando…".to_string(),
                        })
                        .strong(),
                    );
                    ui.end_row();
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if let Some(version) = &available
                    && ui
                        .button(format!("Instalar v{version}"))
                        .on_hover_text("Descarga y reemplaza esta instalación")
                        .clicked()
                {
                    self.start_update();
                }

                if ui
                    .button("Buscar actualización")
                    .on_hover_text("Consulta la última release sin descargar nada")
                    .clicked()
                {
                    self.start_check(true);
                }
            });

            if let Some((ok, message)) = &self.update_note {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(message).small().color(if *ok {
                    self.palette.good
                } else {
                    self.palette.bad
                }));
            }
        });

        if self.notes_check.is_some() {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new("leyendo las notas…").color(self.palette.muted));
            });
        }

        if let Some(error) = &self.notes_error {
            alert(
                ui,
                self.palette.bad,
                "No se pudieron leer las novedades",
                &format!("{error}. ¿Hay conexión con GitHub?"),
            );
            if ui.button("Reintentar").clicked() {
                self.retry_notes();
            }
        }

        if let Some(notes) = self.notes.clone() {
            card(ui, Some("ÚLTIMA RELEASE"), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(&notes.tag)
                            .size(20.0)
                            .strong()
                            .color(self.palette.accent),
                    );
                    if !notes.published.is_empty() {
                        ui.label(
                            egui::RichText::new(release_date(&notes.published))
                                .small()
                                .color(self.palette.muted),
                        );
                    }
                    ui.hyperlink_to(
                        "Ver en GitHub",
                        format!(
                            "https://github.com/SyberFoxx/mh-overlay/releases/tag/{}",
                            notes.tag
                        ),
                    );
                });
                ui.add_space(10.0);
                release_body(ui, &notes.body);
            });
        }

        if let Some(version) = &available {
            alert(
                ui,
                self.palette.accent,
                &format!("La versión v{version} está disponible"),
                "Ábrela desde el botón de arriba o desde la barra lateral; la configuración se conserva.",
            );
        }
    }

    /// Informe de una pieza con lo que hace falta para reproducir un problema.
    fn diagnostic_report(&self) -> String {
        let mut report = format!("mh-overlay {VERSION}\n");
        report.push_str(&format!(
            "juego: {} ({}) · detección automática {}\n",
            display_game(self.game()),
            self.game().module(),
            if self.config.game.auto_detect {
                "sí"
            } else {
                "no"
            },
        ));
        report.push_str(&format!(
            "registro: {}\n",
            log::path()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "sin ruta".to_string())
        ));
        report.push_str(&format!(
            "interfaz: tema {} · fondo {}\n",
            self.config.hub.theme,
            match self.config.hub.backdrop {
                Backdrop::Ambient => "ambiente",
                Backdrop::Aurora => "aurora",
                Backdrop::Ember => "brasa",
                Backdrop::Mist => "niebla",
                Backdrop::None => "ninguno",
            }
        ));

        match (self.status.pid, self.status.base) {
            (Some(pid), Some(base)) => {
                report.push_str(&format!("proceso: pid {pid} · base 0x{base:016X}\n"));
            }
            _ => report.push_str("proceso: no encontrado\n"),
        }

        report.push_str(&format!(
            "mapa: {} · {}\n",
            self.status.map.as_deref().unwrap_or("ninguno"),
            validation_label(self.status.validation)
        ));

        report.push_str(&format!(
            "lectura: {} · {} · monstruos {} · jugador {} · cacería {}\n",
            self.status
                .checked_at
                .map(|at| seconds_ago(at.elapsed()))
                .unwrap_or_else(|| "—".to_string()),
            self.status
                .read_ms
                .map(|ms| format!("{ms} ms"))
                .unwrap_or_else(|| "—".to_string()),
            self.status.snapshot.monsters.len(),
            if self.status.snapshot.player.is_some() {
                "sí"
            } else {
                "no"
            },
            if self.status.snapshot.hunt.is_some() {
                "sí"
            } else {
                "no"
            },
        ));

        if let Some(error) = &self.status.error {
            report.push_str(&format!("error: {error}\n"));
        }

        if let Some(seen) = &self.seen {
            report.push_str("procesos vistos:\n");

            if seen.is_empty() {
                report.push_str("  ninguno menciona el ejecutable\n");
            }

            for process in seen {
                report.push_str(&format!("  pid {} · {}", process.pid, process.comm));
                if let Some((module, base)) = &process.mapping {
                    report.push_str(&format!(" · mapea {module} en 0x{base:X}"));
                }
                report.push('\n');

                if !process.cmdline.is_empty() {
                    report.push_str(&format!("    {}\n", process.cmdline));
                }
            }
        }

        report
    }

    /// Lo mismo que `mh-overlay --ps`, sin pedir una terminal.
    fn scan_processes(&mut self) {
        let names = GameKind::ALL.map(GameKind::module);
        self.seen = Some(lookalikes(&names));
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
                        ui.label(
                            egui::RichText::new(title)
                                .size(20.0)
                                .strong()
                                .color(self.palette.text),
                        );
                        ui.label(
                            egui::RichText::new(detail)
                                .size(12.5)
                                .color(self.palette.muted),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Copiar diagnóstico").clicked() {
                            ui.ctx().copy_text(self.diagnostic_report());
                            self.copied_at = Some(Instant::now());
                        }
                        if self
                            .copied_at
                            .is_some_and(|at| at.elapsed() < Duration::from_secs(3))
                        {
                            ui.label(
                                egui::RichText::new("copiado")
                                    .small()
                                    .color(self.palette.good),
                            );
                        }
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
                    ui.label(egui::RichText::new("Juego").color(self.palette.muted));
                    ui.label(format!(
                        "{} · {}",
                        display_game(self.game()),
                        self.game().module()
                    ));
                    ui.end_row();

                    ui.label(egui::RichText::new("Proceso").color(self.palette.muted));
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

                    ui.label(egui::RichText::new("Verificación").color(self.palette.muted));
                    let (text, color) = match self.status.validation {
                        Some(MapValidation::Verified) => ("CRC verificado", self.palette.good),
                        Some(MapValidation::Mismatch) => {
                            ("no coincide con esta build", self.palette.bad)
                        }
                        Some(MapValidation::Unavailable) => {
                            ("sin CRC que comparar", self.palette.muted)
                        }
                        None => ("—", self.palette.muted),
                    };
                    ui.colored_label(color, text);
                    ui.end_row();
                });
        });

        card(ui, Some("PROCESOS"), |ui| {
            ui.label(
                egui::RichText::new(
                    "Lo mismo que mira la detección: nombre del proceso, línea de comandos y si \
                     tiene mapeado el ejecutable del juego.",
                )
                .small()
                .color(self.palette.muted),
            );
            ui.add_space(4.0);

            if ui.button("Buscar procesos").clicked() {
                self.scan_processes();
            }

            let Some(seen) = &self.seen else {
                return;
            };

            ui.add_space(4.0);

            if seen.is_empty() {
                ui.label(
                    egui::RichText::new("ninguno menciona el ejecutable del juego")
                        .small()
                        .color(self.palette.muted),
                );
            }

            for process in seen {
                let state = match &process.mapping {
                    Some((module, base)) => format!("mapea {module} en 0x{base:X}"),
                    None => "sin el ejecutable mapeado".to_string(),
                };

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("pid {}", process.pid))
                            .monospace()
                            .color(self.palette.muted),
                    );
                    ui.label(egui::RichText::new(&process.comm).strong());
                    ui.label(egui::RichText::new(state).small().color(self.palette.muted));
                });
            }
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
                    self.palette.accent,
                );
                stat_tile(ui, "Jugador", &player, self.palette.good);
                stat_tile(ui, "Cacería", &hunt, self.palette.text);
                stat_tile(ui, "Última lectura", &last, self.palette.muted);
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
                    .color(self.palette.muted),
                );
            }
            if let Some(hunt) = &self.status.snapshot.hunt {
                ui.label(
                    egui::RichText::new(format!(
                        "vidas del equipo {}/{}",
                        hunt.deaths, hunt.max_deaths
                    ))
                    .small()
                    .color(self.palette.muted),
                );
            }
        });

        if let Some(error) = &self.status.error {
            alert(ui, self.palette.bad, "La lectura falló", error);
        }

        if self.status.validation == Some(MapValidation::Mismatch) {
            alert(
                ui,
                self.palette.accent,
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
                        .color(self.palette.accent),
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
                                .color(self.palette.muted),
                        );
                    });
                }

                if let Some(note) = &self.access_note {
                    ui.label(egui::RichText::new(note).small().color(
                        if note.contains("concedido") {
                            self.palette.good
                        } else {
                            self.palette.bad
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
                .fill(self.palette.surface)
                .stroke(egui::Stroke::new(1.0, self.palette.border))
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
                        .color(self.palette.muted),
                    );
                });
            return;
        };

        egui::Frame::new()
            .fill(self.palette.surface)
            .stroke(egui::Stroke::new(1.0, self.palette.border))
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
                                .color(self.palette.text),
                        );
                        ui.label(
                            egui::RichText::new(if weapon.kind.has_sharpness() {
                                "Cuerpo a cuerpo"
                            } else {
                                "A distancia"
                            })
                            .size(12.5)
                            .color(self.palette.muted),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(sharpness) = &weapon.sharpness {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Filo")
                                        .small()
                                        .color(self.palette.muted),
                                );
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
                    .color(self.palette.muted),
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
                            ui.label(egui::RichText::new(label).color(self.palette.muted));
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
                        stat_tile(ui, "Ataque", &format!("{attack:.0}"), self.palette.accent);
                    }
                    if let Some(affinity) = detail.affinity {
                        stat_tile(ui, "Afinidad", &format!("{affinity:+}%"), self.palette.good);
                    }
                    if let Some(element) = detail.element {
                        stat_tile(ui, "Elemento", &format!("{element:.0}"), self.palette.text);
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
                        ui.label(
                            egui::RichText::new("Tipo")
                                .small()
                                .color(self.palette.muted),
                        );
                        ui.label(
                            egui::RichText::new("Cargada")
                                .small()
                                .color(self.palette.muted),
                        );
                        ui.label(
                            egui::RichText::new("Reserva")
                                .small()
                                .color(self.palette.muted),
                        );
                        ui.label(
                            egui::RichText::new("Máximo")
                                .small()
                                .color(self.palette.muted),
                        );
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
                            ui.label(egui::RichText::new(label).color(self.palette.muted));
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
                                ui.label(
                                    egui::RichText::new("ID").small().color(self.palette.muted),
                                );
                                ui.label(
                                    egui::RichText::new("Notas")
                                        .small()
                                        .color(self.palette.muted),
                                );
                                ui.label(
                                    egui::RichText::new("Longitud")
                                        .small()
                                        .color(self.palette.muted),
                                );
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
                .color(self.palette.muted),
            );
        }

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "En World, munición, proyectiles y melodías salen de los offsets de HunterPie \
                 legacy. En Rise solo se muestran los campos que HunterPie v2 lee.",
            )
            .small()
            .color(self.palette.muted),
        );
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        let before = self.config.clone();
        let snapshot = if self.status.snapshot.player.is_some()
            || self.status.snapshot.hunt.is_some()
            || !self.status.snapshot.monsters.is_empty()
        {
            self.status.snapshot.clone()
        } else {
            sample()
        };

        Self::page_header(
            ui,
            "Ajustes",
            "Se guarda solo: el panel abierto aplica los cambios en el siguiente refresco.",
        );

        card(ui, Some("JUEGO"), |ui| {
            ui.label(egui::RichText::new("Juego").color(self.palette.muted));
            ui.add_space(5.0);
            // Dos opciones del mismo ancho: con la tarjeta estrecha, dos etiquetas seguidas se
            // salían del borde y «Monster Hunter World» aparecía cortado a media palabra.
            ui.spacing_mut().item_spacing.x = 6.0;
            let option = ((ui.available_width() - 6.0) / 2.0).max(90.0);
            ui.horizontal(|ui| {
                for kind in [GameKind::Rise, GameKind::World] {
                    if ui
                        .add_sized(
                            [option, 30.0],
                            egui::Button::new(display_game(kind)).selected(self.game() == kind),
                        )
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
                ui.label(egui::RichText::new("Refresco").color(self.palette.muted));
                ui.add(
                    egui::DragValue::new(&mut self.config.game.interval_ms)
                        .range(100..=5000)
                        .suffix(" ms"),
                );
            });
        });

        card(ui, Some("INTERFAZ"), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Tema").color(self.palette.muted));

                for (theme, label) in [
                    (Theme::Auto, "Automático"),
                    (Theme::Dark, "Oscuro"),
                    (Theme::Light, "Claro"),
                ] {
                    if ui
                        .selectable_label(self.config.hub.theme == theme, label)
                        .clicked()
                    {
                        self.config.hub.theme = theme;
                    }
                }

                ui.label(
                    egui::RichText::new(if self.applied_dark { "oscuro" } else { "claro" })
                        .small()
                        .color(self.palette.muted),
                )
                .on_hover_text("Lo que está aplicado ahora; con «Automático» manda el escritorio");
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Fondo").color(self.palette.muted));

                for backdrop in Backdrop::ALL {
                    if ui
                        .selectable_label(self.config.hub.backdrop == backdrop, backdrop.label())
                        .clicked()
                    {
                        self.config.hub.backdrop = backdrop;
                    }
                }

                if self.config.hub.backdrop != Backdrop::None {
                    ui.add(
                        egui::Slider::new(&mut self.config.hub.backdrop_opacity, 0.0..=1.0)
                            .fixed_decimals(2),
                    )
                    .on_hover_text("Intensidad del degradado; 0 lo apaga sin cambiar de modo");
                }
            });

            ui.add(
                egui::Label::new(
                    egui::RichText::new(
                        "El fondo se genera al dibujar: no es una imagen, así que acompaña al tema \
                         y no pesa nada en el paquete.",
                    )
                    .small()
                    .color(self.palette.muted),
                )
                .wrap(),
            );
        });

        card(ui, Some("PANEL"), |ui| {
            egui::Grid::new("panel")
                .num_columns(2)
                .spacing([24.0, 10.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Disposición").color(self.palette.muted));
                    egui::ComboBox::from_id_salt("layout")
                        .width(220.0)
                        .selected_text(match self.config.overlay.layout {
                            Layout::Panel => "Panel único",
                            Layout::Free => "Barras sueltas",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.config.overlay.layout,
                                Layout::Panel,
                                "Panel único",
                            );
                            ui.selectable_value(
                                &mut self.config.overlay.layout,
                                Layout::Free,
                                "Barras sueltas",
                            )
                            .on_hover_text(
                                "Una placa por barra. Las posiciones se editan en \
                                 [[overlay.bars]] del TOML; monster.0 mueve vida y partes juntas.",
                            );
                        });
                    ui.end_row();

                    ui.label(egui::RichText::new("Anclaje").color(self.palette.muted));
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

                    ui.label(egui::RichText::new("Márgenes").color(self.palette.muted));
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

                    ui.label(egui::RichText::new("Escala").color(self.palette.muted));
                    ui.add(
                        egui::Slider::new(&mut self.config.overlay.scale, 0.5..=3.0)
                            .fixed_decimals(2)
                            .suffix("×"),
                    );
                    ui.end_row();

                    ui.label(egui::RichText::new("Monstruos").color(self.palette.muted));
                    egui::ComboBox::from_id_salt("monstruos")
                        .width(220.0)
                        .selected_text(self.config.overlay.monsters.label())
                        .show_ui(ui, |ui| {
                            for mode in Monsters::ALL {
                                ui.selectable_value(
                                    &mut self.config.overlay.monsters,
                                    mode,
                                    mode.label(),
                                )
                                .on_hover_text(mode.detail());
                            }
                        });
                    ui.end_row();

                    ui.label(egui::RichText::new("Monstruos a la vez").color(self.palette.muted));
                    ui.add(egui::Slider::new(
                        &mut self.config.overlay.max_monsters,
                        1..=8,
                    ));
                    ui.end_row();
                });

            ui.add_space(4.0);
            if ui.available_width() < 520.0 {
                ui.checkbox(&mut self.config.overlay.show_parts, "Mostrar partes");
                ui.checkbox(
                    &mut self.config.overlay.show_part_health,
                    "Vida de las partes en números",
                );
                ui.checkbox(&mut self.config.overlay.show_ailments, "Mostrar estados");
                ui.checkbox(
                    &mut self.config.overlay.show_health_numbers,
                    "Vida del monstruo en números",
                );
                ui.checkbox(
                    &mut self.config.overlay.click_through,
                    "Los clics atraviesan el panel",
                );
                ui.checkbox(
                    &mut self.config.overlay.hide_on_empty,
                    "Ocultar el panel sin monstruos",
                );
            } else {
                ui.columns(2, |columns| {
                    columns[0].checkbox(&mut self.config.overlay.show_parts, "Mostrar partes");
                    columns[0].checkbox(
                        &mut self.config.overlay.show_part_health,
                        "Vida de las partes en números",
                    );
                    columns[0].checkbox(&mut self.config.overlay.show_ailments, "Mostrar estados");
                    columns[1].checkbox(
                        &mut self.config.overlay.show_health_numbers,
                        "Vida del monstruo en números",
                    );
                    columns[1].checkbox(
                        &mut self.config.overlay.click_through,
                        "Los clics atraviesan el panel",
                    );
                    columns[1].checkbox(
                        &mut self.config.overlay.hide_on_empty,
                        "Ocultar el panel sin monstruos",
                    );
                });
            }
        });

        if self.config.overlay.layout == Layout::Free {
            card(ui, Some("EDITOR VISUAL"), |ui| {
                ui.label(
                    egui::RichText::new(
                        "Arrastra una placa. Se imanta a la rejilla de 10 px, al centro y a los \
                         bordes de la pantalla; monster.N mueve vida, partes y estados juntos.",
                    )
                    .small()
                    .color(self.palette.muted),
                );
                ui.add_space(6.0);

                let detected = self.detected_screen.unwrap_or((1920, 1080));
                let screen = self.editor_screen.unwrap_or(detected);
                let mut width = screen.0;
                let mut height = screen.1;

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Lienzo").color(self.palette.muted));
                    ui.add(
                        egui::DragValue::new(&mut width)
                            .range(640..=16_384)
                            .speed(4)
                            .prefix("x "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut height)
                            .range(360..=16_384)
                            .speed(4)
                            .prefix("y "),
                    );

                    if self.editor_screen.is_some() && ui.small_button("Detectada").clicked() {
                        self.editor_screen = None;
                    }
                });

                ui.label(
                    egui::RichText::new(match self.detected_screen {
                        Some((width, height)) => format!(
                            "Monitor detectado: {width}×{height}. Si no coincide, ajusta el lienzo."
                        ),
                        None => {
                            "No se pudo detectar el monitor; ajusta el lienzo a mano.".to_string()
                        }
                    })
                    .small()
                    .color(self.palette.muted),
                );
                ui.add_space(6.0);

                if (width, height) != screen {
                    self.editor_screen = Some((width, height));
                }

                self.visual_editor(ui, &snapshot, (width, height));
            });

            card(ui, Some("BARRAS SUELTAS"), |ui| {
                ui.label(
                    egui::RichText::new(
                        "Mueve la vida del monstruo y sus partes juntas. \"down\" las pone debajo; \
                         \"right\", a la derecha.",
                    )
                    .small()
                    .color(self.palette.muted),
                );
                ui.add_space(6.0);

                let margin_x = self.config.overlay.margin_x;
                let margin_y = self.config.overlay.margin_y;
                let mut remove = None;

                for (index, id) in FREE_BARS.iter().enumerate() {
                    let position = self
                        .config
                        .overlay
                        .bars
                        .iter()
                        .position(|bar| bar.id == *id);

                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(*id)
                                .monospace()
                                .color(self.palette.text),
                        );

                        if let Some(position) = position {
                            let bar = &mut self.config.overlay.bars[position];
                            ui.add(
                                egui::DragValue::new(&mut bar.x)
                                    .range(0..=8000)
                                    .prefix("x "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut bar.y)
                                    .range(0..=8000)
                                    .prefix("y "),
                            );

                            if id.starts_with("monster.") {
                                egui::ComboBox::from_id_salt(format!("dir-{id}"))
                                    .width(86.0)
                                    .selected_text(match bar.direction {
                                        Direction::Down => "abajo",
                                        Direction::Right => "derecha",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut bar.direction,
                                            Direction::Down,
                                            "abajo",
                                        );
                                        ui.selectable_value(
                                            &mut bar.direction,
                                            Direction::Right,
                                            "derecha",
                                        );
                                    });
                            }

                            if ui.small_button("Quitar").clicked() {
                                remove = Some(position);
                            }
                        } else if ui.small_button("Colocar").clicked() {
                            self.config.overlay.bars.push(BarPlacement {
                                id: (*id).to_string(),
                                x: margin_x,
                                y: margin_y + index as i32 * 40,
                                direction: Direction::Down,
                            });
                        }
                    });
                }

                if let Some(position) = remove {
                    self.config.overlay.bars.remove(position);
                }
            });

            card(ui, Some("PRESETS"), |ui| {
                let mut presets = builtin_presets();
                let builtin = presets.len();
                presets.extend(self.config.overlay.presets.clone());
                let mut selected = self
                    .preset_selected
                    .unwrap_or(0)
                    .min(presets.len().saturating_sub(1));

                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_id_salt("preset")
                        .width(220.0)
                        .selected_text(presets[selected].name.as_str())
                        .show_ui(ui, |ui| {
                            for (index, preset) in presets.iter().enumerate() {
                                ui.selectable_value(&mut selected, index, preset.name.as_str());
                            }
                        });

                    if ui.button("Aplicar").clicked() {
                        self.config.overlay.layout = Layout::Free;
                        self.config.overlay.bars = presets[selected].bars.clone();
                    }

                    if selected >= builtin && ui.button("Eliminar").clicked() {
                        self.config.overlay.presets.remove(selected - builtin);
                        selected = 0;
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.preset_name)
                            .hint_text("nombre del preset")
                            .desired_width(180.0),
                    );

                    if ui.button("Guardar actual").clicked() {
                        let name = self.preset_name.trim().to_string();

                        if !name.is_empty() {
                            let preset = LayoutPreset {
                                name: name.clone(),
                                bars: self.config.overlay.bars.clone(),
                            };

                            if let Some(existing) = self
                                .config
                                .overlay
                                .presets
                                .iter_mut()
                                .find(|preset| preset.name == name)
                            {
                                *existing = preset;
                            } else {
                                self.config.overlay.presets.push(preset);
                            }

                            self.preset_name.clear();
                        }
                    }
                });

                self.preset_selected = Some(selected);
            });
        }

        card(ui, Some("ESTILO"), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Preset").color(self.palette.muted));
                for preset in StylePreset::ALL {
                    if ui
                        .button(preset.label())
                        .on_hover_text("Reemplaza los colores de abajo; no toca el umbral")
                        .clicked()
                    {
                        self.config.style = preset.style();
                    }
                }
            });

            ui.add_space(8.0);
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
                ui.label(egui::RichText::new("Vida baja por debajo de").color(self.palette.muted));
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

        card(ui, Some("GUÍA"), |ui| {
            ui.label(
                egui::RichText::new(
                    "La guía inicial explica el juego, el permiso de memoria y cómo abrir el \
                     panel. Puedes volver a verla cuando quieras.",
                )
                .small()
                .color(self.palette.muted),
            );
            ui.add_space(4.0);
            if ui.button("Ver la guía otra vez").clicked() {
                self.replay_onboarding();
            }
        });

        if self.config != before {
            self.dirty = true;
            self.preview_stale = true;
        }

        // En una ventana estrecha la vista previa va al final y con un ancho contenido: arriba
        // empujaba todos los ajustes fuera de la vista, que es justo lo que se viene a hacer aquí.
        if packed_preview(ui.ctx().content_rect().width()) {
            ui.add_space(6.0);
            ui.scope(|ui| {
                ui.set_max_width(560.0);
                self.preview(ui);
            });
            ui.add_space(12.0);
        }

        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| match (&self.save_error, self.last_saved) {
            (Some(error), _) => {
                ui.colored_label(self.palette.bad, error);
            }
            (None, Some(at)) => {
                ui.label(
                    egui::RichText::new(format!(
                        "guardado hace {} en {}",
                        seconds_ago(at.elapsed()),
                        self.config_path.display()
                    ))
                    .small()
                    .color(self.palette.muted),
                );
            }
            (None, None) => {
                ui.label(
                    egui::RichText::new("los cambios se guardan solos")
                        .small()
                        .color(self.palette.muted),
                );
            }
        });
    }

    fn visual_editor(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot, screen: (u32, u32)) {
        let screen_w = screen.0.max(320) as i32;
        let screen_h = screen.1.max(200) as i32;
        let width = ui.available_width().clamp(280.0, 920.0);
        let scale = width / screen_w as f32;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(screen_w as f32 * scale, screen_h as f32 * scale),
            egui::Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);
        let to_screen = |x: i32, y: i32| rect.min + egui::vec2(x as f32 * scale, y as f32 * scale);

        painter.rect_filled(rect, 6.0, self.palette.bg);
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, self.palette.border),
            egui::StrokeKind::Inside,
        );

        let step = (screen_w / 16).max(80) as usize;

        for x in (0..=screen_w).step_by(step) {
            let top = to_screen(x, 0);
            let bottom = to_screen(x, screen_h);
            painter.line_segment(
                [top, bottom],
                egui::Stroke::new(1.0, self.palette.border.gamma_multiply(0.7)),
            );
        }

        for y in (0..=screen_h).step_by(step) {
            let left = to_screen(0, y);
            let right = to_screen(screen_w, y);
            painter.line_segment(
                [left, right],
                egui::Stroke::new(1.0, self.palette.border.gamma_multiply(0.7)),
            );
        }

        // Mientras se arrastra, los rectángulos solo cambian de posición. Reutilizarlos evita
        // volver a maquetar todas las placas en cada frame; al soltar se recalculan. Cuando no
        // se arrastra, la caché solo se invalida si cambia la configuración o la lectura.
        let cache_matches = !self.editor_bounds.is_empty()
            && self.editor_bounds_config == self.config
            && self.editor_bounds_at == self.status.checked_at
            && self.editor_bounds_screen == Some(screen);
        let bars = if (self.drag_bar.is_some() || cache_matches) && !self.editor_bounds.is_empty() {
            self.editor_bounds.clone()
        } else {
            let bars = view::free_bounds(&self.config, &self.text, snapshot, None);
            self.editor_bounds = bars.clone();
            self.editor_bounds_config = self.config.clone();
            self.editor_bounds_at = self.status.checked_at;
            self.editor_bounds_screen = Some(screen);
            bars
        };

        for bar in &bars {
            let min = to_screen(bar.bounds.x, bar.bounds.y);
            let max = to_screen(
                bar.bounds.x + bar.bounds.width as i32,
                bar.bounds.y + bar.bounds.height as i32,
            );
            let plate = egui::Rect::from_min_max(min, max);
            let selected = self.drag_bar.as_deref() == Some(bar.id.as_str());
            let color = if selected {
                self.palette.accent
            } else {
                self.palette.text
            };

            painter.rect_filled(plate, 4.0, color.gamma_multiply(0.18));
            painter.rect_stroke(
                plate,
                4.0,
                egui::Stroke::new(if selected { 1.5 } else { 1.0 }, color),
                egui::StrokeKind::Inside,
            );
            painter.text(
                plate.min + egui::vec2(4.0, 3.0),
                egui::Align2::LEFT_TOP,
                &bar.id,
                egui::FontId::proportional(10.0),
                color,
            );
        }

        let pointer = response.interact_pointer_pos();

        if response.drag_started()
            && let Some(position) = pointer
        {
            self.drag_bar = bars
                .iter()
                .rev()
                .find(|bar| {
                    let min = to_screen(bar.bounds.x, bar.bounds.y);
                    let max = to_screen(
                        bar.bounds.x + bar.bounds.width as i32,
                        bar.bounds.y + bar.bounds.height as i32,
                    );
                    egui::Rect::from_min_max(min, max).contains(position)
                })
                .map(|bar| bar.id.clone());

            self.drag_offset = self
                .drag_bar
                .as_ref()
                .and_then(|id| {
                    bars.iter()
                        .find(|bar| &bar.id == id)
                        .map(|bar| (position - to_screen(bar.bounds.x, bar.bounds.y)) / scale)
                })
                .unwrap_or_default();
        }

        if let (Some(id), Some(position), true) = (&self.drag_bar, pointer, response.dragged()) {
            let bounds = bars.iter().find(|bar| &bar.id == id).map(|bar| bar.bounds);

            if let Some(bounds) = bounds {
                let raw_x = (position.x - rect.min.x) / scale - self.drag_offset.x;
                let raw_y = (position.y - rect.min.y) / scale - self.drag_offset.y;
                let x = snap_axis(raw_x, bounds.width, screen_w);
                let y = snap_axis(raw_y, bounds.height, screen_h);

                if let Some(bar) = self
                    .config
                    .overlay
                    .bars
                    .iter_mut()
                    .find(|bar| &bar.id == id)
                {
                    bar.x = x;
                    bar.y = y;
                } else {
                    self.config.overlay.bars.push(BarPlacement {
                        id: id.clone(),
                        x,
                        y,
                        direction: Direction::Down,
                    });
                }

                self.snap_guide = Some((x, y));

                if let Some(cached) = self.editor_bounds.iter_mut().find(|bar| &bar.id == id) {
                    cached.bounds.x = x;
                    cached.bounds.y = y;
                }
            }
        }

        if response.drag_stopped() {
            self.drag_bar = None;
            self.snap_guide = None;
        }

        if let Some((x, y)) = self.snap_guide {
            let top = to_screen(x, 0);
            let bottom = to_screen(x, screen_h);
            let left = to_screen(0, y);
            let right = to_screen(screen_w, y);

            painter.line_segment([top, bottom], egui::Stroke::new(1.0, self.palette.accent));
            painter.line_segment([left, right], egui::Stroke::new(1.0, self.palette.accent));
        }

        if self.drag_bar.is_some() {
            ui.ctx().request_repaint();
        }
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
            .color(self.palette.muted),
        );
        if self.drag_bar.is_some() {
            ui.label(
                egui::RichText::new("Suelta para actualizar la vista previa")
                    .small()
                    .color(self.palette.accent),
            );
        }
        ui.add_space(10.0);

        let Some(texture) = &self.preview else {
            ui.spinner();
            return;
        };

        let available = (ui.available_width() - 24.0).max(80.0);
        let scale = (available / self.preview_size.x).clamp(0.2, 1.0);
        egui::Frame::new()
            .fill(self.palette.surface_2)
            .stroke(egui::Stroke::new(1.0, self.palette.border))
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
                self.hidden = false;
                // Lo que se enseñe al volver tiene que ser de ahora, no de cuando se escondió.
                self.last_refresh = None;
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

        self.shortcuts(ctx);

        // El tamaño se guarda al cerrar, no mientras se arrastra el borde: es lo último que el
        // usuario eligió y no hay razón para escribir el TOML en cada frame.
        if self.closing || ctx.input(|input| input.viewport().close_requested()) {
            self.remember_geometry(ctx);
        }

        // Cerrar solo oculta la ventana si hay un panel vivo que conservar. Sin panel no hay
        // nada que mantener en segundo plano: cerrar termina, como espera Super+Q.
        // `Minimized` no existe en Wayland; para esconder de verdad se oculta la superficie.
        if !self.closing
            && ctx.input(|input| input.viewport().close_requested())
            && close_hides_window(self.tray.is_some(), self.overlay_running())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.hidden = true;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.sync_theme(&ctx);

        // El overlay cuenta píxeles físicos; egui da el monitor en puntos lógicos.
        let monitor = ctx.input(|input| {
            let viewport = input.viewport();
            viewport
                .monitor_size
                .map(|size| (size, viewport.native_pixels_per_point.unwrap_or(1.0)))
        });

        if let Some((size, native)) = monitor {
            let width = (size.x * native).round() as u32;
            let height = (size.y * native).round() as u32;

            if (640..=16_384).contains(&width) && (360..=16_384).contains(&height) {
                self.detected_screen = Some((width, height));
            }
        }

        self.poll_update();
        self.poll_check();
        self.poll_notes();
        self.poll_access();

        // Escondido en la bandeja no se pinta nada: la lectura y el repintado quedarían para
        // nadie. Se despierta una vez por segundo solo para recoger el panel si se cierra solo,
        // y al volver se fuerza una lectura nueva.
        if !self.hidden
            && self.last_refresh.is_none_or(|last| {
                last.elapsed() >= Duration::from_millis(self.config.game.interval_ms)
            })
        {
            self.refresh();
        }

        // Guardar cuando el usuario suelta el control: arrastrar un deslizador no debe escribir
        // el archivo decenas de veces por segundo.
        if self.dirty && !ctx.input(|input| input.pointer.any_down()) {
            self.save();
        }

        // Arrastrar una placa o un control no debe rehacer el lienzo completo en cada frame:
        // la vista previa se actualiza al soltar, y los cambios continuos de un control se
        // agrupan para no competir con el editor.
        let pointer_down = ctx.input(|input| input.pointer.any_down());
        let preview_due = self
            .preview_built_at
            .is_none_or(|at| at.elapsed() >= Duration::from_millis(120));

        // La vista previa solo se dibuja en Ajustes y con la ventana a la vista: fuera de ahí,
        // reconstruirla es un lienzo de 1920×1080 tirado a la basura (medido: 12 ms el dibujo)
        // cada refresco. Al volver, `preview_stale` sigue en pie y se rehace antes de pintarla.
        if self.preview_stale
            && !self.hidden
            && self.tab == Tab::Settings
            && !pointer_down
            && preview_due
        {
            self.rebuild_preview(&ctx);
        }

        egui::Panel::left("navegacion")
            .exact_size(216.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(self.palette.surface)
                    .inner_margin(egui::Margin::symmetric(14, 14)),
            )
            .show(ui, |ui| self.sidebar(ui));

        if self.tab == Tab::Settings && !packed_preview(ctx.content_rect().width()) {
            egui::Panel::right("preview")
                .resizable(true)
                .default_size(360.0)
                .min_size(240.0)
                .max_size(420.0)
                .show_separator_line(false)
                .frame(
                    egui::Frame::new()
                        .fill(self.palette.bg)
                        .inner_margin(egui::Margin::same(20)),
                )
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.preview(ui));
                });
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.palette.bg)
                    .inner_margin(egui::Margin::same(26)),
            )
            .show(ui, |ui| {
                if self.config.hub.backdrop != Backdrop::None {
                    theme::paint_backdrop(
                        ui,
                        self.palette,
                        self.config.hub.backdrop,
                        self.config.hub.backdrop_opacity,
                    );
                }

                let tab = match self.tab {
                    Tab::News => "novedades",
                    Tab::Diagnostics => "diagnostico",
                    Tab::Weapon => "arma",
                    Tab::Settings => "ajustes",
                };
                let mut scroll = egui::ScrollArea::vertical()
                    .id_salt(tab)
                    .auto_shrink([false, false]);

                if std::mem::take(&mut self.scroll_to_top) {
                    scroll = scroll.vertical_scroll_offset(0.0);
                }

                scroll.show(ui, |ui| match self.tab {
                    Tab::News => self.news(ui),
                    Tab::Diagnostics => self.diagnostics(ui),
                    Tab::Weapon => self.weapon(ui),
                    Tab::Settings => self.settings(ui),
                });
            });

        self.onboarding(&ctx);
        ctx.request_repaint_after(if self.hidden {
            Duration::from_secs(1)
        } else {
            Duration::from_millis(250)
        });
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
    let palette = Palette::current();

    egui::Frame::new()
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(title) = title {
                ui.label(
                    egui::RichText::new(title)
                        .size(11.0)
                        .strong()
                        .color(palette.muted),
                );
                ui.add_space(8.0);
            }
            add(ui);
        });
    ui.add_space(10.0);
}

/// Tamaño fijo de los resúmenes: cuatro cifras seguidas no cambian de tamaño según el texto y
/// caben cuatro en una fila a la anchura mínima de la ventana.
const TILE_WIDTH: f32 = 144.0;
const TILE_HEIGHT: f32 = 76.0;

/// Tamaño de la cifra; una cadena larga baja un escalón en vez de salirse del recuadro.
fn stat_size(value: &str) -> f32 {
    if value.chars().count() > 8 {
        15.0
    } else {
        19.0
    }
}

/// Cifra grande con su etiqueta debajo, para los resúmenes de la pestaña de diagnóstico.
fn stat_tile(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    let palette = Palette::current();

    // Caja de tamaño exacto: el fondo se pinta primero y el texto va dentro con un rect fijo, así
    // las cuatro cifras quedan alineadas aunque una sea más larga que las demás.
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(TILE_WIDTH, TILE_HEIGHT), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(10), palette.surface_2);

    // `new_child` en vez de `scope_builder`: el rect ya está reservado arriba y volver a
    // reservarlo desde un hijo descoloca la fila entera.
    let mut tile = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(12.0, 11.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    tile.spacing_mut().item_spacing.y = 4.0;
    tile.add(
        egui::Label::new(
            egui::RichText::new(value)
                .size(stat_size(value))
                .strong()
                .color(color),
        )
        .truncate(),
    );
    tile.add(
        egui::Label::new(egui::RichText::new(label).size(11.0).color(palette.muted)).truncate(),
    );
}

/// Aviso con color propio: error de lectura, mapa desajustado, permiso pendiente.
fn alert(ui: &mut egui::Ui, color: egui::Color32, title: &str, body: &str) {
    let palette = Palette::current();
    egui::Frame::new()
        .fill(color.gamma_multiply(0.08))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.45)))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(title).strong().color(color));
            ui.label(egui::RichText::new(body).size(12.5).color(palette.muted));
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
    let palette = Palette::current();
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(5), palette.input);

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
        egui::Stroke::new(1.0, palette.border),
        egui::StrokeKind::Inside,
    );
}

/// Cerrar la ventana solo la esconde si hay bandeja y un panel que conservar; si no, termina.
fn close_hides_window(has_tray: bool, overlay_running: bool) -> bool {
    has_tray && overlay_running
}

/// Convierte `2026-11-20T14:39:43Z` en algo legible sin añadir una dependencia de fechas.
fn release_date(published: &str) -> String {
    let date = published.get(..10).unwrap_or(published);

    date.to_string()
}

/// Quita el énfasis y el código en línea de una línea de markdown.
fn clean_inline_markdown(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(character) = chars.next() {
        match character {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
            }
            '*' | '`' => {}
            other => out.push(other),
        }
    }

    out
}

/// Cuerpo de la release con un markdown mínimo: títulos, viñetas y líneas normales.
fn release_body(ui: &mut egui::Ui, body: &str) {
    let palette = Palette::current();
    if body.trim().is_empty() {
        ui.label(egui::RichText::new("La release no trae notas.").color(palette.muted));
        return;
    }

    for line in body.lines() {
        let line = line.trim();

        if line.is_empty() {
            ui.add_space(5.0);
        } else if let Some(title) = line.strip_prefix("### ") {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(title.to_uppercase())
                    .size(12.0)
                    .strong()
                    .color(palette.accent),
            );
        } else if let Some(title) = line.strip_prefix("## ").or_else(|| line.strip_prefix("# ")) {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(clean_inline_markdown(title))
                    .size(15.0)
                    .strong()
                    .color(palette.text),
            );
        } else if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            // Viñeta en su propia columna: así las líneas que envuelven quedan alineadas con el
            // texto y no con el punto, que es lo que hacía ilegible una lista larga.
            ui.horizontal_top(|ui| {
                ui.add_space(2.0);
                ui.label(egui::RichText::new("•").color(palette.accent));
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(clean_inline_markdown(item)).wrap());
                });
            });
            ui.add_space(3.0);
        } else {
            ui.label(clean_inline_markdown(line));
        }
    }
}

fn parse_release_notes(output: &str) -> Option<ReleaseInfo> {
    let (head, body) = output.split_once("\n---\n")?;
    let field = |name: &str| {
        head.lines()
            .find_map(|line| line.strip_prefix(name))
            .map(|value| value.trim().to_string())
    };

    Some(ReleaseInfo {
        tag: field("tag:")?,
        version: field("version:")?,
        published: field("published:").unwrap_or_default(),
        body: body.trim_end().to_string(),
    })
}

/// Extrae la versión de la salida de `--check-update`; `None` si ya es la última.
fn parse_available(output: &str) -> Option<String> {
    output
        .trim()
        .strip_prefix("actualización disponible: ")
        .map(|version| version.trim().to_string())
        .filter(|version| !version.is_empty())
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

/// Ajusta una coordenada a la rejilla de 10 px, al centro o a los bordes si está cerca.
fn snap_axis(value: f32, extent: u32, screen: i32) -> i32 {
    let max = (screen - extent as i32).max(0);
    let grid = (value / 10.0).round() * 10.0;
    let candidates = [0.0, max as f32 / 2.0, max as f32, grid];
    let snapped = candidates
        .into_iter()
        .min_by(|a, b| (value - a).abs().total_cmp(&(value - b).abs()))
        .unwrap_or(grid);
    let result = if (value - snapped).abs() <= 12.0 {
        snapped
    } else {
        grid
    };

    result.round().clamp(0.0, max as f32) as i32
}

/// Disposiciones que vienen con el hub; no se guardan en el TOML hasta que se aplican.
fn builtin_presets() -> Vec<LayoutPreset> {
    let bar = |id: &str, x: i32, y: i32, direction| BarPlacement {
        id: id.to_string(),
        x,
        y,
        direction,
    };

    vec![
        LayoutPreset {
            name: "Compacto".to_string(),
            bars: vec![
                bar("hunt", 24, 24, Direction::Down),
                bar("monster.0", 360, 24, Direction::Down),
                bar("weapon", 24, 610, Direction::Down),
                bar("player.health", 24, 770, Direction::Down),
                bar("player.stamina", 24, 810, Direction::Down),
                bar("buffs", 24, 860, Direction::Down),
            ],
        },
        LayoutPreset {
            name: "Monstruo a la derecha".to_string(),
            bars: vec![
                bar("hunt", 700, 24, Direction::Down),
                bar("monster.0", 24, 24, Direction::Right),
                bar("weapon", 24, 610, Direction::Down),
                bar("player.health", 24, 770, Direction::Down),
                bar("player.stamina", 24, 810, Direction::Down),
                bar("buffs", 24, 860, Direction::Down),
            ],
        },
    ]
}

/// Barras fijas que el hub puede colocar sin editar el TOML a mano.
const FREE_BARS: [&str; 10] = [
    "hunt",
    "player.health",
    "player.stamina",
    "weapon",
    "buffs",
    "status",
    "monster.0",
    "monster.1",
    "monster.2",
    "monster.3",
];

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
    rathalos.crown = Crown::Gold;
    rathalos.target = true;
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
        health: Some(Health::new(100.0 - damaged, 100.0)),
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
    fn the_preview_only_sits_beside_the_form_when_there_is_room() {
        // Por debajo del umbral la vista previa va debajo: al lado dejaba el formulario sin ancho
        // y los párrafos se cortaban a media palabra.
        assert!(packed_preview(920.0));
        assert!(packed_preview(1179.0));
        assert!(!packed_preview(1180.0));
        assert!(!packed_preview(1920.0));
    }

    #[test]
    fn the_sidebar_footer_never_leaves_negative_space() {
        assert_eq!(footer_space(600.0, 178.0), 422.0);
        assert_eq!(footer_space(178.0, 178.0), 0.0);
        assert_eq!(footer_space(120.0, 178.0), 0.0);
    }

    #[test]
    fn a_long_stat_keeps_its_box() {
        // La cifra baja de tamaño antes que salirse del recuadro.
        assert_eq!(stat_size("0"), 19.0);
        assert_eq!(stat_size("1.600/2.056"), 15.0);
        assert_eq!(stat_size("menos de 1 s"), 15.0);
    }

    /// Pulsa una tecla en un contexto egui sin ventana y devuelve la pestaña que resolvería.
    fn pressed_key(key: egui::Key, modifiers: egui::Modifiers) -> Option<Tab> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        };

        let mut found = None;
        ctx.run_pass(input, |ctx| {
            found = ctx.input_mut(tab_shortcut);
        })
        .drop_without_applying_deltas();

        found
    }

    #[test]
    fn each_control_number_picks_its_tab() {
        for (key, tab) in [
            (egui::Key::Num1, Tab::News),
            (egui::Key::Num2, Tab::Diagnostics),
            (egui::Key::Num3, Tab::Weapon),
            (egui::Key::Num4, Tab::Settings),
        ] {
            assert_eq!(pressed_key(key, egui::Modifiers::CTRL), Some(tab));
        }
    }

    #[test]
    fn a_number_without_control_is_not_a_shortcut() {
        assert_eq!(pressed_key(egui::Key::Num1, egui::Modifiers::NONE), None);
        assert_eq!(pressed_key(egui::Key::Num1, egui::Modifiers::ALT), None);
    }

    #[test]
    fn a_shortcut_key_is_consumed_once() {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Num3,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            }],
            ..Default::default()
        };

        let mut pressed: Option<(Option<Tab>, Option<Tab>)> = None;
        ctx.run_pass(input, |ctx| {
            ctx.input_mut(|input| {
                pressed = Some((tab_shortcut(input), tab_shortcut(input)));
            });
        })
        .drop_without_applying_deltas();

        assert_eq!(pressed, Some((Some(Tab::Weapon), None)));
    }

    #[test]
    fn every_validation_state_has_its_own_label() {
        let labels = [
            validation_label(None),
            validation_label(Some(MapValidation::Verified)),
            validation_label(Some(MapValidation::Mismatch)),
            validation_label(Some(MapValidation::Unavailable)),
        ];

        for (index, label) in labels.iter().enumerate() {
            assert!(!label.is_empty());
            assert!(!labels[..index].contains(label), "{label} repetido");
        }
    }

    #[test]
    fn a_remembered_window_size_stays_inside_the_window() {
        assert_eq!(window_size(1120.0, 720.0), [1120.0, 720.0]);
        assert_eq!(window_size(10.0, 10.0), [860.0, 560.0]);
        assert_eq!(window_size(9000.0, 9000.0), [4096.0, 2160.0]);
    }

    #[test]
    fn closing_quits_without_a_panel_and_hides_with_one() {
        assert!(!close_hides_window(true, false));
        assert!(!close_hides_window(false, true));
        assert!(close_hides_window(true, true));
    }

    #[test]
    fn the_update_check_output_is_parsed() {
        assert_eq!(
            parse_available("actualización disponible: 0.2.0\n"),
            Some("0.2.0".to_string())
        );
        assert_eq!(parse_available("sin actualizaciones: 0.1.0\n"), None);
        assert_eq!(parse_available(""), None);
    }

    #[test]
    fn release_notes_are_split_from_their_header() {
        let notes = parse_release_notes(
            "tag: v0.3.0\nversion: 0.3.0\npublished: 2026-11-20T14:39:43Z\n---\n## Cambios\n- Vida\n",
        )
        .unwrap();

        assert_eq!(notes.tag, "v0.3.0");
        assert_eq!(notes.version, "0.3.0");
        assert_eq!(notes.published, "2026-11-20T14:39:43Z");
        assert_eq!(notes.body, "## Cambios\n- Vida");
        assert!(parse_release_notes("sin cabecera").is_none());
        assert_eq!(
            clean_inline_markdown("**Novedades** y `--flag`"),
            "Novedades y --flag"
        );
    }

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
