//! Actualización en sitio desde las releases de GitHub.
//!
//! La instalación no se desinstala: `--update` baja el paquete de la última release, comprueba su
//! SHA-256, verifica la firma minisign con la clave del proyecto y reemplaza lo que haya en su
//! sitio. Si el programa corre desde un AppImage (`$APPIMAGE` lo dice), el archivo que se
//! reemplaza es la imagen entera; si corre desde el tarball instalado, el binario y los mapas que
//! están junto a él. El archivo de configuración vive en `$XDG_CONFIG_HOME`, fuera de la
//! instalación, así que se conserva tal cual.
//!
//! No se añade un cliente HTTP: `curl`, `tar` y `sha256sum` están en cualquier escritorio Linux y
//! evitan meter TLS y un analizador de JSON en el binario del overlay. La firma sí se verifica en
//! proceso: `minisign` no viene instalado en ningún escritorio base, y pedirle al usuario que lo
//! instale antes de actualizar convertiría la comprobación en opcional.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use minisign_verify::{PublicKey, Signature};

/// Repositorio del que se publican los paquetes.
pub const REPOSITORY: &str = "SyberFoxx/mh-overlay";
const API: &str = "https://api.github.com/repos/SyberFoxx/mh-overlay/releases/latest";
const DOWNLOADS: &str = "https://github.com/SyberFoxx/mh-overlay/releases/download";
/// Variables para probar el actualizador contra un servidor local y para usar un espejo.
const API_OVERRIDE: &str = "MH_OVERLAY_UPDATE_API";
const DOWNLOADS_OVERRIDE: &str = "MH_OVERLAY_UPDATE_DOWNLOADS";
/// Versión del binario que está ejecutando la actualización.
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// Clave pública minisign del proyecto, tal cual se publica en `minisign.pub`.
///
/// Se puede pegar el archivo entero (con su línea `untrusted comment:`) o solo el base64. Mientras
/// esté vacía no hay con qué saber quién publicó el paquete, así que la actualización se queda en
/// el SHA-256 y lo avisa; en cuanto tenga valor la firma pasa a ser obligatoria y una release sin
/// `.minisig`, o con una firma que no corresponda, no se instala.
const PUBLIC_KEY: &str = "RWSiP1TP7hUZZxIX34xkvP/goq0S6gAVkbrY1M2r31zs9sV5mG2bW6ap";

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
    #[error(
        "la release {tag} no publica la firma de {name}; no se instala nada sin poder comprobarla"
    )]
    SignatureMissing { tag: String, name: String },
    #[error("no se pudo bajar la firma del paquete; reintenta la actualización: {0}")]
    SignatureDownload(String),
    #[error("la firma del paquete no verifica: {0}")]
    Signature(String),
    #[error("no se pudo descomprimir el paquete: {0}")]
    Unpack(String),
    #[error(
        "no se puede escribir en {0}; mueve el AppImage a una carpeta tuya o actualiza con permisos sobre esa carpeta"
    )]
    NotWritable(String),
    #[error("el AppImage indicado por $APPIMAGE ya no existe: {0}")]
    MissingAppImage(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Forma en la que está instalado el programa, que decide qué archivo se reemplaza.
enum Install {
    /// Imagen portable: se reemplaza el archivo que el runtime señala en `$APPIMAGE`.
    AppImage(PathBuf),
    /// Paquete instalado: se reemplazan el binario y `data/` junto a él.
    Directory(PathBuf),
}

impl Install {
    /// El AppImage se reconoce por la variable que pone su propio runtime, no por la ruta del
    /// ejecutable: dentro de la imagen montada el binario es de solo lectura.
    fn detect() -> Result<Self, UpdateError> {
        if let Some(path) = std::env::var_os("APPIMAGE").map(PathBuf::from) {
            if !path.is_file() {
                return Err(UpdateError::MissingAppImage(path.display().to_string()));
            }

            return Ok(Self::AppImage(path));
        }

        let executable = std::env::current_exe()?;
        let directory = executable
            .parent()
            .ok_or_else(|| UpdateError::NotWritable(executable.display().to_string()))?;

        Ok(Self::Directory(directory.to_path_buf()))
    }

    /// Carpeta donde vive lo que hay que reemplazar.
    fn directory(&self) -> &Path {
        match self {
            Self::AppImage(path) => path.parent().unwrap_or(Path::new(".")),
            Self::Directory(directory) => directory,
        }
    }

    /// Nombre del archivo que publica `scripts/package.sh` o `scripts/appimage.sh`.
    fn asset_name(&self, version: &str, arch: &str) -> String {
        match self {
            Self::AppImage(_) => format!("mh-overlay-{version}-linux-{arch}.AppImage"),
            Self::Directory(_) => format!("mh-overlay-{version}-linux-{arch}.tar.gz"),
        }
    }
}

/// Datos de la última release que el hub enseña en Novedades.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseNotes {
    pub tag: String,
    pub version: String,
    pub published: String,
    pub body: String,
}

