use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use clap::Parser;
use mh_overlay_config::{Config, ConfigError, Loaded, Origin, Position};
use mh_overlay_games::{GameKind, Hunt, Player, Snapshot};
use mh_overlay_memory::{AddressMap, MapValidation, Memory, MemoryError, Process, maps_directory};
use mh_overlay_ui::OverlayError;
use mh_overlay_ui::Panel;
use mh_overlay_ui::editor::Editor;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::window::{SettingsWindow, UiEvent};

/// Cualquiera de las capas puede fallar al arrancar; la CLI las presenta igual.
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error(transparent)]
    Memory(#[from] MemoryError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Overlay(#[from] OverlayError),
    #[error(transparent)]
    Hub(#[from] mh_overlay_app::AppError),
}

#[derive(Parser)]
#[command(
    name = "mh-overlay",
    version,
    about = "Vida, partes y estados de los monstruos de Monster Hunter en Linux"
)]
struct Arguments {
    /// Juego al que adjuntarse: rise o world
    #[arg(short, long, value_name = "JUEGO")]
    game: Option<GameKind>,

    /// Pid explícito, si hay más de una instancia corriendo
    #[arg(long, value_name = "PID")]
    pid: Option<i32>,

    /// Address map concreto; por defecto, el más nuevo compatible de data/maps
    #[arg(long, value_name = "ARCHIVO")]
    map: Option<PathBuf>,

    /// Base del módulo en hexadecimal, si /proc no la expone
    #[arg(long, value_name = "HEX", value_parser = parse_hex)]
    base: Option<u64>,

    /// Refrescar continuamente en lugar de leer una vez
    #[arg(long)]
    watch: bool,

    /// Dibujar el panel sobre el juego en una ventana X11
    #[arg(long)]
    overlay: bool,

    /// Abrir el hub de escritorio con diagnóstico y ajustes
    #[arg(long)]
    hub: bool,

    /// Abrir el editor de ajustes; no necesita el juego abierto
    #[arg(long)]
    settings: bool,

    /// Periodo de refresco en milisegundos; por defecto, el del archivo de configuración
    #[arg(long, value_name = "MS")]
    interval: Option<u64>,

    /// Anclaje del panel: top-left, top-right, bottom-left, bottom-right o center
    #[arg(long, value_name = "POSICIÓN")]
    position: Option<Position>,

    /// Archivo de configuración; por defecto, $XDG_CONFIG_HOME/mh-overlay/config.toml
    #[arg(long, value_name = "ARCHIVO")]
    config: Option<PathBuf>,

    /// Escribir la configuración documentada por defecto y salir
    #[arg(long)]
    init_config: bool,

    /// Mostrar los address maps disponibles y salir
    #[arg(long)]
    list_maps: bool,
}

