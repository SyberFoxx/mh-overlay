//! Tablas derivadas de los `MonsterData.xml` de HunterPie (Apache-2.0, ver `NOTICE`).
//!
//! Generado por `scripts/generate_tables.py`; no editar a mano. Rise identifica las
//! partes por su posición en el array (igual que `MHRMonster.Parts[i]`); World las
//! identifica por el índice que reporta el juego (`MHWMonsterPart`).

/// Parte de World: índice del juego y nombre corto en castellano.
pub struct WorldPart {
    pub index: usize,
    pub name: &'static str,
}

/// Porcentaje de vida por debajo del cual se puede capturar.
const WORLD_CAPTURE: &[(i32, u8)] = &[
    (0, 25),
    (1, 20),
    (4, 0),
    (7, 30),
    (9, 25),
    (10, 25),
    (11, 15),
    (12, 20),
    (13, 15),
    (14, 0),
    (15, 0),
    (16, 0),
    (17, 0),
    (18, 0),
    (19, 20),
    (20, 20),
    (21, 25),
    (22, 20),
    (23, 0),
    (24, 25),
    (25, 0),
    (26, 0),
    (27, 30),
    (28, 30),
    (29, 25),
    (30, 25),
    (31, 25),
    (32, 20),
    (33, 30),
    (34, 20),
    (35, 25),
    (36, 0),
    (37, 30),
    (38, 0),
    (39, 30),
    (51, 0),
    (61, 20),
    (62, 20),
    (63, 20),
    (64, 10),
    (65, 20),
    (66, 20),
    (67, 20),
    (68, 15),
    (69, 25),
    (70, 0),
    (71, 25),
    (72, 25),
    (73, 15),
    (74, 15),
    (75, 0),
    (76, 10),
    (77, 25),
    (78, 25),
    (79, 0),
    (80, 0),
    (81, 0),
    (87, 0),
    (88, 10),
    (89, 10),
    (90, 20),
    (91, 15),
    (92, 10),
    (93, 15),
    (94, 20),
    (95, 20),
    (96, 0),
    (97, 0),
    (98, 0),
    (99, 15),
    (100, 20),
    (101, 0),
];

