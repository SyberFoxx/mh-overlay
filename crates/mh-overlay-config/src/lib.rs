//! Configuración del overlay: valores por defecto, archivo TOML y validación.
//!
//! El archivo vive en `$XDG_CONFIG_HOME/mh-overlay/config.toml` (o `$MH_OVERLAY_CONFIG` si se
//! define) y se relee cuando cambia, así que editar el TOML con el juego abierto se aplica sin
//! reiniciar. Un archivo que no parsea o trae un valor imposible es un error: aplicar valores a
//! medias esconde la errata que el usuario acaba de escribir.

mod color;
mod position;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mh_overlay_games::{GameKind, GameKindError};
use serde::{Deserialize, Serialize};

pub use color::Color;
pub use position::Position;

/// Archivo de configuración documentado que se escribe la primera vez.
pub const TEMPLATE: &str = r##"# mh-overlay · configuración
#
# Se relee sola al guardar: no hace falta reiniciar el overlay.
# Los colores son #rgb, #rrggbb o #rrggbbaa (el alfa permite paneles translúcidos).

[game]
# rise o world; se usa si no hay ningún juego abierto o si auto_detect = false.
kind = "rise"
# Detectar si el juego abierto es Rise o World. Elegir juego a mano lo desactiva.
auto_detect = true
# Cada cuántos milisegundos se lee la memoria del juego.
interval_ms = 500

[overlay]
# top-left | top-right | bottom-left | bottom-right | center
position = "top-left"
margin_x = 24
margin_y = 24
# Escala del panel: 0.5 a 3.0.
scale = 1.0
# Monstruos que se muestran a la vez: 1 a 8.
max_monsters = 4
show_parts = true
show_ailments = true
# Los clics atraviesan el panel en vez de quedarse en él.
click_through = true
# Ocultar el panel cuando no hay monstruos grandes en la zona.
hide_on_empty = true

[style]
panel = "#0b0d12e6"
text = "#f2f4f8"
muted = "#93a0b4"
health = "#7fd47f"
# Color de la vida por debajo de low_health_percent.
low_health = "#e2565c"
low_health_percent = 25.0
capture = "#e6b45e"
part = "#4f8dd6"
severable = "#d69a4f"
"##;

/// Sección `[game]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GameSection {
    /// `rise` o `world`; se valida con [`GameKind::from_str`].
    pub kind: String,
    /// Mirar qué ejecutable está corriendo antes de usar `kind`.
    pub auto_detect: bool,
    pub interval_ms: u64,
}

impl Default for GameSection {
    fn default() -> Self {
        Self {
            kind: "rise".to_string(),
            auto_detect: true,
            interval_ms: 500,
        }
    }
}

/// Sección `[overlay]`: dónde y cómo se dibuja el panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OverlaySection {
    pub position: Position,
    pub margin_x: i32,
    pub margin_y: i32,
    pub scale: f32,
    pub max_monsters: usize,
    pub show_parts: bool,
    pub show_ailments: bool,
    pub click_through: bool,
    pub hide_on_empty: bool,
}

impl Default for OverlaySection {
    fn default() -> Self {
        Self {
            position: Position::TopLeft,
            margin_x: 24,
            margin_y: 24,
            scale: 1.0,
            max_monsters: 4,
            show_parts: true,
            show_ailments: true,
            click_through: true,
            hide_on_empty: true,
        }
    }
}

/// Sección `[style]`: colores del panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StyleSection {
    pub panel: Color,
    pub text: Color,
    pub muted: Color,
    pub health: Color,
    pub low_health: Color,
    /// Porcentaje de vida por debajo del cual la barra usa `low_health`.
    pub low_health_percent: f32,
    pub capture: Color,
    pub part: Color,
    pub severable: Color,
}

impl Default for StyleSection {
    fn default() -> Self {
        Self {
            panel: Color::rgb(0x0b, 0x0d, 0x12).with_alpha(0xe6),
            text: Color::rgb(0xf2, 0xf4, 0xf8),
            muted: Color::rgb(0x93, 0xa0, 0xb4),
            health: Color::rgb(0x7f, 0xd4, 0x7f),
            low_health: Color::rgb(0xe2, 0x56, 0x5c),
            low_health_percent: 25.0,
            capture: Color::rgb(0xe6, 0xb4, 0x5e),
            part: Color::rgb(0x4f, 0x8d, 0xd6),
            severable: Color::rgb(0xd6, 0x9a, 0x4f),
        }
    }
}

/// Configuración completa del overlay.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub game: GameSection,
    pub overlay: OverlaySection,
    pub style: StyleSection,
}

/// De dónde salió la configuración aplicada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// El archivo existía y se leyó.
    File,
    /// No había archivo: se usan los valores por defecto.
    Defaults,
}

#[derive(Debug)]
pub struct Loaded {
    pub config: Config,
    pub origin: Origin,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("no se pudo leer {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("{path}: configuración inválida: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("no se pudo escribir {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("no se pudo serializar la configuración: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("valor inválido en {field}: {reason}")]
    Invalid { field: &'static str, reason: String },
}

impl Config {
    /// Ruta del archivo: `$MH_OVERLAY_CONFIG`, o `mh-overlay/config.toml` bajo el directorio de
    /// configuración del usuario.
    pub fn default_path() -> PathBuf {
        if let Some(path) = env::var_os("MH_OVERLAY_CONFIG") {
            return PathBuf::from(path);
        }

        let base = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));

        base.join("mh-overlay").join("config.toml")
    }