fn main() -> ExitCode {
    let arguments = Arguments::parse();

    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");

            if let AppError::Memory(MemoryError::Read { source, .. }) = &error
                && source.kind() == std::io::ErrorKind::PermissionDenied
            {
                eprintln!(
                    "pista: el kernel bloquea la lectura (kernel.yama.ptrace_scope=1).\n       \
                     ejecuta `sudo setcap cap_sys_ptrace+ep <binario>` o baja el sysctl."
                );
            }

            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &Arguments) -> Result<(), AppError> {
    let maps_directory = maps_directory()?;

    if arguments.list_maps {
        list_maps(&maps_directory);
        return Ok(());
    }

    let config_path = arguments
        .config
        .clone()
        .unwrap_or_else(Config::default_path);

    if arguments.init_config {
        init_config(&config_path)?;
        return Ok(());
    }

    let loaded = Config::load_or_default(&config_path)?;

    // El hub edita el archivo tal cual está; los ajustes de la CLI son de esta ejecución y no
    // deben acabar guardados en él.
    if arguments.hub {
        let mut config = loaded.config;

        // El juego y el mapa de esta ejecución sirven para inspeccionar el otro título sin
        // tocar la configuración guardada.
        if let Some(kind) = arguments.game {
            config.game.kind = kind.to_string();
        }

        return mh_overlay_app::run(config_path, config, arguments.map.clone())
            .map_err(AppError::from);
    }

    let config = configured(&config_path, loaded, arguments)?;

    if arguments.settings {
        return run_settings(&config_path, config);
    }

    let kind = config.game_kind()?;

    let process = match arguments.pid {
        Some(pid) => Process::from_pid(pid, kind.module())?,
        None => Process::find(&[kind.module()])?,
    };
    let memory = process.memory();
    let base = arguments.base.unwrap_or_else(|| process.base());

    let map = match &arguments.map {
        Some(path) => {
            let map = load_map(path, base)?;
            report_validation(path, map.validate(&memory));
            map
        }
        None => select_map(kind, &maps_directory, memory, base)?,
    };

    if arguments.overlay {
        return run_overlay(&config_path, config, arguments, kind, memory, map);
    }

    if !arguments.watch {
        let snapshot = read_snapshot(kind, memory, &map)?;
        emit(&render(kind, &snapshot));
        return Ok(());
    }

    loop {
        let snapshot = read_snapshot(kind, memory, &map)?;
        emit(&format!("\x1b[2J\x1b[H{}", render(kind, &snapshot)));
        sleep(Duration::from_millis(config.game.interval_ms));
    }
}

/// Editor de ajustes: guarda cada cambio en el TOML, que un overlay en marcha relee solo.
fn run_settings(config_path: &Path, config: Config) -> Result<(), AppError> {
    let text = Text::bundled()?;
    let mut window = SettingsWindow::open("mh-overlay · ajustes")?;
    let mut editor = Editor::new(config);
    let label = display_path(config_path);

    loop {
        window.present(&editor.view(&text, &label))?;

        let changed = match window.next_event()? {
            UiEvent::Click { x, y } => editor.click(x, y),
            UiEvent::Move { x, y } => {
                editor.motion(x, y);
                false
            }
            UiEvent::Key(key) => editor.key(key),
            UiEvent::Close => return Ok(()),
            UiEvent::Ignored => false,
        };

        if changed && let Err(error) = editor.config().save(config_path) {
            eprintln!("aviso: {error}");
        }
    }
}

/// La ruta con `~` delante si cuelga del home, para que quepa en la ventana.
fn display_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME")
        && let Ok(rest) = path.strip_prefix(Path::new(&home))
    {
        return format!("~/{}", rest.display());
    }

    path.display().to_string()
}

/// Escribe la plantilla documentada y no toca un archivo que ya existe.
fn init_config(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        eprintln!("ya existe {}; no se sobrescribe", path.display());
        return Ok(());
    }

    Config::write_template(path)?;
    eprintln!("configuración escrita en {}", path.display());

    Ok(())
}

fn configured(
    config_path: &Path,
    loaded: Loaded,
    arguments: &Arguments,
) -> Result<Config, ConfigError> {
    let mut config = loaded.config;

    // Con el archivo ausente se escribe la plantilla: así el usuario tiene algo que editar con
    // los valores que de verdad están en uso, en vez de buscarlos en la documentación.
    if loaded.origin == Origin::Defaults
        && (arguments.overlay || arguments.watch || arguments.settings)
    {
        match Config::write_template(config_path) {
            Ok(()) => eprintln!("configuración creada en {}", config_path.display()),
            Err(error) => eprintln!("aviso: {error}"),
        }
    }

    apply_overrides(&mut config, arguments);
    config.validate()?;

    Ok(config)
}

fn apply_overrides(config: &mut Config, arguments: &Arguments) {
    if let Some(game) = arguments.game {
        config.game.kind = game.to_string();
    }

    if let Some(interval) = arguments.interval {
        config.game.interval_ms = interval;
    }

    if let Some(position) = arguments.position {
        config.overlay.position = position;
    }
}