/// Etiqueta y versión de la última release publicada.
fn latest() -> Result<(String, String), UpdateError> {
    let metadata = curl_text(&endpoint(API_OVERRIDE, API))?;
    let tag = parse_tag(&metadata).ok_or(UpdateError::NoReleases)?;
    let version = tag.trim_start_matches('v').to_string();

    Ok((tag, version))
}

/// Lee la etiqueta, la fecha y el cuerpo de la última release.
pub fn notes() -> Result<ReleaseNotes, UpdateError> {
    let metadata = curl_text(&endpoint(API_OVERRIDE, API))?;
    let tag = json_string(&metadata, "tag_name").ok_or(UpdateError::NoReleases)?;
    let version = tag.trim_start_matches('v').to_string();

    Ok(ReleaseNotes {
        tag,
        version,
        published: json_string(&metadata, "published_at").unwrap_or_default(),
        body: json_string(&metadata, "body").unwrap_or_default(),
    })
}

/// Versión publicada más nueva que esta, sin descargar nada.
pub fn check() -> Result<Option<String>, UpdateError> {
    let (_tag, version) = latest()?;

    Ok(is_newer(&version, CURRENT).then_some(version))
}

/// Comprueba si hay una versión nueva y, si la hay, la instala en el sitio actual.
pub fn run() -> Result<(), UpdateError> {
    let install = Install::detect()?;
    let directory = install.directory();

    if !is_writable(directory) {
        return Err(UpdateError::NotWritable(directory.display().to_string()));
    }

    let (tag, version) = latest()?;

    if !is_newer(&version, CURRENT) {
        println!("mh-overlay {CURRENT} ya es la última versión publicada ({tag}).");
        return Ok(());
    }

    let arch = std::env::consts::ARCH.to_string();
    let name = install.asset_name(&version, &arch);
    println!("actualizando de {CURRENT} a {version}…");

    let staging = Staging::new()?;
    let temporary = staging.path();
    let archive = temporary.join(&name);
    let checksum = temporary.join(format!("{name}.sha256"));

    let downloads = endpoint(DOWNLOADS_OVERRIDE, DOWNLOADS);
    curl_file(&format!("{downloads}/{tag}/{name}"), &archive)?;
    curl_file(&format!("{downloads}/{tag}/{name}.sha256"), &checksum)?;
    verify_checksum(temporary, &checksum)?;

    match configured_key() {
        Some(public_key) => {
            let signature = temporary.join(format!("{name}.minisig"));
            let fetched = curl_signature(&format!("{downloads}/{tag}/{name}.minisig"), &signature)?;

            match fetched {
                SignatureFetch::Published => {}
                SignatureFetch::NotPublished => {
                    return Err(UpdateError::SignatureMissing {
                        tag: tag.clone(),
                        name: name.clone(),
                    });
                }
                SignatureFetch::Failed(reason) => {
                    return Err(UpdateError::SignatureDownload(reason));
                }
            }

            verify_file(&archive, &signature, public_key)?;
            println!("firma del paquete verificada con la clave del proyecto");
        }
        None => println!(
            "aviso: este binario se compiló sin la clave pública del proyecto, así que el paquete \
             se comprueba por SHA-256 pero no por quién lo publicó."
        ),
    }

    match &install {
        Install::AppImage(target) => {
            install_appimage(&archive, target)?;
            println!("actualizado: {}", target.display());
            println!("cierra y vuelve a abrir mh-overlay para usar la versión nueva.");
        }
        Install::Directory(directory) => {
            let status = Command::new("tar")
                .arg("-xzf")
                .arg(&archive)
                .arg("-C")
                .arg(temporary)
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

            let executable = directory.join("mh-overlay");
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
        }
    }

    Ok(())
}

