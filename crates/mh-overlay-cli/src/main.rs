mod update;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use clap::Parser;
use mh_overlay_config::{Config, ConfigError, Layout, Loaded, Origin, Position};
use mh_overlay_games::{GameKind, Hunt, Player, Snapshot, WeaponGauge, part_labels};
use mh_overlay_memory::{AddressMap, MapValidation, Memory, MemoryError, Process, maps_directory};
use mh_overlay_ui::editor::Editor;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::window::{SettingsWindow, UiEvent};
use mh_overlay_ui::{OverlayError, Panel, Preference};

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
    #[error(transparent)]
    Update(#[from] update::UpdateError),
}

#[derive(Parser)]
#[command(
    name = "mh-overlay",
    version,
    about = "Vida, partes y estados de los monstruos de Monster Hunter en Linux",
    long_about = "Vida, partes y estados de los monstruos de Monster Hunter en Linux.\n\n\
                  Sin argumentos abre el hub gráfico; desde ahí se abre el panel sobre el \
                  juego. Para la consola están --print y --watch."
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

    /// Dibujar el panel sobre el juego
    #[arg(long)]
    overlay: bool,

    /// Backend del panel: auto prefiere layer-shell de Wayland y cae a X11
    #[arg(long, value_name = "BACKEND", value_enum, default_value_t = BackendChoice::Auto)]
    backend: BackendChoice,

    /// Imprimir una lectura en la consola y salir, sin abrir ventana
    #[arg(long)]
    print: bool,

    /// Abrir el hub de escritorio con diagnóstico y ajustes (lo que hace sin argumentos)
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

    /// Listar los procesos que mencionan o mapean el ejecutable del juego y salir
    #[arg(long)]
    ps: bool,

    /// Imprimir un informe del entorno, la configuración y la detección, y salir
    #[arg(long)]
    doctor: bool,

    /// Actualizar esta instalación (AppImage o paquete) y salir
    #[arg(long)]
    update: bool,

    /// Comprobar si hay una versión nueva y salir, sin descargar nada
    #[arg(long)]
    check_update: bool,

    /// Imprimir la etiqueta, la fecha y las notas de la última release y salir
    #[arg(long)]
    release_notes: bool,
}

/// Backend pedido en la línea de órdenes.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum BackendChoice {
    Auto,
    Wayland,
    X11,
}

impl From<BackendChoice> for Preference {
    fn from(choice: BackendChoice) -> Self {
        match choice {
            BackendChoice::Auto => Preference::Auto,
            BackendChoice::Wayland => Preference::Wayland,
            BackendChoice::X11 => Preference::X11,
        }
    }
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
    // Se atiende antes que nada: actualizar no necesita el juego, ni los mapas que va a
    // reemplazar, ni una sesión gráfica.
    if arguments.update {
        return update::run().map_err(AppError::from);
    }

    if arguments.check_update {
        return match update::check()? {
            Some(version) => {
                println!("actualización disponible: {version}");
                Ok(())
            }
            None => {
                println!("sin actualizaciones: {}", update::CURRENT);
                Ok(())
            }
        };
    }

    if arguments.release_notes {
        let notes = update::notes()?;
        println!("tag: {}", notes.tag);
        println!("version: {}", notes.version);
        println!("published: {}", notes.published);
        println!("---");
        println!("{}", notes.body);
        return Ok(());
    }

    // Antes de los mapas: cuando el juego no aparece, lo primero que hay que ver es qué
    // procesos existen, y eso no depende de tener los `.map` instalados.
    if arguments.ps {
        list_processes();
        return Ok(());
    }

    let config_path = arguments
        .config
        .clone()
        .unwrap_or_else(Config::default_path);

    // El informe se pide justo cuando algo no funciona, así que no puede fallar por lo que
    // intenta explicar: ni los mapas ni la configuración tienen que estar bien.
    if arguments.doctor {
        doctor(&config_path);
        return Ok(());
    }

    let maps_directory = maps_directory()?;

    if arguments.list_maps {
        list_maps(&maps_directory);
        return Ok(());
    }

    if arguments.init_config {
        init_config(&config_path)?;
        return Ok(());
    }

    let loaded = Config::load_or_default(&config_path)?;

    if loaded.migrated {
        eprintln!(
            "configuración de un esquema anterior: se usa el actual ({}) y se reescribe al \
             siguiente cambio",
            mh_overlay_config::CONFIG_VERSION
        );
    }