    /// Lee el archivo, o devuelve los valores por defecto si no existe.
    pub fn load_or_default(path: &Path) -> Result<Loaded, ConfigError> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Loaded {
                config: Self::from_toml(&text, &path.display().to_string())?,
                origin: Origin::File,
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Loaded {
                config: Self::default(),
                origin: Origin::Defaults,
            }),
            Err(source) => Err(ConfigError::Read {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    /// Interpreta el TOML y valida los valores.
    pub fn from_toml(text: &str, origin: &str) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: origin.to_string(),
            source,
        })?;

        config.validate()?;

        Ok(config)
    }

    /// Escribe la configuración en `path` de forma atómica (temporal + rename).
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = self.to_toml()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
                path: parent.display().to_string(),
                source,
            })?;
        }

        let temporary = path.with_extension("toml.tmp");
        fs::write(&temporary, text).map_err(|source| ConfigError::Write {
            path: temporary.display().to_string(),
            source,
        })?;

        fs::rename(&temporary, path).map_err(|source| ConfigError::Write {
            path: path.display().to_string(),
            source,
        })
    }

    /// Escribe el archivo documentado (`TEMPLATE`) si no existe todavía.
    pub fn write_template(path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
                path: parent.display().to_string(),
                source,
            })?;
        }

        fs::write(path, TEMPLATE).map_err(|source| ConfigError::Write {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Juego configurado, ya validado.
    pub fn game_kind(&self) -> Result<GameKind, ConfigError> {
        self.game
            .kind
            .parse()
            .map_err(|error: GameKindError| ConfigError::Invalid {
                field: "game.kind",
                reason: error.to_string(),
            })
    }

    /// Rechaza valores que no tienen sentido en pantalla.
    ///
    /// La CLI la llama después de aplicar sus propios flags, porque un valor absurdo por
    /// línea de comandos es tan error como uno en el TOML.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.game_kind()?;

        let invalid = |field: &'static str, reason: String| ConfigError::Invalid { field, reason };

        if !(16..=10_000).contains(&self.game.interval_ms) {
            return Err(invalid(
                "game.interval_ms",
                format!(
                    "{} está fuera de 16..10000; leer más rápido no aporta nada y satura el proceso",
                    self.game.interval_ms
                ),
            ));
        }

        if !self.overlay.scale.is_finite() || !(0.5..=3.0).contains(&self.overlay.scale) {
            return Err(invalid(
                "overlay.scale",
                format!("{} está fuera de 0.5..3.0", self.overlay.scale),
            ));
        }

        if !(1..=8).contains(&self.overlay.max_monsters) {
            return Err(invalid(
                "overlay.max_monsters",
                format!("{} está fuera de 1..8", self.overlay.max_monsters),
            ));
        }

        if !(0..=4096).contains(&self.overlay.margin_x)
            || !(0..=4096).contains(&self.overlay.margin_y)
        {
            return Err(invalid(
                "overlay.margin_x/margin_y",
                format!(
                    "{}x{} está fuera de 0..4096",
                    self.overlay.margin_x, self.overlay.margin_y
                ),
            ));
        }

        if !self.style.low_health_percent.is_finite()
            || !(0.0..=100.0).contains(&self.style.low_health_percent)
        {
            return Err(invalid(
                "style.low_health_percent",
                format!("{} está fuera de 0..100", self.style.low_health_percent),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_documented_template_is_the_default_configuration() {
        let parsed = Config::from_toml(TEMPLATE, "plantilla").unwrap();

        assert_eq!(parsed, Config::default());
    }

    #[test]
    fn a_typo_in_a_key_is_an_error_instead_of_being_ignored() {
        let error =
            Config::from_toml("[overlay]\npositionn = \"top-left\"\n", "prueba").unwrap_err();

        assert!(error.to_string().contains("positionn"), "{error}");
    }

    #[test]
    fn an_impossible_value_names_the_field() {
        let error = Config::from_toml("[overlay]\nscale = 12.0\n", "prueba").unwrap_err();

        assert!(error.to_string().contains("overlay.scale"), "{error}");
    }

    #[test]
    fn an_unknown_game_says_which_ones_exist() {
        let error = Config::from_toml("[game]\nkind = \"wilds\"\n", "prueba").unwrap_err();

        assert!(error.to_string().contains("rise, world"), "{error}");
    }

    #[test]
    fn a_bad_color_names_the_line_it_came_from() {
        let error = Config::from_toml("[style]\npanel = \"azul\"\n", "prueba").unwrap_err();

        assert!(error.to_string().contains("panel"), "{error}");
    }

    #[test]
    fn an_old_file_without_auto_detect_keeps_it_on() {
        let config = Config::from_toml("[game]\nkind = \"world\"\n", "prueba").unwrap();

        assert!(config.game.auto_detect);
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let directory = std::env::temp_dir().join(format!("mh-config-{}", std::process::id()));
        let path = directory.join("no-existe.toml");

        let loaded = Config::load_or_default(&path).unwrap();

        assert_eq!(loaded.origin, Origin::Defaults);
        assert_eq!(loaded.config, Config::default());
    }

    #[test]
    fn a_saved_configuration_loads_back_unchanged() {
        let directory = std::env::temp_dir().join(format!("mh-save-{}", std::process::id()));
        let path = directory.join("config.toml");
        let mut config = Config::default();
        config.overlay.position = Position::BottomRight;
        config.style.panel = Color::rgb(1, 2, 3);

        config.save(&path).unwrap();
        let loaded = Config::load_or_default(&path).unwrap();

        assert_eq!(loaded.origin, Origin::File);
        assert_eq!(loaded.config, config);

        fs::remove_dir_all(&directory).unwrap();
    }
}