const WORLD_PARTS: &[(i32, &[WorldPart])] = &[
    (
        0,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "garganta",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "pata izq.",
            },
            WorldPart {
                index: 5,
                name: "pata der.",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        1,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "ala izq.",
            },
            WorldPart {
                index: 4,
                name: "ala der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        4,
        &[
            WorldPart {
                index: 0,
                name: "rechazo",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "pecho",
            },
            WorldPart {
                index: 4,
                name: "brazo izq.",
            },
            WorldPart {
                index: 5,
                name: "brazo der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
            WorldPart {
                index: 9,
                name: "caparazón",
            },
            WorldPart {
                index: 10,
                name: "órgano de gas central",
            },
            WorldPart {
                index: 11,
                name: "órgano de gas frontal",
            },
            WorldPart {
                index: 12,
                name: "órgano de gas del cráter",
            },
            WorldPart {
                index: 13,
                name: "órgano de gas trasero",
            },
            WorldPart {
                index: 14,
                name: "caparazón débil izq.",
            },
            WorldPart {
                index: 15,
                name: "caparazón débil der.",
            },
        ],
    ),
    (
        7,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cuerpo",
            },
            WorldPart {
                index: 2,
                name: "brazos",
            },
            WorldPart {
                index: 3,
                name: "patas",
            },
            WorldPart {
                index: 4,
                name: "cola",
            },
            WorldPart {
                index: 5,
                name: "abdomen",
            },
        ],
    ),
    (
        9,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "ala izq.",
            },
            WorldPart {
                index: 4,
                name: "ala der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        10,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "ala izq.",
            },
            WorldPart {
                index: 4,
                name: "ala der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        11,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "ala izq.",
            },
            WorldPart {
                index: 4,
                name: "ala der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        12,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        13,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        14,
        &[
            WorldPart {
                index: 0,
                name: "cuerno",
            },
            WorldPart {
                index: 1,
                name: "brazos",
            },
            WorldPart {
                index: 2,
                name: "patas",
            },
        ],
    ),
    (
        15,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cuernos",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "brazo izq.",
            },
            WorldPart {
                index: 5,
                name: "brazo der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        16,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "cola",
            },
            WorldPart {
                index: 4,
                name: "extremidades izq.",
            },
            WorldPart {
                index: 5,
                name: "extremidades der.",
            },
            WorldPart {
                index: 6,
                name: "alas",
            },
        ],
    ),
    (
        17,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "extremidades",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        18,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cuernos",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "brazos",
            },
            WorldPart {
                index: 5,
                name: "patas",
            },
            WorldPart {
                index: 6,
                name: "alas",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        19,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cuerpo",
            },
            WorldPart {
                index: 2,
                name: "abdomen",
            },
            WorldPart {
                index: 3,
                name: "pata izq.",
            },
            WorldPart {
                index: 4,
                name: "pata der.",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
        ],
    ),
    (
        20,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "gran tambaleo",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "pecho",
            },
            WorldPart {
                index: 5,
                name: "zona trasera",
            },
            WorldPart {
                index: 6,
                name: "brazos",
            },
            WorldPart {
                index: 7,
                name: "pata izq.",
            },
            WorldPart {
                index: 8,
                name: "pata der.",
            },
            WorldPart {
                index: 9,
                name: "cola",
            },
        ],
    ),
    (
        21,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cabeza (lodo)",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "cuerpo (lodo)",
            },
            WorldPart {
                index: 6,
                name: "brazos",
            },
            WorldPart {
                index: 7,
                name: "brazos (lodo)",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata izq. (lodo)",
            },
            WorldPart {
                index: 10,
                name: "pata der.",
            },
            WorldPart {
                index: 11,
                name: "pata der. (lodo)",
            },
            WorldPart {
                index: 12,
                name: "cola",
            },
            WorldPart {
                index: 13,
                name: "cola (lodo)",
            },
        ],
    ),
    (
        22,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "mandíbula",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "brazos",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        23,
        &[
            WorldPart {
                index: 0,
                name: "cuerpo y patas",
            },
            WorldPart {
                index: 1,
                name: "astas",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "patas",
            },
            WorldPart {
                index: 4,
                name: "brazos",
            },
        ],
    ),
    (
        24,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        25,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 3,
                name: "cuernos",
            },
            WorldPart {
                index: 4,
                name: "cabeza",
            },
            WorldPart {
                index: 5,
                name: "cuerpo",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "ala izq.",
            },
            WorldPart {
                index: 11,
                name: "ala der.",
            },
            WorldPart {
                index: 12,
                name: "cola",
            },
        ],
    ),
    (
        26,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuello",
            },
            WorldPart {
                index: 4,
                name: "espalda",
            },
            WorldPart {
                index: 5,
                name: "pecho",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "alas",
            },
            WorldPart {
                index: 11,
                name: "cola",
            },
        ],
    ),
    (
        27,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "brazos",
            },
            WorldPart {
                index: 2,
                name: "patas",
            },
            WorldPart {
                index: 3,
                name: "cola",
            },
            WorldPart {
                index: 4,
                name: "roca",
            },
        ],
    ),
    (
        28,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "brazos",
            },
            WorldPart {
                index: 3,
                name: "patas",
            },
            WorldPart {
                index: 4,
                name: "cola",
            },
        ],
    ),
    (
        29,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "torso",
            },
            WorldPart {
                index: 2,
                name: "pata izq.",
            },
            WorldPart {
                index: 3,
                name: "pata der.",
            },
            WorldPart {
                index: 4,
                name: "cola",
            },
            WorldPart {
                index: 5,
                name: "cabeza (lodo)",
            },
            WorldPart {
                index: 6,
                name: "torso (lodo)",
            },
            WorldPart {
                index: 7,
                name: "pata izq. (lodo)",
            },
            WorldPart {
                index: 8,
                name: "pata der. (lodo)",
            },
            WorldPart {
                index: 9,
                name: "cola (lodo)",
            },
        ],
    ),
    (
        30,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cuerpo",
            },
            WorldPart {
                index: 2,
                name: "melena",
            },
            WorldPart {
                index: 3,
                name: "brazos",
            },
            WorldPart {
                index: 4,
                name: "patas",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
        ],
    ),
    (
        31,
        &[
            WorldPart {
                index: 0,
                name: "globo",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "globo",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "pata izq.",
            },
            WorldPart {
                index: 5,
                name: "pata der.",
            },
            WorldPart {
                index: 6,
                name: "ala izq.",
            },
            WorldPart {
                index: 7,
                name: "ala der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        32,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cuerpo",
            },
            WorldPart {
                index: 2,
                name: "ala izq.",
            },
            WorldPart {
                index: 3,
                name: "ala der.",
            },
            WorldPart {
                index: 4,
                name: "pata izq.",
            },
            WorldPart {
                index: 5,
                name: "pata der.",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        33,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazos",
            },
            WorldPart {
                index: 4,
                name: "patas",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
        ],
    ),
    (
        34,
        &[
            WorldPart {
                index: 0,
                name: "contraataque",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "brazos",
            },
            WorldPart {
                index: 5,
                name: "patas",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        35,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 5,
                name: "cabeza",
            },
            WorldPart {
                index: 6,
                name: "cuerpo",
            },
            WorldPart {
                index: 7,
                name: "pata izq.",
            },
            WorldPart {
                index: 8,
                name: "pata der.",
            },
            WorldPart {
                index: 9,
                name: "cola",
            },
            WorldPart {
                index: 10,
                name: "mandíbula",
            },
            WorldPart {
                index: 11,
                name: "espalda",
            },
            WorldPart {
                index: 12,
                name: "hueso izq.",
            },
            WorldPart {
                index: 13,
                name: "hueso der.",
            },
        ],
    ),
    (
        36,
        &[
            WorldPart {
                index: 0,
                name: "emisiones",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "espalda",
            },
            WorldPart {
                index: 4,
                name: "pecho",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "alas",
            },
        ],
    ),
    (
        37,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazos",
            },
            WorldPart {
                index: 4,
                name: "patas",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
        ],
    ),
    (
        38,
        &[
            WorldPart {
                index: 0,
                name: "cuernos 2",
            },
            WorldPart {
                index: 1,
                name: "cuernos",
            },
            WorldPart {
                index: 2,
                name: "cuernos",
            },
            WorldPart {
                index: 3,
                name: "pecho",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "extremidades izq.",
            },
            WorldPart {
                index: 6,
                name: "extremidades der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
            WorldPart {
                index: 8,
                name: "cuernos (dorada)",
            },
            WorldPart {
                index: 9,
                name: "melena (dorada)",
            },
            WorldPart {
                index: 10,
                name: "pecho izq. (dorada)",
            },
            WorldPart {
                index: 11,
                name: "pecho der. (dorada)",
            },
            WorldPart {
                index: 12,
                name: "brazo izq. (dorada)",
            },
            WorldPart {
                index: 13,
                name: "brazo der. (dorada)",
            },
            WorldPart {
                index: 14,
                name: "pata izq. (dorada)",
            },
            WorldPart {
                index: 15,
                name: "pata der. (dorada)",
            },
            WorldPart {
                index: 16,
                name: "cola izq. (dorada)",
            },
            WorldPart {
                index: 17,
                name: "cola der. (dorada)",
            },
        ],
    ),
    (
        39,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "patas",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        51,
        &[
            WorldPart {
                index: 0,
                name: "cuerpo y patas",
            },
            WorldPart {
                index: 1,
                name: "astas",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "patas",
            },
            WorldPart {
                index: 4,
                name: "brazos",
            },
        ],
    ),
    (
        61,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazo izq.",
            },
            WorldPart {
                index: 4,
                name: "brazo der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        62,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazo izq.",
            },
            WorldPart {
                index: 4,
                name: "brazo der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        63,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazo izq.",
            },
            WorldPart {
                index: 4,
                name: "brazo der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        64,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "gran tambaleo",
            },
            WorldPart {
                index: 2,
                name: "contraataque",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "pecho",
            },
            WorldPart {
                index: 6,
                name: "zona trasera",
            },
            WorldPart {
                index: 7,
                name: "brazos",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "cola",
            },
        ],
    ),
    (
        65,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazo izq.",
            },
            WorldPart {
                index: 4,
                name: "brazo der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        66,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "garganta",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "aleta",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "brazos",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        67,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "garganta",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 4,
                name: "aleta",
            },
            WorldPart {
                index: 5,
                name: "cuerpo",
            },
            WorldPart {
                index: 6,
                name: "brazos",
            },
            WorldPart {
                index: 7,
                name: "pata izq.",
            },
            WorldPart {
                index: 8,
                name: "pata der.",
            },
            WorldPart {
                index: 9,
                name: "cola",
            },
        ],
    ),
    (
        68,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "garganta",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "pata izq.",
            },
            WorldPart {
                index: 5,
                name: "pata der.",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        69,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cola inflada",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "ala izq.",
            },
            WorldPart {
                index: 6,
                name: "ala der.",
            },
            WorldPart {
                index: 7,
                name: "pata izq.",
            },
            WorldPart {
                index: 8,
                name: "pata der.",
            },
            WorldPart {
                index: 9,
                name: "cola",
            },
        ],
    ),
    (
        70,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 3,
                name: "cabeza con púas",
            },
            WorldPart {
                index: 4,
                name: "brazo izq. con púas",
            },
            WorldPart {
                index: 5,
                name: "brazo der. con púas",
            },
            WorldPart {
                index: 6,
                name: "ala izq. con púas",
            },
            WorldPart {
                index: 7,
                name: "ala der. con púas",
            },
            WorldPart {
                index: 8,
                name: "cuernos",
            },
            WorldPart {
                index: 9,
                name: "cabeza",
            },
            WorldPart {
                index: 10,
                name: "cuerpo",
            },
            WorldPart {
                index: 11,
                name: "brazo izq.",
            },
            WorldPart {
                index: 12,
                name: "brazo der.",
            },
            WorldPart {
                index: 13,
                name: "pata izq.",
            },
            WorldPart {
                index: 14,
                name: "pata der.",
            },
            WorldPart {
                index: 15,
                name: "ala izq.",
            },
            WorldPart {
                index: 16,
                name: "ala der.",
            },
            WorldPart {
                index: 17,
                name: "cola",
            },
        ],
    ),
    (
        71,
        &[
            WorldPart {
                index: 0,
                name: "cabeza",
            },
            WorldPart {
                index: 1,
                name: "cuerpo",
            },
            WorldPart {
                index: 2,
                name: "melena",
            },
            WorldPart {
                index: 3,
                name: "brazos",
            },
            WorldPart {
                index: 4,
                name: "patas",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
        ],
    ),
    (
        72,
        &[
            WorldPart {
                index: 0,
                name: "globo",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "globo",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "pata izq.",
            },
            WorldPart {
                index: 5,
                name: "pata der.",
            },
            WorldPart {
                index: 6,
                name: "ala izq.",
            },
            WorldPart {
                index: 7,
                name: "ala der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        73,
        &[
            WorldPart {
                index: 0,
                name: "caída",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "ala izq.",
            },
            WorldPart {
                index: 4,
                name: "ala der.",
            },
            WorldPart {
                index: 5,
                name: "patas",
            },
            WorldPart {
                index: 6,
                name: "cola",
            },
        ],
    ),
    (
        74,
        &[
            WorldPart {
                index: 0,
                name: "contraataque",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "brazos",
            },
            WorldPart {
                index: 6,
                name: "patas",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        75,
        &[
            WorldPart {
                index: 0,
                name: "emisiones",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "espalda",
            },
            WorldPart {
                index: 4,
                name: "pecho",
            },
            WorldPart {
                index: 5,
                name: "cola",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "alas",
            },
        ],
    ),
    (
        76,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza brillante",
            },
            WorldPart {
                index: 2,
                name: "cola brillante",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "patas",
            },
            WorldPart {
                index: 6,
                name: "ala izq.",
            },
            WorldPart {
                index: 7,
                name: "ala der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        77,
        &[
            WorldPart {
                index: 0,
                name: "cabeza al emerger",
            },
            WorldPart {
                index: 1,
                name: "cuerpo al emerger",
            },
            WorldPart {
                index: 2,
                name: "cola al emerger",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 4,
                name: "cabeza (nieve)",
            },
            WorldPart {
                index: 5,
                name: "cuerpo",
            },
            WorldPart {
                index: 6,
                name: "cuerpo (nieve)",
            },
            WorldPart {
                index: 7,
                name: "patas",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
            WorldPart {
                index: 9,
                name: "cola (nieve)",
            },
        ],
    ),
    (
        78,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 3,
                name: "cuernos",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        79,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "caída",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cabeza (hielo)",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "cuerpo (hielo)",
            },
            WorldPart {
                index: 6,
                name: "alas",
            },
            WorldPart {
                index: 7,
                name: "alas (hielo)",
            },
            WorldPart {
                index: 8,
                name: "brazos",
            },
            WorldPart {
                index: 9,
                name: "brazos (hielo)",
            },
            WorldPart {
                index: 10,
                name: "patas",
            },
            WorldPart {
                index: 11,
                name: "cola",
            },
        ],
    ),
    (
        80,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "espalda",
            },
            WorldPart {
                index: 3,
                name: "cola",
            },
            WorldPart {
                index: 4,
                name: "brazos",
            },
            WorldPart {
                index: 5,
                name: "patas",
            },
            WorldPart {
                index: 6,
                name: "alas",
            },
        ],
    ),
    (
        81,
        &[
            WorldPart {
                index: 0,
                name: "cabeza (roca)",
            },
            WorldPart {
                index: 1,
                name: "cabeza (roca)",
            },
            WorldPart {
                index: 2,
                name: "pecho (roca)",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 5,
                name: "cuello izq. (roca)",
            },
            WorldPart {
                index: 6,
                name: "cuello der. (roca)",
            },
            WorldPart {
                index: 7,
                name: "cabeza (roca)",
            },
            WorldPart {
                index: 8,
                name: "cola (roca)",
            },
            WorldPart {
                index: 9,
                name: "ala izq. (roca)",
            },
            WorldPart {
                index: 10,
                name: "ala der. (roca)",
            },
            WorldPart {
                index: 11,
                name: "brazo izq. (roca)",
            },
            WorldPart {
                index: 12,
                name: "brazo der. (roca)",
            },
            WorldPart {
                index: 13,
                name: "patas",
            },
            WorldPart {
                index: 14,
                name: "cabeza",
            },
            WorldPart {
                index: 15,
                name: "cuerpo",
            },
            WorldPart {
                index: 16,
                name: "ala izq.",
            },
            WorldPart {
                index: 17,
                name: "ala der.",
            },
            WorldPart {
                index: 18,
                name: "brazo izq.",
            },
            WorldPart {
                index: 19,
                name: "brazo der.",
            },
            WorldPart {
                index: 20,
                name: "cola",
            },
        ],
    ),
    (
        87,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cola",
            },
            WorldPart {
                index: 3,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "explosión debilitada",
            },
            WorldPart {
                index: 4,
                name: "pecho",
            },
            WorldPart {
                index: 5,
                name: "alas",
            },
            WorldPart {
                index: 6,
                name: "brazos",
            },
            WorldPart {
                index: 7,
                name: "patas",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        88,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "gran tambaleo",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        89,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "gran tambaleo",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "ala izq.",
            },
            WorldPart {
                index: 5,
                name: "ala der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
        ],
    ),
    (
        90,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "pata izq.",
            },
            WorldPart {
                index: 4,
                name: "pata der.",
            },
            WorldPart {
                index: 5,
                name: "ala izq.",
            },
            WorldPart {
                index: 6,
                name: "ala der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        91,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cuernos 2",
            },
            WorldPart {
                index: 2,
                name: "cola",
            },
            WorldPart {
                index: 3,
                name: "furia",
            },
            WorldPart {
                index: 4,
                name: "cabeza",
            },
            WorldPart {
                index: 5,
                name: "cuerpo",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "cola",
            },
        ],
    ),
    (
        92,
        &[
            WorldPart {
                index: 0,
                name: "cuernos",
            },
            WorldPart {
                index: 1,
                name: "cuernos 2",
            },
            WorldPart {
                index: 2,
                name: "cola",
            },
            WorldPart {
                index: 3,
                name: "furia",
            },
            WorldPart {
                index: 4,
                name: "cabeza",
            },
            WorldPart {
                index: 5,
                name: "cuerpo",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "cola",
            },
        ],
    ),
    (
        93,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazo izq.",
            },
            WorldPart {
                index: 4,
                name: "brazo der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        94,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "carga",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "espalda",
            },
            WorldPart {
                index: 5,
                name: "brazos",
            },
            WorldPart {
                index: 6,
                name: "patas",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        95,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "carga",
            },
            WorldPart {
                index: 2,
                name: "cabeza",
            },
            WorldPart {
                index: 3,
                name: "cuerpo",
            },
            WorldPart {
                index: 4,
                name: "espalda",
            },
            WorldPart {
                index: 5,
                name: "brazos",
            },
            WorldPart {
                index: 6,
                name: "patas",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        96,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "espalda",
            },
            WorldPart {
                index: 4,
                name: "brazo izq.",
            },
            WorldPart {
                index: 5,
                name: "brazo der.",
            },
            WorldPart {
                index: 6,
                name: "pata izq.",
            },
            WorldPart {
                index: 7,
                name: "pata der.",
            },
            WorldPart {
                index: 8,
                name: "cola",
            },
            WorldPart {
                index: 9,
                name: "punta de la cola",
            },
        ],
    ),
    (
        97,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "abdomen",
            },
            WorldPart {
                index: 4,
                name: "espalda",
            },
            WorldPart {
                index: 5,
                name: "pecho",
            },
            WorldPart {
                index: 6,
                name: "brazo izq.",
            },
            WorldPart {
                index: 7,
                name: "brazo der.",
            },
            WorldPart {
                index: 8,
                name: "pata izq.",
            },
            WorldPart {
                index: 9,
                name: "pata der.",
            },
            WorldPart {
                index: 10,
                name: "ala izq.",
            },
            WorldPart {
                index: 11,
                name: "ala der.",
            },
            WorldPart {
                index: 12,
                name: "cola",
            },
        ],
    ),
    (
        99,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "pata izq.",
            },
            WorldPart {
                index: 4,
                name: "pata der.",
            },
            WorldPart {
                index: 5,
                name: "ala izq.",
            },
            WorldPart {
                index: 6,
                name: "ala der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        100,
        &[
            WorldPart {
                index: 0,
                name: "cola",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuerpo",
            },
            WorldPart {
                index: 3,
                name: "brazo izq.",
            },
            WorldPart {
                index: 4,
                name: "brazo der.",
            },
            WorldPart {
                index: 5,
                name: "pata izq.",
            },
            WorldPart {
                index: 6,
                name: "pata der.",
            },
            WorldPart {
                index: 7,
                name: "cola",
            },
        ],
    ),
    (
        101,
        &[
            WorldPart {
                index: 0,
                name: "derribo",
            },
            WorldPart {
                index: 1,
                name: "cabeza",
            },
            WorldPart {
                index: 2,
                name: "cuello",
            },
            WorldPart {
                index: 3,
                name: "pecho",
            },
            WorldPart {
                index: 4,
                name: "cuerpo",
            },
            WorldPart {
                index: 5,
                name: "brazo izq.",
            },
            WorldPart {
                index: 6,
                name: "brazo der.",
            },
            WorldPart {
                index: 7,
                name: "pata izq.",
            },
            WorldPart {
                index: 8,
                name: "pata der.",
            },
            WorldPart {
                index: 9,
                name: "ala izq.",
            },
            WorldPart {
                index: 10,
                name: "ala der.",
            },
            WorldPart {
                index: 11,
                name: "cola",
            },
        ],
    ),
];

