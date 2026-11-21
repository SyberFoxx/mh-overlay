//! Lectores por juego sobre `mh-overlay-memory`.
//!
//! La CLI elige un [`GameKind`] y llama a [`monsters`]; este crate no sabe nada de argumentos
//! ni de cómo se muestra el resultado.

pub mod buffs;
#[cfg(test)]
mod fixtures;
pub mod model;
pub mod rise;
pub mod tables;
pub mod world;

use std::fmt;
use std::str::FromStr;

use mh_overlay_memory::{AddressMap, Memory, MemoryError, Process, Reads};

pub use model::{
    Ailment, AmmoSlot, BowState, Breaks, Buff, Crown, CrownThresholds, Extracts, GunlanceState,
    Health, HornNote, HornSong, HornState, Hunt, Monster, Part, Player, RangedState, Sharpness,
    SharpnessLevel, Snapshot, Weapon, WeaponDetail, WeaponGauge, WeaponKind,
};

/// Resuelve la raíz de una lista de monstruos: `None` si el juego todavía no la ha creado.
///
/// Fuera de una cacería —menú, aldea, pantalla de carga— el contenedor no existe y la cadena
/// acaba en un nulo. Eso no es un offset roto: es la respuesta correcta, un mundo sin monstruos.
/// Devolver un error ahí convertía «todavía no hay zona» en un fallo rojo en el panel.
pub(crate) fn optional_chain<M: Reads>(
    memory: &M,
    address: u64,
    offsets: &[i32],
) -> Result<Option<u64>, MemoryError> {
    match memory.deref_chain(address, offsets) {
        Ok(0) => Ok(None),
        Ok(pointer) => Ok(Some(pointer)),
        Err(MemoryError::NullPointer { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Juegos con lector implementado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKind {
    Rise,
    World,
}

impl GameKind {
    /// Juegos con lector, en el orden en que se prueban al detectar.
    pub const ALL: [Self; 2] = [Self::Rise, Self::World];

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

    /// Juego al que pertenece el módulo que `/proc` reportó para el proceso.
    pub fn from_module(module: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.module().eq_ignore_ascii_case(module))
    }
}

/// Juego que está corriendo ahora mismo, si hay uno de los dos.
///
/// Si ambos están abiertos devuelve el primero que aparece en `/proc`; el hub prefiere el
/// configurado antes de llegar aquí y la CLI desactiva la detección con `--game`.
pub fn detect_running_game() -> Option<GameKind> {
    let modules = GameKind::ALL.map(GameKind::module);
    let process = Process::find(&modules).ok()?;

    GameKind::from_module(process.module())
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

/// Lee monstruos, jugador y cacería con el lector del juego indicado.
///
/// El `map` debe tener ya aplicada la base del módulo con [`AddressMap::set_base`].
pub fn snapshot(kind: GameKind, memory: Memory, map: &AddressMap) -> Result<Snapshot, MemoryError> {
    match kind {
        GameKind::Rise => rise::Rise::new(memory, map).snapshot(),
        GameKind::World => world::World::new(memory, map).snapshot(),
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

    #[test]
    fn a_module_name_maps_back_to_its_game() {
        for kind in GameKind::ALL {
            assert_eq!(GameKind::from_module(kind.module()), Some(kind));
            assert_eq!(
                GameKind::from_module(&kind.module().to_lowercase()),
                Some(kind)
            );
        }

        assert_eq!(GameKind::from_module("otro.exe"), None);
    }

    #[test]
    fn the_newest_maps_define_the_keys_each_reader_needs() {
        use std::path::Path;

        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/maps");

        let rise_path =
            AddressMap::newest_for_game(GameKind::Rise.map_prefix(), &directory).unwrap();
        let rise = AddressMap::load(rise_path).unwrap();

        for key in [
            "MONSTERS_ADDRESS",
            "MONSTER_LIST_OFFSETS",
            "MONSTER_HEALTH_COMPONENT_OFFSETS",
            "MONSTER_HEALTH_COMPONENT_ENCODED_OFFSETS",
            "MONSTER_CAPTURE_HEALTH_THRESHOLD",
            "MONSTER_AILMENTS_OFFSETS",
            "MONSTER_FLINCH_HEALTH_COMPONENT_OFFSETS",
            "MONSTER_BREAK_HEALTH_COMPONENT_OFFSETS",
            "MONSTER_SEVER_HEALTH_COMPONENT_OFFSETS",
        ] {
            assert!(
                rise.address(key).is_ok() || rise.offsets(key).is_ok(),
                "el mapa más nuevo de Rise no define '{key}'"
            );
        }

        let world_path =
            AddressMap::newest_for_game(GameKind::World.map_prefix(), &directory).unwrap();
        let world = AddressMap::load(world_path).unwrap();

        for key in ["MONSTER_LIST_ADDRESS", "MONSTER_LIST_OFFSETS"] {
            assert!(
                world.address(key).is_ok() || world.offsets(key).is_ok(),
                "el mapa más nuevo de World no define '{key}'"
            );
        }
    }
}