/// URL configurada por variable de entorno, si la hay.
fn endpoint(variable: &str, default: &str) -> String {
    std::env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// Extrae `"tag_name": "v1.2.3"` del JSON de la API sin añadir un analizador.
fn parse_tag(json: &str) -> Option<String> {
    json_string(json, "tag_name")
}

/// Campo de texto de un objeto JSON; deshace los escapes que usa la API en el cuerpo.
fn json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let rest = &json[json.find(&needle)? + needle.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut chars = rest.chars();
    let mut out = String::new();

    while let Some(character) = chars.next() {
        match character {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000c}'),
                'u' => {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        hex.push(chars.next()?);
                    }
                    let code = u32::from_str_radix(&hex, 16).ok()?;
                    out.push(char::from_u32(code)?);
                }
                _ => return None,
            },
            other => out.push(other),
        }
    }

    None
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
fn install_appimage(image: &Path, target: &Path) -> Result<(), UpdateError> {
    let directory = target
        .parent()
        .ok_or_else(|| UpdateError::NotWritable(target.display().to_string()))?;

    // Se copia dentro de la misma carpeta y se renombra encima: el `rename` es atómico y el
    // proceso que está corriendo sigue leyendo el inodo viejo hasta que se cierra.
    let pending = directory.join(format!(".mh-overlay-{}.AppImage", std::process::id()));
    let result = fs::copy(image, &pending)
        .and_then(|_| fs::set_permissions(&pending, fs::Permissions::from_mode(0o755)))
        .and_then(|_| fs::rename(&pending, target));

    if let Err(error) = result {
        let _ = fs::remove_file(&pending);
        return Err(UpdateError::Io(error));
    }

    Ok(())
}

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

/// Carpeta temporal de la actualización, que se borra sola.
///
/// Sin esto, un fallo después de bajar el paquete dejaba el artefacto descargado (y rechazado)
/// en `$TMPDIR` hasta la siguiente ejecución, porque el `return` se saltaba la limpieza.
struct Staging(PathBuf);

