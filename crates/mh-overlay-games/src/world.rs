//! Lectura del estado de los monstruos en Monster Hunter: World / Iceborne.
//!
//! World guarda el estado de forma menos uniforme que Rise: la lista de componentes se
//! recorre hasta 128 entradas, el prefijo `em` del modelo identifica a los monstruos grandes
//! y las partes son arrays de tamaño fijo con dos strides distintos.

use std::collections::HashSet;

use mh_overlay_memory::{AddressMap, Memory, MemoryError};

use crate::model::{
    Ailment, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel, Snapshot, Weapon,
    WeaponKind,
};

// Desplazamientos fijos verificados contra la build 421810 (Iceborne), la misma que
// documenta HunterPie para esta rama del lector.

/// Componentes que el juego puede tener registrados a la vez.
const MAX_COMPONENTS: usize = 128;
/// Monstruos grandes distintos que se reportan como máximo.
const MAX_MONSTERS: usize = 8;
/// Puntero al monstruo dentro de su componente.
const COMPONENT_MONSTER: u64 = 0x138;
/// Identificador del monstruo (`Monster.Id`).
const MONSTER_ID: u64 = 0x12280;
/// Ruta del modelo (`em\em001\data`), que distingue monstruos grandes de pequeños.
///
/// En la build 421810 el bloque de rutas del modelo vive aquí, no en `0x2A0`: leer ese
/// desplazamiento devuelve basura y el filtro `em\em` descartaba hasta los Anjanath.
const MONSTER_MODEL: u64 = 0x7749;
const MONSTER_MODEL_LENGTH: usize = 64;
/// Puntero al bloque de partes.
const MONSTER_PARTS: u64 = 0x1D058;
/// Puntero a la estructura de vida del monstruo.
const MONSTER_HEALTH: u64 = 0x7670;
/// Bloque de vida: `[max, current]`.
const MONSTER_HEALTH_VALUES: u64 = 0x60;
/// Array de partes normales y su stride.
const NORMAL_PARTS: u64 = 0x40;
const NORMAL_PART_STRIDE: u64 = 0x1F8;
/// Array de partes cortables y su stride.
const SEVERABLE_PARTS: u64 = 0x1FC8;
const SEVERABLE_PART_STRIDE: u64 = 0x78;
/// Ventana donde HunterPie busca las partes cortables: 0x120 * 32 bytes.
const SEVERABLE_WINDOW: u64 = 0x120 * 32;
/// Slots de partes normales que se recorren antes de darse por terminado el array.
const MAX_NORMAL_PARTS: usize = 16;
/// `MHWSharpnessStructure`: unidades de filo actuales, color y color máximo del arma.
const SHARPNESS_CURRENT: u64 = 0x20F8;
const SHARPNESS_LEVEL: u64 = 0x20FC;
const SHARPNESS_MAX_LEVEL: u64 = 0x1D10;
/// Dentro de la tabla de datos del arma, el array de longitudes de filo empieza aquí.
const SHARPNESS_LENGTHS_OFFSET: u64 = 0x0C;

/// `MHWMonsterPartStructure`: vida máxima, vida actual e índice.
const PART_MAX_HEALTH: u64 = 0x0C;
const PART_HEALTH: u64 = 0x10;
const PART_INDEX: u64 = 0x6C;
/// Tope de cordura para descartar lecturas corruptas.
const MAX_PART_HEALTH: f32 = 10_000_000.0;
/// Índice de parte máximo creíble; por encima, la lectura está desalineada.
const MAX_PART_INDEX: u32 = 0x40;
/// Primer puntero del array de estados alterados del monstruo.
const MONSTER_AILMENTS: u64 = 0x1BC40;
/// Hueco entre el puntero del array y la estructura del estado.
const AILMENT_STRUCTURE: u64 = 0x148;
/// `MHWMonsterAilmentStructure`: dueño, id, duración y acumulación.
const AILMENT_OWNER: u64 = 0x00;
const AILMENT_ID: u64 = 0x10;
const AILMENT_MAX_DURATION: u64 = 0x14;
const AILMENT_BUILDUP: u64 = 0x30;
const AILMENT_MAX_BUILDUP: u64 = 0x40;
const AILMENT_DURATION: u64 = 0x70;
/// Estados que el juego puede registrar a la vez; el array termina en un puntero nulo.
const MAX_AILMENTS: usize = 32;
/// `MHWHudStructure`: vida y aguante del jugador.
const HUD_MAX_HEALTH: u64 = 0x60;
/// Vida recuperable (la parte roja de la barra).
const HUD_RECOVERABLE_HEALTH: u64 = 0x2DD4;
const HUD_HEALTH: u64 = 0x64;
const HUD_STAMINA: u64 = 0x12C;
const HUD_MAX_STAMINA: u64 = 0x130;
/// `MHWQuestStructure`: estado de la cacería.
const QUEST_STATE: u64 = 0x54;
/// `MHWQuestDataStructure`: vidas del equipo, en la estructura extra.
const QUEST_MAX_DEATHS: u64 = 0x00;
const QUEST_DEATHS: u64 = 0x04;
/// `QuestState.InQuest` en World (None, Ready, InQuest, ...).
const QUEST_STATE_IN_QUEST: i32 = 2;
/// El reloj de World se guarda en tics de 60 Hz.
const TIMER_TICKS_PER_SECOND: f32 = 60.0;