const RISE_PARTS: &[(i32, &[Option<&'static str>])] = &[
    (
        0,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        1,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        2,
        &[
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cuello"),
            Some("cabeza"),
            Some("cola"),
        ],
    ),
    (
        3,
        &[
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cuello"),
            Some("cabeza"),
            Some("cola"),
        ],
    ),
    (
        4,
        &[
            Some("cabeza"),
            Some("cuello"),
            Some("torso"),
            Some("pata izq."),
            Some("pata der."),
            Some("ala izq."),
            Some("ala der."),
            Some("cola"),
        ],
    ),
    (
        5,
        &[
            Some("espalda"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cabeza"),
            Some("pecho"),
            Some("cola"),
        ],
    ),
    (
        6,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        7,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        8,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        9,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata izq."),
            Some("pata der."),
            Some("alas"),
            Some("cola"),
        ],
    ),
    (
        10,
        &[
            Some("cabeza"),
            Some("pecho"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
            Some("alas"),
        ],
    ),
    (
        11,
        &[
            Some("cabeza"),
            Some("pecho"),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("alas"),
            Some("cola"),
        ],
    ),
    (
        12,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
        ],
    ),
    (
        13,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("membrana izq."),
            Some("cola"),
            Some("patas delanteras"),
            Some("pata trasera izq."),
            Some("membrana der."),
            Some("pata trasera der."),
        ],
    ),
    (
        14,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
        ],
    ),
    (
        15,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("brazos"),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
            Some("cabeza (lodo)"),
            Some("torso (lodo)"),
            Some("brazos (lodo)"),
            Some("pata izq. (lodo)"),
            Some("pata der. (lodo)"),
            Some("cola (lodo)"),
        ],
    ),
    (
        16,
        &[
            Some("cabeza"),
            Some("esponja"),
            Some("torso"),
            Some("patas izq."),
            Some("patas der."),
            Some("cola"),
        ],
    ),
    (17, &[Some("cabeza"), Some("cuerpo"), Some("cola")]),
    (
        18,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("espalda"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
        ],
    ),
    (
        19,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("espalda"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
        ],
    ),
    (20, &[Some("cabeza"), Some("cuerpo"), Some("cola")]),
    (
        21,
        &[
            Some("cabeza"),
            Some("cuerpo alto"),
            Some("patas delanteras"),
            Some("zona trasera"),
            Some("cuerpo bajo"),
        ],
    ),
    (
        22,
        &[
            Some("cabeza"),
            Some("cuerpo alto"),
            Some("patas delanteras"),
            Some("zona trasera"),
            Some("cuerpo bajo"),
        ],
    ),
    (
        23,
        &[
            Some("cabeza"),
            Some("cuerpo alto"),
            Some("patas delanteras"),
            Some("zona trasera"),
            Some("cuerpo bajo"),
        ],
    ),
    (
        24,
        &[
            Some("lomo"),
            Some("cabeza"),
            Some("patas delanteras"),
            Some("lomo bajo"),
            Some("patas traseras"),
            Some("rodadura"),
        ],
    ),
    (
        25,
        &[
            Some("cabeza"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("torso"),
            Some("cola"),
            Some("aleta dorsal"),
        ],
    ),
    (
        26,
        &[
            Some("cabeza"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("torso"),
            Some("cola"),
            Some("aleta dorsal"),
        ],
    ),
    (
        27,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("cola"),
            Some("patas traseras"),
            Some("pecho"),
        ],
    ),
    (
        28,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata delantera der."),
            Some("pata delantera izq."),
            Some("espalda"),
            Some("cola"),
            Some("patas traseras"),
        ],
    ),
    (
        29,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata delantera der."),
            Some("pata delantera izq."),
            Some("pata trasera der."),
            Some("pata trasera izq."),
            Some("cola"),
        ],
    ),
    (
        30,
        &[
            Some("cabeza"),
            Some("cuello"),
            Some("ala izq."),
            Some("ala der."),
            Some("torso"),
            Some("cola"),
            Some("patas"),
        ],
    ),
    (
        31,
        &[
            Some("cabeza"),
            Some("brazo der."),
            Some("brazo izq."),
            Some("pata der."),
            Some("pata izq."),
            Some("caparazón"),
            Some("pecho"),
            Some("cola"),
        ],
    ),
    (
        32,
        &[
            Some("torso"),
            Some("cabeza"),
            None,
            Some("brazo izq."),
            Some("brazo der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        33,
        &[
            Some("cabeza"),
            Some("garra"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("abdomen"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("abdomen"),
            Some("pecho"),
        ],
    ),
    (
        34,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
            Some("punta de la cola"),
            Some("bola de lodo"),
        ],
    ),
    (
        35,
        &[
            Some("cabeza"),
            Some("brazo der."),
            Some("brazo izq."),
            Some("torso bajo"),
            Some("espalda"),
            Some("cola"),
            Some("cola con saco de viento"),
            Some("pecho con saco de viento"),
            Some("espalda con saco de viento"),
        ],
    ),
    (
        36,
        &[
            Some("cabeza"),
            Some("espalda"),
            Some("brazo izq."),
            Some("brazo izq. (hielo)"),
            Some("brazo der."),
            Some("brazo der. (hielo)"),
            Some("pata izq."),
            Some("pata der."),
            Some("torso"),
        ],
    ),
    (
        37,
        &[Some("cabeza"), Some("cuerpo"), Some("brazos"), Some("cola")],
    ),
    (
        38,
        &[
            Some("cabeza"),
            Some("aletas de la pata trasera"),
            Some("brazo der."),
            Some("brazo izq."),
            Some("cola"),
            Some("torso"),
            Some("espalda"),
            Some("bolas de trueno"),
            Some("pecho"),
        ],
    ),
    (
        39,
        &[
            Some("cabeza"),
            Some("aletas de la pata trasera"),
            Some("brazo der."),
            Some("brazo izq."),
            Some("cola"),
            Some("torso"),
            Some("espalda"),
            Some("bolas de trueno"),
            Some("pecho"),
        ],
    ),
    (
        40,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
            Some("ala izq."),
            Some("ala der."),
        ],
    ),
    (
        41,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        42,
        &[
            Some("cabeza"),
            Some("brazos"),
            Some("cuerpo"),
            Some("cola"),
            Some("roca"),
            Some("olla"),
        ],
    ),
    (
        43,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
            Some("cabeza (lodo)"),
            Some("torso (lodo)"),
            Some("pata izq. (lodo)"),
            Some("pata der. (lodo)"),
            Some("cola (lodo)"),
        ],
    ),
    (
        44,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("espalda"),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("cola"),
        ],
    ),
    (
        45,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("patas"),
            Some("ala izq."),
            Some("ala der."),
            Some("cola"),
        ],
    ),
    (
        46,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("patas"),
            Some("ala izq."),
            Some("ala der."),
            Some("cola"),
        ],
    ),
    (
        76,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        77,
        &[
            Some("cuerpo"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cuello"),
            Some("cabeza"),
            Some("cola"),
        ],
    ),
    (
        78,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("caparazón"),
            Some("pata izq."),
            Some("pata der."),
            Some("garra izq."),
            Some("garra der."),
            Some("patas delanteras"),
        ],
    ),
    (
        79,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("caparazón"),
            Some("pata izq."),
            Some("pata der."),
            Some("garra izq."),
            Some("garra der."),
            Some("patas delanteras"),
            Some("caparazón"),
            Some("caparazón"),
        ],
    ),
    (
        80,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
        ],
    ),
    (
        81,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("membrana izq."),
            Some("cola"),
            Some("pata delantera"),
            Some("pata trasera"),
            Some("membrana der."),
            Some("pata trasera"),
        ],
    ),
    (
        82,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("cola"),
            Some("antena"),
        ],
    ),
    (
        83,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("cola"),
        ],
    ),
    (
        84,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata izq."),
            Some("pata der."),
            Some("ala izq."),
            Some("ala der."),
            Some("cola"),
        ],
    ),
    (
        85,
        &[
            Some("cresta"),
            Some("torso"),
            Some("ala der."),
            Some("ala izq."),
            Some("pata der."),
            Some("pata izq."),
            Some("cola"),
            Some("punta de la cola"),
        ],
    ),
    (
        86,
        &[
            Some("cabeza"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("torso"),
            Some("cola"),
            Some("aleta dorsal"),
        ],
    ),
    (
        87,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("brazo der."),
            Some("brazo izq."),
            Some("espalda"),
            Some("cola"),
            Some("patas traseras"),
        ],
    ),
    (
        88,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("brazo der."),
            Some("brazo izq."),
            Some("pata trasera der."),
            Some("pata trasera izq."),
            Some("cola"),
        ],
    ),
    (
        89,
        &[
            Some("torso"),
            Some("cabeza"),
            Some("cuello"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("pata izq."),
            Some("pata der."),
            Some("cola"),
        ],
    ),
    (
        90,
        &[
            Some("cabeza"),
            Some("garra"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("abdomen"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("abdomen"),
            Some("pecho"),
        ],
    ),
    (
        91,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
            Some("punta de la cola"),
            Some("bola de lodo"),
        ],
    ),
    (
        92,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("patas"),
            Some("ala izq."),
            Some("ala der."),
            Some("cola"),
        ],
    ),
    (
        93,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("patas traseras"),
            Some("alas"),
            Some("cola"),
        ],
    ),
    (
        94,
        &[
            Some("cabeza"),
            Some("cuerpo"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("patas traseras"),
            Some("cola"),
            Some("abdomen"),
            Some("espalda"),
        ],
    ),
    (
        95,
        &[
            Some("cabeza"),
            Some("cabeza"),
            Some("torso"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("pata trasera"),
            Some("pata trasera"),
            Some("cola"),
        ],
    ),
    (
        96,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("garra alar"),
            Some("garra alar"),
            Some("garra alar"),
            Some("garra alar"),
            Some("pata delantera"),
            Some("pata delantera"),
            Some("pata delantera"),
            Some("pata delantera"),
            Some("pata trasera"),
            Some("pata trasera"),
            Some("cola"),
            Some("cabeza"),
            Some("espalda"),
        ],
    ),
    (
        97,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("alas"),
            Some("patas"),
            Some("cola"),
        ],
    ),
    (
        98,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("alas"),
            Some("patas"),
            Some("cola"),
        ],
    ),
    (
        107,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata izq."),
            Some("pata der."),
            Some("alas"),
            Some("cola"),
        ],
    ),
    (
        108,
        &[
            Some("cabeza"),
            Some("pecho"),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("pata trasera izq."),
            Some("pata trasera der."),
            Some("cola"),
            Some("alas"),
        ],
    ),
    (
        109,
        &[
            Some("cabeza"),
            Some("pecho"),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("alas"),
            Some("cola"),
        ],
    ),
    (
        110,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("cola"),
        ],
    ),
    (
        111,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("pata delantera izq."),
            Some("pata delantera der."),
            Some("cola"),
            Some("patas traseras"),
            Some("pecho"),
        ],
    ),
    (
        112,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("patas traseras"),
            Some("alas"),
            Some("cola"),
        ],
    ),
    (
        113,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("ala izq."),
            Some("ala der."),
            Some("patas delanteras"),
            Some("patas traseras"),
            Some("cola"),
            Some("antena"),
        ],
    ),
    (
        114,
        &[
            Some("cabeza"),
            Some("torso"),
            Some("pata delantera"),
            Some("patas"),
            Some("alas"),
            Some("cola"),
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    ),
    (
        115,
        &[
            Some("cabeza"),
            Some("brazo izq."),
            Some("brazo der."),
            Some("abdomen"),
            Some("cola"),
            Some("espalda"),
            None,
            Some("pata izq."),
            Some("pata der."),
        ],
    ),
];