impl Staging {
    fn new() -> Result<Self, UpdateError> {
        Ok(Self(staging_dir()?))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
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

/// Qué pasó al intentar bajar la firma. Una release que no la publica es un caso distinto de
/// una descarga que falló: el primero se arregla esperando otra release, el segundo reintentando.
#[derive(Debug)]
enum SignatureFetch {
    Published,
    NotPublished,
    Failed(String),
}

fn curl_signature(url: &str, destination: &Path) -> Result<SignatureFetch, UpdateError> {
    // Sin `-f` a propósito: con `-f`, un 404 (la release no publica la firma) y un 500 (el
    // servidor está roto) salen con el mismo código, y confundirlos manda a buscar el problema
    // donde no está.
    let output = Command::new("curl")
        .args([
            "-sSL",
            "-H",
            "User-Agent: mh-overlay",
            "-o",
            &destination.display().to_string(),
            "-w",
            "%{http_code}",
            url,
        ])
        .output()?;

    if !output.status.success() {
        let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();

        // Con `file://` (el espejo de pruebas) no hay código HTTP: si curl no puede abrir el
        // archivo, es que no está.
        return Ok(if url.starts_with("file://") {
            SignatureFetch::NotPublished
        } else {
            SignatureFetch::Failed(reason)
        });
    }

    // `000` es la respuesta de los esquemas sin HTTP (`file://`).
    Ok(classify_status(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

/// Traduce el código HTTP que devuelve `curl -w` a lo que hay que hacer con la firma.
///
/// Separado de la descarga para poder fijarlo con un test: confundir «no está publicada» con
/// «no se pudo bajar» manda a diagnosticar el problema en el sitio equivocado.
fn classify_status(status: &str) -> SignatureFetch {
    match status {
        "404" | "410" => SignatureFetch::NotPublished,
        "200" | "000" => SignatureFetch::Published,
        other => SignatureFetch::Failed(format!("HTTP {other}")),
    }
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

/// Clave pública compilada en el binario, si la hay.
fn configured_key() -> Option<&'static str> {
    compiled_key(PUBLIC_KEY)
}

/// Normaliza lo que haya en [`PUBLIC_KEY`]: espacios de más y una clave sin pegar dan `None`.
fn compiled_key(raw: &'static str) -> Option<&'static str> {
    let key = raw.trim();

    (!key.is_empty()).then_some(key)
}

/// Lee la clave pública escrita como `minisign.pub` o como su base64 suelto.
///
/// Se aceptan las dos formas porque quien la pega no va a mirar el formato.
fn parse_key(raw: &str) -> Result<PublicKey, minisign_verify::Error> {
    PublicKey::decode(raw).or_else(|_| PublicKey::from_base64(raw))
}

/// Estado de la firma en el binario que se está ejecutando, para `--doctor`.
pub enum SignatureKey {
    /// No lleva clave: la actualización se queda en el SHA-256 y lo avisa.
    Missing,
    /// La clave se lee: la firma es obligatoria al actualizar.
    Ready,
    /// Hay algo pegado, pero no es una clave: la actualización fallaría, así que se dice antes.
    Unreadable(String),
}

/// Lo que puede comprobar este binario al actualizarse.
pub fn signature_key() -> SignatureKey {
    let Some(raw) = configured_key() else {
        return SignatureKey::Missing;
    };

    match parse_key(raw) {
        Ok(_) => SignatureKey::Ready,
        Err(error) => SignatureKey::Unreadable(error.to_string()),
    }
}

/// Comprueba que `signature` corresponde a `archive` con la clave pública del proyecto.
///
/// `minisign -S` firma con pre-hash desde hace años, así que las firmas en el formato antiguo se
/// rechazan: aceptarlas solo amplía la superficie sin cubrir ningún caso real de la release.
fn verify_file(archive: &Path, signature: &Path, public_key: &str) -> Result<(), UpdateError> {
    let key = parse_key(public_key).map_err(|error| {
        UpdateError::Signature(format!("la clave pública no se pudo leer: {error}"))
    })?;
    let signature = Signature::from_file(signature)
        .map_err(|error| UpdateError::Signature(format!("la firma no se pudo leer: {error}")))?;
    let content = fs::read(archive)?;

    key.verify(&content, &signature, false)
        .map_err(|error| UpdateError::Signature(error.to_string()))
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
    fn release_notes_unescape_the_json_body() {
        let json = r#"{"tag_name":"v0.2.0","published_at":"2026-11-20T14:39:43Z","body":"Cambios\n- Vida\n- Cola"}"#;
        let notes = json_string(json, "body").unwrap();

        assert_eq!(notes, "Cambios\n- Vida\n- Cola");
        assert_eq!(
            json_string(json, "published_at").unwrap(),
            "2026-11-20T14:39:43Z"
        );
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
        let installed = Install::Directory(PathBuf::from("/tmp/mh-overlay"));
        let portable = Install::AppImage(PathBuf::from("/tmp/mh-overlay.AppImage"));

        assert_eq!(
            installed.asset_name("0.2.0", "x86_64"),
            "mh-overlay-0.2.0-linux-x86_64.tar.gz"
        );
        assert_eq!(
            portable.asset_name("0.2.0", "x86_64"),
            "mh-overlay-0.2.0-linux-x86_64.AppImage"
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

    #[test]
    fn updating_an_appimage_replaces_the_file_atomically() {
        let root = std::env::temp_dir().join(format!("mh-overlay-appimage-{}", std::process::id()));
        let target = root.join("mh-overlay.AppImage");
        let download = root.join("download.AppImage");

        fs::create_dir_all(&root).unwrap();
        fs::write(&target, b"viejo").unwrap();
        fs::write(&download, b"nuevo").unwrap();

        install_appimage(&download, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"nuevo");
        assert!(!root.read_dir().unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".mh-overlay-")
        }));

        let mode = fs::metadata(&target).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);

        let _ = fs::remove_dir_all(&root);
    }

    /// Vector publicado por `minisign-verify`: una clave, una firma y su contenido reales, así
    /// que el test recorre Ed25519 de verdad y no un doble.
    const SAMPLE_KEY: &str = "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
    const SAMPLE_SIGNATURE: &str = concat!(
        "untrusted comment: signature from minisign secret key\n",
        "RUQf6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/",
        "z7SrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\n",
        "trusted comment: timestamp:1633700835\tfile:test\tprehashed\n",
        "wLMDjy9FLAuxZ3q4NlEvkgtyhrr0gtTu6KC4KBJdITbbOeAi1zBIYo0v4iTgt8jJpIidRJnp94ABQkJAgAooBQ==\n",
    );
    /// Lo que firma ese vector.
    const SAMPLE_PAYLOAD: &[u8] = b"test";

    fn signature_dir(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("mh-overlay-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn a_package_signed_with_the_project_key_verifies() {
        let root = signature_dir("firma-ok");
        let archive = root.join("paquete.AppImage");
        let signature = root.join("paquete.AppImage.minisig");

        fs::write(&archive, SAMPLE_PAYLOAD).unwrap();
        fs::write(&signature, SAMPLE_SIGNATURE).unwrap();

        assert!(verify_file(&archive, &signature, SAMPLE_KEY).is_ok());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_package_that_does_not_match_the_signature_is_rejected() {
        let root = signature_dir("firma-manipulada");
        let archive = root.join("paquete.AppImage");
        let signature = root.join("paquete.AppImage.minisig");

        // Un byte distinto es todo lo que hace falta para que la firma deje de cubrir el archivo.
        fs::write(&archive, b"tesT").unwrap();
        fs::write(&signature, SAMPLE_SIGNATURE).unwrap();

        assert!(verify_file(&archive, &signature, SAMPLE_KEY).is_err());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_signature_that_was_altered_is_rejected() {
        let root = signature_dir("firma-alterada");
        let archive = root.join("paquete.AppImage");
        let signature = root.join("paquete.AppImage.minisig");

        fs::write(&archive, SAMPLE_PAYLOAD).unwrap();
        fs::write(&signature, SAMPLE_SIGNATURE.replace("RUQf6LRC", "RUQf6LRD")).unwrap();

        assert!(verify_file(&archive, &signature, SAMPLE_KEY).is_err());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_whole_minisign_pub_file_is_accepted_as_the_key() {
        let root = signature_dir("clave-con-comentario");
        let archive = root.join("paquete.AppImage");
        let signature = root.join("paquete.AppImage.minisig");
        let pasted = format!("untrusted comment: minisign public key 1\n{SAMPLE_KEY}\n");

        fs::write(&archive, SAMPLE_PAYLOAD).unwrap();
        fs::write(&signature, SAMPLE_SIGNATURE).unwrap();

        assert!(verify_file(&archive, &signature, &pasted).is_ok());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn an_empty_key_means_there_is_nothing_to_verify_with() {
        assert_eq!(compiled_key(""), None);
        assert_eq!(compiled_key("  \n\t "), None);
        assert_eq!(compiled_key(SAMPLE_KEY), Some(SAMPLE_KEY));
    }

    #[test]
    fn a_missing_signature_is_reported_as_such() {
        let error = UpdateError::SignatureMissing {
            tag: "v9.9.9".to_string(),
            name: "mh-overlay-9.9.9-linux-x86_64.AppImage".to_string(),
        }
        .to_string();

        assert!(error.contains("no publica la firma"), "{error}");
        assert!(error.contains("v9.9.9"), "{error}");
    }

    /// Misma clave y mismo contenido, pero firmado sin pre-hash: el formato que `minisign -S` ya
    /// no produce. Aceptarlo ampliaría la superficie sin cubrir ninguna release real.
    const LEGACY_SIGNATURE: &str = concat!(
        "untrusted comment: signature from minisign secret key\n",
        "RWQf6LRCGA9i59SLOFxz6NxvASXDJeRtuZykwQepbDEGt87ig1BNpWaVWuNrm73YiIiJbq71Wi+dP9eKL8OC351vwIasSSbXxwA=\n",
        "trusted comment: timestamp:1555779966\tfile:test\n",
        "QtKMXWyYcwdpZAlPF7tE2ENJkRd1ujvKjlj1m9RtHTBnZPa5WKU5uWRs5GoP5M/VqE81QFuMKI5k/SfNQUaOAA==\n",
    );
    /// La firma del vector con el identificador de clave alterado: la matemática Ed25519 ni
    /// siquiera llega a ejecutarse, así que solo un chequeo explícito del identificador la para.
    const OTHER_KEY_ID_SIGNATURE: &str = concat!(
        "untrusted comment: signature from minisign secret key\n",
        "RUTg6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/",
        "z7SrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\n",
        "trusted comment: timestamp:1633700835\tfile:test\tprehashed\n",
        "wLMDjy9FLAuxZ3q4NlEvkgtyhrr0gtTu6KC4KBJdITbbOeAi1zBIYo0v4iTgt8jJpIidRJnp94ABQkJAgAooBQ==\n",
    );

    fn signed_pair(name: &str, signature: &str) -> (PathBuf, PathBuf, PathBuf) {
        let root = signature_dir(name);
        let archive = root.join("paquete.AppImage");
        let file = root.join("paquete.AppImage.minisig");

        fs::write(&archive, SAMPLE_PAYLOAD).unwrap();
        fs::write(&file, signature).unwrap();

        (root, archive, file)
    }

    #[test]
    fn a_legacy_signature_is_rejected() {
        let (root, archive, signature) = signed_pair("firma-legacy", LEGACY_SIGNATURE);
        let error = verify_file(&archive, &signature, SAMPLE_KEY)
            .unwrap_err()
            .to_string();

        assert!(error.to_lowercase().contains("algorithm"), "{error}");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_signature_with_another_key_id_is_rejected_as_such() {
        let (root, archive, signature) = signed_pair("otro-key-id", OTHER_KEY_ID_SIGNATURE);
        let error = verify_file(&archive, &signature, SAMPLE_KEY)
            .unwrap_err()
            .to_string();

        assert!(error.contains("different key"), "{error}");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_published_signature_is_fetched_and_a_missing_one_is_told_apart() {
        let (root, _archive, signature) = signed_pair("curl", SAMPLE_SIGNATURE);
        let destination = root.join("descargada.minisig");

        let fetched = curl_signature(&format!("file://{}", signature.display()), &destination);
        assert!(matches!(fetched, Ok(SignatureFetch::Published)));
        assert_eq!(fs::read(&destination).unwrap(), SAMPLE_SIGNATURE.as_bytes());

        let missing = curl_signature(
            &format!("file://{}", root.join("no-existe").display()),
            &root.join("otra.minisig"),
        );
        assert!(matches!(missing, Ok(SignatureFetch::NotPublished)));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_signature_that_cannot_be_downloaded_is_not_reported_as_missing() {
        let root = signature_dir("curl-roto");
        let result = curl_signature("esquema-invalido://x.minisig", &root.join("x.minisig"));

        // Un fallo de transporte no es «la release no la publica»: reintentar sí sirve y el
        // mensaje tiene que decir eso.
        assert!(
            matches!(result, Ok(SignatureFetch::Failed(_))),
            "{result:?}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_key_that_is_not_a_key_is_reported_instead_of_silently_failing_later() {
        assert!(parse_key("no es una clave").is_err());
        assert!(parse_key("RWSiP1TP7hUZZxIX34xkvP/goq0S6gAVkbrY1M2r31zs9sV5mG2bW6ap").is_ok());
        assert!(
            parse_key("untrusted comment: minisign public key 1\nRWSiP1TP7hUZZxIX34xkvP/goq0S6gAVkbrY1M2r31zs9sV5mG2bW6ap").is_ok()
        );
    }

    #[test]
    fn a_missing_signature_and_a_broken_server_are_told_apart() {
        // 404 y 410 son «la release no la publica»; 500 y el resto son «reintenta».
        assert!(matches!(
            classify_status("404"),
            SignatureFetch::NotPublished
        ));
        assert!(matches!(
            classify_status("410"),
            SignatureFetch::NotPublished
        ));
        assert!(matches!(classify_status("500"), SignatureFetch::Failed(_)));
        assert!(matches!(classify_status("502"), SignatureFetch::Failed(_)));
        assert!(matches!(classify_status("200"), SignatureFetch::Published));
        // `file://` no tiene código HTTP y curl informa `000`.
        assert!(matches!(classify_status("000"), SignatureFetch::Published));
    }
}
