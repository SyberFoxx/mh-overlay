//! Lectores por juego sobre `mh-overlay-memory`.
//!
//! La CLI elige un [`GameKind`] y llama a [`monsters`]; este crate no sabe nada de argumentos
//! ni de cómo se muestra el resultado.

pub mod model;
pub mod rise;
pub mod world;

use std::fmt;
use std::str::FromStr;

use mh_overlay_memory::{AddressMap, Memory, MemoryError};

pub use model::{Ailment, Health, Monster, Part};

/// Juegos con lector implementado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKind {
    Rise,
    World,
}

impl GameKind {
    /// Nombre del ejecutable tal como lo ve `/proc` bajo Proton.
    pub fn module(self) -> &'static str {
        match self {
            Self::Rise => "MonsterHunterRise.exe",
            Self::World => "MonsterHunterWorld.exe",
        }
    }

    /// Prefijo de los archivos de `data/maps/` para este juego.
    pub fn map_prefix(self) -> &'static str {
        match self {
            Self::Rise => "MonsterHunterRise",
            Self::World => "MonsterHunterWorld",
        }
    }
}

impl fmt::Display for GameKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rise => formatter.write_str("rise"),
            Self::World => formatter.write_str("world"),
        }
    }
}

impl FromStr for GameKind {
    type Err = GameKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "rise" | "mhrise" | "sunbreak" => Ok(Self::Rise),
            "world" | "mhw" | "iceborne" => Ok(Self::World),
            other => Err(GameKindError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("juego desconocido '{0}' (usa: rise, world)")]
pub struct GameKindError(String);

/// Lee los monstruos presentes con el lector del juego indicado.
///
/// El `map` debe tener ya aplicada la base del módulo con [`AddressMap::set_base`].
pub fn monsters(
    kind: GameKind,
    memory: Memory,
    map: &AddressMap,
) -> Result<Vec<Monster>, MemoryError> {
    match kind {
        GameKind::Rise => rise::Rise::new(memory, map).monsters(),
        GameKind::World => world::World::new(memory, map).monsters(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_names_accept_the_common_aliases() {
        assert_eq!("rise".parse::<GameKind>().unwrap(), GameKind::Rise);
        assert_eq!("Sunbreak".parse::<GameKind>().unwrap(), GameKind::Rise);
        assert_eq!("iceborne".parse::<GameKind>().unwrap(), GameKind::World);
    }

    #[test]
    fn an_unknown_game_lists_the_valid_ones() {
        let error = "wilds".parse::<GameKind>().unwrap_err();

        assert!(error.to_string().contains("rise, world"));
    }
}
