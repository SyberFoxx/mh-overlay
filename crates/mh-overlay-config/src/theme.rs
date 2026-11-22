//! Tema visual y fondo del hub.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Tema del hub: el del escritorio, u oscuro o claro fijos.
///
/// `auto` no mira la variable de entorno: deja que la ventana siga el tema del sistema y
/// resuelve la paleta con lo que el escritorio diga en cada momento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Auto,
    Dark,
    Light,
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" | "system" | "sistema" => Ok(Self::Auto),
            "dark" | "oscuro" => Ok(Self::Dark),
            "light" | "claro" => Ok(Self::Light),
            other => Err(format!(
                "tema desconocido '{other}' (usa: auto, dark, light)"
            )),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Auto => "auto",
            Self::Dark => "dark",
            Self::Light => "light",
        })
    }
}

/// Fondo de la ventana del hub.
///
/// Todos los fondos se generan al dibujar: no traen ninguna imagen ni licencia que repartir y
/// escalan a cualquier tamaño de ventana. Cambian el par de focos y su color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backdrop {
    /// El del proyecto: dos focos con el acento y el color frío de la paleta.
    #[default]
    Ambient,
    /// Verde y azul, como una aurora.
    Aurora,
    /// Naranja y rojo de brasa.
    Ember,
    /// Grises fríos, casi sin color.
    Mist,
    None,
}

impl Backdrop {
    /// Nombre corto para la interfaz, en castellano.
    pub fn label(self) -> &'static str {
        match self {
            Self::Ambient => "Ambiente",
            Self::Aurora => "Aurora",
            Self::Ember => "Brasa",
            Self::Mist => "Niebla",
            Self::None => "Sin fondo",
        }
    }

    /// Todos los fondos, en el orden en que se ofrecen.
    pub const ALL: [Self; 5] = [
        Self::Ambient,
        Self::Aurora,
        Self::Ember,
        Self::Mist,
        Self::None,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_names_accept_the_documented_ones() {
        let config: crate::Config =
            crate::Config::from_toml("[hub]\nbackdrop = \"none\"\n", "prueba").unwrap();
        assert_eq!(config.hub.backdrop, Backdrop::None);

        let config: crate::Config =
            crate::Config::from_toml("[hub]\nbackdrop = \"ambient\"\n", "prueba").unwrap();
        assert_eq!(config.hub.backdrop, Backdrop::Ambient);
    }

    #[test]
    fn every_backdrop_has_a_name_and_round_trips_through_the_toml() {
        for backdrop in Backdrop::ALL {
            let name = backdrop.label().to_lowercase();
            let style = match backdrop {
                Backdrop::Ambient => "ambient",
                Backdrop::Aurora => "aurora",
                Backdrop::Ember => "ember",
                Backdrop::Mist => "mist",
                Backdrop::None => "none",
            };
            let config: crate::Config =
                crate::Config::from_toml(&format!("[hub]\nbackdrop = \"{style}\"\n"), "prueba")
                    .unwrap();

            assert_eq!(config.hub.backdrop, backdrop, "{name}");
        }
    }

    #[test]
    fn every_bar_style_has_a_name_and_round_trips_through_the_toml() {
        for (style, name) in [
            (crate::BarStyle::Rounded, "rounded"),
            (crate::BarStyle::Blocks, "blocks"),
            (crate::BarStyle::Pill, "pill"),
        ] {
            let config: crate::Config =
                crate::Config::from_toml(&format!("[style]\nbar_style = \"{name}\"\n"), "prueba")
                    .unwrap();

            assert_eq!(config.style.bar_style, style);
            assert!(!style.label().is_empty());
        }
    }

    #[test]
    fn every_panel_theme_has_a_name_and_round_trips_through_the_toml() {
        for (theme, name) in [
            (crate::PanelTheme::Classic, "classic"),
            (crate::PanelTheme::Glass, "glass"),
            (crate::PanelTheme::Brutal, "brutal"),
        ] {
            let config: crate::Config =
                crate::Config::from_toml(&format!("[style]\npanel_theme = \"{name}\"\n"), "prueba")
                    .unwrap();

            assert_eq!(config.style.panel_theme, theme);
            assert!(!theme.label().is_empty());
        }
    }

    #[test]
    fn an_old_file_without_a_panel_theme_keeps_the_classic_one() {
        let config: crate::Config =
            crate::Config::from_toml("[style]\nhealth = \"#112233\"\n", "prueba").unwrap();

        assert_eq!(config.style.panel_theme, crate::PanelTheme::Classic);
    }

    #[test]
    fn the_theme_names_accept_spanish_and_english() {
        assert_eq!("auto".parse::<Theme>().unwrap(), Theme::Auto);
        assert_eq!("Sistema".parse::<Theme>().unwrap(), Theme::Auto);
        assert_eq!("dark".parse::<Theme>().unwrap(), Theme::Dark);
        assert_eq!("Claro".parse::<Theme>().unwrap(), Theme::Light);
        assert!("neon".parse::<Theme>().is_err());
    }

    #[test]
    fn a_theme_round_trips_through_its_name() {
        for theme in [Theme::Auto, Theme::Dark, Theme::Light] {
            assert_eq!(theme.to_string().parse::<Theme>().unwrap(), theme);
        }
    }
}