/// Lector de World sobre un proceso ya adjuntado.
pub struct World<'map> {
    memory: Memory,
    map: &'map AddressMap,
}

impl<'map> World<'map> {
    /// El `map` debe tener ya aplicada la base del módulo.
    pub fn new(memory: Memory, map: &'map AddressMap) -> Self {
        Self { memory, map }
    }

    /// Todo lo que el panel necesita de una lectura; jugador y cacería son opcionales.
    pub fn snapshot(&self) -> Result<Snapshot, MemoryError> {
        Ok(Snapshot {
            player: self.player().ok().flatten(),
            hunt: self.hunt().ok().flatten(),
            monsters: self.monsters()?,
        })
    }

    /// Vida y aguante del jugador local, leídos del HUD.
    pub fn player(&self) -> Result<Option<Player>, MemoryError> {
        let hud = self.memory.deref_chain(
            self.map.absolute("EQUIPMENT_ADDRESS")?,
            self.map.offsets("PLAYER_BASIC_INFORMATION_OFFSETS")?,
        )?;

        if hud == 0 {
            return Ok(None);
        }

        let recoverable = self.memory.read_f32(hud + HUD_RECOVERABLE_HEALTH).ok();

        let player = Player {
            health: Health::new(
                self.memory.read_f32(hud + HUD_HEALTH)?,
                self.memory.read_f32(hud + HUD_MAX_HEALTH)?,
            ),
            stamina: Health::new(
                self.memory.read_f32(hud + HUD_STAMINA)?,
                self.memory.read_f32(hud + HUD_MAX_STAMINA)?,
            ),
            recoverable: recoverable.filter(|value| value.is_finite() && *value >= 0.0),
            weapon: self.weapon(),
        };

        Ok(player.is_plausible().then_some(player))
    }

    /// Arma equipada y su filo. Un arma sin datos (todavía cargando) no rompe la lectura del
    /// jugador: el panel simplemente no dibuja la sección.
    fn weapon(&self) -> Option<Weapon> {
        let root = self.map.absolute("WEAPON_ADDRESS").ok()?;
        let address = self
            .memory
            .deref_chain(root, self.map.offsets("WEAPON_OFFSETS").ok()?)
            .ok()?;

        if address == 0 {
            return None;
        }

        let kind = WeaponKind::from_id(self.memory.read_u8(address).ok()?)?;
        let sharpness = kind.has_sharpness().then(|| self.sharpness(root)).flatten();

        Some(Weapon { kind, sharpness })
    }

    /// El filo se arma con dos piezas: el estado actual (`unidades` y color) y la tabla de
    /// longitudes por arma, indexada por el id del arma concreta.
    fn sharpness(&self, root: u64) -> Option<Sharpness> {
        let address = self
            .memory
            .deref_chain(root, self.map.offsets("WEAPON_SHARPNESS_OFFSETS").ok()?)
            .ok()?;
        if address == 0 {
            return None;
        }

        let level =
            SharpnessLevel::from_index(self.memory.read_i32(address + SHARPNESS_LEVEL).ok()?)?;
        let max_level =
            SharpnessLevel::from_index(self.memory.read_i32(address + SHARPNESS_MAX_LEVEL).ok()?)?;
        let current = self.memory.read_i32(address + SHARPNESS_CURRENT).ok()?;

        // Unidades negativas vienen de una lectura desalineada: mejor no dibujar filo que
        // dibujarlo mal.
        if current < 0 {
            return None;
        }

        // Hay armas que reportan un color máximo por debajo del actual mientras se cambia de
        // arma; el color actual manda y la barra se queda en ese nivel.
        let max_level = if SharpnessLevel::ALL
            .iter()
            .position(|item| *item == max_level)
            < SharpnessLevel::ALL.iter().position(|item| *item == level)
        {
            level
        } else {
            max_level
        };

        let weapon_id = self
            .memory
            .read_u32(
                self.memory
                    .deref_chain(root, self.map.offsets("WEAPON_ID_OFFSETS").ok()?)
                    .ok()?,
            )
            .ok()?;
        let data = self
            .memory
            .deref_chain(
                self.map.absolute("WEAPON_DATA_ADDRESS").ok()?,
                self.map.offsets("WEAPON_DATA_OFFSETS").ok()?,
            )
            .ok()?;
        if data == 0 {
            return None;
        }

        let lengths_address =
            self.memory.read_u64(data + u64::from(weapon_id) * 8).ok()? + SHARPNESS_LENGTHS_OFFSET;
        let mut lengths = [0u16; 7];

        for (index, length) in lengths.iter_mut().enumerate() {
            let value = self
                .memory
                .read_u16(lengths_address + (index as u64) * 2)
                .ok()?;
            *length = value;
        }

        let sharpness = Sharpness {
            lengths,
            level,
            max_level,
            current: current as f32,
        };

        (sharpness.total() > 0.0).then_some(sharpness)
    }

