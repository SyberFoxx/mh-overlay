//! Lectura del estado de los monstruos en Monster Hunter: World / Iceborne.
//!
//! World guarda el estado de forma menos uniforme que Rise: la lista de componentes se
//! recorre hasta 128 entradas, el prefijo `em` del modelo identifica a los monstruos grandes
//! y las partes son arrays de tamaño fijo con dos strides distintos.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use mh_overlay_memory::{AddressMap, Memory, MemoryError};

use crate::buffs::WORLD_BUFFS;
use crate::model::{
    Ailment, Buff, Extracts, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel,
    Snapshot, Weapon, WeaponGauge, WeaponKind,
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
/// `MHWSharpnessStructure`: unidades de filo actuales, color y color máximo del arma.
const SHARPNESS_CURRENT: u64 = 0x20F8;
const SHARPNESS_LEVEL: u64 = 0x20FC;
const SHARPNESS_MAX_LEVEL: u64 = 0x1D10;
/// Dentro de la tabla de datos del arma, el array de longitudes de filo empieza aquí.
const SHARPNESS_LENGTHS_OFFSET: u64 = 0x0C;

// Medidores de clase, todos dentro del bloque `WEAPON_MECHANICS_OFFSETS` (HunterPie:
// `MHWLongSwordStructure`, `MHWChargeBladeStructure`, `MHWSwitchAxeStructure`,
// `MHWDualBladesStructure`, `MHWInsectGlaiveStructure`).

/// Espada larga: carga, nivel, temporizador del nivel y regeneración (iai / helm breaker).
const LS_BUILDUP: u64 = 0x2368;
const LS_SPIRIT_LEVEL: u64 = 0x2370;
const LS_LEVEL_TIMER: u64 = 0x2374;
const LS_IAI_TIMER: u64 = 0x2378;
const LS_HELM_TIMER: u64 = 0x2388;
/// Duración de cada nivel de espíritu, en segundos, sin la habilidad de prolongación.
const LS_LEVEL_DURATIONS: [f32; 4] = [0.0, 200.0, 140.0, 70.0];
/// Espada cargada: carga, frascos y buffs de escudo, espada y hacha.
const CB_CHARGE: u64 = 0x2370;
const CB_PHIALS: u64 = 0x2374;
const CB_SHIELD: u64 = 0x2378;
const CB_SWORD: u64 = 0x237C;
const CB_AXE: u64 = 0x2470;
/// Frascos por defecto; la habilidad de capacidad añade uno.
const CB_MAX_PHIALS: u8 = 5;
/// Hacha espada: carga de la hoja, carga amped y su temporizador.
const SA_BUILDUP: u64 = 0x2350;
const SA_CHARGE: u64 = 0x2360;
const SA_TIMER: u64 = 0x2364;
/// Espadas dobles: modo demonio, archidemonio y el medidor.
const DB_DEMON: u64 = 0x2368;
const DB_ARCH_DEMON: u64 = 0x2369;
const DB_BUILDUP: u64 = 0x236C;
/// Glaive insecto: puntero al kinsecto y temporizadores de las esencias.
const IG_KINSECT: u64 = 0x2350;
const IG_ATTACK_TIMER: u64 = 0x2368;
const IG_SPEED_TIMER: u64 = 0x236C;
const IG_DEFENSE_TIMER: u64 = 0x2370;
const IG_BUFF_QUEUE: u64 = 0x2378;
const IG_QUEUE_LENGTH: usize = 3;
const IG_QUEUED_INDEX: u64 = 0x2388;
const IG_QUEUED_COUNT: u64 = 0x2390;
/// `MHWKinsectStructure`: aguante del kinsecto.
const KINSECT_STAMINA: u64 = 0x0A8C;
/// `MHWInsectGlaiveExtraStructure`: aguante máximo, en la cadena extra.
const IG_MAX_STAMINA: u64 = 0x12B4;
/// Tope de cordura para un temporizador de buff: más de una hora es basura.
const MAX_BUFF_TIMER: f32 = 3600.0;

/// Normaliza una fracción del juego (0..1) a porcentaje y descarta NaN.
fn percent(value: f32) -> f32 {
    if value.is_finite() {
        (value * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Los medidores que el juego ya guarda en porcentaje (carga de la espada cargada, hojas del
/// hacha espada) solo necesitan descartar NaN y valores fuera de rango.
fn clamp_percent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// `MHWMonsterPartStructure`: vida máxima, vida actual, contador de golpes e índice.
const PART_MAX_HEALTH: u64 = 0x0C;
const PART_HEALTH: u64 = 0x10;
const PART_COUNTER: u64 = 0x18;
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
    /// Duración máxima vista de cada buff, por offset: el juego solo guarda lo que queda y
    /// una barra necesita el total.
    seen_buffs: Mutex<HashMap<u32, f32>>,
}

impl<'map> World<'map> {
    /// El `map` debe tener ya aplicada la base del módulo.
    pub fn new(memory: Memory, map: &'map AddressMap) -> Self {
        Self {
            memory,
            map,
            seen_buffs: Mutex::new(HashMap::new()),
        }
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
            buffs: self.buffs(),
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
        let gauge = self.gauge(kind);

        Some(Weapon {
            kind,
            sharpness,
            gauge,
        })
    }

    /// Base del bloque de medidores de clase, común a todas las armas.
    fn mechanics(&self) -> Option<u64> {
        let address = self
            .memory
            .deref_chain(
                self.map.absolute("WEAPON_MECHANICS_ADDRESS").ok()?,
                self.map.offsets("WEAPON_MECHANICS_OFFSETS").ok()?,
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
                let level = self.memory.read_i32(address + LS_SPIRIT_LEVEL).ok()?;
                if !(0..=3).contains(&level) {
                    return None;
                }

                let buildup = self.memory.read_f32(address + LS_BUILDUP).ok()?;
                let timer = self.memory.read_f32(address + LS_LEVEL_TIMER).ok()?;
                let regen = self
                    .memory
                    .read_f32(address + LS_IAI_TIMER)
                    .unwrap_or(0.0)
                    .max(self.memory.read_f32(address + LS_HELM_TIMER).unwrap_or(0.0));

                Some(WeaponGauge::Spirit {
                    buildup: percent(buildup),
                    level: level as u8,
                    timer: timer.max(0.0),
                    max_timer: LS_LEVEL_DURATIONS[level as usize],
                    regen: regen.max(0.0),
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
                    shield: self.memory.read_f32(address + CB_SHIELD).ok()?.max(0.0),
                    sword: self.memory.read_f32(address + CB_SWORD).ok()?.max(0.0),
                    axe: self.memory.read_f32(address + CB_AXE).ok()?.max(0.0),
                })
            }
            WeaponKind::SwitchAxe => Some(WeaponGauge::SwitchAxe {
                buildup: clamp_percent(self.memory.read_f32(address + SA_BUILDUP).ok()?),
                charge: clamp_percent(self.memory.read_f32(address + SA_CHARGE).ok()?),
                timer: self.memory.read_f32(address + SA_TIMER).ok()?.max(0.0),
            }),
            WeaponKind::DualBlades => Some(WeaponGauge::DualBlades {
                demon: self.memory.read_u8(address + DB_DEMON).ok()? != 0,
                arch_demon: self.memory.read_u8(address + DB_ARCH_DEMON).ok()? != 0,
                buildup: percent(self.memory.read_f32(address + DB_BUILDUP).ok()?),
            }),
            WeaponKind::InsectGlaive => self.insect_glaive(address),
            _ => None,
        }
    }

    /// Esencias, tiempos y aguante del kinsecto; el aguante máximo está en la cadena extra.
    fn insect_glaive(&self, address: u64) -> Option<WeaponGauge> {
        let mut extracts = Extracts::default();
        let queued = self.memory.read_i32(address + IG_QUEUED_COUNT).ok()?;
        let start = self.memory.read_i32(address + IG_QUEUED_INDEX).ok()?;

        if (1..=IG_QUEUE_LENGTH as i32).contains(&queued)
            && (0..IG_QUEUE_LENGTH as i32).contains(&start)
        {
            for step in 0..queued {
                let slot = (start + step) % IG_QUEUE_LENGTH as i32;
                let kind = self
                    .memory
                    .read_i32(address + IG_BUFF_QUEUE + (slot as u64) * 4)
                    .ok()?;

                match kind {
                    0 => extracts.attack = true,
                    1 => extracts.speed = true,
                    2 => extracts.defense = true,
                    3 => extracts.heal = true,
                    _ => {}
                }
            }
        }

        let max_stamina = self
            .memory
            .deref_chain(
                self.map.absolute("WEAPON_MECHANICS_ADDRESS").ok()?,
                self.map
                    .offsets("WEAPON_EXTRA_MECHANICS_DATA_OFFSETS")
                    .ok()?,
            )
            .ok()
            .filter(|address| *address != 0)
            .and_then(|address| self.memory.read_f32(address + IG_MAX_STAMINA).ok())
            .unwrap_or(0.0);

        let kinsect = self.memory.read_u64(address + IG_KINSECT).ok()?;
        let stamina = if kinsect == 0 {
            0.0
        } else {
            self.memory
                .read_f32(kinsect + KINSECT_STAMINA)
                .unwrap_or(0.0)
        };

        Some(WeaponGauge::InsectGlaive {
            extracts,
            attack: self
                .memory
                .read_f32(address + IG_ATTACK_TIMER)
                .ok()?
                .max(0.0),
            speed: self
                .memory
                .read_f32(address + IG_SPEED_TIMER)
                .ok()?
                .max(0.0),
            defense: self
                .memory
                .read_f32(address + IG_DEFENSE_TIMER)
                .ok()?
                .max(0.0),
            stamina: stamina.max(0.0),
            max_stamina: max_stamina.max(0.0),
        })
    }

    /// Buffs y estados activos, del más duradero al más corto y con los negativos al final.
    fn buffs(&self) -> Vec<Buff> {
        let base = self
            .memory
            .deref_chain(
                self.map.absolute("ABNORMALITY_ADDRESS").ok().unwrap_or(0),
                self.map.offsets("ABNORMALITY_OFFSETS").ok().unwrap_or(&[]),
            )
            .unwrap_or(0);

        let canteen = self
            .memory
            .deref_chain(
                self.map.absolute("CANTEEN_ADDRESS").ok().unwrap_or(0),
                self.map
                    .offsets("ABNORMALITY_CANTEEN_OFFSETS")
                    .ok()
                    .unwrap_or(&[]),
            )
            .unwrap_or(0);

        if base == 0 {
            return Vec::new();
        }

        let mut seen = self
            .seen_buffs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut buffs = Vec::new();

        for entry in WORLD_BUFFS {
            let address = if entry.canteen { canteen } else { base };
            if address == 0 {
                continue;
            }

            if let Some((depends_on, value)) = entry.depends {
                let actual = self
                    .memory
                    .read_i32(address + u64::from(depends_on))
                    .unwrap_or(0);
                if actual != value {
                    continue;
                }
            }

            let timer = if entry.infinite {
                1.0
            } else {
                self.memory
                    .read_f32(address + u64::from(entry.offset))
                    .unwrap_or(0.0)
            };

            if !timer.is_finite() || timer <= 0.0 || timer > MAX_BUFF_TIMER {
                seen.remove(&entry.offset);
                continue;
            }

            let max_timer = seen
                .entry(entry.offset)
                .and_modify(|known| *known = known.max(timer))
                .or_insert(timer);

            buffs.push(Buff {
                name: entry.name,
                timer,
                max_timer: *max_timer,
                debuff: entry.debuff,
                infinite: entry.infinite,
            });
        }

        // Los negativos al final, y dentro de cada grupo lo que antes se va.
        buffs.sort_by(|left, right| {
            left.debuff.cmp(&right.debuff).then(
                left.timer
                    .partial_cmp(&right.timer)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        buffs
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

    /// Se reportan las partes que el juego tiene inicializadas y que además se pueden romper
    /// o cortar. La tabla del monstruo es la que pone nombre e índice: las partes normales no
    /// guardan su índice y el juego las registra en el mismo orden que la tabla, saltándose
    /// las cortables, que viven en su propia zona.
    fn parts(&self, monster: u64, id: i32) -> Result<Vec<Part>, MemoryError> {
        let block = self.memory.read_u64(monster + MONSTER_PARTS)?;
        let definitions = crate::tables::world_parts(id);

        if block == 0 || definitions.is_empty() {
            return Ok(Vec::new());
        }

        let mut parts = Vec::new();

        for (slot, definition) in definitions
            .iter()
            .filter(|part| !part.severable)
            .enumerate()
        {
            let address = block + NORMAL_PARTS + slot as u64 * NORMAL_PART_STRIDE;

            let Some(health) = self.part_health(address) else {
                break;
            };

            // Cuello, brazos y demás partes que solo cuentan para el aturdimiento no se
            // rompen: mostrarlas como una barra al 100 % solo confunde.
            if let Some((breakable, breaks)) = self.break_progress(definition, address, health) {
                parts.push(Part {
                    index: definition.index,
                    name: Some(definition.name),
                    flinch: Some(health),
                    breakable: Some(breakable),
                    severable: None,
                    breaks: Some(breaks),
                });
            }
        }

        // Las cortables viven en otra zona y sus entradas aparecen desplazadas 8 bytes cuando
        // la referencia inicial es pequeña; HunterPie aplica el mismo ajuste antes de leer. El
        // índice del juego es lo que las identifica.
        let mut address = block + SEVERABLE_PARTS;
        let window_end = address + SEVERABLE_WINDOW;

        for definition in definitions.iter().filter(|part| part.severable) {
            while address < window_end {
                if self
                    .memory
                    .read_u32(address)
                    .is_ok_and(|reference| reference <= 0xA0)
                {
                    address += 8;
                }

                if self.part_index(address) == Some(definition.index)
                    && let Some(health) = self.part_health(address)
                {
                    parts.push(Part {
                        index: definition.index,
                        name: Some(definition.name),
                        flinch: None,
                        breakable: None,
                        severable: Some(health),
                        breaks: None,
                    });

                    address += SEVERABLE_PART_STRIDE;
                    break;
                }

                address += SEVERABLE_PART_STRIDE;
            }
        }

        parts.sort_by_key(|part| part.index);

        Ok(parts)
    }

    /// Vida de la barra de rompimiento y golpes hechos, escalados al siguiente umbral como
    /// hace HunterPie: la vida de una parte se regenera, así que lo que importa es cuántos
    /// rompimientos ha aguantado y cuánto lleva del siguiente.
    fn break_progress(
        &self,
        definition: &crate::tables::WorldPart,
        address: u64,
        health: Health,
    ) -> Option<(Health, (u32, u32))> {
        if definition.breaks.is_empty() {
            return None;
        }

        let counter = self.memory.read_u32(address + PART_COUNTER).ok()?;
        let total = u32::from(*definition.breaks.last()?);
        if total == 0 {
            return None;
        }

        let done = counter.min(total);
        let next = definition
            .breaks
            .iter()
            .map(|threshold| u32::from(*threshold))
            .find(|threshold| *threshold > done);

        // Sin umbral pendiente la parte está rota del todo: la barra se queda llena.
        let Some(next) = next else {
            let max = health.max * total as f32;
            return Some((Health::new(max, max), (done, total)));
        };

        let max = health.max * next as f32;
        let current = (next - done - 1) as f32 * health.max + health.current;

        Some((Health::new(current, max), (done, total)))
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
