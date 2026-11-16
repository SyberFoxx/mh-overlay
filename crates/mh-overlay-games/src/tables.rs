//! Tablas derivadas de los datos de HunterPie: `MonsterData.xml` (Apache-2.0) y la
//! localización `es-es.xml` (MIT), ver `NOTICE`.
//!
//! Generado por `scripts/generate_tables.py`; no editar a mano. Rise identifica las
//! partes por su posición en el array (igual que `MHRMonster.Parts[i]`); World las
//! identifica por el índice que reporta el juego (`MHWMonsterPart`).

/// Parte de World: índice del juego y nombre en castellano.
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
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Garganta",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        1,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        4,
        &[
            WorldPart {
                index: 0,
                name: "Repeler",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Pecho",
            },
            WorldPart {
                index: 4,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 5,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
            WorldPart {
                index: 9,
                name: "Caparazon",
            },
            WorldPart {
                index: 10,
                name: "Núcleo de magma (Centro)",
            },
            WorldPart {
                index: 11,
                name: "Núcleo de magma (Cabeza)",
            },
            WorldPart {
                index: 12,
                name: "Núcleo de magma (Crater)",
            },
            WorldPart {
                index: 13,
                name: "Núcleo de magma (Trasero)",
            },
            WorldPart {
                index: 14,
                name: "Caparazón débil izquierdo",
            },
            WorldPart {
                index: 15,
                name: "Caparazón débil derecho",
            },
        ],
    ),
    (
        7,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
            },
            WorldPart {
                index: 2,
                name: "Brazos",
            },
            WorldPart {
                index: 3,
                name: "Piernas",
            },
            WorldPart {
                index: 4,
                name: "Cola",
            },
            WorldPart {
                index: 5,
                name: "Abdomen",
            },
        ],
    ),
    (
        9,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        10,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        11,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        12,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        13,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        14,
        &[
            WorldPart {
                index: 0,
                name: "Cuerno",
            },
            WorldPart {
                index: 1,
                name: "Brazos",
            },
            WorldPart {
                index: 2,
                name: "Piernas",
            },
        ],
    ),
    (
        15,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cuernos",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 5,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        16,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Cola",
            },
            WorldPart {
                index: 4,
                name: "Extremidades izquierdas",
            },
            WorldPart {
                index: 5,
                name: "Extremidades derechas",
            },
            WorldPart {
                index: 6,
                name: "Alas",
            },
        ],
    ),
    (
        17,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Extremidades",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        18,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cuernos",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Brazos",
            },
            WorldPart {
                index: 5,
                name: "Piernas",
            },
            WorldPart {
                index: 6,
                name: "Alas",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        19,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
            },
            WorldPart {
                index: 2,
                name: "Abdomen",
            },
            WorldPart {
                index: 3,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 4,
                name: "Pata derecha",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
        ],
    ),
    (
        20,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Pecho",
            },
            WorldPart {
                index: 5,
                name: "Trasero",
            },
            WorldPart {
                index: 6,
                name: "Brazos",
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
            },
            WorldPart {
                index: 9,
                name: "Cola",
            },
        ],
    ),
    (
        21,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cabeza (lodo)",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo (lodo)",
            },
            WorldPart {
                index: 6,
                name: "Brazos",
            },
            WorldPart {
                index: 7,
                name: "Brazos (lodo)",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata izquierda (lodo)",
            },
            WorldPart {
                index: 10,
                name: "Pata derecha",
            },
            WorldPart {
                index: 11,
                name: "Pata derecha (lodo)",
            },
            WorldPart {
                index: 12,
                name: "Cola",
            },
            WorldPart {
                index: 13,
                name: "Cola (lodo)",
            },
        ],
    ),
    (
        22,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Mandibula",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Brazos",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        23,
        &[
            WorldPart {
                index: 0,
                name: "Cuerpo & Patas",
            },
            WorldPart {
                index: 1,
                name: "Cornamenta",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Piernas",
            },
            WorldPart {
                index: 4,
                name: "Brazos",
            },
        ],
    ),
    (
        24,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        25,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 3,
                name: "Cuernos",
            },
            WorldPart {
                index: 4,
                name: "Cabeza",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 11,
                name: "Ala derecha",
            },
            WorldPart {
                index: 12,
                name: "Cola",
            },
        ],
    ),
    (
        26,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuello",
            },
            WorldPart {
                index: 4,
                name: "Espalda",
            },
            WorldPart {
                index: 5,
                name: "Pecho",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Alas",
            },
            WorldPart {
                index: 11,
                name: "Cola",
            },
        ],
    ),
    (
        27,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Brazos",
            },
            WorldPart {
                index: 2,
                name: "Piernas",
            },
            WorldPart {
                index: 3,
                name: "Cola",
            },
            WorldPart {
                index: 4,
                name: "Roca",
            },
        ],
    ),
    (
        28,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Brazos",
            },
            WorldPart {
                index: 3,
                name: "Piernas",
            },
            WorldPart {
                index: 4,
                name: "Cola",
            },
        ],
    ),
    (
        29,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Torso",
            },
            WorldPart {
                index: 2,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 3,
                name: "Pata derecha",
            },
            WorldPart {
                index: 4,
                name: "Cola",
            },
            WorldPart {
                index: 5,
                name: "Cabeza (lodo)",
            },
            WorldPart {
                index: 6,
                name: "Torso (lodo)",
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda (lodo)",
            },
            WorldPart {
                index: 8,
                name: "Pata derecha (lodo)",
            },
            WorldPart {
                index: 9,
                name: "Cola (lodo)",
            },
        ],
    ),
    (
        30,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
            },
            WorldPart {
                index: 2,
                name: "Melena",
            },
            WorldPart {
                index: 3,
                name: "Brazos",
            },
            WorldPart {
                index: 4,
                name: "Piernas",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
        ],
    ),
    (
        31,
        &[
            WorldPart {
                index: 0,
                name: "Globo",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Globo",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
            },
            WorldPart {
                index: 6,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 7,
                name: "Ala derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        32,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
            },
            WorldPart {
                index: 2,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 3,
                name: "Ala derecha",
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        33,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazos",
            },
            WorldPart {
                index: 4,
                name: "Piernas",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
        ],
    ),
    (
        34,
        &[
            WorldPart {
                index: 0,
                name: "Contraataque",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Brazos",
            },
            WorldPart {
                index: 5,
                name: "Piernas",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        35,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 5,
                name: "Cabeza",
            },
            WorldPart {
                index: 6,
                name: "Cuerpo",
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
            },
            WorldPart {
                index: 9,
                name: "Cola",
            },
            WorldPart {
                index: 10,
                name: "Mandibula",
            },
            WorldPart {
                index: 11,
                name: "Espalda",
            },
            WorldPart {
                index: 12,
                name: "Hueso izquierdo",
            },
            WorldPart {
                index: 13,
                name: "Hueso derecho",
            },
        ],
    ),
    (
        36,
        &[
            WorldPart {
                index: 0,
                name: "Emisiones de efluvio",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Espalda",
            },
            WorldPart {
                index: 4,
                name: "Pecho",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Alas",
            },
        ],
    ),
    (
        37,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazos",
            },
            WorldPart {
                index: 4,
                name: "Piernas",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
        ],
    ),
    (
        38,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos #2",
            },
            WorldPart {
                index: 1,
                name: "Cuernos",
            },
            WorldPart {
                index: 2,
                name: "Cuernos",
            },
            WorldPart {
                index: 3,
                name: "Pecho",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Extremidades izquierdas",
            },
            WorldPart {
                index: 6,
                name: "Extremidades derechas",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
            WorldPart {
                index: 8,
                name: "Cuernos Dorados",
            },
            WorldPart {
                index: 9,
                name: "Melena Dorada",
            },
            WorldPart {
                index: 10,
                name: "Pecho Izquierdo Dorado",
            },
            WorldPart {
                index: 11,
                name: "Pecho Derecho Dorado",
            },
            WorldPart {
                index: 12,
                name: "Brazo Izquierdo Dorado",
            },
            WorldPart {
                index: 13,
                name: "Brazo Derecho Dorado",
            },
            WorldPart {
                index: 14,
                name: "Pata Izquierda Dorada",
            },
            WorldPart {
                index: 15,
                name: "Pata Derecha Dorada",
            },
            WorldPart {
                index: 16,
                name: "Cola Dorada (Izq.)",
            },
            WorldPart {
                index: 17,
                name: "Cola Dorada (Der.)",
            },
        ],
    ),
    (
        39,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Piernas",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        51,
        &[
            WorldPart {
                index: 0,
                name: "Cuerpo & Patas",
            },
            WorldPart {
                index: 1,
                name: "Cornamenta",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Piernas",
            },
            WorldPart {
                index: 4,
                name: "Brazos",
            },
        ],
    ),
    (
        61,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        62,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        63,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        64,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
            },
            WorldPart {
                index: 2,
                name: "Contraataque",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Pecho",
            },
            WorldPart {
                index: 6,
                name: "Trasero",
            },
            WorldPart {
                index: 7,
                name: "Brazos",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Cola",
            },
        ],
    ),
    (
        65,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        66,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Garganta",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Aleta",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Brazos",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        67,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Garganta",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Aleta",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
            },
            WorldPart {
                index: 6,
                name: "Brazos",
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
            },
            WorldPart {
                index: 9,
                name: "Cola",
            },
        ],
    ),
    (
        68,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Garganta",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        69,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cola inflada",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 6,
                name: "Ala derecha",
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
            },
            WorldPart {
                index: 9,
                name: "Cola",
            },
        ],
    ),
    (
        70,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 3,
                name: "Espinas Plateadas (Cabeza)",
            },
            WorldPart {
                index: 4,
                name: "Espinas Plateadas (B. Izq.)",
            },
            WorldPart {
                index: 5,
                name: "Espinas Plateadas (B. Der.)",
            },
            WorldPart {
                index: 6,
                name: "Espinas Plateadas (Ala Izq.)",
            },
            WorldPart {
                index: 7,
                name: "Espinas Plateadas (Ala Der.)",
            },
            WorldPart {
                index: 8,
                name: "Cuernos",
            },
            WorldPart {
                index: 9,
                name: "Cabeza",
            },
            WorldPart {
                index: 10,
                name: "Cuerpo",
            },
            WorldPart {
                index: 11,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 12,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 13,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 14,
                name: "Pata derecha",
            },
            WorldPart {
                index: 15,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 16,
                name: "Ala derecha",
            },
            WorldPart {
                index: 17,
                name: "Cola",
            },
        ],
    ),
    (
        71,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
            },
            WorldPart {
                index: 2,
                name: "Melena",
            },
            WorldPart {
                index: 3,
                name: "Brazos",
            },
            WorldPart {
                index: 4,
                name: "Piernas",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
        ],
    ),
    (
        72,
        &[
            WorldPart {
                index: 0,
                name: "Globo",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Globo",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
            },
            WorldPart {
                index: 6,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 7,
                name: "Ala derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        73,
        &[
            WorldPart {
                index: 0,
                name: "Caída del cielo",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
            },
            WorldPart {
                index: 5,
                name: "Piernas",
            },
            WorldPart {
                index: 6,
                name: "Cola",
            },
        ],
    ),
    (
        74,
        &[
            WorldPart {
                index: 0,
                name: "Contraataque",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Brazos",
            },
            WorldPart {
                index: 6,
                name: "Piernas",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        75,
        &[
            WorldPart {
                index: 0,
                name: "Emisiones de efluvio",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Espalda",
            },
            WorldPart {
                index: 4,
                name: "Pecho",
            },
            WorldPart {
                index: 5,
                name: "Cola",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Alas",
            },
        ],
    ),
    (
        76,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza Brillante",
            },
            WorldPart {
                index: 2,
                name: "Cola Brillante",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Piernas",
            },
            WorldPart {
                index: 6,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 7,
                name: "Ala derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        77,
        &[
            WorldPart {
                index: 0,
                name: "Emerger desde la nieve (Cabeza)",
            },
            WorldPart {
                index: 1,
                name: "Emerger desde la nieve (Cuerpo)",
            },
            WorldPart {
                index: 2,
                name: "Emerger desde la nieve (Cola)",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Cabeza (Nieve)",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
            },
            WorldPart {
                index: 6,
                name: "Cuerpo (Nieve)",
            },
            WorldPart {
                index: 7,
                name: "Piernas",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
            WorldPart {
                index: 9,
                name: "Cola (Nieve)",
            },
        ],
    ),
    (
        78,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 3,
                name: "Cuernos",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        79,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Caída del cielo",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cabeza (Hielo)",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo (Hielo)",
            },
            WorldPart {
                index: 6,
                name: "Alas",
            },
            WorldPart {
                index: 7,
                name: "Alas (Hielo)",
            },
            WorldPart {
                index: 8,
                name: "Brazos",
            },
            WorldPart {
                index: 9,
                name: "Brazos (Hielo)",
            },
            WorldPart {
                index: 10,
                name: "Piernas",
            },
            WorldPart {
                index: 11,
                name: "Cola",
            },
        ],
    ),
    (
        80,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Espalda",
            },
            WorldPart {
                index: 3,
                name: "Cola",
            },
            WorldPart {
                index: 4,
                name: "Brazos",
            },
            WorldPart {
                index: 5,
                name: "Piernas",
            },
            WorldPart {
                index: 6,
                name: "Alas",
            },
        ],
    ),
    (
        81,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza (Roca)",
            },
            WorldPart {
                index: 1,
                name: "Cabeza (Roca)",
            },
            WorldPart {
                index: 2,
                name: "Pecho (Roca)",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 5,
                name: "Cuello Izq. (Roca)",
            },
            WorldPart {
                index: 6,
                name: "Cuello Der. (Roca)",
            },
            WorldPart {
                index: 7,
                name: "Cabeza (Roca)",
            },
            WorldPart {
                index: 8,
                name: "Cola (Roca)",
            },
            WorldPart {
                index: 9,
                name: "Ala Izq. (Roca)",
            },
            WorldPart {
                index: 10,
                name: "Ala Der. (Roca)",
            },
            WorldPart {
                index: 11,
                name: "Brazo Izq. (Roca)",
            },
            WorldPart {
                index: 12,
                name: "Brazo Der. (Roca)",
            },
            WorldPart {
                index: 13,
                name: "Piernas",
            },
            WorldPart {
                index: 14,
                name: "Cabeza",
            },
            WorldPart {
                index: 15,
                name: "Cuerpo",
            },
            WorldPart {
                index: 16,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 17,
                name: "Ala derecha",
            },
            WorldPart {
                index: 18,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 19,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 20,
                name: "Cola",
            },
        ],
    ),
    (
        87,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cola",
            },
            WorldPart {
                index: 3,
                name: "Explosión Elemental",
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
            },
            WorldPart {
                index: 4,
                name: "Pecho",
            },
            WorldPart {
                index: 5,
                name: "Alas",
            },
            WorldPart {
                index: 6,
                name: "Brazos",
            },
            WorldPart {
                index: 7,
                name: "Piernas",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        88,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        89,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
        ],
    ),
    (
        90,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 4,
                name: "Pata derecha",
            },
            WorldPart {
                index: 5,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 6,
                name: "Ala derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        91,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cuernos #2",
            },
            WorldPart {
                index: 2,
                name: "Cola",
            },
            WorldPart {
                index: 3,
                name: "Ira",
            },
            WorldPart {
                index: 4,
                name: "Cabeza",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Cola",
            },
        ],
    ),
    (
        92,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
            },
            WorldPart {
                index: 1,
                name: "Cuernos #2",
            },
            WorldPart {
                index: 2,
                name: "Cola",
            },
            WorldPart {
                index: 3,
                name: "Ira",
            },
            WorldPart {
                index: 4,
                name: "Cabeza",
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Cola",
            },
        ],
    ),
    (
        93,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        94,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Carga",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Espalda",
            },
            WorldPart {
                index: 5,
                name: "Brazos",
            },
            WorldPart {
                index: 6,
                name: "Piernas",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        95,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Carga",
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
            },
            WorldPart {
                index: 4,
                name: "Espalda",
            },
            WorldPart {
                index: 5,
                name: "Brazos",
            },
            WorldPart {
                index: 6,
                name: "Piernas",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        96,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Espalda",
            },
            WorldPart {
                index: 4,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 5,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
            },
            WorldPart {
                index: 8,
                name: "Cola",
            },
            WorldPart {
                index: 9,
                name: "Punta de la cola",
            },
        ],
    ),
    (
        97,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Abdomen",
            },
            WorldPart {
                index: 4,
                name: "Espalda",
            },
            WorldPart {
                index: 5,
                name: "Pecho",
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
            },
            WorldPart {
                index: 10,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 11,
                name: "Ala derecha",
            },
            WorldPart {
                index: 12,
                name: "Cola",
            },
        ],
    ),
    (
        99,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 4,
                name: "Pata derecha",
            },
            WorldPart {
                index: 5,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 6,
                name: "Ala derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        100,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
            },
            WorldPart {
                index: 7,
                name: "Cola",
            },
        ],
    ),
    (
        101,
        &[
            WorldPart {
                index: 0,
                name: "Noqueo",
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
            },
            WorldPart {
                index: 2,
                name: "Cuello",
            },
            WorldPart {
                index: 3,
                name: "Pecho",
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
            },
            WorldPart {
                index: 5,
                name: "Brazo izquierdo",
            },
            WorldPart {
                index: 6,
                name: "Brazo derecho",
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
            },
            WorldPart {
                index: 9,
                name: "Ala izquierda",
            },
            WorldPart {
                index: 10,
                name: "Ala derecha",
            },
            WorldPart {
                index: 11,
                name: "Cola",
            },
        ],
    ),
];

