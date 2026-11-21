//! Memoria de mentira con la disposición que esperan los lectores.
//!
//! La fixture es un mapa disperso de bytes sobre una base fija: `AddressMap::set_base` apunta
//! ahí y los lectores recorren las mismas cadenas que contra un proceso vivo, así que un test
//! cubre a la vez el `.map` que se carga, las cadenas de punteros y el lector. Las direcciones
//! que nadie escribe se leen como cero, igual que una página vacía.

use std::collections::BTreeMap;
use std::path::Path;

use mh_overlay_memory::{AddressMap, MemoryError, Reads};

/// Base del módulo en la fixture, la misma que usa Rise al cargar.
pub const BASE: u64 = 0x1_4000_0000;

/// Imagen de memoria para un test: bytes por dirección, sin tamaño reservado.
pub struct Fixture {
    bytes: BTreeMap<u64, u8>,
    next: u64,
}

impl Fixture {
    pub fn new() -> Self {
        Self {
            bytes: BTreeMap::new(),
            next: BASE + 0x1000_0000,
        }
    }

    /// Una dirección libre para colocar una estructura.
    pub fn allocate(&mut self) -> u64 {
        let address = self.next;
        self.next += 0x100;
        address
    }

    pub fn write_bytes(&mut self, address: u64, bytes: &[u8]) {
        for (index, byte) in bytes.iter().enumerate() {
            self.bytes.insert(address + index as u64, *byte);
        }
    }

    pub fn write_u64(&mut self, address: u64, value: u64) {
        self.write_bytes(address, &value.to_le_bytes());
    }

    pub fn write_i32(&mut self, address: u64, value: i32) {
        self.write_bytes(address, &value.to_le_bytes());
    }

    pub fn write_f32(&mut self, address: u64, value: f32) {
        self.write_bytes(address, &value.to_le_bytes());
    }

    pub fn write_cstring(&mut self, address: u64, text: &str) {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        self.write_bytes(address, &bytes);
    }

    /// Coloca una cadena `deref_chain` desde `start` que acaba en `target`.
    pub fn seed_deref(&mut self, start: u64, offsets: &[i32], target: u64) {
        let mut value = target;

        for index in (0..offsets.len()).rev() {
            let slot = if index == 0 { start } else { self.allocate() };
            let written = value.wrapping_sub_signed(i64::from(offsets[index]));
            self.write_u64(slot, written);
            value = slot;
        }
    }

    /// Coloca una cadena `follow_ptr_chain` desde `start` que acaba en `target`.
    ///
    /// Reutiliza los nodos de un prefijo ya escrito: en el juego, la vida y las tres
    /// resistencias de las partes comparten los primeros saltos y solo se separan al final.
    ///
    /// Un último salto se escribe siempre, así que una cadena de un solo paso (captura,
    /// corona) no se siembra aparte: se siembra la larga y se lee dónde acaba con
    /// [`Reads::follow_ptr_chain`], que es el nodo que ambas comparten.
    pub fn seed_follow(&mut self, start: u64, offsets: &[i32], target: u64) {
        let mut current = start;

        for (index, offset) in offsets.iter().enumerate() {
            let slot = current.wrapping_add_signed(i64::from(*offset));
            let last = index + 1 == offsets.len();

            let next = if last {
                self.write_u64(slot, target);
                target
            } else {
                match self.read_u64(slot) {
                    Ok(existing) if existing != 0 => existing,
                    _ => {
                        let fresh = self.allocate();
                        self.write_u64(slot, fresh);
                        fresh
                    }
                }
            };

            current = next;
        }
    }
}

impl Reads for Fixture {
    fn read_into(&self, address: u64, buffer: &mut [u8]) -> Result<(), MemoryError> {
        for (index, slot) in buffer.iter_mut().enumerate() {
            *slot = self
                .bytes
                .get(&(address + index as u64))
                .copied()
                .unwrap_or(0);
        }

        Ok(())
    }
}

fn map_for(game: &str) -> AddressMap {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/maps");
    let path = AddressMap::newest_for_game(game, &directory).expect("hay mapas del juego");
    let mut map = AddressMap::load(path).expect("el mapa más nuevo parsea");
    map.set_base(BASE);
    map
}