/// Bucle del panel: lee, dibuja y sube la imagen; sobrevive a errores de lectura y a
/// configuraciones a medio guardar, porque cerrar el overlay en mitad de una cacería es peor
/// que quedarse con el último estado conocido.
fn run_overlay(
    config_path: &Path,
    config: Config,
    arguments: &Arguments,
    kind: GameKind,
    memory: Memory,
    map: AddressMap,
) -> Result<(), AppError> {
    let text = Text::bundled()?;
    let mut window = Panel::open("mh-overlay")?;
    let mut config = config;
    let mut stamp = modified_at(config_path);
    let mut last_error: Option<String> = None;

    loop {
        if let Some(modified) = modified_at(config_path)
            && stamp.as_ref() != Some(&modified)
        {
            stamp = Some(modified);

            match Config::load_or_default(config_path) {
                Ok(loaded) => {
                    let mut reloaded = loaded.config;
                    apply_overrides(&mut reloaded, arguments);

                    match reloaded.validate() {
                        Ok(()) => {
                            config = reloaded;
                            eprintln!("configuración recargada");
                        }
                        Err(error) => {
                            eprintln!("aviso: {error}; sigo con la configuración anterior")
                        }
                    }
                }
                Err(error) => {
                    eprintln!("aviso: {error}; sigo con la configuración anterior")
                }
            }
        }

        let snapshot = match mh_overlay_games::snapshot(kind, memory, &map) {
            Ok(snapshot) => {
                last_error = None;
                snapshot
            }
            Err(error) => {
                let message = error.to_string();

                if last_error.as_deref() != Some(&message) {
                    eprintln!("aviso: {message}");
                    last_error = Some(message);
                }

                Snapshot::default()
            }
        };

        let status = last_error.as_deref().map(|_| "sin conexión con el juego");

        if snapshot.monsters.is_empty() && status.is_none() && config.overlay.hide_on_empty {
            window.hide()?;
            sleep(Duration::from_millis(config.game.interval_ms));
            continue;
        }

        let canvas = mh_overlay_ui::view::render(&config, &text, &snapshot, status);
        let origin = config.overlay.position.origin(
            window.screen_size(),
            (canvas.width(), canvas.height()),
            (config.overlay.margin_x, config.overlay.margin_y),
        );

        window.present(&canvas, origin, true, config.overlay.click_through)?;

        sleep(Duration::from_millis(config.game.interval_ms));
    }
}