    // Sin una acción explícita, la aplicación es su interfaz gráfica: es lo que espera quien
    // pulsa el lanzador del menú. La consola queda en --print y --watch.
    if opens_hub(arguments) {
        let origin = loaded.origin;
        let mut config = loaded.config;

        // El juego y el mapa de esta ejecución sirven para inspeccionar el otro título sin
        // tocar la configuración guardada.
        if let Some(kind) = arguments.game {
            config.game.kind = kind.to_string();
        }

        if origin == Origin::Defaults {
            match Config::write_template(&config_path) {
                Ok(()) => eprintln!("configuración creada en {}", config_path.display()),
                Err(error) => eprintln!("aviso: {error}"),
            }
        }

        return mh_overlay_app::run(config_path, config, arguments.map.clone(), arguments.game)
            .map_err(AppError::from);
    }

    let config = configured(&config_path, loaded, arguments)?;

    if arguments.settings {
        return run_settings(&config_path, config);
    }

    let configured = config.game_kind()?;
    // Sin `--game` ni `--pid`, lo que manda es lo que está abierto. Un `--game` explícito es
    // una elección del usuario y no se detecta por encima de ella.
    let auto = arguments.game.is_none() && arguments.pid.is_none() && config.game.auto_detect;
    let detected = auto.then(mh_overlay_games::detect_running_game).flatten();
    let kind = detected.unwrap_or(configured);

    if detected.is_some_and(|found| found != configured) {
        eprintln!("juego detectado: {kind}");
    }