const RISE_PARTS: &[(i32, &[Option<&'static str>])] = &[
    (
        0,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        1,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        2,
        &[
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cuello"),
            Some("Cabeza"),
            Some("Cola"),
        ],
    ),
    (
        3,
        &[
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cuello"),
            Some("Cabeza"),
            Some("Cola"),
        ],
    ),
    (
        4,
        &[
            Some("Cabeza"),
            Some("Cuello"),
            Some("Torso"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Cola"),
        ],
    ),
    (
        5,
        &[
            Some("Espalda"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cabeza"),
            Some("Pecho"),
            Some("Cola"),
        ],
    ),
    (
        6,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        7,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        8,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        9,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Alas"),
            Some("Cola"),
        ],
    ),
    (
        10,
        &[
            Some("Cabeza"),
            Some("Pecho"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
            Some("Alas"),
        ],
    ),
    (
        11,
        &[
            Some("Cabeza"),
            Some("Pecho"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Alas"),
            Some("Cola"),
        ],
    ),
    (
        12,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
        ],
    ),
    (
        13,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala cuchilla izquierda"),
            Some("Cola"),
            Some("Pata delantera"),
            Some("Pata trasera izquierda"),
            Some("Ala cuchilla derecha"),
            Some("Pata trasera derecha"),
        ],
    ),
    (
        14,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
        ],
    ),
    (
        15,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazos"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
            Some("Cabeza (lodo)"),
            Some("Torso (lodo)"),
            Some("Brazos (lodo)"),
            Some("Pata izquierda (lodo)"),
            Some("Pata derecha (lodo)"),
            Some("Cola (lodo)"),
        ],
    ),
    (
        16,
        &[
            Some("Cabeza"),
            Some("Esponja"),
            Some("Torso"),
            Some("Patas izquierdas"),
            Some("Patas derechas"),
            Some("Cola"),
        ],
    ),
    (17, &[Some("Cabeza"), Some("Cuerpo"), Some("Cola")]),
    (
        18,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Espalda"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
        ],
    ),
    (
        19,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Espalda"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
        ],
    ),
    (20, &[Some("Cabeza"), Some("Cuerpo"), Some("Cola")]),
    (
        21,
        &[
            Some("Cabeza"),
            Some("Cuerpo superior"),
            Some("Pata delantera"),
            Some("Trasero"),
            Some("Cuerpo inferior"),
        ],
    ),
    (
        22,
        &[
            Some("Cabeza"),
            Some("Cuerpo superior"),
            Some("Pata delantera"),
            Some("Trasero"),
            Some("Cuerpo inferior"),
        ],
    ),
    (
        23,
        &[
            Some("Cabeza"),
            Some("Cuerpo superior"),
            Some("Pata delantera"),
            Some("Trasero"),
            Some("Cuerpo inferior"),
        ],
    ),
    (
        24,
        &[
            Some("Trasero superior"),
            Some("Cabeza"),
            Some("Pata delantera"),
            Some("Trasero inferior"),
            Some("Pata trasera"),
            Some("Rodar"),
        ],
    ),
    (
        25,
        &[
            Some("Cabeza"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Torso"),
            Some("Cola"),
            Some("Aleta dorsal"),
        ],
    ),
    (
        26,
        &[
            Some("Cabeza"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Torso"),
            Some("Cola"),
            Some("Aleta dorsal"),
        ],
    ),
    (
        27,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Cola"),
            Some("Pata trasera"),
            Some("Pecho"),
        ],
    ),
    (
        28,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata delantera derecha"),
            Some("Pata delantera izquierda"),
            Some("Espalda"),
            Some("Cola"),
            Some("Pata trasera"),
        ],
    ),
    (
        29,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata delantera derecha"),
            Some("Pata delantera izquierda"),
            Some("Pata trasera derecha"),
            Some("Pata trasera izquierda"),
            Some("Cola"),
        ],
    ),
    (
        30,
        &[
            Some("Cabeza"),
            Some("Cuello"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Torso"),
            Some("Cola"),
            Some("Piernas"),
        ],
    ),
    (
        31,
        &[
            Some("Cabeza"),
            Some("Brazo derecho"),
            Some("Brazo izquierdo"),
            Some("Pata derecha"),
            Some("Pata izquierda"),
            Some("Caparazon"),
            Some("Pecho"),
            Some("Cola"),
        ],
    ),
    (
        32,
        &[
            Some("Torso"),
            Some("Cabeza"),
            None,
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        33,
        &[
            Some("Cabeza"),
            Some("Garra"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Abdomen"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Abdomen"),
            Some("Pecho"),
        ],
    ),
    (
        34,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
            Some("Punta de la cola"),
            Some("Bola lodosa"),
        ],
    ),
    (
        35,
        &[
            Some("Cabeza"),
            Some("Brazo derecho"),
            Some("Brazo izquierdo"),
            Some("Torso inferior"),
            Some("Espalda"),
            Some("Cola"),
            Some("Punta cola (vesic.)"),
            Some("Pecho (vesicula)"),
            Some("Espalda (vesicula)"),
        ],
    ),
    (
        36,
        &[
            Some("Cabeza"),
            Some("Espalda"),
            Some("Brazo izquierdo"),
            Some("Brazo izquierdo (hielo)"),
            Some("Brazo derecho"),
            Some("Brazo derecho (hielo)"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Torso"),
        ],
    ),
    (
        37,
        &[Some("Cabeza"), Some("Cuerpo"), Some("Brazos"), Some("Cola")],
    ),
    (
        38,
        &[
            Some("Cabeza"),
            Some("Aleta de la pata trasera"),
            Some("Brazo derecho"),
            Some("Brazo izquierdo"),
            Some("Cola"),
            Some("Torso"),
            Some("Espalda"),
            Some("Rayoesfera"),
            Some("Pecho"),
        ],
    ),
    (
        39,
        &[
            Some("Cabeza"),
            Some("Aleta de la pata trasera"),
            Some("Brazo derecho"),
            Some("Brazo izquierdo"),
            Some("Cola"),
            Some("Torso"),
            Some("Espalda"),
            Some("Rayoesfera"),
            Some("Pecho"),
        ],
    ),
    (
        40,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
        ],
    ),
    (
        41,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        42,
        &[
            Some("Cabeza"),
            Some("Brazos"),
            Some("Cuerpo"),
            Some("Cola"),
            Some("Roca"),
            Some("Vasija"),
        ],
    ),
    (
        43,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
            Some("Cabeza (lodo)"),
            Some("Torso (lodo)"),
            Some("Pata izquierda (lodo)"),
            Some("Pata derecha (lodo)"),
            Some("Cola (lodo)"),
        ],
    ),
    (
        44,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Espalda"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Cola"),
        ],
    ),
    (
        45,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Piernas"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Cola"),
        ],
    ),
    (
        46,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Piernas"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Cola"),
        ],
    ),
    (
        76,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        77,
        &[
            Some("Cuerpo"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cuello"),
            Some("Cabeza"),
            Some("Cola"),
        ],
    ),
    (
        78,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Caparazon"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Garra Izq."),
            Some("Garra Der."),
            Some("Pata delantera"),
        ],
    ),
    (
        79,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Caparazon"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Garra Izq."),
            Some("Garra Der."),
            Some("Pata delantera"),
            Some("Caparazon"),
            Some("Caparazon"),
        ],
    ),
    (
        80,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
        ],
    ),
    (
        81,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Ala cuchilla izquierda"),
            Some("Cola"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Ala cuchilla derecha"),
            Some("Pata trasera"),
        ],
    ),
    (
        82,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Cola"),
            Some("Antena"),
        ],
    ),
    (
        83,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Cola"),
        ],
    ),
    (
        84,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Cola"),
        ],
    ),
    (
        85,
        &[
            Some("Cresta"),
            Some("Torso"),
            Some("Ala derecha"),
            Some("Ala izquierda"),
            Some("Pata derecha"),
            Some("Pata izquierda"),
            Some("Cola"),
            Some("Punta de la cola"),
        ],
    ),
    (
        86,
        &[
            Some("Cabeza"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Torso"),
            Some("Cola"),
            Some("Aleta dorsal"),
        ],
    ),
    (
        87,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazo derecho"),
            Some("Brazo izquierdo"),
            Some("Espalda"),
            Some("Cola"),
            Some("Pata trasera"),
        ],
    ),
    (
        88,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Brazo derecho"),
            Some("Brazo izquierdo"),
            Some("Pata trasera derecha"),
            Some("Pata trasera izquierda"),
            Some("Cola"),
        ],
    ),
    (
        89,
        &[
            Some("Torso"),
            Some("Cabeza"),
            Some("Cuello"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Cola"),
        ],
    ),
    (
        90,
        &[
            Some("Cabeza"),
            Some("Garra"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Abdomen"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Abdomen"),
            Some("Pecho"),
        ],
    ),
    (
        91,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
            Some("Punta de la cola"),
            Some("Bola lodosa"),
        ],
    ),
    (
        92,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Piernas"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Cola"),
        ],
    ),
    (
        93,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata trasera"),
            Some("Alas"),
            Some("Cola"),
        ],
    ),
    (
        94,
        &[
            Some("Cabeza"),
            Some("Cuerpo"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata trasera"),
            Some("Cola"),
            Some("Abdomen"),
            Some("Espalda"),
        ],
    ),
    (
        95,
        &[
            Some("Cabeza"),
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata trasera"),
            Some("Pata trasera"),
            Some("Cola"),
        ],
    ),
    (
        96,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Garraala"),
            Some("Garraala"),
            Some("Garraala"),
            Some("Garraala"),
            Some("Pata delantera"),
            Some("Pata delantera"),
            Some("Pata delantera"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Pata trasera"),
            Some("Cola"),
            Some("Cabeza"),
            Some("Espalda"),
        ],
    ),
    (
        97,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Alas"),
            Some("Piernas"),
            Some("Cola"),
        ],
    ),
    (
        98,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Alas"),
            Some("Piernas"),
            Some("Cola"),
        ],
    ),
    (
        107,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata izquierda"),
            Some("Pata derecha"),
            Some("Alas"),
            Some("Cola"),
        ],
    ),
    (
        108,
        &[
            Some("Cabeza"),
            Some("Pecho"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Pata trasera izquierda"),
            Some("Pata trasera derecha"),
            Some("Cola"),
            Some("Alas"),
        ],
    ),
    (
        109,
        &[
            Some("Cabeza"),
            Some("Pecho"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Alas"),
            Some("Cola"),
        ],
    ),
    (
        110,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Cola"),
        ],
    ),
    (
        111,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata delantera izquierda"),
            Some("Pata delantera derecha"),
            Some("Cola"),
            Some("Pata trasera"),
            Some("Pecho"),
        ],
    ),
    (
        112,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Pata trasera"),
            Some("Alas"),
            Some("Cola"),
        ],
    ),
    (
        113,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Ala izquierda"),
            Some("Ala derecha"),
            Some("Pata delantera"),
            Some("Pata trasera"),
            Some("Cola"),
            Some("Antena"),
        ],
    ),
    (
        114,
        &[
            Some("Cabeza"),
            Some("Torso"),
            Some("Pata delantera"),
            Some("Piernas"),
            Some("Alas"),
            Some("Cola"),
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
            Some("Cabeza"),
            Some("Brazo izquierdo"),
            Some("Brazo derecho"),
            Some("Abdomen"),
            Some("Cola"),
            Some("Espalda"),
            None,
            Some("Pata izquierda"),
            Some("Pata derecha"),
        ],
    ),
];

