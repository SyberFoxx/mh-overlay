//! Buffs, estados y canciones de World, con el offset al que el juego guarda su temporizador
//! dentro del bloque de estados (`ABNORMALITY_ADDRESS` + `ABNORMALITY_OFFSETS`).
//!
//! La lista es la de `HunterPie/Game/World/Data/AbnormalityData.xml` (Apache-2.0, ver
//! `NOTICE`): cada entrada tiene un `Offset` en hexadecimal y, en algunos casos, un
//! `DependsOn`/`WithValue` que distingue dos buffs que comparten temporizador (semilla del
//! poder frente a pastilla, por ejemplo). Los nombres son la traducción castellana de las
//! claves `ABNORMALITY_*` de ese mismo archivo.

/// Grupo de estados de Rise: cada bloque cuelga de un puntero distinto del jugador
/// (`CONS_ABNORMALITIES_OFFSETS` y `DEBUFF_ABNORMALITIES_OFFSETS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiseGroup {
    Consumable,
    Debuff,
}

/// Un buff o estado del jugador en Rise, con el offset del `float` que guarda su temporizador
/// dentro del bloque de su grupo.
///
/// La lista sale de `HunterPie/Game/Rise/Data/AbnormalityData.xml` (Apache-2.0, ver `NOTICE`).
/// Los temporizadores de Rise están en tics de 60 por segundo y el lector los pasa a segundos.
pub struct RiseBuff {
    pub group: RiseGroup,
    pub offset: u32,
    pub name: &'static str,
    /// Otro offset del bloque y el valor que debe tener para que este buff sea el activo
    /// (semilla frente a mantequilla, dango fortificante frente a semilla del poder...).
    pub depends: Option<(u32, i32)>,
    pub debuff: bool,
    /// Buffs sin temporizador propio (drogas): se muestran como activos mientras dependan.
    pub infinite: bool,
    /// En los estados que guardan el tiempo transcurrido, aquí va la duración total.
    pub max_timer: Option<f32>,
}

/// Buffs, objetos, habilidades y comidas de Rise. Los estados que HunterPie distingue solo por
/// un flag de condición (veneno severo frente a veneno, burbujas grandes frente a pequeñas)
/// quedan con un solo nombre por offset: el flag no se lee todavía, así que mostrar los dos
/// duplicaría la fila sin poder elegir la correcta.
pub const RISE_BUFFS: &[RiseBuff] = &[
    rise_consumable(0x90, "Semilla del poder", Some((0x84, 10))),
    rise_consumable(0x90, "Mantequilla ardiente", Some((0x84, 25))),
    rise_consumable(0x94, "Semilla de adamantio", None),
    rise_consumable(0x98, "Mosca de seda", None),
    rise_consumable(0x9C, "Zumo de energía", None),
    rise_consumable(0xA8, "Polvo demoníaco", None),
    rise_consumable(0xAC, "Polvo de caparazón", None),
    rise_consumable(0xB0, "Firmeza de disparo de arco", None),
    rise_consumable(0xB4, "Calamar rojo", None),
    rise_consumable(0xBC, "Calamar amarillo", None),
    rise_consumable(0xC4, "Mosca cortante", None),
    rise_consumable(0xC8, "Afinidad de disparo de arco", None),
    rise_consumable(0xD4, "Mofeta hedionda", None),
    rise_consumable(0xDC, "Inmunidad", None),
    rise_consumable(0xE4, "Pez gourmet", None),
    rise_consumable(0xE8, "Curación natural", None),
    rise_consumable(0xEC, "A por la victoria", None),
    rise_consumable(0xF0, "Munición demoníaca", None),
    rise_consumable(0xF4, "Munición de armadura", None),
    rise_consumable(0xF8, "Guardia ofensiva", None),
    rise_consumable(0x100, "Tambor de poder", None),
    rise_consumable(0x104, "Rugido alentador", None),
    rise_consumable(0x114, "Afinidad deslizante", None),
    rise_consumable(0x11C, "Ataque al correr por muros", None),
    rise_consumable(0x120, "Contraataque", None),
    rise_consumable(0x12C, "Alma de Kushala Daora", None),
    rise_consumable(0x134, "Alma de Chameleos", None),
    rise_consumable(0x138, "Dango potenciador", None),
    rise_consumable(0x144, "Dango glotón", None),
    rise_consumable(0x158, "Dango cazador", None),
    rise_consumable(0x160, "Dango conector", None),
    rise_consumable(0x168, "Coalescencia", None),
    rise_consumable(0x16C, "Crítico en cadena", None),
    rise_consumable(0x174, "Subidón de adrenalina", None),
    rise_timed(0x17C, "Llamada de spiribird", 60.0),
    rise_consumable(0x180, "Curación de sangrado", None),
    rise_consumable(0x18C, "Aguante infinito (furia)", None),
    rise_consumable(0x190, "Afilado rápido", None),
    rise_consumable(0x194, "Inmunidad al frenesí", None),
    rise_consumable(0x1AC, "Activador de estado", None),
    rise_debuff(0x7EC, "Sangre"),
    rise_debuff(0x820, "Somnolencia"),
    rise_debuff(0x8A4, "Veneno"),
    rise_debuff(0x8A8, "Aturdimiento"),
    rise_debuff(0x8AC, "Sueño"),
    rise_debuff(0x8B0, "Parálisis"),
    rise_debuff(0x8BC, "Defensa -"),
    rise_debuff(0x8C0, "Resistencias -"),
    rise_debuff(0x8C4, "Hedor"),
    rise_debuff(0x8C8, "Fuego infernal"),
    rise_debuff(0x8CC, "Explosión"),
    rise_debuff(0x8D0, "Telaraña"),
    rise_debuff(0x8D4, "Fuego"),
    rise_debuff(0x8D8, "Agua"),
    rise_debuff(0x8DC, "Hielo"),
    rise_debuff(0x8E0, "Rayo"),
    rise_debuff(0x8E4, "Dragón"),
    rise_debuff(0x8F0, "Burbujas"),
    rise_debuff(0x914, "Sangrado"),
    rise_debuff(0x92C, "Frenesí"),
    rise_debuff(0x1D8, "Pulido protector"),
    rise_debuff(0x1DC, "Poder latente"),
];