    /// Cacería en curso: reloj en tics de 60 Hz y vidas del equipo.
    pub fn hunt(&self) -> Result<Option<Hunt>, MemoryError> {
        let base = self.map.absolute("QUEST_DATA_ADDRESS")?;
        let quest = self
            .memory
            .deref_chain(base, self.map.offsets("QUEST_DATA_OFFSETS")?)?;

        if quest == 0 || self.memory.read_i32(quest + QUEST_STATE)? != QUEST_STATE_IN_QUEST {
            return Ok(None);
        }

        let extra = self
            .memory
            .deref_chain(base, self.map.offsets("QUEST_EXTRA_DATA_OFFSETS")?)?;
        if extra == 0 {
            return Ok(None);
        }

        let max_deaths = self.memory.read_i32(extra + QUEST_MAX_DEATHS)?;
        let deaths = self.memory.read_i32(extra + QUEST_DEATHS)?;
        if max_deaths <= 0 || deaths < 0 {
            return Ok(None);
        }

        let timer = self
            .memory
            .deref_chain(base, self.map.offsets("QUEST_TIMER_OFFSETS")?)?;
        if timer == 0 {
            return Ok(None);
        }

        let hunt = Hunt {
            time_left: self.memory.read_u64(timer)? as f32 / TIMER_TICKS_PER_SECOND,
            deaths: deaths as u32,
            max_deaths: max_deaths as u32,
        };

        Ok(hunt.is_plausible().then_some(hunt))
    }

    pub fn monsters(&self) -> Result<Vec<Monster>, MemoryError> {
        let list = self.memory.deref_chain(
            self.map.absolute("MONSTER_LIST_ADDRESS")?,
            self.map.offsets("MONSTER_LIST_OFFSETS")?,
        )?;

        let components = self.memory.read_pointers(list, MAX_COMPONENTS)?;
        let mut monsters: Vec<Monster> = Vec::new();
        let mut seen: HashSet<u64> = HashSet::new();

        for component in components {
            if component == 0 {
                continue;
            }

            if monsters.len() >= MAX_MONSTERS {
                break;
            }

            let Ok(monster) = self.memory.read_u64(component + COMPONENT_MONSTER) else {
                continue;
            };
            if monster == 0 {
                continue;
            }

            // `em\ems` son los monstruos pequeños; solo interesan los grandes.
            let model = match self
                .memory
                .read_cstring(monster + MONSTER_MODEL, MONSTER_MODEL_LENGTH)
            {
                Ok(model) => model,
                Err(_) => continue,
            };
            if !model.starts_with("em\\em") || model.starts_with("em\\ems") {
                continue;
            }

            // El puntero identifica la instancia: dos monstruos de la misma especie comparten
            // ruta de modelo y aun así son dos barras distintas.
            if !seen.insert(monster) {
                continue;
            }

            if let Some(monster_state) = self.monster(monster, model) {
                monsters.push(monster_state);
            }
        }

        Ok(monsters)
    }

    fn monster(&self, address: u64, model: String) -> Option<Monster> {
        let id = self.memory.read_i32(address + MONSTER_ID).ok()?;
        let health = self.health(address).ok()?;

        // Una vida imposible viene de un desplazamiento desfasado, no de un monstruo.
        if !health.is_plausible() {
            return None;
        }

        let mut monster = Monster::new(id, health);

        // La localización es la fuente del nombre legible; el código `em` queda como respaldo
        // para ids que todavía no tienen traducción.
        monster.name = crate::tables::world_monster_name(id)
            .map(str::to_owned)
            .or(Some(model));
        monster.capture_percent = crate::tables::world_capture_percent(id);
        monster.parts = self.parts(address, id).unwrap_or_default();
        monster.ailments = self.ailments(address).unwrap_or_default();

        Some(monster)
    }

    fn health(&self, monster: u64) -> Result<Health, MemoryError> {
        let values = self.memory.read_u64(monster + MONSTER_HEALTH)?;
        let health = self
            .memory
            .read_f32_array(values + MONSTER_HEALTH_VALUES, 2)?;

        Ok(Health::new(health[1], health[0]))
    }

