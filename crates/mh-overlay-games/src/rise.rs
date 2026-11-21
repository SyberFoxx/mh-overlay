//! Lectura del estado de los monstruos en Monster Hunter Rise / Sunbreak.
//!
//! Todos los desplazamientos fijos de esta sección fueron verificados contra la build
//! 16.0.2.0 y están documentados junto a la estructura de la que salen.

use mh_overlay_memory::{AddressMap, Memory, MemoryError, Reads};

use std::collections::HashMap;
use std::sync::Mutex;

use crate::buffs::{RISE_BUFFS, RiseGroup};
use crate::model::{
    Ailment, Buff, Crown, Extracts, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel,
    Snapshot, Weapon, WeaponDetail, WeaponGauge, WeaponKind,
};
use crate::optional_chain;

/// Monstruos grandes que pueden coexistir en una zona.
const MAX_MONSTERS: usize = 5;

/// `MHRSizeStructure` dentro del bloque de `MONSTER_CROWN_OFFSETS`: multiplicador de tamaño
/// sorteado y la corrección que el juego aplica en algunas misiones.
const MONSTER_SIZE_MULTIPLIER: u64 = 0x24;
const MONSTER_SIZE_CORRECTION: u64 = 0x28;

/// Estados alterados de Rise, en el orden exacto del array en memoria.
const AILMENTS: [&str; 17] = [
    "parálisis",
    "sueño",
    "aturdimiento",
    "destello",
    "veneno",
    "explosión",
    "agotamiento",
    "montura",
    "agua",
    "fuego",
    "hielo",
    "rayo",
    "trampa de foso",
    "trampa eléctrica",
    "tranquilizante",
    "estiércol",
    "colmillo de acero",
];

/// `MHRMonster.Id` (campo i32).
const MONSTER_ID: u64 = 0x2D4;
/// Vida máxima, en el propio componente de vida.
const HEALTH_MAX: u64 = 0x18;
/// Vida actual, en la estructura apuntada por el componente codificado.
const HEALTH_CURRENT: u64 = 0x18;
/// Longitud del array de partes, en el contenedor.
const PART_ARRAY_LENGTH: u64 = 0x1C;
/// Inicio de los punteros de partes, en el contenedor.
const PART_ARRAY_DATA: u64 = 0x20;
/// Umbral de captura, en su componente.
const CAPTURE_THRESHOLD_VALUE: u64 = 0x1C;
/// `MHRMonsterAilmentStructure` (LayoutKind.Explicit).
const AILMENT_MAX_TIMER: u64 = 0x44;
const AILMENT_TIMER: u64 = 0x48;
const AILMENT_BUILDUP_PTR: u64 = 0x68;
const AILMENT_MAX_BUILDUP_PTR: u64 = 0x78;
/// Valor dentro de las estructuras de contador y acumulación.
const AILMENT_VALUE: u64 = 0x20;
/// Tope defensivo: un array de partes mayor es una lectura corrupta.
const MAX_PARTS: i32 = 64;
/// `MHRPlayerHudStructure` (Pack = 1): vida y aguante del HUD.
const HUD_HEALTH: u64 = 0x00;
const HUD_MAX_HEALTH: u64 = 0x08;
const HUD_STAMINA: u64 = 0x28;
const HUD_MAX_STAMINA: u64 = 0x2C;
/// `MHRQuestStructure`: estado, vidas y reloj de la cacería.
const QUEST_STATE: u64 = 0x110;
const QUEST_MAX_DEATHS: u64 = 0x15C;
const QUEST_DEATHS: u64 = 0x160;
const QUEST_TIME_ELAPSED: u64 = 0x170;
const QUEST_TIME_LIMIT: u64 = 0x178;
/// `QuestState.InQuest` en Rise (Idle, Ready, InQuest, ...).
const QUEST_STATE_IN_QUEST: i32 = 2;
/// Tipo de arma equipada, en el bloque `WEAPON_ADDRESS` + `WEAPON_OFFSETS`.
const WEAPON_ID: u64 = 0x8C;
/// `MHRSharpnessStructure` (Sequential): color actual, unidades gastadas y máximo del arma.
const SHARPNESS_LEVEL: u64 = 0x00;
const SHARPNESS_HITS: u64 = 0x04;
/// Todo array de Rise guarda su tamaño en +0x1C y sus datos en +0x20.
const ARRAY_LENGTH: u64 = 0x1C;
const ARRAY_DATA: u64 = 0x20;
/// Los temporizadores de Rise van en tics de 60 por segundo.
const TIMER_DIVISOR: f32 = 60.0;
/// Tope de cordura para un temporizador de buff: más de una hora es basura.
const MAX_BUFF_TIMER: f32 = 3600.0;
/// `MHRHHAbnormality`: el temporizador va detrás de un puntero y dos enteros.
const SONG_TIMER: u64 = 0x10;
/// Ranura de la tabla de visto-por-primera-vez reservada a las canciones.
const SONG_KEY: u32 = 0x10000;
/// `MHRPlayerStatusContext`: ataque, elemento y afinidad que el juego calcula al equipar.
const STATUS_ATTACK: u64 = 0x44;
const STATUS_ELEMENT: u64 = 0x50;
const STATUS_ELEMENT_SECONDARY: u64 = 0x54;
const STATUS_AFFINITY: u64 = 0x68;

