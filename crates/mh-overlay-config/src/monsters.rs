//! Qué monstruos entran al panel.

use serde::{Deserialize, Serialize};

/// Modo de selección de monstruos.
///
/// El modo solo elige qué se dibuja: los lectores siguen leyendo la zona entera, porque el
/// objetivo puede cambiar en cualquier momento y releer el proceso es más caro que filtrar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Monsters {
    /// Todos los monstruos grandes de la zona, hasta `max_monsters`.
    #[default]
    Zone,
    /// Solo el que el juego marca como objetivo (fijado en el mapa o de la cacería).
    Target,
}

impl Monsters {
    pub const ALL: [Self; 2] = [Self::Zone, Self::Target];

    /// Nombre corto para el desplegable del hub.
    pub fn label(self) -> &'static str {
        match self {
            Self::Zone => "toda la zona",
            Self::Target => "solo el objetivo",
        }
    }

    /// Una línea de ayuda, porque «objetivo» no dice de dónde sale.
    pub fn detail(self) -> &'static str {
        match self {
            Self::Zone => "una fila por monstruo grande presente",
            Self::Target => {
                "el fijado en el mapa del juego; sin objetivo marcado se enseña la zona"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_are_the_ones_the_configuration_accepts() {
        let zone = crate::Config::from_toml("[overlay]\nmonsters = \"zone\"\n", "prueba").unwrap();
        let target =
            crate::Config::from_toml("[overlay]\nmonsters = \"target\"\n", "prueba").unwrap();

        assert_eq!(zone.overlay.monsters, Monsters::Zone);
        assert_eq!(target.overlay.monsters, Monsters::Target);
        assert_eq!(Monsters::default(), Monsters::Zone);
    }

    #[test]
    fn an_unknown_mode_lists_the_valid_ones() {
        let error =
            crate::Config::from_toml("[overlay]\nmonsters = \"todos\"\n", "prueba").unwrap_err();

        assert!(error.to_string().contains("monsters"), "{error}");
    }

    #[test]
    fn every_mode_says_what_it_does() {
        for mode in Monsters::ALL {
            assert!(!mode.label().is_empty());
            assert!(!mode.detail().is_empty());
        }
    }
}