    /// Los estados viven en un array de punteros que termina en nulo. La estructura se lee
    /// desplazada del puntero y su primer campo apunta de vuelta al monstruo: si no coincide,
    /// el array pertenece a otro monstruo y se corta.
    fn ailments(&self, monster: u64) -> Result<Vec<Ailment>, MemoryError> {
        let mut ailments = Vec::new();

        for slot in 0..MAX_AILMENTS {
            let element = monster + MONSTER_AILMENTS + (slot as u64) * 8;
            let pointer = self.memory.read_u64(element)?;

            if pointer <= 1 {
                break;
            }

            let structure = pointer + AILMENT_STRUCTURE;
            if self.memory.read_u64(structure + AILMENT_OWNER)? != monster {
                break;
            }

            if let Ok(Some(ailment)) = self.ailment(structure)
                && ailment.is_active()
            {
                ailments.push(ailment);
            }
        }

        Ok(ailments)
    }

    /// Un id sin nombre es un estado que el juego no muestra (trampas internas, contadores):
    /// se descarta en vez de inventarle una etiqueta.
    fn ailment(&self, structure: u64) -> Result<Option<Ailment>, MemoryError> {
        let id = self.memory.read_i32(structure + AILMENT_ID)?;
        let Some(name) = crate::tables::world_ailment_name(id) else {
            return Ok(None);
        };

        Ok(Some(Ailment {
            name,
            buildup: self.memory.read_f32(structure + AILMENT_BUILDUP)?,
            max_buildup: self.memory.read_f32(structure + AILMENT_MAX_BUILDUP)?,
            timer: self.memory.read_f32(structure + AILMENT_DURATION)?,
            max_timer: self.memory.read_f32(structure + AILMENT_MAX_DURATION)?,
        }))
    }

    /// Se reportan las partes que el juego tiene inicializadas. Las normales exponen su vida
    /// como `breakable`; las cortables, solo como `severable`. La tabla del monstruo pone el
    /// nombre que el juego solo guarda como índice.
    fn parts(&self, monster: u64, id: i32) -> Result<Vec<Part>, MemoryError> {
        let block = self.memory.read_u64(monster + MONSTER_PARTS)?;
        if block == 0 {
            return Ok(Vec::new());
        }

        let mut parts = Vec::new();

        // Las partes normales son un array contiguo: la primera entrada sin vida marca el
        // final, no un hueco que haya que saltar. El índice que la tabla usa para nombrarlas
        // es la posición en el array: el campo `Index` de la estructura solo se rellena en
        // las cortables y en las normales vale siempre cero.
        for slot in 0..MAX_NORMAL_PARTS {
            let address = block + NORMAL_PARTS + (slot as u64) * NORMAL_PART_STRIDE;
            let Some(health) = self.part_health(address) else {
                break;
            };

            parts.push(Part {
                index: slot,
                name: crate::tables::world_part_name(id, slot),
                flinch: None,
                breakable: Some(health),
                severable: None,
            });
        }

        // Las cortables viven en otra zona y sus entradas aparecen desplazadas 8 bytes cuando
        // la referencia inicial es pequeña; HunterPie aplica el mismo ajuste antes de leer.
        let mut address = block + SEVERABLE_PARTS;
        let window_end = address + SEVERABLE_WINDOW;

        while address < window_end && parts.len() < MAX_NORMAL_PARTS * 4 {
            if self
                .memory
                .read_u32(address)
                .is_ok_and(|reference| reference <= 0xA0)
            {
                address += 8;
            }

            // El índice del juego identifica qué parte de la tabla es: la misma que ya
            // reportó la lista normal, así que la vida de corte se suma a esa entrada.
            if let Some(index) = self.part_index(address)
                && let Some(health) = self.part_health(address)
                && let Some(existing) = parts.iter_mut().find(|part| part.index == index)
            {
                existing.severable = Some(health);
            }

            address += SEVERABLE_PART_STRIDE;
        }

        parts.sort_by_key(|part| part.index);

        Ok(parts)
    }

    fn part_health(&self, address: u64) -> Option<Health> {
        let max = self.memory.read_f32(address + PART_MAX_HEALTH).ok()?;
        if !max.is_finite() || max <= 0.0 || max > MAX_PART_HEALTH {
            return None;
        }

        let current = self.memory.read_f32(address + PART_HEALTH).ok()?;
        let health = Health::new(current, max);
        health.is_plausible().then_some(health)
    }

    fn part_index(&self, address: u64) -> Option<usize> {
        let index = self.memory.read_u32(address + PART_INDEX).ok()?;
        (index <= MAX_PART_INDEX).then_some(index as usize)
    }
}