/// Canciones de la cornamusa, en el orden exacto del array del juego: cada ranura es un
/// puntero a su estado, y las que el juego no usa quedan vacías.
pub const RISE_SONGS: [Option<&str>; 28] = [
    Some("Automejora"),
    Some("Ataque +"),
    Some("Defensa +"),
    Some("Afinidad +"),
    Some("Ataque elemental +"),
    Some("Ataque y defensa +"),
    Some("Ataque y afinidad +"),
    Some("Empuje anulado"),
    Some("Tapones"),
    Some("Tapones +"),
    Some("Temblores anulados"),
    Some("Presión del viento anulada"),
    Some("Aturdimiento anulado"),
    Some("Estados anulados"),
    Some("Protección divina"),
    None,
    None,
    None,
    Some("Regeneración"),
    Some("Uso de aguante -"),
    Some("Recuperación de aguante +"),
    Some("Menos desgaste de filo"),
    Some("Daño ambiental anulado"),
    None,
    Some("Barrera sónica"),
    Some("Melodía infernal"),
    Some("Regeneración de filo"),
    Some("Extensión de filo"),
];

const fn rise_consumable(offset: u32, name: &'static str, depends: Option<(u32, i32)>) -> RiseBuff {
    RiseBuff {
        group: RiseGroup::Consumable,
        offset,
        name,
        depends,
        debuff: false,
        infinite: false,
        max_timer: None,
    }
}

const fn rise_debuff(offset: u32, name: &'static str) -> RiseBuff {
    RiseBuff {
        group: RiseGroup::Debuff,
        offset,
        name,
        depends: None,
        debuff: true,
        infinite: false,
        max_timer: None,
    }
}

const fn rise_timed(offset: u32, name: &'static str, max_timer: f32) -> RiseBuff {
    RiseBuff {
        group: RiseGroup::Consumable,
        offset,
        name,
        depends: None,
        debuff: false,
        infinite: false,
        max_timer: Some(max_timer),
    }
}

/// Un buff (o estado negativo) del jugador en World.
pub struct WorldBuff {
    /// Offset del `float` con el temporizador, relativo a la base de estados.
    pub offset: u32,
    pub name: &'static str,
    /// Otro offset del bloque y el valor que debe tener para que este buff sea el activo.
    pub depends: Option<(u32, i32)>,
    /// `true` en los estados negativos: se dibujan en rojo y al final de la lista.
    pub debuff: bool,
    /// Buffs sin temporizador propio (drogas): se muestran como activos mientras dependan.
    pub infinite: bool,
    /// La comida vive en otro bloque (el de la cantina), no en el de estados.
    pub canteen: bool,
}