// Medidores de clase: todos se leen desde `LOCAL_PLAYER_DATA_ADDRESS` con
// `CURRENT_WEAPON_OFFSETS` (HunterPie: `MHRLongSwordStructure`, `MHRChargeBladeStructure`,
// `MHRSwitchAxeStructure`, `MHRDualBladesStructure`, `MHRInsectGlaiveDataStructure`).

/// Espada larga: temporizador del nivel, medidor de espíritu, nivel y su tabla de duraciones.
const LS_LEVEL_TIMER: u64 = 0x9F0;
const LS_BUILDUP: u64 = 0x9F4;
const LS_LEVEL: u64 = 0x9F8;
const LS_LEVEL_MAX_TIMERS: u64 = 0xA00;
const LS_BUFF_TIMER: u64 = 0xA08;
const LS_LEVELS: usize = 4;
/// Espada cargada: frascos, carga y los buffs de escudo y espada.
const CB_PHIALS: u64 = 0x9E8;
const CB_CHARGE: u64 = 0xA08;
const CB_SHIELD: u64 = 0xA28;
const CB_SWORD: u64 = 0xA2C;
/// Frascos por defecto; la habilidad de capacidad añade uno.
const CB_MAX_PHIALS: u8 = 5;
/// Hacha espada: carga de la hoja (entero), carga amped y temporizador de la carga.
const SA_BUILDUP: u64 = 0xA70;
const SA_CHARGE: u64 = 0xA7C;
const SA_TIMER: u64 = 0xA84;
/// Espadas dobles: medidor máximo, actual, modo demonio y modo archidemonio.
const DB_MAX_BUILDUP: u64 = 0x9D0;
const DB_BUILDUP: u64 = 0x9E8;
const DB_DEMON: u64 = 0x9EC;
const DB_ARCH_DEMON: u64 = 0xA48;
/// Glaive insecto: array de esencias y los tres temporizadores.
const IG_EXTRACTS: u64 = 0xA08;
const IG_ATTACK_TIMER: u64 = 0xA44;
const IG_SPEED_TIMER: u64 = 0xA48;
const IG_DEFENSE_TIMER: u64 = 0xA4C;
const IG_EXTRACTS_COUNT: usize = 2;
/// `MHRKinsectStaminaStructure` (Sequential): actual y máximo, en la cadena del kinsecto.
const KINSECT_STAMINA_CURRENT: u64 = 0x00;
const KINSECT_STAMINA_MAX: u64 = 0x04;

/// Los temporizadores de Rise vienen en tics de 60 Hz y el panel los dibuja en segundos.
fn seconds(ticks: f32) -> f32 {
    if ticks.is_finite() {
        (ticks / TIMER_DIVISOR).max(0.0)
    } else {
        0.0
    }
}

/// Los medidores que el juego ya guarda en porcentaje solo necesitan descartar NaN y valores
/// fuera de rango.
fn clamp_percent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Lector de Rise sobre un proceso ya adjuntado, o sobre una fixture en los tests.
pub struct Rise<'map, M: Reads = Memory> {
    memory: M,
    map: &'map AddressMap,
    /// Duración máxima vista de cada estado, por offset: el juego solo guarda lo que queda y
    /// una barra necesita el total.
    seen_buffs: Mutex<HashMap<u32, f32>>,
}

impl<'map, M: Reads> Rise<'map, M> {
    /// El `map` debe tener ya aplicada la base del módulo.
    pub fn new(memory: M, map: &'map AddressMap) -> Self {
        Self {
            memory,
            map,
            seen_buffs: Mutex::new(HashMap::new()),
        }
    }