fn capture(monster: i32) -> Option<u8> {
    WORLD_CAPTURE
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|index| WORLD_CAPTURE[index].1)
}

/// Umbral de captura de World en porcentaje de vida, o `None` si el id no está en la tabla.
pub fn world_capture_percent(monster: i32) -> Option<f32> {
    capture(monster).map(f32::from)
}

/// Nombre de la parte de World con ese índice, si la tabla lo conoce.
pub fn world_part_name(monster: i32, index: usize) -> Option<&'static str> {
    let entry = WORLD_PARTS
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|position| WORLD_PARTS[position].1)?;

    entry
        .iter()
        .find(|part| part.index == index)
        .map(|part| part.name)
}

/// Nombre de la parte de Rise por su posición en el array de partes.
pub fn rise_part_name(monster: i32, position: usize) -> Option<&'static str> {
    let parts = RISE_PARTS
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|index| RISE_PARTS[index].1)?;

    parts.get(position).copied().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_world_definition_has_a_capture_threshold() {
        assert_eq!(WORLD_CAPTURE.len(), 72);
        assert!(world_capture_percent(0).is_some());
    }

    #[test]
    fn known_parts_resolve_to_names() {
        assert!(world_part_name(0, 0).is_some());
        assert!(rise_part_name(110, 0).is_some());
    }

    #[test]
    fn unknown_ids_and_positions_are_none() {
        assert_eq!(world_capture_percent(-1), None);
        assert_eq!(world_part_name(0, 999), None);
        assert_eq!(rise_part_name(-1, 0), None);
    }

    #[test]
    fn rise_part_names_keep_their_positions() {
        let parts = RISE_PARTS[0].1;
        assert!(parts.iter().flatten().count() > 0);
    }
}
