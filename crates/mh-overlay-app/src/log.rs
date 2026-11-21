//! Registro del hub en `$XDG_STATE_HOME/mh-overlay/hub.log`.
//!
//! El hub vive sin terminal: cuando la ventana se cierra sola o un refresco empieza a fallar,
//! este archivo es el único rastro. No crece sin freno: al pasar de 256 KiB el anterior pasa a
//! `hub.log.1` y el nuevo empieza vacío.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tamaño a partir del cual se rota; un registro más grande ocupa más que la aplicación.
const LIMIT: u64 = 256 * 1024;

/// Archivo abierto por [`install`]; `None` mientras no haya dónde escribir.
static SINK: Mutex<Option<File>> = Mutex::new(None);

/// Ruta del registro, exista todavía o no.
pub fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;

    Some(base.join("mh-overlay").join("hub.log"))
}

/// Abre el registro y lo deja listo para [`line`]. Devuelve dónde escribe.
///
/// Un registro que no se puede abrir no es un error de arranque: el hub funciona igual, solo
/// pierde el rastro. Por eso devuelve `None` en vez de fallar.
pub fn install() -> Option<PathBuf> {
    let path = path()?;
    fs::create_dir_all(path.parent()?).ok()?;
    rotate(&path, LIMIT);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;

    *SINK.lock().ok()? = Some(file);

    Some(path)
}

/// Escribe una línea con fecha UTC, nivel y mensaje. Sin registro abierto no hace nada.
pub fn line(level: &str, message: &str) {
    let Ok(mut sink) = SINK.lock() else {
        return;
    };
    let Some(file) = sink.as_mut() else {
        return;
    };

    let stamp = timestamp(SystemTime::now());

    // Un mensaje de varias líneas se numera por líneas: si no, la fecha solo acompañaría a la
    // primera y el resto parecería de otro momento.
    for part in message.lines() {
        let _ = writeln!(file, "{stamp} · {level} · {part}");
    }

    let _ = file.flush();
}

pub fn info(message: &str) {
    line("info", message);
}

pub fn warn(message: &str) {
    line("aviso", message);
}

/// Cualquier pánico deja su mensaje y la pila en el registro, y luego sigue al hook anterior.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        line("pánico", &panic.to_string());

        let backtrace = std::backtrace::Backtrace::force_capture();
        line("pánico", &backtrace.to_string());

        previous(panic);
    }));
}

/// Si el archivo pasó de `limit`, lo mueve a `.1` para que el siguiente arranque empiece limpio.
fn rotate(path: &Path, limit: u64) {
    let oversized = fs::metadata(path)
        .map(|meta| meta.len() > limit)
        .unwrap_or(false);

    if !oversized {
        return;
    }

    let previous = rotated(path);
    let _ = fs::remove_file(&previous);
    let _ = fs::rename(path, previous);
}

fn rotated(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".1");
    PathBuf::from(name)
}

/// Fecha UTC en `YYYY-MM-DD HH:MM:SSZ`; no hay zona horaria que consultar sin dependencias.
fn timestamp(now: SystemTime) -> String {
    let seconds = now
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);

    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    let rest = seconds % 86_400;

    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}Z",
        rest / 3_600,
        (rest % 3_600) / 60,
        rest % 60
    )
}

/// Días desde 1970 a fecha civil, por el algoritmo de Hinnant (`civil_from_days`).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    };

    (year + i64::from(month <= 2), month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_is_january_1970() {
        assert_eq!(timestamp(UNIX_EPOCH), "1970-01-01 00:00:00Z");
    }

    #[test]
    fn a_known_instant_keeps_its_date_and_time() {
        // 2026-11-21T13:14:15Z
        let now = UNIX_EPOCH + std::time::Duration::from_secs(1_795_266_855);

        assert_eq!(timestamp(now), "2026-11-21 13:14:15Z");
    }

    #[test]
    fn leap_days_are_counted() {
        // 2024-02-29T00:00:00Z
        let now = UNIX_EPOCH + std::time::Duration::from_secs(1_709_164_800);

        assert_eq!(timestamp(now), "2024-02-29 00:00:00Z");
    }

    #[test]
    fn the_rotated_file_keeps_the_whole_name() {
        assert_eq!(
            rotated(Path::new("/tmp/hub.log")),
            PathBuf::from("/tmp/hub.log.1")
        );
    }
}