/// Canciones de la cornamusa y buffs que reparte el grupo.
pub const WORLD_BUFFS: &[WorldBuff] = &[
    buff(0x38, "Automejora"),
    buff(0x3C, "Ataque +"),
    buff(0x40, "Ataque + (grande)"),
    buff(0x44, "Vida +"),
    buff(0x48, "Vida + (grande)"),
    buff(0x4C, "Uso de aguante -"),
    buff(0x50, "Uso de aguante - (grande)"),
    buff(0x54, "Presión del viento anulada"),
    buff(0x58, "Presión del viento anulada +"),
    buff(0x5C, "Defensa +"),
    buff(0x60, "Defensa + (grande)"),
    buff(0x64, "Uso de herramientas -"),
    buff(0x68, "Uso de herramientas - (grande)"),
    buff(0x80, "Regeneración"),
    buff(0x84, "Regeneración (grande)"),
    buff(0x88, "Tapones"),
    buff(0x8C, "Tapones +"),
    buff(0x90, "Protección divina"),
    buff(0x94, "Buscainsectos +"),
    buff(0x98, "Daño ambiental anulado"),
    buff(0x9C, "Aturdimiento anulado"),
    buff(0xA0, "Parálisis anulada"),
    buff(0xA4, "Temblores anulados"),
    buff(0xA8, "Resistencia profunda"),
    buff(0xAC, "Resistencia al fuego"),
    buff(0xB0, "Resistencia al fuego +"),
    buff(0xB4, "Resistencia al agua"),
    buff(0xB8, "Resistencia al agua +"),
    buff(0xBC, "Resistencia al rayo"),
    buff(0xC0, "Resistencia al rayo +"),
    buff(0xC4, "Resistencia al hielo"),
    buff(0xC8, "Resistencia al hielo +"),
    buff(0xCC, "Resistencia al dragón"),
    buff(0xD0, "Resistencia al dragón +"),
    buff(0xD4, "Ataque elemental +"),
    buff(0xD8, "Poder anulado"),
    buff(0xE4, "Empuje anulado"),
    buff(0xEC, "Todas las resistencias +"),
    buff(0xF0, "Afinidad +"),
    buff(0xF4, "Estados anulados"),
    buff(0xF8, "Tapones"),
    buff(0xFC, "Ataque de estados +"),
    buff(0x10C, "Recuperación de aguante +"),
    buff(0x110, "Recuperación extendida"),
    buff(0x114, "Evasión +"),
    buff(0x118, "Resistencias y elemento +"),
    // Estados negativos.
    debuff(0x5DC, "Veneno"),
    debuff(0x5E0, "Veneno severo"),
    debuff(0x5EC, "Fuego"),
    debuff(0x5F0, "Rayo"),
    debuff(0x5F4, "Agua"),
    debuff(0x5F8, "Hielo"),
    debuff(0x5FC, "Dragón"),
    debuff(0x600, "Sangrado"),
    debuff(0x608, "Efluvio"),
    debuff(0x60C, "Defensa -"),
    debuff(0x614, "Resistencias -"),
    debuff(0x620, "Explosión"),
    dependent_debuff(0x63C, "Azote explosivo", 0x62C, 1),
    // Objetos.
    buff(0x690, "Zumo de energía"),
    buff(0x694, "Lichi vibrante"),
    buff(0x698, "Cecina de Astera"),
    dependent(0x6A0, "Semilla del poder", 0x6A4, 10),
    dependent(0x6A0, "Pastilla del poder", 0x6A4, 25),
    dependent(0x6B0, "Semilla de adamantio", 0x6B4, 20),
    dependent(0x6B0, "Pastilla de adamantio", 0x6BC, 1),
    buff(0x6C4, "Polvo demoníaco"),
    buff(0x6C8, "Polvo coraza"),
    dependent_infinite(0x6CC, "Droga demoníaca", 0x6D4, 1),
    dependent_infinite(0x6CC, "Mega droga demoníaca", 0x6D4, 2),
    dependent_infinite(0x6D0, "Piel de armadura", 0x6D8, 1),
    dependent_infinite(0x6D0, "Mega piel de armadura", 0x6D8, 2),
    buff(0x6EC, "Bebida fría"),
    buff(0x6F0, "Bebida caliente"),
    buff(0x6F8, "Regeneración (objeto)"),
    buff(0x6FC, "Resistencia al frío"),
    buff(0x708, "Cabeza de nieve"),
    buff(0x718, "Cono de poder"),
    buff(0x71C, "Resistencia al hielo +"),
    // Habilidades de armadura.
    buff(0x6CC, "Munición demoníaca"),
    buff(0x6D0, "Munición de armadura"),
    buff(0x764, "Fortificar"),
    buff(0x76C, "Pulido protector"),
    buff(0x770, "Afinidad deslizante"),
    buff(0x730, "Aceleración elemental"),
    buff(0x738, "Poder latente"),
    buff(0x754, "Adrenalina"),
    buff(0x788, "Arte de escarcha"),
    buff(0x79C, "Guardia ofensiva"),
    buff(0x7A0, "Coalescencia"),
    // Comida y herramientas.
    WorldBuff {
        offset: 0xE4,
        name: "Comida",
        depends: None,
        debuff: false,
        infinite: false,
        canteen: true,
    },
    buff(0xFC4, "Manto de evasión"),
    buff(0xFC8, "Refuerzo de afinidad"),
];