/// Nombres de los monstruos de World por el id que reporta el juego.
const WORLD_MONSTERS: &[(i32, &str)] = &[
    (0, "Anjanath"),
    (1, "Rathalos"),
    (4, "Zorah Magdaros"),
    (7, "Gran Jagras"),
    (9, "Rathian"),
    (10, "Rathian Rosa"),
    (11, "Rathalos Celeste"),
    (12, "Diablos"),
    (13, "Diablos Negra"),
    (14, "Kirin"),
    (15, "Bégimo"),
    (16, "Kushala Daora"),
    (17, "Lunastra"),
    (18, "Teostra"),
    (19, "Lavasioth"),
    (20, "Deviljho"),
    (21, "Barroth"),
    (22, "Uragaan"),
    (23, "Leshen"),
    (24, "Pukei-Pukei"),
    (25, "Nergigante"),
    (26, "Xeno'jiiva"),
    (27, "Kulu-Ya-Ku"),
    (28, "Tzitzi-Ya-Ku"),
    (29, "Jyuratodus"),
    (30, "Tobi-Kadachi"),
    (31, "Paolumu"),
    (32, "Legiana"),
    (33, "Gran Girros"),
    (34, "Odogaron"),
    (35, "Radobaan"),
    (36, "Vaal Hazak"),
    (37, "Dodogama"),
    (38, "Kulve Taroth"),
    (39, "Bazelgeuse"),
    (51, "Leshen antiguo"),
    (61, "Tigrex"),
    (62, "Nargacuga"),
    (63, "Barioth"),
    (64, "Deviljho Salvaje"),
    (65, "Brachydios"),
    (66, "Glavenus"),
    (67, "Glavenus Acídico"),
    (68, "Anjanath Fulgúreo"),
    (69, "Pukei-Pukei Coralino"),
    (70, "Nergigante Regio"),
    (71, "Tobi-Kadachi Vípero"),
    (72, "Paolumu Somnus"),
    (73, "Legiana Aullador"),
    (74, "Odogaron Ébano"),
    (75, "Vaal Hazak Espóreo"),
    (76, "Bazelgeuse Magma"),
    (77, "Beotodus"),
    (78, "Banbaro"),
    (79, "Velkhana"),
    (80, "Namielle"),
    (81, "Shara Ishvalda"),
    (87, "Alatreon"),
    (88, "Rathian Dorada"),
    (89, "Rathalos Plateado"),
    (90, "Yian Garuga"),
    (91, "Rajang"),
    (92, "Rajang Rabioso"),
    (93, "Tigrex Pardo"),
    (94, "Zinogre"),
    (95, "Zinogre Estigio"),
    (96, "Brachydios Colérico"),
    (97, "Safi'jiiva"),
    (98, "Poste de entrenamiento"),
    (99, "Yian Garuga Naevus"),
    (100, "Barioth Cynodon"),
    (101, "Fatalis"),
];

