//! Tablas derivadas de los datos de HunterPie: `MonsterData.xml` (Apache-2.0) y la
//! localización `es-es.xml` (MIT), ver `NOTICE`.
//!
//! Generado por `scripts/generate_tables.py`; no editar a mano. Rise identifica las
//! partes por su posición en el array (igual que `MHRMonster.Parts[i]`); World las
//! identifica por el índice que reporta el juego (`MHWMonsterPart`).

use crate::model::CrownThresholds;

/// Parte de World: índice del juego, nombre en castellano y cómo se rompe.
pub struct WorldPart {
    pub index: usize,
    pub name: &'static str,
    /// `IsSeverable`: el juego guarda su vida en la zona de partes cortables.
    pub severable: bool,
    /// Rompimientos que aguanta la parte, en unidades de su propia vida (`Break`).
    pub breaks: &'static [u8],
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
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Garganta",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        1,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        4,
        &[
            WorldPart {
                index: 0,
                name: "Repeler",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Pecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Brazo derecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Caparazon",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Núcleo de magma (Centro)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 11,
                name: "Núcleo de magma (Cabeza)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 12,
                name: "Núcleo de magma (Crater)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 13,
                name: "Núcleo de magma (Trasero)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 14,
                name: "Caparazón débil izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 15,
                name: "Caparazón débil derecho",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        7,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Abdomen",
                severable: false,
                breaks: &[3],
            },
        ],
    ),
    (
        9,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        10,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        11,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        12,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        13,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        14,
        &[
            WorldPart {
                index: 0,
                name: "Cuerno",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        15,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cuernos",
                severable: false,
                breaks: &[2, 3],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 5,
                name: "Brazo derecho",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        16,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Extremidades izquierdas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Extremidades derechas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Alas",
                severable: false,
                breaks: &[2],
            },
        ],
    ),
    (
        17,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Extremidades",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        18,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cuernos",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Alas",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        19,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[6],
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Abdomen",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Pata izquierda",
                severable: false,
                breaks: &[4],
            },
            WorldPart {
                index: 4,
                name: "Pata derecha",
                severable: false,
                breaks: &[4],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[4],
            },
        ],
    ),
    (
        20,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pecho",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 5,
                name: "Trasero",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        21,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cabeza (lodo)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo (lodo)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Brazos (lodo)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 9,
                name: "Pata izquierda (lodo)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 10,
                name: "Pata derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 11,
                name: "Pata derecha (lodo)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 12,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 13,
                name: "Cola (lodo)",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        22,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Mandibula",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        23,
        &[
            WorldPart {
                index: 0,
                name: "Cuerpo & Patas",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cornamenta",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        24,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        25,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[2, 3],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Cuernos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Ala izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 11,
                name: "Ala derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 12,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        26,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Cuello",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Espalda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Alas",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 11,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        27,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Roca",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        28,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        29,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[4],
            },
            WorldPart {
                index: 1,
                name: "Torso",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Pata izquierda",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Pata derecha",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Cola",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 5,
                name: "Cabeza (lodo)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Torso (lodo)",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda (lodo)",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 8,
                name: "Pata derecha (lodo)",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 9,
                name: "Cola (lodo)",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        30,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Melena",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[4],
            },
        ],
    ),
    (
        31,
        &[
            WorldPart {
                index: 0,
                name: "Globo",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Globo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        32,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        33,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        34,
        &[
            WorldPart {
                index: 0,
                name: "Contraataque",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Piernas",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        35,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Mandibula",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 11,
                name: "Espalda",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 12,
                name: "Hueso izquierdo",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 13,
                name: "Hueso derecho",
                severable: false,
                breaks: &[1, 2],
            },
        ],
    ),
    (
        36,
        &[
            WorldPart {
                index: 0,
                name: "Emisiones de efluvio",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Espalda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Alas",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        37,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        38,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos #2",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cuernos",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cuernos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Pecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Extremidades izquierdas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Extremidades derechas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Cuernos Dorados",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 9,
                name: "Melena Dorada",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Pecho Izquierdo Dorado",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 11,
                name: "Pecho Derecho Dorado",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 12,
                name: "Brazo Izquierdo Dorado",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 13,
                name: "Brazo Derecho Dorado",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 14,
                name: "Pata Izquierda Dorada",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 15,
                name: "Pata Derecha Dorada",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 16,
                name: "Cola Dorada (Izq.)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 17,
                name: "Cola Dorada (Der.)",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        39,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        51,
        &[
            WorldPart {
                index: 0,
                name: "Cuerpo & Patas",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cornamenta",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        61,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        62,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[4],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        63,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        64,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Contraataque",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 6,
                name: "Trasero",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        65,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[4],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        66,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Garganta",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 3,
                name: "Aleta",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        67,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Garganta",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 4,
                name: "Aleta",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        68,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Garganta",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        69,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cola inflada",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        70,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[2, 4],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Espinas Plateadas (Cabeza)",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Espinas Plateadas (B. Izq.)",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Espinas Plateadas (B. Der.)",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Espinas Plateadas (Ala Izq.)",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Espinas Plateadas (Ala Der.)",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Cuernos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 11,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 12,
                name: "Brazo derecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 13,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 14,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 15,
                name: "Ala izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 16,
                name: "Ala derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 17,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        71,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 1,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Melena",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        72,
        &[
            WorldPart {
                index: 0,
                name: "Globo",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Globo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        73,
        &[
            WorldPart {
                index: 0,
                name: "Caída del cielo",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        74,
        &[
            WorldPart {
                index: 0,
                name: "Contraataque",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: true,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Brazos",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Piernas",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        75,
        &[
            WorldPart {
                index: 0,
                name: "Emisiones de efluvio",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 3,
                name: "Espalda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Alas",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        76,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza Brillante",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cola Brillante",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        77,
        &[
            WorldPart {
                index: 0,
                name: "Emerger desde la nieve (Cabeza)",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 1,
                name: "Emerger desde la nieve (Cuerpo)",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Emerger desde la nieve (Cola)",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Cabeza (Nieve)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Cuerpo (Nieve)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Piernas",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 9,
                name: "Cola (Nieve)",
                severable: false,
                breaks: &[1],
            },
        ],
    ),
    (
        78,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Cuernos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        79,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Caída del cielo",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Cabeza (Hielo)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo (Hielo)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Alas",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 7,
                name: "Alas (Hielo)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Brazos (Hielo)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 11,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        80,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 2,
                name: "Espalda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazos",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Alas",
                severable: false,
                breaks: &[2],
            },
        ],
    ),
    (
        81,
        &[
            WorldPart {
                index: 0,
                name: "Cabeza (Roca)",
                severable: true,
                breaks: &[1, 2, 3, 4],
            },
            WorldPart {
                index: 1,
                name: "Cabeza (Roca)",
                severable: true,
                breaks: &[1, 2, 3, 4],
            },
            WorldPart {
                index: 2,
                name: "Pecho (Roca)",
                severable: true,
                breaks: &[1, 2, 3, 4, 5, 6],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Cuello Izq. (Roca)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Cuello Der. (Roca)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cabeza (Roca)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola (Roca)",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Ala Izq. (Roca)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 10,
                name: "Ala Der. (Roca)",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 11,
                name: "Brazo Izq. (Roca)",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 12,
                name: "Brazo Der. (Roca)",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 13,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 14,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 15,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 16,
                name: "Ala izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 17,
                name: "Ala derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 18,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 19,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 20,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        87,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 1,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Explosión Elemental",
                severable: true,
                breaks: &[3],
            },
            WorldPart {
                index: 3,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Alas",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 6,
                name: "Brazos",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        88,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        89,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Retroceso grande",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 4,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        90,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2, 4],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        91,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cuernos #2",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Ira",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        92,
        &[
            WorldPart {
                index: 0,
                name: "Cuernos",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cuernos #2",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cola",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Ira",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Cabeza",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 10,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        93,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        94,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Carga",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2, 4],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Espalda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Brazos",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 6,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        95,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Carga",
                severable: true,
                breaks: &[],
            },
            WorldPart {
                index: 2,
                name: "Cabeza",
                severable: false,
                breaks: &[2, 4],
            },
            WorldPart {
                index: 3,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Espalda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Brazos",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 6,
                name: "Piernas",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        96,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Espalda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Brazo derecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 6,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Punta de la cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        97,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[5, 10],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Abdomen",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Espalda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 5,
                name: "Pecho",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[7],
            },
            WorldPart {
                index: 7,
                name: "Brazo derecho",
                severable: false,
                breaks: &[7],
            },
            WorldPart {
                index: 8,
                name: "Pata izquierda",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 9,
                name: "Pata derecha",
                severable: false,
                breaks: &[5],
            },
            WorldPart {
                index: 10,
                name: "Ala izquierda",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 11,
                name: "Ala derecha",
                severable: false,
                breaks: &[3],
            },
            WorldPart {
                index: 12,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        99,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[2, 4],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 3,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 4,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 6,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        100,
        &[
            WorldPart {
                index: 0,
                name: "Cola",
                severable: true,
                breaks: &[1],
            },
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[1, 2],
            },
            WorldPart {
                index: 2,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Brazo derecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 5,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Cola",
                severable: false,
                breaks: &[],
            },
        ],
    ),
    (
        101,
        &[
            WorldPart {
                index: 1,
                name: "Cabeza",
                severable: false,
                breaks: &[3, 6],
            },
            WorldPart {
                index: 2,
                name: "Cuello",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 3,
                name: "Pecho",
                severable: false,
                breaks: &[2],
            },
            WorldPart {
                index: 4,
                name: "Cuerpo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 5,
                name: "Brazo izquierdo",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 6,
                name: "Brazo derecho",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 7,
                name: "Pata izquierda",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 8,
                name: "Pata derecha",
                severable: false,
                breaks: &[],
            },
            WorldPart {
                index: 9,
                name: "Ala izquierda",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 10,
                name: "Ala derecha",
                severable: false,
                breaks: &[1],
            },
            WorldPart {
                index: 11,
                name: "Cola",
                severable: false,
                breaks: &[],
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

/// Umbrales de corona de World por el id que reporta el juego.
const WORLD_CROWNS: &[(i32, CrownThresholds)] = &[
    (
        0,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        1,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        4,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        7,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        9,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        10,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        11,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        12,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        13,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        14,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        15,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        16,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        17,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        18,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        19,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        20,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        21,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        22,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        23,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        24,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        25,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        26,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        27,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        28,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        29,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        30,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        31,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        32,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        33,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        34,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        35,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        36,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        37,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        38,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        39,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        51,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        61,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        62,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        63,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        64,
        CrownThresholds {
            mini: Some(0.99),
            silver: 1.14,
            gold: Some(1.2),
        },
    ),
    (
        65,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        66,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        67,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        68,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        69,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        70,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        71,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        },
    ),
    (
        72,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        73,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        74,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        75,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        76,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        77,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        78,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        79,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        80,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        81,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        87,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        88,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        89,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        90,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        91,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.11,
            gold: Some(1.18),
        },
    ),
    (
        92,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.11,
            gold: Some(1.18),
        },
    ),
    (
        93,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        94,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        95,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        96,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        97,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        98,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        99,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        100,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
    (
        101,
        CrownThresholds {
            mini: Some(0.9),
            silver: 1.15,
            gold: Some(1.23),
        },
    ),
];

/// Umbrales de corona de Rise por `MHRMonster.Id`.
const RISE_CROWNS: &[(i32, CrownThresholds)] = &[
    (
        1,
        CrownThresholds {
            mini: Some(0.96),
            silver: 1.1,
            gold: Some(1.14),
        },
    ),
    (
        3,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        4,
        CrownThresholds {
            mini: None,
            silver: 1.09,
            gold: Some(1.14),
        },
    ),
    (
        7,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        8,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        9,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        10,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        11,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        19,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        22,
        CrownThresholds {
            mini: Some(0.96),
            silver: 1.03,
            gold: Some(1.05),
        },
    ),
    (
        26,
        CrownThresholds {
            mini: Some(0.96),
            silver: 1.1,
            gold: Some(1.14),
        },
    ),
    (
        27,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        29,
        CrownThresholds {
            mini: None,
            silver: 1.09,
            gold: Some(1.14),
        },
    ),
    (
        30,
        CrownThresholds {
            mini: Some(0.765),
            silver: 0.9775,
            gold: Some(1.0455),
        },
    ),
    (
        76,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        77,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        80,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        81,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        82,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        83,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        86,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        87,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        88,
        CrownThresholds {
            mini: None,
            silver: 1.09,
            gold: Some(1.14),
        },
    ),
    (
        93,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        95,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        107,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        108,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        109,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.17,
            gold: None,
        },
    ),
    (
        110,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        111,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        112,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        113,
        CrownThresholds {
            mini: None,
            silver: 1.11,
            gold: Some(1.17),
        },
    ),
    (
        114,
        CrownThresholds {
            mini: Some(0.97),
            silver: 1.15,
            gold: None,
        },
    ),
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

/// Partes de World con ese id, en el orden en que el juego las registra.
pub fn world_parts(monster: i32) -> &'static [WorldPart] {
    WORLD_PARTS
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map_or(&[], |position| WORLD_PARTS[position].1)
}

/// Nombre de la parte de World con ese índice, si la tabla lo conoce.
pub fn world_part_name(monster: i32, index: usize) -> Option<&'static str> {
    world_parts(monster)
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

/// Umbrales de corona de World, o `None` si el monstruo no tiene tamaño variable.
pub fn world_crown_thresholds(monster: i32) -> Option<CrownThresholds> {
    WORLD_CROWNS
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|index| WORLD_CROWNS[index].1)
}

/// Lo mismo para Rise.
pub fn rise_crown_thresholds(monster: i32) -> Option<CrownThresholds> {
    RISE_CROWNS
        .binary_search_by_key(&monster, |entry| entry.0)
        .ok()
        .map(|index| RISE_CROWNS[index].1)
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
    fn fatalis_only_breaks_the_parts_the_game_breaks() {
        let broken: Vec<_> = world_parts(101)
            .iter()
            .filter(|part| !part.breaks.is_empty())
            .map(|part| part.name)
            .collect();

        assert_eq!(broken, ["Cabeza", "Pecho", "Ala izquierda", "Ala derecha"]);
        // La cabeza es la primera parte de la tabla y aguanta tres rompimientos.
        assert_eq!(world_parts(101)[0].breaks, [3, 6]);
        assert!(world_parts(101).iter().all(|part| !part.severable));
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

    #[test]
    fn every_crown_table_is_sorted_and_bounded() {
        assert!(WORLD_CROWNS.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(RISE_CROWNS.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(RISE_CROWNS.iter().all(|(_, crown)| {
            crown.mini.is_none_or(|mini| mini < crown.silver)
                && crown.gold.is_none_or(|gold| gold > crown.silver)
        }));
    }

    #[test]
    fn monsters_without_crowns_have_no_thresholds() {
        assert!(rise_crown_thresholds(0).is_none());
        assert!(world_crown_thresholds(0).is_some());
        assert!(rise_crown_thresholds(-1).is_none());
    }
}
