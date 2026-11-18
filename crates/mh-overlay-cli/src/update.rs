//! Actualización en sitio desde las releases de GitHub.
//!
//! La instalación no se desinstala: `--update` baja el paquete de la última release, comprueba su
//! SHA-256, y reemplaza el binario y los mapas que están junto a él. El archivo de configuración
//! vive en `$XDG_CONFIG_HOME`, fuera de la instalación, así que se conserva tal cual.
//!
//! No se añade un cliente HTTP: `curl`, `tar` y `sha256sum` están en cualquier escritorio Linux y
//! evitan meter TLS y un analizador de JSON en el binario del overlay.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Repositorio del que se publican los paquetes.
pub const REPOSITORY: &str = "SyberFoxx/mh-overlay";
const API: &str = "https://api.github.com/repos/SyberFoxx/mh-overlay/releases/latest";
const DOWNLOADS: &str = "https://github.com/SyberFoxx/mh-overlay/releases/download";
/// Variables para probar el actualizador contra un servidor local y para usar un espejo.
const API_OVERRIDE: &str = "MH_OVERLAY_UPDATE_API";
const DOWNLOADS_OVERRIDE: &str = "MH_OVERLAY_UPDATE_DOWNLOADS";
/// Versión del binario que está ejecutando la actualización.
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("no se pudo consultar las releases de {REPOSITORY}: {0}")]
    Network(String),
    #[error("{REPOSITORY} no tiene releases publicadas todavía")]
    NoReleases,
    #[error("la release más reciente no trae un paquete para {arch}: {name}")]
    NoPackage { arch: String, name: String },
    #[error("el SHA-256 del paquete no coincide; no se instala nada")]
    Checksum,
    #[error("no se pudo descomprimir el paquete: {0}")]
    Unpack(String),
    #[error(
        "no se puede escribir en {0}; instala con `install.sh` o actualiza con permisos sobre esa carpeta"
    )]
    NotWritable(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Comprueba si hay una versión nueva y, si la hay, la instala sobre este mismo binario.
pub fn run() -> Result<(), UpdateError> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| UpdateError::NotWritable(executable.display().to_string()))?;

    if !is_writable(directory) {
        return Err(UpdateError::NotWritable(directory.display().to_string()));
    }

    let metadata = curl_text(&endpoint(API_OVERRIDE, API))?;
    let tag = parse_tag(&metadata).ok_or(UpdateError::NoReleases)?;
    let version = tag.trim_start_matches('v').to_string();

    if !is_newer(&version, CURRENT) {
        println!("mh-overlay {CURRENT} ya es la última versión publicada ({tag}).");
        return Ok(());
    }

    let arch = std::env::consts::ARCH.to_string();
    let name = asset_name(&version, &arch);
    println!("actualizando de {CURRENT} a {version}…");

    let temporary = staging_dir()?;
    let archive = temporary.join(&name);
    let checksum = temporary.join(format!("{name}.sha256"));

    let downloads = endpoint(DOWNLOADS_OVERRIDE, DOWNLOADS);
    curl_file(&format!("{downloads}/{tag}/{name}"), &archive)?;
    curl_file(&format!("{downloads}/{tag}/{name}.sha256"), &checksum)?;
    verify_checksum(&temporary, &checksum)?;

    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&temporary)
        .status()?;
    if !status.success() {
        return Err(UpdateError::Unpack("`tar` terminó con error".into()));
    }

    let extracted = temporary.join(format!("mh-overlay-{version}-linux-{arch}"));
    if !extracted.is_dir() {
        return Err(UpdateError::Unpack(format!(
            "el paquete no trae la carpeta {}",
            extracted.display()
        )));
    }

    let capabilities = file_capabilities(&executable);
    install_from_dir(&extracted, directory)?;

    if let Some(capabilities) = capabilities {
        restore_capabilities(&executable, &capabilities);
    }

    let updated = Command::new(&executable).arg("--version").output();
    match updated {
        Ok(output) if output.status.success() => println!(
            "actualizado: {}",
            String::from_utf8_lossy(&output.stdout).trim()
        ),
        _ => println!("actualizado en {}", executable.display()),
    }

    let _ = fs::remove_dir_all(&temporary);
    Ok(())
}

