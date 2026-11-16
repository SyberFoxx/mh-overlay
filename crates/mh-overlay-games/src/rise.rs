//! Lectura del estado de los monstruos en Monster Hunter Rise / Sunbreak.
//!
//! Todos los desplazamientos fijos de esta sección fueron verificados contra la build
//! 16.0.2.0 y están documentados junto a la estructura de la que salen.

use mh_overlay_memory::{AddressMap, Memory, MemoryError};

use crate::model::{Ailment, Health, Monster, Part};

/// Monstruos grandes que pueden coexistir en una zona.
const MAX_MONSTERS: usize = 5;

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

/// Lector de Rise sobre un proceso ya adjuntado.
pub struct Rise<'map> {
    memory: Memory,
    map: &'map AddressMap,
}

impl<'map> Rise<'map> {
    /// El `map` debe tener ya aplicada la base del módulo.
    pub fn new(memory: Memory, map: &'map AddressMap) -> Self {
        Self { memory, map }
    }

    /// Monstruos grandes presentes ahora mismo. Una zona sin monstruos devuelve una lista
    /// vacía, no un error.
    pub fn monsters(&self) -> Result<Vec<Monster>, MemoryError> {
        let list = self.memory.deref_chain(
            self.map.absolute("MONSTERS_ADDRESS")?,
            self.map.offsets("MONSTER_LIST_OFFSETS")?,
        )?;

        let pointers = self.memory.read_pointers(list, MAX_MONSTERS)?;
        let mut monsters = Vec::new();

        for pointer in pointers {
            if pointer == 0 {
                continue;
            }

            match self.monster(pointer) {
                // Una vida imposible viene de un desplazamiento desfasado, no de un monstruo.
                Ok(monster) if monster.health.is_plausible() => monsters.push(monster),
                Ok(_) => continue,
                // Un monstruo despawnando deja punteros a medio limpiar.
                Err(MemoryError::NullPointer { .. }) => continue,
                Err(error) => return Err(error),
            }
        }

        Ok(monsters)
    }

    fn monster(&self, address: u64) -> Result<Monster, MemoryError> {
        let id = self.memory.read_i32(address + MONSTER_ID)?;
        let health = self.health(address)?;
        let mut monster = Monster::new(id, health);

        monster.parts = self.parts(address, id).unwrap_or_default();
        monster.ailments = self.ailments(address).unwrap_or_default();
        monster.capture_percent = self.capture_percent(address, health);

        Ok(monster)
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

            parts.push(Part {
                index,
                name: crate::tables::rise_part_name(id, index),
                flinch: self.current_component(flinch[index], encoded, max_flinch),
                breakable: self.current_component(breakable[index], encoded, max_breakable),
                severable: self.current_component(severable[index], encoded, max_severable),
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
