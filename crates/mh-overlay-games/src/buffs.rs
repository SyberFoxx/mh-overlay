//! Buffs, estados y canciones de World, con el offset al que el juego guarda su temporizador
//! dentro del bloque de estados (`ABNORMALITY_ADDRESS` + `ABNORMALITY_OFFSETS`).
//!
//! La lista es la de `HunterPie/Game/World/Data/AbnormalityData.xml` (Apache-2.0, ver
//! `NOTICE`): cada entrada tiene un `Offset` en hexadecimal y, en algunos casos, un
//! `DependsOn`/`WithValue` que distingue dos buffs que comparten temporizador (semilla del
//! poder frente a pastilla, por ejemplo). Los nombres son la traducción castellana de las
//! claves `ABNORMALITY_*` de ese mismo archivo.

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