fn modified_at(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn load_map(path: &Path, base: u64) -> Result<AddressMap, MemoryError> {
    let mut map = AddressMap::load(path)?;
    map.set_base(base);
    Ok(map)
}

/// Elige el mapa más nuevo cuyo CRC coincida con la memoria del juego.
///
/// Una build y un mapa no son intercambiables: usar el archivo más nuevo contra un juego viejo
/// no falla, devuelve basura plausible, que es el peor resultado posible.
fn select_map(
    kind: GameKind,
    directory: &Path,
    memory: Memory,
    base: u64,
) -> Result<AddressMap, MemoryError> {
    let selected = AddressMap::select_for_game(kind.map_prefix(), directory, &memory, base)?;
    report_validation(&selected.path, selected.validation);
    Ok(selected.map)
}

fn report_validation(path: &Path, validation: MapValidation) {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    match validation {
        MapValidation::Verified => eprintln!("mapa: {name} (versión verificada por CRC)"),
        MapValidation::Mismatch => eprintln!(
            "aviso: {name} no coincide con los CRC del juego: es de otra build.\n       \
             si los números salen mal, pasa --map con el mapa de tu versión."
        ),
        MapValidation::Unavailable => {
            eprintln!("aviso: {name} no trae CRC; no se pudo verificar que corresponda a tu build.")
        }
    }
}

fn read_snapshot(
    kind: GameKind,
    memory: Memory,
    map: &AddressMap,
) -> Result<Snapshot, MemoryError> {
    mh_overlay_games::snapshot(kind, memory, map).inspect_err(|error| {
        if let MemoryError::MissingKey { key } = error {
            eprintln!(
                "aviso: el mapa {} (versión {}) no define '{key}';\n       \
                 esa build no está soportada por este lector, prueba --map con otra versión.",
                map.game(),
                map.version()
            );
        }
    })
}

fn hunt_line(hunt: &Hunt) -> String {
    format!(
        "cacería  {:>6} restante   vidas {}/{}\n\n",
        hunt.time_label(),
        hunt.deaths,
        hunt.max_deaths
    )
}

fn player_line(player: &Player) -> String {
    format!(
        "tú  HP {:>5.1}%  ({:.0} / {:.0})   aguante {:>5.1}%  ({:.0} / {:.0})\n\n",
        player.health.percent(),
        player.health.current,
        player.health.max,
        player.stamina.percent(),
        player.stamina.current,
        player.stamina.max,
    )
}

fn render(kind: GameKind, snapshot: &Snapshot) -> String {
    let monsters = &snapshot.monsters;
    let mut output = format!("mh-overlay · {kind} · {} monstruo(s)\n\n", monsters.len());

    if let Some(hunt) = snapshot.hunt {
        output.push_str(&hunt_line(&hunt));
    }
    if let Some(player) = snapshot.player {
        output.push_str(&player_line(&player));
    }

    if monsters.is_empty() {
        output.push_str("sin monstruos grandes en la zona\n");
        return output;
    }

    for monster in monsters {
        let name = monster
            .name
            .as_deref()
            .map(|name| format!(" {name}"))
            .unwrap_or_default();
        let capture = monster
            .capture_percent
            .map(|percent| format!("   captura ≤ {percent:.0}%"))
            .unwrap_or_default();

        output.push_str(&format!(
            "id {:<4}{name}  HP {:>5.1}%  ({:.0} / {:.0}){capture}\n",
            monster.id,
            monster.health.percent(),
            monster.health.current,
            monster.health.max,
        ));

        for part in &monster.parts {
            output.push_str(&format!(
                "  {:<14}",
                part.name
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("parte {:02}", part.index))
            ));

            if let Some(flinch) = part.flinch {
                output.push_str(&format!("  flinch {:>5.1}%", flinch.percent()));
            }
            if let Some(breakable) = part.breakable {
                output.push_str(&format!("  romper {:>5.1}%", breakable.percent()));
            }
            if let Some(severable) = part.severable {
                output.push_str(&format!("  cortar {:>5.1}%", severable.percent()));
            }

            output.push('\n');
        }

        for ailment in &monster.ailments {
            output.push_str(&format!(
                "  {} {:>5.1}%  ({:.0} / {:.0})\n",
                ailment.name,
                ailment.buildup_percent(),
                ailment.buildup,
                ailment.max_buildup,
            ));
        }

        output.push('\n');
    }

    output.push_str("datos leídos del proceso del juego; nada se escribe en él\n");

    output
}

fn list_maps(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        eprintln!("no se pudo leer {}", directory.display());
        return;
    };

    let mut names = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .filter(|name| name.ends_with(".map"))
        .collect::<Vec<_>>();
    names.sort();

    let mut output = format!("address maps en {}:\n", directory.display());

    for name in names {
        output.push_str(&format!("  {name}\n"));
    }

    emit(&output);
}

/// Escribe en stdout ignorando `EPIPE`, para que `mh-overlay --list-maps | head` termine en
/// silencio en vez de entrar en pánico al cerrarse la tubería.
fn emit(text: &str) {
    use std::io::Write;

    let _ = std::io::stdout().lock().write_all(text.as_bytes());
}

fn parse_hex(value: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(
        value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
            .unwrap_or(value),
        16,
    )
}