/// Nombres de los monstruos de Rise por `MHRMonster.Id`.
const RISE_MONSTERS: &[(i32, &str)] = &[
    (0, "Rathian"),
    (1, "Rathian Apex"),
    (2, "Rathalos"),
    (3, "Rathalos Apex"),
    (4, "Khezu"),
    (5, "Basarios"),
    (6, "Diablos"),
    (7, "Diablos Apex"),
    (8, "Rajang"),
    (9, "Kushala Daora"),
    (10, "Chameleos"),
    (11, "Teostra"),
    (12, "Tigrex"),
    (13, "Nargacuga"),
    (14, "Barioth"),
    (15, "Barroth"),
    (16, "Ludroth Real"),
    (17, "Gran Baggi"),
    (18, "Zinogre"),
    (19, "Zinogre Apex"),
    (20, "Gran Wroggi"),
    (21, "Arzuros"),
    (22, "Arzuros Apex"),
    (23, "Lagombi"),
    (24, "Volvidon"),
    (25, "Mizutsune"),
    (26, "Mizutsune Apex"),
    (27, "Valstrax Carmesí"),
    (28, "Magnamalo"),
    (29, "Bishaten"),
    (30, "Aknosom"),
    (31, "Tetranadon"),
    (32, "Somnacanth"),
    (33, "Rakna-Kadaki"),
    (34, "Almudron"),
    (35, "Ibushi de viento"),
    (36, "Goss Harag"),
    (37, "Gran Izuchi"),
    (38, "Narwa de trueno"),
    (39, "Narwa la creadora"),
    (40, "Anjanath"),
    (41, "Pukei-Pukei"),
    (42, "Kulu-Ya-Ku"),
    (43, "Jyuratodus"),
    (44, "Tobi-Kadachi"),
    (45, "Bazelgeuse"),
    (46, "Saprobot"),
    (76, "Rathian Dorada"),
    (77, "Rathalos Plateado"),
    (78, "Daimyo Hermitaur"),
    (79, "Shogun Ceanataur"),
    (80, "Rajang Rabioso"),
    (81, "Nargacuga Lunar"),
    (82, "Gore Magala"),
    (83, "Shagaru Magala"),
    (84, "Seregios"),
    (85, "Astalos"),
    (86, "Mizutsune Violeta"),
    (87, "Magnamalo Humillado"),
    (88, "Bishaten Sanguina"),
    (89, "Somnacanth Aurora"),
    (90, "Rakna-kadaki Pira"),
    (91, "Almudron Magma"),
    (92, "Bezelgeuse Magma"),
    (93, "Malzeno"),
    (94, "Lunagaron"),
    (95, "Garangolm"),
    (96, "Gaismagorm"),
    (97, "Espinas"),
    (98, "Espinas Llameante"),
    (107, "Kushala Daora elevado"),
    (108, "Chameleos elevado"),
    (109, "Teostra elevado"),
    (110, "Shagaru Magala elevado"),
    (111, "Valstrax Carmesí elevado"),
    (112, "Malzeno primordial"),
    (113, "Gore Magala caótico"),
    (114, "Velkhana"),
    (115, "Amatsu"),
];