const fn buff(offset: u32, name: &'static str) -> WorldBuff {
    WorldBuff {
        offset,
        name,
        depends: None,
        debuff: false,
        infinite: false,
        canteen: false,
    }
}

const fn debuff(offset: u32, name: &'static str) -> WorldBuff {
    WorldBuff {
        offset,
        name,
        depends: None,
        debuff: true,
        infinite: false,
        canteen: false,
    }
}

const fn dependent(offset: u32, name: &'static str, depends_on: u32, value: i32) -> WorldBuff {
    WorldBuff {
        offset,
        name,
        depends: Some((depends_on, value)),
        debuff: false,
        infinite: false,
        canteen: false,
    }
}

const fn dependent_debuff(
    offset: u32,
    name: &'static str,
    depends_on: u32,
    value: i32,
) -> WorldBuff {
    WorldBuff {
        offset,
        name,
        depends: Some((depends_on, value)),
        debuff: true,
        infinite: false,
        canteen: false,
    }
}

const fn dependent_infinite(
    offset: u32,
    name: &'static str,
    depends_on: u32,
    value: i32,
) -> WorldBuff {
    WorldBuff {
        offset,
        name,
        depends: Some((depends_on, value)),
        debuff: false,
        infinite: true,
        canteen: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_buff_has_a_name() {
        for entry in WORLD_BUFFS {
            assert!(
                !entry.name.is_empty(),
                "offset {:#x} sin nombre",
                entry.offset
            );
        }
    }

    #[test]
    fn every_rise_buff_has_a_name() {
        for entry in RISE_BUFFS {
            assert!(
                !entry.name.is_empty(),
                "offset {:#x} sin nombre",
                entry.offset
            );
        }
    }

    #[test]
    fn rise_buffs_only_share_an_offset_to_disambiguate() {
        for (index, entry) in RISE_BUFFS.iter().enumerate() {
            let shared = RISE_BUFFS
                .iter()
                .skip(index + 1)
                .filter(|other| other.group == entry.group && other.offset == entry.offset)
                .all(|other| other.depends.is_some() && other.depends != entry.depends);

            assert!(
                shared,
                "{:#x} comparte offset sin un `depends` que los distinga",
                entry.offset
            );
        }
    }

    #[test]
    fn every_rise_song_slot_has_a_name_or_is_empty() {
        assert_eq!(RISE_SONGS.len(), 28);
        assert!(RISE_SONGS.iter().flatten().all(|name| !name.is_empty()));
    }

    #[test]
    fn dependent_buffs_share_their_offset_on_purpose() {
        // Semilla y pastilla comparten temporizador; lo que las distingue es el valor de
        // `DependsOn`, y la tabla tiene que conservar las dos entradas.
        let shared: Vec<&WorldBuff> = WORLD_BUFFS
            .iter()
            .filter(|entry| entry.offset == 0x6A0)
            .collect();

        assert_eq!(shared.len(), 2);
        assert_ne!(shared[0].depends, shared[1].depends);
    }
}
