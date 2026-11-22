//! `--update` de punta a punta contra una release falsa servida por `file://`: sin red, sin
//! `minisign` instalado y sin tocar el `$HOME` real.
//!
//! Cubre la propiedad que importa: el paquete llega al destino solo cuando las comprobaciones
//! pasan, y un paquete rechazado deja el archivo anterior intacto.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VERSION: &str = "9.9.9";
const ASSET: &str = "mh-overlay-9.9.9-linux-x86_64.AppImage";

fn fake_release(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("mh-update-e2e-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let assets = root.join("release").join(format!("v{VERSION}"));
    fs::create_dir_all(&assets).unwrap();
    fs::create_dir_all(root.join("home")).unwrap();
    fs::write(assets.join(ASSET), b"test").unwrap();
    write_checksum(&assets);
    fs::write(root.join("release.json"), r#"{"tag_name":"v9.9.9"}"#).unwrap();
    fs::write(root.join("mh-overlay.AppImage"), b"viejo").unwrap();

    root
}

fn write_checksum(assets: &Path) {
    let hash = Command::new("sha256sum")
        .arg(ASSET)
        .current_dir(assets)
        .output()
        .unwrap();
    assert!(hash.status.success());

    fs::write(assets.join(format!("{ASSET}.sha256")), hash.stdout).unwrap();
}

fn update(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mh-overlay"))
        .arg("--update")
        .env("APPIMAGE", root.join("mh-overlay.AppImage"))
        .env("HOME", root.join("home"))
        .env(
            "MH_OVERLAY_UPDATE_API",
            format!("file://{}", root.join("release.json").display()),
        )
        .env(
            "MH_OVERLAY_UPDATE_DOWNLOADS",
            format!("file://{}", root.join("release").display()),
        )
        .output()
        .unwrap()
}

/// Si el binario compilado lleva la clave pública, la firma es obligatoria; si no, se queda en el
/// SHA-256. El test se adapta para no depender de si la clave ya se pegó.
fn verifies_signatures() -> bool {
    let doctor = Command::new(env!("CARGO_BIN_EXE_mh-overlay"))
        .arg("--doctor")
        .env("HOME", std::env::temp_dir())
        .output()
        .unwrap();

    String::from_utf8_lossy(&doctor.stdout).contains("clave del proyecto compilada")
}

#[test]
fn a_release_without_signature_only_installs_when_the_binary_cannot_verify() {
    let root = fake_release("sin-firma");
    let output = update(&root);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let installed = fs::read(root.join("mh-overlay.AppImage")).unwrap();

    if verifies_signatures() {
        assert_eq!(output.status.code(), Some(1), "{stdout}");
        assert_eq!(installed, b"viejo", "se instaló un paquete sin firma");
    } else {
        assert_eq!(output.status.code(), Some(0), "{stdout}");
        assert!(stdout.contains("aviso:"), "{stdout}");
        assert_eq!(installed, b"test");
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_checksum_that_does_not_match_never_touches_the_target() {
    let root = fake_release("checksum");
    let assets = root.join("release").join(format!("v{VERSION}"));
    fs::write(
        assets.join(format!("{ASSET}.sha256")),
        format!("{}  {ASSET}\n", "0".repeat(64)),
    )
    .unwrap();

    let output = update(&root);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("SHA-256"));
    assert_eq!(
        fs::read(root.join("mh-overlay.AppImage")).unwrap(),
        b"viejo"
    );

    let _ = fs::remove_dir_all(&root);
}
