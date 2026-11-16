use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use mh_overlay_games::{GameKind, Monster};
use mh_overlay_memory::{AddressMap, MapValidation, Memory, MemoryError, Process};

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

    /// Periodo de refresco en milisegundos
    #[arg(long, default_value_t = 500, value_name = "MS")]
    interval: u64,

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

            if let MemoryError::Read { source, .. } = &error
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

fn run(arguments: &Arguments) -> Result<(), MemoryError> {
    let maps_directory = maps_directory()?;

    if arguments.list_maps {
        list_maps(&maps_directory);
        return Ok(());
    }

    let kind = arguments.game.unwrap_or(GameKind::Rise);

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

    if !arguments.watch {
        let monsters = read_monsters(kind, memory, &map)?;
        emit(&render(kind, &monsters));
        return Ok(());
    }

    loop {
        let monsters = read_monsters(kind, memory, &map)?;
        emit(&format!("\x1b[2J\x1b[H{}", render(kind, &monsters)));
        sleep(Duration::from_millis(arguments.interval));
    }
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
    let candidates = AddressMap::candidates_for_game(kind.map_prefix(), directory)?;
    let mut fallback = None;

    for path in &candidates {
        let map = load_map(path, base)?;
        let validation = map.validate(&memory);

        if validation == MapValidation::Verified {
            report_validation(path, validation);
            return Ok(map);
        }

        if fallback.is_none() {
            fallback = Some((path.clone(), map, validation));
        }
    }

    let (path, map, validation) = fallback.ok_or_else(|| MemoryError::NoMapsForGame {
        game: kind.map_prefix().to_string(),
        directory: directory.display().to_string(),
    })?;

    report_validation(&path, validation);
    Ok(map)
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

fn read_monsters(
    kind: GameKind,
    memory: Memory,
    map: &AddressMap,
) -> Result<Vec<Monster>, MemoryError> {
    mh_overlay_games::monsters(kind, memory, map).inspect_err(|error| {
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

fn render(kind: GameKind, monsters: &[Monster]) -> String {
    let mut output = format!("mh-overlay · {kind} · {} monstruo(s)\n\n", monsters.len());

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
            output.push_str(&format!("  parte {:<2}", part.index));

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

/// Directorio de los `.map`: `$MH_OVERLAY_MAPS`, `./data/maps`, o el del repositorio
/// relativo al ejecutable (para cuando se lanza desde `target/`).
fn maps_directory() -> Result<PathBuf, MemoryError> {
    if let Ok(directory) = std::env::var("MH_OVERLAY_MAPS")
        && Path::new(&directory).is_dir()
    {
        return Ok(PathBuf::from(directory));
    }

    let mut candidates = vec![PathBuf::from("data/maps")];

    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
    {
        candidates.push(directory.join("../../data/maps"));
        candidates.push(directory.join("data/maps"));
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.is_dir())
        .ok_or_else(|| MemoryError::NoMapsForGame {
            game: "cualquiera".to_string(),
            directory: "data/maps".to_string(),
        })
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