#[test]
fn a_world_without_a_zone_has_no_monsters_instead_of_an_error() {
    use crate::rise::Rise;
    use crate::world::World;

    let rise = map_for("MonsterHunterRise");
    let world = map_for("MonsterHunterWorld");

    // Una imagen sin nada escrito es un juego en el menú o en la aldea: todas las cadenas
    // acaban en cero, y eso significa «sin monstruos», no «offset roto».
    assert!(
        Rise::new(Fixture::new(), &rise)
            .monsters()
            .unwrap()
            .is_empty()
    );
    assert!(
        World::new(Fixture::new(), &world)
            .monsters()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rise_reads_a_snapshot_from_a_memory_image() {
    use crate::rise::Rise;

    let map = map_for("MonsterHunterRise");
    let mut fixture = Fixture::new();
    let id = (1..2000)
        .find(|id| crate::tables::rise_monster_name(*id).is_some())
        .expect("la tabla de Rise tiene monstruos");

    // HUD del jugador (`MHRPlayerHudStructure`, Pack = 1).
    let hud = fixture.allocate();
    fixture.seed_deref(
        map.absolute("UI_ADDRESS").unwrap(),
        map.offsets("PLAYER_HUD_OFFSETS").unwrap(),
        hud,
    );
    fixture.write_f32(hud, 120.0);
    fixture.write_f32(hud + 0x08, 150.0);
    fixture.write_f32(hud + 0x28, 80.0);
    fixture.write_f32(hud + 0x2C, 100.0);

    // Cacería en curso (`MHRQuestStructure`).
    let quest = fixture.allocate();
    fixture.seed_deref(
        map.absolute("QUEST_ADDRESS").unwrap(),
        map.offsets("QUEST_OFFSETS").unwrap(),
        quest,
    );
    fixture.write_i32(quest + 0x110, 2);
    fixture.write_i32(quest + 0x15C, 3);
    fixture.write_i32(quest + 0x160, 1);
    fixture.write_f32(quest + 0x170, 60.0);
    fixture.write_f32(quest + 0x178, 3000.0);

    // Lista de monstruos: una sola ranura ocupada.
    let list = fixture.allocate();
    fixture.seed_deref(
        map.absolute("MONSTERS_ADDRESS").unwrap(),
        map.offsets("MONSTER_LIST_OFFSETS").unwrap(),
        list,
    );
    let monster = fixture.allocate();
    fixture.write_u64(list, monster);
    fixture.write_i32(monster + 0x2D4, id);

    // Vida: el componente comparte prefijo con las resistencias de las partes.
    let component = fixture.allocate();
    fixture.seed_follow(
        monster,
        map.offsets("MONSTER_HEALTH_COMPONENT_OFFSETS").unwrap(),
        component,
    );
    fixture.write_f32(component + 0x18, 2000.0);
    let encoded = fixture.allocate();
    fixture.seed_follow(
        component,
        map.offsets("MONSTER_HEALTH_COMPONENT_ENCODED_OFFSETS")
            .unwrap(),
        encoded,
    );
    fixture.write_f32(encoded + 0x18, 500.0);

    // Captura y corona viven en el mismo nodo que el componente de vida: su cadena es el
    // primer salto, compartido, y el valor va detrás.
    let spine = fixture
        .follow_ptr_chain(
            monster,
            map.offsets("MONSTER_CAPTURE_HEALTH_THRESHOLD").unwrap(),
        )
        .unwrap();
    fixture.write_f32(spine + 0x1C, 200.0);

    let crown = crate::tables::rise_crown_thresholds(id);
    if let Some(thresholds) = crown {
        fixture.write_f32(spine + 0x24, thresholds.silver);
        fixture.write_f32(spine + 0x28, 1.0);
    }

    // Una parte con sus tres resistencias: flinch 100, romper 200, cortar 300.
    for (key, max, current) in [
        ("MONSTER_FLINCH_HEALTH_COMPONENT_OFFSETS", 100.0, 50.0),
        ("MONSTER_BREAK_HEALTH_COMPONENT_OFFSETS", 200.0, 100.0),
        ("MONSTER_SEVER_HEALTH_COMPONENT_OFFSETS", 300.0, 150.0),
    ] {
        let container = fixture.allocate();
        fixture.seed_follow(monster, map.offsets(key).unwrap(), container);
        fixture.write_i32(container + 0x1C, 1);

        let part = fixture.allocate();
        fixture.write_u64(container + 0x20, part);
        fixture.write_f32(part + 0x18, max);

        let part_encoded = fixture.allocate();
        fixture.seed_follow(
            part,
            map.offsets("MONSTER_HEALTH_COMPONENT_ENCODED_OFFSETS")
                .unwrap(),
            part_encoded,
        );
        fixture.write_f32(part_encoded + 0x18, current);
    }

    // Un estado alterado: parálisis al 40 % con 30 tics de temporizador.
    let ailments = fixture.allocate();
    fixture.seed_follow(
        monster,
        map.offsets("MONSTER_AILMENTS_OFFSETS").unwrap(),
        ailments,
    );
    let ailment = fixture.allocate();
    fixture.write_u64(ailments, ailment);
    let buildup = fixture.allocate();
    let max_buildup = fixture.allocate();
    fixture.write_u64(ailment + 0x68, buildup);
    fixture.write_u64(ailment + 0x78, max_buildup);
    fixture.write_f32(buildup + 0x20, 40.0);
    fixture.write_f32(max_buildup + 0x20, 100.0);
    fixture.write_f32(ailment + 0x44, 60.0);
    fixture.write_f32(ailment + 0x48, 30.0);

    let snapshot = Rise::new(fixture, &map).snapshot().unwrap();

    let player = snapshot.player.expect("el HUD da un jugador");
    assert_eq!(player.health.percent(), 80.0);
    assert_eq!(player.stamina.percent(), 80.0);
    assert!(player.weapon.is_none(), "la fixture no trae arma");

    let hunt = snapshot.hunt.expect("la cacería está en curso");
    assert_eq!(hunt.time_left, 2940.0);
    assert_eq!((hunt.deaths, hunt.max_deaths), (1, 3));

    assert_eq!(snapshot.monsters.len(), 1);
    let monster = &snapshot.monsters[0];
    assert_eq!(monster.id, id);
    assert_eq!(
        monster.name.as_deref(),
        crate::tables::rise_monster_name(id)
    );
    assert_eq!(monster.health.current, 500.0);
    assert_eq!(monster.health.max, 2000.0);
    assert_eq!(monster.health.percent(), 25.0);
    assert_eq!(monster.capture_percent, Some(10.0));
    assert!(!monster.target);

    match crown {
        Some(_) => assert_eq!(monster.crown, crate::model::Crown::Silver),
        None => assert_eq!(monster.crown, crate::model::Crown::None),
    }

    assert_eq!(monster.parts.len(), 1);
    let part = &monster.parts[0];
    assert_eq!(part.flinch.map(|health| health.current), Some(50.0));
    assert_eq!(part.breakable.map(|health| health.current), Some(100.0));
    assert_eq!(part.severable.map(|health| health.current), Some(150.0));
    assert_eq!(
        part.health.map(|health| health.current),
        Some(100.0),
        "la parte enseña la vida de romper"
    );

    assert_eq!(monster.ailments.len(), 1);
    assert_eq!(monster.ailments[0].name, "parálisis");
    assert_eq!(monster.ailments[0].buildup_percent(), 40.0);
}

#[test]
fn world_reads_a_snapshot_from_a_memory_image() {
    use crate::world::World;

    let map = map_for("MonsterHunterWorld");
    let mut fixture = Fixture::new();
    let id = (1..2000)
        .find(|id| crate::tables::world_monster_name(*id).is_some())
        .expect("la tabla de World tiene monstruos");

    // HUD del jugador (`MHWPlayerBasicInformationStructure`).
    let hud = fixture.allocate();
    fixture.seed_deref(
        map.absolute("EQUIPMENT_ADDRESS").unwrap(),
        map.offsets("PLAYER_BASIC_INFORMATION_OFFSETS").unwrap(),
        hud,
    );
    fixture.write_f32(hud + 0x60, 150.0);
    fixture.write_f32(hud + 0x64, 120.0);
    fixture.write_f32(hud + 0x12C, 80.0);
    fixture.write_f32(hud + 0x130, 100.0);
    fixture.write_f32(hud + 0x2DD4, 30.0);

    // Lista de componentes: uno solo, apuntando a su monstruo.
    let list = fixture.allocate();
    fixture.seed_deref(
        map.absolute("MONSTER_LIST_ADDRESS").unwrap(),
        map.offsets("MONSTER_LIST_OFFSETS").unwrap(),
        list,
    );
    let component = fixture.allocate();
    let monster = fixture.allocate();
    fixture.write_u64(list, component);
    fixture.write_u64(component + 0x138, monster);

    fixture.write_cstring(monster + 0x7749, "em\\em001\\data");
    fixture.write_i32(monster + 0x12280, id);
    fixture.write_f32(monster + 0x7730, 1.0);
    fixture.write_f32(monster + 0x184, 1.0);

    let values = fixture.allocate();
    fixture.write_u64(monster + 0x7670, values);
    fixture.write_f32(values + 0x60, 3000.0);
    fixture.write_f32(values + 0x64, 750.0);

    let snapshot = World::new(fixture, &map).snapshot().unwrap();

    let player = snapshot.player.expect("el HUD da un jugador");
    assert_eq!(player.health.percent(), 80.0);
    assert_eq!(player.stamina.percent(), 80.0);
    assert_eq!(player.recoverable, Some(30.0));

    assert_eq!(snapshot.monsters.len(), 1);
    let monster = &snapshot.monsters[0];
    assert_eq!(monster.id, id);
    assert_eq!(
        monster.name.as_deref(),
        crate::tables::world_monster_name(id)
    );
    assert_eq!(monster.health.current, 750.0);
    assert_eq!(monster.health.max, 3000.0);
    assert_eq!(monster.health.percent(), 25.0);
    assert!(monster.ailments.is_empty());
    assert!(monster.parts.is_empty());
}