    /// Todo lo que el panel necesita de una lectura. El jugador y la cacería son opcionales:
    /// en el menú principal no existen y el overlay tiene que seguir enseñando los monstruos.
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
            self.map.absolute("UI_ADDRESS")?,
            self.map.offsets("PLAYER_HUD_OFFSETS")?,
        )?;

        if hud == 0 {
            return Ok(None);
        }

        let player = Player {
            health: Health::new(
                self.memory.read_f32(hud + HUD_HEALTH)?,
                self.memory.read_f32(hud + HUD_MAX_HEALTH)?,
            ),
            stamina: Health::new(
                self.memory.read_f32(hud + HUD_STAMINA)?,
                self.memory.read_f32(hud + HUD_MAX_STAMINA)?,
            ),
            recoverable: None,
            weapon: self.weapon(),
            buffs: self.buffs(),
        };

        Ok(player.is_plausible().then_some(player))
    }

    /// Arma equipada, su filo y el medidor de su clase.
    fn weapon(&self) -> Option<Weapon> {
        let address = self
            .memory
            .deref_chain(
                self.map.absolute("WEAPON_ADDRESS").ok()?,
                self.map.offsets("WEAPON_OFFSETS").ok()?,
            )
            .ok()?;

        if address == 0 {
            return None;
        }

        let kind = WeaponKind::from_rise_id(self.memory.read_i32(address + WEAPON_ID).ok()?)?;
        let sharpness = kind.has_sharpness().then(|| self.sharpness()).flatten();
        let gauge = self.gauge(kind);

        Some(Weapon {
            kind,
            sharpness,
            gauge,
            detail: self.weapon_detail(),
        })
    }

    /// Ataque, afinidad y elemento que el juego calcula para el arma equipada. Rise los expone en
    /// el contexto de estado del jugador; World no tiene una estructura equivalente en el lector.
    fn weapon_detail(&self) -> WeaponDetail {
        let (Ok(base), Ok(local_offsets), Ok(status_offsets)) = (
            self.map.absolute("LOCAL_PLAYER_DATA_ADDRESS"),
            self.map.offsets("Player::Local"),
            self.map.offsets("Player::Status"),
        ) else {
            return WeaponDetail::default();
        };
        let Ok(player) = self.memory.deref_chain(base, local_offsets) else {
            return WeaponDetail::default();
        };
        let Ok(status) = self.memory.deref_chain(player, status_offsets) else {
            return WeaponDetail::default();
        };
        if status == 0 {
            return WeaponDetail::default();
        }

        let attack = self.memory.read_f32(status + STATUS_ATTACK).ok();
        let element = self
            .memory
            .read_f32(status + STATUS_ELEMENT)
            .ok()
            .zip(self.memory.read_f32(status + STATUS_ELEMENT_SECONDARY).ok())
            .map(|(primary, secondary)| primary.max(secondary));
        let affinity = self.memory.read_i32(status + STATUS_AFFINITY).ok();

        WeaponDetail {
            attack: attack.filter(|value| value.is_finite() && (0.0..=10_000.0).contains(value)),
            element: element.filter(|value| value.is_finite() && (0.0..=10_000.0).contains(value)),
            affinity: affinity.filter(|value| (-100..=100).contains(value)),
            ..WeaponDetail::default()
        }
    }

    /// El filo se arma con dos piezas: el color y las unidades actuales, y la tabla de
    /// longitudes por color del arma concreta.
    fn sharpness(&self) -> Option<Sharpness> {
        let root = self.map.absolute("SHARPNESS_ADDRESS").ok()?;
        let state = self
            .memory
            .deref_chain(root, self.map.offsets("SHARPNESS_OFFSETS").ok()?)
            .ok()?;
        if state == 0 {
            return None;
        }

        let level =
            SharpnessLevel::from_index(self.memory.read_i32(state + SHARPNESS_LEVEL).ok()?)?;
        let current = self.memory.read_i32(state + SHARPNESS_HITS).ok()?;
        // Unidades negativas vienen de una lectura desalineada: mejor no dibujar filo que
        // dibujarlo mal.
        if current < 0 {
            return None;
        }

        let lengths_ptr = self
            .memory
            .deref_chain(root, self.map.offsets("SHARPNESS_ARRAY_OFFSETS").ok()?)
            .ok()?;
        if lengths_ptr == 0 {
            return None;
        }

        let mut lengths = [0u16; 7];
        for (index, length) in lengths.iter_mut().enumerate() {
            let address = lengths_ptr + ARRAY_DATA + (index as u64) * 4;
            let value = self.memory.read_i32(address).ok()?;
            *length = value.clamp(0, i32::from(u16::MAX)) as u16;
        }

        // El color máximo es el último con longitud; el actual manda si el arma se está
        // cambiando y el juego todavía no actualizó la tabla.
        let highest = lengths
            .iter()
            .rposition(|length| *length > 0)
            .and_then(|index| SharpnessLevel::from_index(index as i32))
            .unwrap_or(level);
        let max_level = if SharpnessLevel::ALL.iter().position(|item| *item == highest)
            < SharpnessLevel::ALL.iter().position(|item| *item == level)
        {
            level
        } else {
            highest
        };

        let sharpness = Sharpness {
            lengths,
            level,
            max_level,
            current: current as f32,
        };

        (sharpness.total() > 0.0).then_some(sharpness)
    }

    /// Base del bloque de medidores de clase, común a todas las armas.
    fn mechanics(&self) -> Option<u64> {
        let address = self
            .memory
            .deref_chain(
                self.map.absolute("LOCAL_PLAYER_DATA_ADDRESS").ok()?,
                self.map.offsets("CURRENT_WEAPON_OFFSETS").ok()?,
            )
            .ok()?;

        (address != 0).then_some(address)
    }

    /// Medidores propios de la clase de arma. Solo se leen los que HunterPie también lee; el
    /// resto (lanza, martillo, ballestas...) no guarda nada extra que mostrar.
    fn gauge(&self, kind: WeaponKind) -> Option<WeaponGauge> {
        let address = self.mechanics()?;

        match kind {
            WeaponKind::Longsword => {
                let level = self.memory.read_i32(address + LS_LEVEL).ok()?;
                if !(0..LS_LEVELS as i32).contains(&level) {
                    return None;
                }

                let buildup = self.memory.read_f32(address + LS_BUILDUP).ok()?;
                let timer = self.memory.read_f32(address + LS_LEVEL_TIMER).ok()?;
                let timers = self.memory.read_u64(address + LS_LEVEL_MAX_TIMERS).ok()?;
                let max_timer = self
                    .array_f32(timers, LS_LEVELS)
                    .and_then(|values| values.get(level as usize).copied())
                    .unwrap_or(0.0);
                let regen = self.memory.read_f32(address + LS_BUFF_TIMER).unwrap_or(0.0);

                Some(WeaponGauge::Spirit {
                    buildup: clamp_percent(buildup),
                    level: level as u8,
                    timer: seconds(timer),
                    max_timer: seconds(max_timer),
                    regen: seconds(regen),
                })
            }
            WeaponKind::ChargeBlade => {
                let phials = self.memory.read_i32(address + CB_PHIALS).ok()?;
                if !(0..=8).contains(&phials) {
                    return None;
                }

                Some(WeaponGauge::ChargeBlade {
                    phials: phials as u8,
                    max_phials: CB_MAX_PHIALS.max(phials as u8),
                    charge: clamp_percent(self.memory.read_f32(address + CB_CHARGE).ok()?),
                    shield: seconds(self.memory.read_f32(address + CB_SHIELD).ok()?),
                    sword: seconds(self.memory.read_f32(address + CB_SWORD).ok()?),
                    // Rise no guarda un temporizador propio del hacha.
                    axe: 0.0,
                })
            }
            WeaponKind::SwitchAxe => Some(WeaponGauge::SwitchAxe {
                buildup: clamp_percent(self.memory.read_i32(address + SA_BUILDUP).ok()? as f32),
                charge: clamp_percent(self.memory.read_f32(address + SA_CHARGE).ok()?),
                timer: seconds(self.memory.read_f32(address + SA_TIMER).ok()?),
            }),
            WeaponKind::DualBlades => {
                let max = self.memory.read_f32(address + DB_MAX_BUILDUP).ok()?;
                let buildup = self.memory.read_f32(address + DB_BUILDUP).ok()?;

                Some(WeaponGauge::DualBlades {
                    demon: self.memory.read_u8(address + DB_DEMON).ok()? != 0,
                    arch_demon: self.memory.read_u8(address + DB_ARCH_DEMON).ok()? != 0,
                    buildup: if max > 0.0 {
                        (buildup / max * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    },
                })
            }
            WeaponKind::InsectGlaive => self.insect_glaive(address),
            _ => None,
        }
    }

    /// Esencias, tiempos y aguante del kinsecto; el aguante máximo va en la misma estructura.
    fn insect_glaive(&self, address: u64) -> Option<WeaponGauge> {
        let mut extracts = Extracts::default();
        let array = self.memory.read_u64(address + IG_EXTRACTS).ok()?;

        if array != 0 {
            for index in 0..IG_EXTRACTS_COUNT {
                let kind = self
                    .memory
                    .read_i32(array + ARRAY_DATA + (index as u64) * 4)
                    .ok()?;

                match kind {
                    // `KinsectExtract`: rojo (ataque), blanco (velocidad), naranja (defensa).
                    0 => extracts.attack = true,
                    1 => extracts.speed = true,
                    2 => extracts.defense = true,
                    _ => {}
                }
            }
        }

        let stamina = self
            .memory
            .deref_chain(
                self.map.absolute("LOCAL_PLAYER_DATA_ADDRESS").ok()?,
                self.map.offsets("KINSECT_STAMINA_OFFSETS").ok()?,
            )
            .ok();
        let (current, max) = match stamina.filter(|address| *address != 0) {
            Some(stamina) => (
                self.memory
                    .read_f32(stamina + KINSECT_STAMINA_CURRENT)
                    .unwrap_or(0.0),
                self.memory
                    .read_f32(stamina + KINSECT_STAMINA_MAX)
                    .unwrap_or(0.0),
            ),
            None => (0.0, 0.0),
        };

        Some(WeaponGauge::InsectGlaive {
            extracts,
            attack: seconds(self.memory.read_f32(address + IG_ATTACK_TIMER).ok()?),
            speed: seconds(self.memory.read_f32(address + IG_SPEED_TIMER).ok()?),
            defense: seconds(self.memory.read_f32(address + IG_DEFENSE_TIMER).ok()?),
            stamina: current.max(0.0),
            max_stamina: max.max(0.0),
        })
    }

    /// Buffs y estados activos, del más duradero al más corto y con los negativos al final.
    fn buffs(&self) -> Vec<Buff> {
        let consumable = self.abnormality_base("CONS_ABNORMALITIES_OFFSETS");
        let debuff = self.abnormality_base("DEBUFF_ABNORMALITIES_OFFSETS");
        let mut buffs = Vec::new();

        for entry in RISE_BUFFS {
            let base = match entry.group {
                RiseGroup::Consumable => consumable,
                RiseGroup::Debuff => debuff,
            };
            let Some(base) = base else {
                continue;
            };

            if let Some((offset, value)) = entry.depends {
                let actual = self.memory.read_i32(base + u64::from(offset)).ok();
                if actual != Some(value) {
                    continue;
                }
            }

            if entry.infinite {
                buffs.push(Buff {
                    name: entry.name,
                    timer: 0.0,
                    max_timer: 0.0,
                    debuff: entry.debuff,
                    infinite: true,
                });
                continue;
            }

            let raw = match self.memory.read_f32(base + u64::from(entry.offset)) {
                Ok(value) if value.is_finite() => value,
                _ => continue,
            };
            let (timer, max_timer) = match entry.max_timer {
                // En los estados que guardan el tiempo transcurrido, el máximo es fijo.
                Some(max_timer) => (max_timer - seconds(raw), max_timer),
                None => {
                    let remaining = seconds(raw);
                    (remaining, self.remember_timer(entry.offset, remaining))
                }
            };

            if timer <= 0.0 || timer > MAX_BUFF_TIMER {
                continue;
            }

            buffs.push(Buff {
                name: entry.name,
                timer,
                max_timer: max_timer.max(timer),
                debuff: entry.debuff,
                infinite: false,
            });
        }

        buffs.extend(self.songs());
        buffs.sort_by(|left, right| {
            left.debuff.cmp(&right.debuff).then(
                right
                    .fraction()
                    .partial_cmp(&left.fraction())
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });
        buffs
    }

    /// Canciones de la cornamusa: un puntero por canción, en el orden de la tabla de HunterPie.
    fn songs(&self) -> Vec<Buff> {
        let Some(base) = self.abnormality_base("HH_ABNORMALITIES_OFFSETS") else {
            return Vec::new();
        };
        let count = match self.memory.read_i32(base + ARRAY_LENGTH) {
            Ok(count) if count > 0 => (count as usize).min(crate::buffs::RISE_SONGS.len()),
            _ => return Vec::new(),
        };
        let pointers = match self.memory.read_pointers(base + ARRAY_DATA, count) {
            Ok(pointers) => pointers,
            Err(_) => return Vec::new(),
        };
        let mut songs = Vec::new();

        for (index, pointer) in pointers.into_iter().enumerate() {
            let Some(name) = crate::buffs::RISE_SONGS[index] else {
                continue;
            };
            if pointer == 0 {
                continue;
            }

            let raw = match self.memory.read_f32(pointer + SONG_TIMER) {
                Ok(value) if value.is_finite() => value,
                _ => continue,
            };
            let timer = seconds(raw);
            if timer <= 0.0 || timer > MAX_BUFF_TIMER {
                continue;
            }

            let max_timer = self.remember_timer(SONG_KEY + index as u32, timer);
            songs.push(Buff {
                name,
                timer,
                max_timer: max_timer.max(timer),
                debuff: false,
                infinite: false,
            });
        }

        songs
    }

    /// Puntero al bloque de un grupo de estados (consumibles, estados negativos o canciones).
    fn abnormality_base(&self, key: &str) -> Option<u64> {
        let address = self
            .memory
            .deref_chain(
                self.map.absolute("ABNORMALITIES_ADDRESS").ok()?,
                self.map.offsets(key).ok()?,
            )
            .ok()?;

        (address != 0).then_some(address)
    }

    /// Duración máxima vista de un estado; crece cuando el buff se refresca.
    fn remember_timer(&self, key: u32, value: f32) -> f32 {
        let mut seen = match self.seen_buffs.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = seen.entry(key).or_insert(value);
        if value > *entry {
            *entry = value;
        }

        *entry
    }

    /// Lee un array de `f32` con cabecera de tamaño (Rise la guarda en +0x1C).
    fn array_f32(&self, address: u64, limit: usize) -> Option<Vec<f32>> {
        if address == 0 {
            return None;
        }
        let count = self.memory.read_i32(address + ARRAY_LENGTH).ok()?;
        let count = usize::try_from(count).ok()?.min(limit);

        self.memory.read_f32_array(address + ARRAY_DATA, count).ok()
    }

    /// Estado de la cacería (`QuestState`), si el juego tiene ya la estructura.
    fn quest_state(&self) -> Option<i32> {
        let quest = self
            .memory
            .deref_chain(
                self.map.absolute("QUEST_ADDRESS").ok()?,
                self.map.offsets("QUEST_OFFSETS").ok()?,
            )
            .ok()?;

        if quest == 0 {
            return None;
        }

        self.memory.read_i32(quest + QUEST_STATE).ok()
    }

    /// Cacería en curso: el reloj corre hacia atrás y las vidas se cuentan por el equipo.
    pub fn hunt(&self) -> Result<Option<Hunt>, MemoryError> {
        let quest = self.memory.deref_chain(
            self.map.absolute("QUEST_ADDRESS")?,
            self.map.offsets("QUEST_OFFSETS")?,
        )?;

        if quest == 0 || self.memory.read_i32(quest + QUEST_STATE)? != QUEST_STATE_IN_QUEST {
            return Ok(None);
        }

        let max_deaths = self.memory.read_i32(quest + QUEST_MAX_DEATHS)?;
        let deaths = self.memory.read_i32(quest + QUEST_DEATHS)?;
        if max_deaths <= 0 || deaths < 0 {
            return Ok(None);
        }

        let hunt = Hunt {
            time_left: self.memory.read_f32(quest + QUEST_TIME_LIMIT)?
                - self.memory.read_f32(quest + QUEST_TIME_ELAPSED)?,
            deaths: deaths as u32,
            max_deaths: max_deaths as u32,
        };

        Ok(hunt.is_plausible().then_some(hunt))
    }

    /// Monstruos grandes presentes ahora mismo. Una zona sin monstruos devuelve una lista
    /// vacía, no un error.
    pub fn monsters(&self) -> Result<Vec<Monster>, MemoryError> {
        // Sin cacería no hay zona: el juego mantiene los contenedores a medio inicializar y
        // leerlos devuelve punteros que no son monstruos. Preguntar por el estado de la cacería
        // es lo que distingue «no hay monstruos» de «los offsets están mal».
        if self.quest_state() != Some(QUEST_STATE_IN_QUEST) {
            return Ok(Vec::new());
        }

        let Some(list) = optional_chain(
            &self.memory,
            self.map.absolute("MONSTERS_ADDRESS")?,
            self.map.offsets("MONSTER_LIST_OFFSETS")?,
        )?
        else {
            return Ok(Vec::new());
        };

        let pointers = self.memory.read_pointers(list, MAX_MONSTERS)?;
        let target = self.target();
        let mut monsters = Vec::new();

        for pointer in pointers {
            if pointer == 0 {
                continue;
            }

            match self.monster(pointer) {
                // Una vida imposible viene de un desplazamiento desfasado, no de un monstruo.
                Ok(mut monster) if monster.health.is_plausible() => {
                    monster.target = Some(pointer) == target;
                    monsters.push(monster);
                }
                Ok(_) => continue,
                // Un monstruo despawneando, o una zona a medio cargar, deja punteros a medio
                // limpiar: uno ilegible no invalida los demás ni el resto del panel.
                Err(MemoryError::NullPointer { .. } | MemoryError::Read { .. }) => continue,
                Err(error) => return Err(error),
            }
        }

        Ok(monsters)
    }

    /// Dirección del monstruo que el juego marca como objetivo de la cacería.
    ///
    /// `QUEST_AUTOMATIC_TARGET` solo está definido en los mapas de 16.0.1.1 en adelante; con
    /// uno anterior no hay objetivo que leer y el panel enseña la zona entera.
    fn target(&self) -> Option<u64> {
        let base = self.map.absolute("QUEST_GUI_ADDRESS").ok()?;
        let offsets = self.map.offsets("QUEST_AUTOMATIC_TARGET").ok()?;

        self.memory
            .deref_chain(base, offsets)
            .ok()
            .filter(|address| *address != 0)
    }

    fn monster(&self, address: u64) -> Result<Monster, MemoryError> {
        let id = self.memory.read_i32(address + MONSTER_ID)?;
        let health = self.health(address)?;
        let mut monster = Monster::new(id, health);

        monster.name = crate::tables::rise_monster_name(id).map(str::to_owned);
        monster.parts = self.parts(address, id).unwrap_or_default();
        monster.ailments = self.ailments(address).unwrap_or_default();
        monster.capture_percent = self.capture_percent(address, health);
        monster.crown = self.crown(address, id);

        Ok(monster)
    }

    /// Corona según el tamaño sorteado para esta instancia. Los monstruos de talla fija no
    /// tienen umbrales en la tabla y nunca llevan corona.
    fn crown(&self, monster: u64, id: i32) -> Crown {
        let Some(thresholds) = crate::tables::rise_crown_thresholds(id) else {
            return Crown::None;
        };
        let Ok(offsets) = self.map.offsets("MONSTER_CROWN_OFFSETS") else {
            return Crown::None;
        };
        let Ok(size) = self.memory.follow_ptr_chain(monster, offsets) else {
            return Crown::None;
        };
        let (Ok(multiplier), Ok(correction)) = (
            self.memory.read_f32(size + MONSTER_SIZE_MULTIPLIER),
            self.memory.read_f32(size + MONSTER_SIZE_CORRECTION),
        ) else {
            return Crown::None;
        };

        Crown::of(multiplier * correction, thresholds)
    }

    fn health(&self, monster: u64) -> Result<Health, MemoryError> {
        let component = self.memory.follow_ptr_chain(
            monster,
            self.map.offsets("MONSTER_HEALTH_COMPONENT_OFFSETS")?,
        )?;
        let encoded = self.memory.follow_ptr_chain(
            component,
            self.map
                .offsets("MONSTER_HEALTH_COMPONENT_ENCODED_OFFSETS")?,
        )?;

        Ok(Health::new(
            self.memory.read_f32(encoded + HEALTH_CURRENT)?,
            self.memory.read_f32(component + HEALTH_MAX)?,
        ))
    }

    fn capture_percent(&self, monster: u64, health: Health) -> Option<f32> {
        let threshold = self
            .memory
            .follow_ptr_chain(
                monster,
                self.map.offsets("MONSTER_CAPTURE_HEALTH_THRESHOLD").ok()?,
            )
            .ok()?;
        let capture_health = self
            .memory
            .read_f32(threshold + CAPTURE_THRESHOLD_VALUE)
            .ok()?;

        let percent = (capture_health / health.max) * 100.0;
        percent.is_finite().then_some(percent)
    }

    /// Las tres resistencias de una parte viven en arrays paralelos: flinch, romper y cortar.
    /// Se recorren juntos porque el índice `i` describe la misma parte en los tres.
    fn parts(&self, monster: u64, id: i32) -> Result<Vec<Part>, MemoryError> {
        let flinch = self.part_array(monster, "MONSTER_FLINCH_HEALTH_COMPONENT_OFFSETS")?;
        let breakable = self.part_array(monster, "MONSTER_BREAK_HEALTH_COMPONENT_OFFSETS")?;
        let severable = self.part_array(monster, "MONSTER_SEVER_HEALTH_COMPONENT_OFFSETS")?;

        if flinch.len() != breakable.len() || flinch.len() != severable.len() {
            return Ok(Vec::new());
        }

        let encoded = self
            .map
            .offsets("MONSTER_HEALTH_COMPONENT_ENCODED_OFFSETS")?;
        let mut parts = Vec::with_capacity(flinch.len());

        for index in 0..flinch.len() {
            let Some(max_flinch) = self.max_component(flinch[index]) else {
                continue;
            };
            let Some(max_breakable) = self.max_component(breakable[index]) else {
                continue;
            };
            let Some(max_severable) = self.max_component(severable[index]) else {
                continue;
            };

            // HunterPie descarta la entrada cuando las tres resistencias son negativas: es un
            // hueco del array, no una parte.
            if max_flinch < 0.0 && max_breakable < 0.0 && max_severable < 0.0 {
                continue;
            }

            let flinch_health = self.current_component(flinch[index], encoded, max_flinch);
            let breakable_health = self.current_component(breakable[index], encoded, max_breakable);
            let severable_health = self.current_component(severable[index], encoded, max_severable);

            // La vida de la parte es la del componente que rompe (el que enseña HunterPie);
            // si esa parte no se rompe, vale la de corte y, en último caso, la de flinch.
            let health = breakable_health.or(severable_health).or(flinch_health);

            parts.push(Part {
                index,
                name: crate::tables::rise_part_name(id, index),
                flinch: flinch_health,
                breakable: breakable_health,
                severable: severable_health,
                health,
                breaks: None,
            });
        }

        Ok(parts)
    }

    fn part_array(&self, monster: u64, key: &str) -> Result<Vec<u64>, MemoryError> {
        let container = self
            .memory
            .follow_ptr_chain(monster, self.map.offsets(key)?)?;
        let length = self.memory.read_i32(container + PART_ARRAY_LENGTH)?;

        if length <= 0 || length > MAX_PARTS {
            return Ok(Vec::new());
        }

        self.memory
            .read_pointers(container + PART_ARRAY_DATA, length as usize)
    }

    fn max_component(&self, part: u64) -> Option<f32> {
        if part == 0 {
            return None;
        }

        self.memory.read_f32(part + HEALTH_MAX).ok()
    }

    fn current_component(&self, part: u64, encoded: &[i32], max: f32) -> Option<Health> {
        if part == 0 || max <= 0.0 {
            return None;
        }

        let encoded_ptr = self.memory.follow_ptr_chain(part, encoded).ok()?;
        let current = self.memory.read_f32(encoded_ptr + HEALTH_CURRENT).ok()?;

        Some(Health::new(current, max))
    }

    fn ailments(&self, monster: u64) -> Result<Vec<Ailment>, MemoryError> {
        let array = self
            .memory
            .follow_ptr_chain(monster, self.map.offsets("MONSTER_AILMENTS_OFFSETS")?)?;
        let pointers = self.memory.read_pointers(array, AILMENTS.len())?;
        let mut ailments = Vec::new();

        for (index, pointer) in pointers.into_iter().enumerate() {
            if pointer == 0 {
                continue;
            }

            let Some(ailment) = self.ailment(pointer, AILMENTS[index]) else {
                continue;
            };

            if ailment.is_active() {
                ailments.push(ailment);
            }
        }

        Ok(ailments)
    }

    fn ailment(&self, pointer: u64, name: &'static str) -> Option<Ailment> {
        let buildup_ptr = self.memory.read_u64(pointer + AILMENT_BUILDUP_PTR).ok()?;
        let max_buildup_ptr = self
            .memory
            .read_u64(pointer + AILMENT_MAX_BUILDUP_PTR)
            .ok()?;

        if buildup_ptr == 0 || max_buildup_ptr == 0 {
            return None;
        }

        let buildup = self.memory.read_f32(buildup_ptr + AILMENT_VALUE).ok()?;
        let max_buildup = self.memory.read_f32(max_buildup_ptr + AILMENT_VALUE).ok()?;
        let timer = self.memory.read_f32(pointer + AILMENT_TIMER).ok()?;
        let max_timer = self.memory.read_f32(pointer + AILMENT_MAX_TIMER).ok()?;

        Some(Ailment {
            name,
            buildup,
            max_buildup,
            timer,
            max_timer,
        })
    }
}