/// Estados alterados de World por `MHWMonsterAilmentStructure.Id`.
const WORLD_AILMENTS: &[(i32, &str)] = &[
    (1, "Veneno"),
    (2, "Parálisis"),
    (3, "Sueño"),
    (4, "Nitro"),
    (5, "Montar"),
    (6, "Fatiga"),
    (7, "Aturdir"),
    (8, "Tranquilizar"),
    (9, "Flash"),
    (10, "Flash"),
    (11, "Derribo"),
    (12, "Bomba boñiga"),
    (14, "Trampa eléctrica"),
    (15, "Trampa de escollo"),
    (16, "Trampa de enredaderas"),
    (18, "Sello de ancianos"),
    (19, "Fumadera"),
    (21, "Trampa felyne"),
    (22, "Garra retráctil"),
    (23, "Derribo de garra"),
    (24, "Cápsula de expulsión"),
];

fn lookup(table: &[(i32, &'static str)], key: i32) -> Option<&'static str> {
    table
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .map(|index| table[index].1)
}

/// Umbral de captura de World en porcentaje de vida, o `None` si el id no está en la tabla.
pub fn world_capture_percent(monster: i32) -> Option<f32> {
    WORLD_CAPTURE
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|index| f32::from(WORLD_CAPTURE[index].1))
}

/// Nombre de la parte de World con ese índice, si la tabla lo conoce.
pub fn world_part_name(monster: i32, index: usize) -> Option<&'static str> {
    let parts = WORLD_PARTS
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|position| WORLD_PARTS[position].1)?;

    parts
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

/// Nombre legible del monstruo, si la localización trae ese id.
pub fn world_monster_name(monster: i32) -> Option<&'static str> {
    lookup(WORLD_MONSTERS, monster)
}