    let process = match arguments.pid {
        Some(pid) => Process::from_pid(pid, kind.module())?,
        // Si la detección no encontró nada, se buscan los dos para que el error diga los dos
        // ejecutables que se miraron, no solo el que estaba en la configuración.
        None if auto && detected.is_none() => {
            Process::find(&mh_overlay_games::GameKind::ALL.map(mh_overlay_games::GameKind::module))?
        }
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

    if arguments.print || !arguments.watch {
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

/// Sin `--overlay`, `--watch`, `--print` ni `--settings` no hay trabajo de consola: se abre el
/// hub. `--hub` sigue siendo explícito para scripts y lanzadores antiguos.
fn opens_hub(arguments: &Arguments) -> bool {
    arguments.hub
        || !(arguments.overlay || arguments.watch || arguments.print || arguments.settings)
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
    let mut window = Panel::open_with("mh-overlay", arguments.backend.into())?;
    eprintln!("panel: {}", window.backend_name());

    // El panel corre sin terminal cuando lo lanza el hub: lo que pase aquí tiene que quedar en
    // el mismo registro, o el cierre no deja rastro.
    mh_overlay_app::log::install();
    mh_overlay_app::log::info(&format!("panel: abierto en {}", window.backend_name()));

    let mut visible = true;
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
            if visible {
                mh_overlay_app::log::info("panel: oculto hasta que haya monstruos en la zona");
                visible = false;
            }

            window.hide()?;
            sleep(Duration::from_millis(config.game.interval_ms));
            continue;
        }

        if !visible {
            mh_overlay_app::log::info("panel: visible");
            visible = true;
        }

        let screen = window.screen_size();
        let free = config.overlay.layout == Layout::Free;
        let canvas = if free {
            mh_overlay_ui::view::render_free(&config, &text, &snapshot, status, screen)
        } else {
            mh_overlay_ui::view::render(&config, &text, &snapshot, status)
        };

        if free {
            // Las coordenadas de [[overlay.bars]] son absolutas; el anclaje y los márgenes solo
            // se usan para la columna por defecto.
            window.present_free(&canvas, (0, 0), config.overlay.click_through)?;
        } else {
            let origin = config.overlay.position.origin(
                screen,
                (canvas.width(), canvas.height()),
                (config.overlay.margin_x, config.overlay.margin_y),
            );
            window.present(&canvas, origin, true, config.overlay.click_through)?;
        }

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
            "aviso: {name} no coincide con los CRC del juego: el módulo no está como lo dejó \
             Capcom.\n       \
             puede ser otra build o un mod que parchea código (REFramework, Fluffy).\n       \
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
    let mut output = format!(
        "tú  HP {:>5.1}%  ({})   aguante {:>5.1}%  ({})\n",
        player.health.percent(),
        player.health.label(),
        player.stamina.percent(),
        player.stamina.label(),
    );

    if let Some(weapon) = &player.weapon {
        output.push_str(&format!("    arma   {}\n", weapon.kind.label()));

        if let Some(sharpness) = &weapon.sharpness {
            output.push_str(&format!(
                "    filo   {:.0} · {}",
                sharpness.hits_left(),
                sharpness.level.label()
            ));
            if let Some((level, hits)) = sharpness.next_segment() {
                output.push_str(&format!(" · luego {} {hits}", level.label()));
            }
            output.push('\n');
        }

        if let Some(gauge) = &weapon.gauge {
            output.push_str(&format!("    medidor {}\n", gauge_label(gauge)));
        }
    }

    for buff in &player.buffs {
        output.push_str(&format!(
            "    buff   {:<30} {:>5.0}s\n",
            buff.name, buff.timer
        ));
    }

    output.push('\n');
    output
}

/// Resumen de una línea del medidor de clase, para la salida de consola.
fn gauge_label(gauge: &WeaponGauge) -> String {
    match gauge {
        WeaponGauge::Spirit { buildup, level, .. } => {
            format!("espíritu {buildup:.0}% nivel {level}")
        }
        WeaponGauge::ChargeBlade {
            phials,
            max_phials,
            charge,
            ..
        } => format!("{phials}/{max_phials} frascos, carga {charge:.0}%"),
        WeaponGauge::SwitchAxe {
            buildup, charge, ..
        } => format!("filo {buildup:.0}%, amped {charge:.0}%"),
        WeaponGauge::DualBlades {
            demon,
            arch_demon,
            buildup,
        } => format!(
            "modo {buildup:.0}%{}",
            if *arch_demon {
                " (archidemonio)"
            } else if *demon {
                " (demonio)"
            } else {
                ""
            }
        ),
        WeaponGauge::InsectGlaive {
            extracts, stamina, ..
        } => format!("esencias {}, kinsecto {stamina:.0}", extracts.label()),
    }
}

fn render(kind: GameKind, snapshot: &Snapshot) -> String {
    let monsters = &snapshot.monsters;
    let mut output = format!("mh-overlay · {kind} · {} monstruo(s)\n\n", monsters.len());

    if let Some(hunt) = snapshot.hunt {
        output.push_str(&hunt_line(&hunt));
    }
    if let Some(player) = &snapshot.player {
        output.push_str(&player_line(player));
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
        let crown = monster
            .crown
            .label()
            .map(|label| format!("   {label}"))
            .unwrap_or_default();
        let target = if monster.target { "   objetivo" } else { "" };

        output.push_str(&format!(
            "id {:<4}{name}  HP {:>5.1}%  ({}){capture}{crown}{target}\n",
            monster.id,
            monster.health.percent(),
            monster.health.label(),
        ));

        let labels = part_labels(&monster.parts);
        for (part, label) in monster.parts.iter().zip(&labels) {
            output.push_str(&format!("  {label:<14}"));

            if part.is_broken() {
                output.push_str("  roto\n");
                continue;
            }

            if let Some(health) = part.health {
                output.push_str(&format!("  vida {}", health.label()));
            }
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
            if ailment.is_elemental() {
                continue;
            }

            if ailment.is_triggered() {
                output.push_str(&format!(
                    "  {:<14} {:>3.0} s de estado\n",
                    ailment.name, ailment.timer
                ));
            } else {
                output.push_str(&format!(
                    "  {:<14} faltan {:.0}  ({:.0} / {:.0})\n",
                    ailment.name,
                    ailment.missing_buildup(),
                    ailment.buildup,
                    ailment.max_buildup,
                ));
            }
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

/// Procesos que mencionan o mapean los ejecutables del juego: lo mismo que mira la detección
/// automática, para cuando el juego corre y aun así no aparece.
fn list_processes() {
    let names = GameKind::ALL.map(GameKind::module);
    let seen = mh_overlay_memory::lookalikes(&names);

    let mut output = format!("procesos que mencionan o mapean {}:\n", names.join(" o "));

    if seen.is_empty() {
        output.push_str(
            "  ninguno: el juego no está corriendo, corre con otro usuario, o /proc no deja\n  \
             ver su línea de comandos\n",
        );
    }

    for process in seen {
        let state = match &process.mapping {
            Some((module, base)) => format!("mapea {module} en 0x{base:X}"),
            None => "sin el ejecutable mapeado".to_string(),
        };

        output.push_str(&format!(
            "  pid {:>6}  {:<20} {state}\n",
            process.pid, process.comm
        ));

        if !process.cmdline.is_empty() {
            output.push_str(&format!("          {}\n", shorten(&process.cmdline, 200)));
        }
    }

    emit(&output);
}

/// Corta una línea larga sin partir un carácter a la mitad.
fn shorten(text: &str, limit: usize) -> String {
    match text.char_indices().nth(limit) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_string(),
    }
}

/// Escribe en stdout ignorando `EPIPE`, para que `mh-overlay --list-maps | head` termine en
/// silencio en vez de entrar en pánico al cerrarse la tubería.
fn emit(text: &str) {
    use std::io::Write;

    let _ = std::io::stdout().lock().write_all(text.as_bytes());
}

/// Informe de una pieza: sesión, permisos, configuración, mapas, procesos y registro.
///
/// Todo lo que aquí se imprime se puede pegar en un issue sin quitar nada, así que no incluye
/// memoria del usuario ni nombres de personajes: solo rutas y hechos del sistema.
fn doctor(config_path: &Path) {
    let mut output = format!("mh-overlay {} · doctor\n", update::CURRENT);

    output.push_str("\nsesión\n");
    for (label, variable) in [
        ("X11", "DISPLAY"),
        ("Wayland", "WAYLAND_DISPLAY"),
        ("tipo", "XDG_SESSION_TYPE"),
        ("escritorio", "XDG_CURRENT_DESKTOP"),
    ] {
        let value = std::env::var(variable).unwrap_or_else(|_| "(sin definir)".to_string());
        output.push_str(&format!("  {label}: {value}\n"));
    }

    output.push_str("\npermisos\n");
    match mh_overlay_memory::env::ptrace_scope() {
        Some(0) => output.push_str("  kernel.yama.ptrace_scope: 0 (cualquier proceso puede leer)\n"),
        Some(scope) => output.push_str(&format!(
            "  kernel.yama.ptrace_scope: {scope} (leer otro proceso exige CAP_SYS_PTRACE o ser su padre)\n"
        )),
        None => output.push_str("  kernel.yama.ptrace_scope: sin Yama (sin restricción por este lado)\n"),
    }

    let capable = mh_overlay_memory::env::has_cap_sys_ptrace();
    output.push_str(&format!(
        "  CAP_SYS_PTRACE efectiva: {}\n",
        if capable { "sí" } else { "no" }
    ));

    if !capable && mh_overlay_memory::env::ptrace_scope().is_some_and(|scope| scope > 0) {
        output.push_str(
            "  pista: sudo setcap cap_sys_ptrace+ep $(command -v mh-overlay)  ·  o baja el sysctl\n",
        );
    }

    output.push_str("\nactualizaciones\n");
    let signature = match update::signature_key() {
        update::SignatureKey::Ready => {
            "clave del proyecto compilada (obligatoria al actualizar)".to_string()
        }
        update::SignatureKey::Missing => "sin clave del proyecto: solo SHA-256".to_string(),
        update::SignatureKey::Unreadable(error) => {
            format!("clave del proyecto ilegible, la actualización fallaría: {error}")
        }
    };
    output.push_str(&format!("  firma de los paquetes: {signature}\n"));

    output.push_str("\nconfiguración\n");
    output.push_str(&format!("  archivo: {}\n", config_path.display()));

    match Config::load_or_default(config_path) {
        Ok(loaded) => {
            let origin = match loaded.origin {
                Origin::File => "leída del archivo",
                Origin::Defaults => "valores por defecto (el archivo no existe)",
            };
            output.push_str(&format!("  estado: válida · {origin}\n"));
            output.push_str(&format!(
                "  juego: {} · detección automática {} · refresco {} ms\n",
                loaded.config.game.kind,
                if loaded.config.game.auto_detect {
                    "sí"
                } else {
                    "no"
                },
                loaded.config.game.interval_ms,
            ));
        }
        Err(error) => output.push_str(&format!("  estado: error: {error}\n")),
    }

    output.push_str("\nmapas\n");
    match maps_directory() {
        Ok(directory) => {
            output.push_str(&format!("  carpeta: {}\n", directory.display()));

            let mut names = std::fs::read_dir(&directory)
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                        .filter(|name| name.ends_with(".map"))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            names.sort();

            output.push_str(&format!("  {} archivo(s)\n", names.len()));
            for name in names.iter().rev().take(3) {
                output.push_str(&format!("    {name}\n"));
            }
            if names.len() > 3 {
                output.push_str(&format!("    … {} más\n", names.len() - 3));
            }
        }
        Err(error) => output.push_str(&format!("  error: {error}\n")),
    }

    output.push_str("\nprocesos\n");
    let names = GameKind::ALL.map(GameKind::module);
    let seen = mh_overlay_memory::lookalikes(&names);

    if seen.is_empty() {
        output.push_str(&format!(
            "  ninguno menciona {} · ¿juego cerrado, en otra cuenta, o /proc sin permiso?\n",
            names.join(" o ")
        ));
    }

    for process in &seen {
        let state = match &process.mapping {
            Some((module, base)) => format!("mapea {module} en 0x{base:X}"),
            None => "sin el ejecutable mapeado".to_string(),
        };
        output.push_str(&format!(
            "  pid {} · {} · {state}\n",
            process.pid, process.comm
        ));
    }

    output.push_str("\nregistro\n");
    match mh_overlay_app::log::path() {
        Some(path) => {
            let size = std::fs::metadata(&path).map(|meta| meta.len()).ok();
            output.push_str(&format!(
                "  {} ({})\n",
                path.display(),
                match size {
                    Some(bytes) => format!("{} KiB", bytes / 1024),
                    None => "todavía no existe".to_string(),
                }
            ));
        }
        None => output.push_str("  sin ruta: define XDG_STATE_HOME o HOME\n"),
    }

    emit(&output);
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