/// URL configurada por variable de entorno, si la hay.
fn endpoint(variable: &str, default: &str) -> String {
    std::env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// Nombre del paquete que publica `scripts/package.sh`.
fn asset_name(version: &str, arch: &str) -> String {
    format!("mh-overlay-{version}-linux-{arch}.tar.gz")
}

/// Extrae `"tag_name": "v1.2.3"` del JSON de la API sin añadir un analizador.
fn parse_tag(json: &str) -> Option<String> {
    let key = "\"tag_name\"";
    let rest = &json[json.find(key)? + key.len()..];
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;

    Some(rest[..end].to_string())
}

/// Compara versiones `x.y.z` numéricamente; el sufijo de una etiqueta no cuenta.
fn is_newer(latest: &str, current: &str) -> bool {
    fn parts(version: &str) -> Vec<u64> {
        version
            .trim_start_matches('v')
            .split(['-', '+'])
            .next()
            .unwrap_or_default()
            .split('.')
            .map(|part| part.parse().unwrap_or(0))
            .collect()
    }

    let latest = parts(latest);
    let current = parts(current);

    for index in 0..latest.len().max(current.len()) {
        let next = latest.get(index).copied().unwrap_or(0);
        let now = current.get(index).copied().unwrap_or(0);

        if next != now {
            return next > now;
        }
    }

    false
}

/// Reemplaza el binario y los mapas de la instalación con los del paquete descomprimido.
///
/// El binario se escribe al lado con otro nombre y se renombra encima: así una actualización a
/// mitad de cacería no deja un ejecutable truncado.
fn install_from_dir(staged: &Path, directory: &Path) -> Result<(), UpdateError> {
    let binary = staged.join("mh-overlay");
    let destination = directory.join("mh-overlay");

    if !binary.is_file() {
        return Err(UpdateError::Unpack(format!(
            "el paquete no trae {}",
            binary.display()
        )));
    }

    let pending = directory.join(".mh-overlay.new");
    fs::copy(&binary, &pending)?;
    fs::set_permissions(&pending, fs::Permissions::from_mode(0o755))?;
    fs::rename(&pending, &destination)?;

    let maps = staged.join("data");
    if maps.is_dir() {
        let target = directory.join("data");
        let previous = directory.join(".data.old");

        let _ = fs::remove_dir_all(&previous);
        if target.is_dir() {
            fs::rename(&target, &previous)?;
        }
        fs::rename(&maps, &target)?;
        let _ = fs::remove_dir_all(&previous);
    }

    Ok(())
}

fn staging_dir() -> Result<PathBuf, UpdateError> {
    let directory = std::env::temp_dir().join(format!("mh-overlay-update-{}", std::process::id()));

    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory)?;

    Ok(directory)
}

fn curl_text(url: &str) -> Result<String, UpdateError> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: mh-overlay",
            url,
        ])
        .output()?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();

        // Un 404 en la API significa que el repositorio no tiene releases (o no es público),
        // que es un caso normal mientras no se publique la primera.
        if message.contains("404") {
            return Err(UpdateError::NoReleases);
        }

        return Err(UpdateError::Network(message));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn curl_file(url: &str, destination: &Path) -> Result<(), UpdateError> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "User-Agent: mh-overlay",
            "-o",
            &destination.display().to_string(),
            url,
        ])
        .output()?;

    if !output.status.success() {
        return Err(UpdateError::NoPackage {
            arch: std::env::consts::ARCH.to_string(),
            name: url.to_string(),
        });
    }

    Ok(())
}

fn verify_checksum(directory: &Path, checksum: &Path) -> Result<(), UpdateError> {
    let status = Command::new("sha256sum")
        .arg("-c")
        .arg(checksum)
        .arg("--quiet")
        .current_dir(directory)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(UpdateError::Checksum)
    }
}

/// Capacidades del binario actual (`cap_sys_ptrace` es la que se necesita para leer la memoria).
///
/// Reemplazar el archivo crea un inodo nuevo y pierde lo que puso `setcap`, así que se vuelve a
/// aplicar si se puede y, si no, se avisa con la orden exacta.
fn file_capabilities(path: &Path) -> Option<String> {
    let output = Command::new("getcap").arg(path).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    text.contains("cap_sys_ptrace").then_some(text)
}

fn restore_capabilities(path: &Path, capabilities: &str) {
    let applied = Command::new("setcap")
        .arg("cap_sys_ptrace+ep")
        .arg(path)
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if !applied {
        println!(
            "aviso: el binario nuevo perdió las capacidades ({capabilities}).\n       \
             vuelve a aplicarlas una vez: sudo setcap cap_sys_ptrace+ep {}",
            path.display()
        );
    }
}

fn is_writable(directory: &Path) -> bool {
    // `nix` no está en el árbol y `access(2)` es la comprobación que hace el propio `rename`.
    let path = directory.join(".mh-overlay-probe");
    match fs::write(&path, b"") {
        Ok(()) => {
            let _ = fs::remove_file(path);
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tag_is_read_from_the_release_json() {
        assert_eq!(
            parse_tag(r#"{"tag_name": "v0.2.0", "name": "0.2.0"}"#),
            Some("v0.2.0".to_string())
        );
        assert_eq!(parse_tag(r#"{"message": "Not Found"}"#), None);
    }

    #[test]
    fn versions_are_compared_number_by_number() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
        assert!(!is_newer("0.2.0-rc1", "0.2.0"));
    }

    #[test]
    fn the_asset_matches_the_packager() {
        assert_eq!(
            asset_name("0.2.0", "x86_64"),
            "mh-overlay-0.2.0-linux-x86_64.tar.gz"
        );
    }

    #[test]
    fn installing_replaces_the_binary_and_the_maps() {
        let root = std::env::temp_dir().join(format!("mh-overlay-install-{}", std::process::id()));
        let staged = root.join("staged");
        let installed = root.join("installed");

        fs::create_dir_all(staged.join("data/maps")).unwrap();
        fs::create_dir_all(installed.join("data/maps")).unwrap();
        fs::write(staged.join("mh-overlay"), b"nuevo").unwrap();
        fs::write(staged.join("data/maps/nuevo.map"), b"mapa").unwrap();
        fs::write(installed.join("data/maps/viejo.map"), b"mapa").unwrap();

        install_from_dir(&staged, &installed).unwrap();

        assert_eq!(fs::read(installed.join("mh-overlay")).unwrap(), b"nuevo");
        assert!(installed.join("data/maps/nuevo.map").is_file());
        assert!(!installed.join("data/maps/viejo.map").exists());
        assert!(!installed.join(".mh-overlay.new").exists());

        let mode = fs::metadata(installed.join("mh-overlay"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o755);

        let _ = fs::remove_dir_all(&root);
    }
}