/// Lo mismo para Rise.
pub fn rise_monster_name(monster: i32) -> Option<&'static str> {
    lookup(RISE_MONSTERS, monster)
}

/// Nombre del estado alterado de World, si el id corresponde a uno conocido.
pub fn world_ailment_name(ailment: i32) -> Option<&'static str> {
    lookup(WORLD_AILMENTS, ailment)
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
    fn known_monsters_have_their_localized_name() {
        assert_eq!(world_monster_name(0), Some("Anjanath"));
        assert_eq!(rise_monster_name(0), Some("Rathian"));
    }

    #[test]
    fn known_parts_resolve_to_names() {
        assert_eq!(world_part_name(1, 1), Some("Cabeza"));
        assert!(rise_part_name(110, 0).is_some());
    }

    #[test]
    fn unknown_ids_and_positions_are_none() {
        assert_eq!(world_capture_percent(-1), None);
        assert_eq!(world_monster_name(-1), None);
        assert_eq!(rise_monster_name(-1), None);
        assert_eq!(world_part_name(0, 999), None);
        assert_eq!(rise_part_name(-1, 0), None);
    }

    #[test]
    fn world_ailments_resolve_except_the_ones_the_game_does_not_name() {
        assert_eq!(world_ailment_name(1), Some("Veneno"));
        assert_eq!(world_ailment_name(0), None);
        assert_eq!(world_ailment_name(13), None);
    }

    #[test]
    fn rise_part_names_keep_their_positions() {
        let parts = RISE_PARTS[0].1;
        assert!(parts.iter().flatten().count() > 0);
    }
}
