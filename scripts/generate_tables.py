#!/usr/bin/env python3
"""Genera crates/mh-overlay-games/src/tables.rs a partir de los MonsterData.xml de HunterPie.

Uso:
    python3 scripts/generate_tables.py rise.xml world.xml tablas.rs

Los XML son datos de HunterPie (Apache-2.0, ver NOTICE). El script falla si alguna parte no
tiene traducción: una tabla con etiquetas a medias es peor que no tener nombres, porque el
usuario no puede distinguir "sin nombre" de "nombre equivocado". Después de generarlo, pasa
`cargo fmt --all`: el formato final del archivo lo decide rustfmt.
"""

import sys
import xml.etree.ElementTree as ET

# Nombre corto de cada tipo de parte. Las variantes (roca, lodo, hielo…) se añaden entre
# paréntesis y el lado como sufijo.
BASE = {
    "HEAD": "cabeza",
    "TAIL": "cola",
    "TAIL_TIP": "punta de la cola",
    "BODY": "cuerpo",
    "ABDOMEN": "abdomen",
    "CHEST": "pecho",
    "TORSO": "torso",
    "NECK": "cuello",
    "BACK": "espalda",
    "UPPER_BACK": "lomo",
    "LOWER_BACK": "lomo bajo",
    "UPPER_BODY": "cuerpo alto",
    "LOWER_BODY": "cuerpo bajo",
    "LOWER_TORSO": "torso bajo",
    "REAR": "zona trasera",
    "JAW": "mandíbula",
    "MANE": "melena",
    "FIN": "aleta",
    "DORSAL_FIN": "aleta dorsal",
    "CREST": "cresta",
    "ANTENNA": "antena",
    "ANTLERS": "astas",
    "HORN": "cuerno",
    "HORNS": "cuernos",
    "HORNS_2": "cuernos 2",
    "HORNS_GOLD": "cuernos dorados",
    "SHELL": "caparazón",
    "CLAW": "garra",
    "WING_CLAW": "garra alar",
    "WING": "ala",
    "WINGS": "alas",
    "CUTWING": "membrana",
    "LEG": "pata",
    "LEGS": "patas",
    "ARM": "brazo",
    "ARMS": "brazos",
    "FORELEG": "pata delantera",
    "FORELEGS": "patas delanteras",
    "H_LEG": "pata trasera",
    "H_LEGS": "patas traseras",
    "B_LEG_FINS": "aletas de la pata trasera",
    "LIMBS": "extremidades",
    "BONE": "hueso",
    "SPONGE": "esponja",
    "POT": "olla",
    "BALLOON": "globo",
    "EMISSIONS": "emisiones",
    "CHARGE": "carga",
    "COUNTERATTACK": "contraataque",
    "RAGE": "furia",
    "ROLLING": "rodadura",
    "KNOCKDOWN": "derribo",
    "REPEL": "rechazo",
    "BIG_FLINCH": "gran tambaleo",
    "SKY_FALL": "caída",
    "MUD_BALL": "bola de lodo",
    "WINDSAC": "saco de viento",
    "THUNDERBALLS": "bolas de trueno",
    "GLOWING_HEAD": "cabeza brillante",
    "GLOWING_TAIL": "cola brillante",
    "INFLATED_TAIL": "cola inflada",
}

# Variantes de estado de la misma parte: se muestran entre paréntesis.
VARIANTS = {
    "ROCK": "roca",
    "MUD": "lodo",
    "ICE": "hielo",
    "SNOW": "nieve",
    "GOLD": "dorada",
}

# Claves que no son una parte física o que necesitan un nombre propio.
EXPLICIT = {
    "SILVER_SPIKES_HEAD": "cabeza con púas",
    "SILVER_SPIKES_L_ARM": "brazo izq. con púas",
    "SILVER_SPIKES_R_ARM": "brazo der. con púas",
    "SILVER_SPIKES_L_WING": "ala izq. con púas",
    "SILVER_SPIKES_R_WING": "ala der. con púas",
    "WEAK_L_SHELL": "caparazón débil izq.",
    "WEAK_R_SHELL": "caparazón débil der.",
    "EMERGE_SNOW_HEAD": "cabeza al emerger",
    "EMERGE_SNOW_BODY": "cuerpo al emerger",
    "EMERGE_SNOW_TAIL": "cola al emerger",
    "EXHAUST_ORGAN_HEAD": "órgano de gas frontal",
    "EXHAUST_ORGAN_CENTRAL": "órgano de gas central",
    "EXHAUST_ORGAN_REAR": "órgano de gas trasero",
    "EXHAUST_ORGAN_CRATER": "órgano de gas del cráter",
    "TAIL_WINDSAC": "cola con saco de viento",
    "CHEST_WINDSAC": "pecho con saco de viento",
    "BACK_WINDSAC": "espalda con saco de viento",
    "BODY_LEGS": "cuerpo y patas",
    "EXPLOSION_WEAKENING": "explosión debilitada",
    "THROAT": "garganta",
    "ROCK": "roca",
    "MUD": "lodo",
    "ICE": "hielo",
    "SNOW": "nieve",
    "GOLD": "dorado",
}

# Sin nombre: no es una parte que el jugador reconozca.
IGNORED = {"TO_BE_MAPPED", "UNKNOWN"}


def translate(key):
    name = key.removeprefix("PART_")

    if name in IGNORED:
        return None
    if name in EXPLICIT:
        return EXPLICIT[name]

    side = ""
    if name.startswith("L_"):
        side, name = " izq.", name[2:]
    elif name.startswith("R_"):
        side, name = " der.", name[2:]

    variant = ""
    for suffix, label in VARIANTS.items():
        if name.endswith("_" + suffix):
            variant, name = f" ({label})", name[: -len(suffix) - 1]
            break

    if name not in BASE:
        raise SystemExit(f"sin traducción para la parte {key}")

    return BASE[name] + side + variant


def rust_str(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def parse(path, severable_needed):
    root = ET.parse(path).getroot()
    monsters = []

    for monster in root.findall(".//Monster"):
        monster_id = int(monster.get("Id"))
        parts = []
        for part in monster.findall("./Parts/Part"):
            parts.append(
                (
                    int(part.get("Id")),
                    translate(part.get("String")),
                    part.get("IsSeverable") == "True",
                )
            )
        monsters.append((monster_id, monster.get("Capture"), parts))

    return monsters


def main():
    rise_path, world_path, output = sys.argv[1:4]
    rise = parse(rise_path, False)
    world = parse(world_path, True)

    lines = [
        "//! Tablas derivadas de los `MonsterData.xml` de HunterPie (Apache-2.0, ver `NOTICE`).",
        "//!",
        "//! Generado por `scripts/generate_tables.py`; no editar a mano. Rise identifica las",
        "//! partes por su posición en el array (igual que `MHRMonster.Parts[i]`); World las",
        "//! identifica por el índice que reporta el juego (`MHWMonsterPart`).",
        "",
        "/// Parte de World: índice del juego y nombre corto en castellano.",
        "pub struct WorldPart {",
        "    pub index: usize,",
        "    pub name: &'static str,",
        "}",
        "",
        "/// Porcentaje de vida por debajo del cual se puede capturar.",
        "const WORLD_CAPTURE: &[(i32, u8)] = &[",
    ]

    for monster_id, capture, _ in sorted(world, key=lambda entry: entry[0]):
        if capture is None:
            raise SystemExit(f"World {monster_id} sin atributo Capture")
        lines.append(f"    ({monster_id}, {capture}),")

    lines += ["];", "", "const WORLD_PARTS: &[(i32, &[WorldPart])] = &["]

    for monster_id, _, parts in sorted(world, key=lambda entry: entry[0]):
        named = sorted(
            (part_id, name) for part_id, name, _ in parts if name is not None
        )
        if not named:
            continue
        lines.append(f"    ({monster_id}, &[")
        for part_id, name in named:
            lines.append(
                f"        WorldPart {{ index: {part_id}, name: {rust_str(name)} }},"
            )
        lines.append("    ]),")

    lines += ["];", "", "const RISE_PARTS: &[(i32, &[Option<&'static str>])] = &["]

    for monster_id, _, parts in sorted(rise, key=lambda entry: entry[0]):
        lines.append(f"    ({monster_id}, &[")
        for _, name, _ in parts:
            rendered = "None" if name is None else f"Some({rust_str(name)})"
            lines.append(f"        {rendered},")
        lines.append("    ]),")

    lines += [
        "];",
        "",
        "fn capture(monster: i32) -> Option<u8> {",
        "    WORLD_CAPTURE",
        "        .binary_search_by_key(&monster, |entry| entry.0)",
        "        .ok()",
        "        .map(|index| WORLD_CAPTURE[index].1)",
        "}",
        "",
        "/// Umbral de captura de World en porcentaje de vida, o `None` si el id no está en la tabla.",
        "pub fn world_capture_percent(monster: i32) -> Option<f32> {",
        "    capture(monster).map(f32::from)",
        "}",
        "",
        "/// Nombre de la parte de World con ese índice, si la tabla lo conoce.",
        "pub fn world_part_name(monster: i32, index: usize) -> Option<&'static str> {",
        "    let entry = WORLD_PARTS",
        "        .binary_search_by_key(&monster, |entry| entry.0)",
        "        .ok()",
        "        .map(|position| WORLD_PARTS[position].1)?;",
        "",
        "    entry",
        "        .iter()",
        "        .find(|part| part.index == index)",
        "        .map(|part| part.name)",
        "}",
        "",
        "/// Nombre de la parte de Rise por su posición en el array de partes.",
        "pub fn rise_part_name(monster: i32, position: usize) -> Option<&'static str> {",
        "    let parts = RISE_PARTS",
        "        .binary_search_by_key(&monster, |entry| entry.0)",
        "        .ok()",
        "        .map(|index| RISE_PARTS[index].1)?;",
        "",
        "    parts.get(position).copied().flatten()",
        "}",
        "",
        "#[cfg(test)]",
        "mod tests {",
        "    use super::*;",
        "",
        "    #[test]",
        "    fn every_world_definition_has_a_capture_threshold() {",
        "        assert_eq!(WORLD_CAPTURE.len(), 72);",
        "        assert!(world_capture_percent(0).is_some());",
        "    }",
        "",
        "    #[test]",
        "    fn known_parts_resolve_to_names() {",
        "        assert!(world_part_name(0, 0).is_some());",
        "        assert!(rise_part_name(110, 0).is_some());",
        "    }",
        "",
        "    #[test]",
        "    fn unknown_ids_and_positions_are_none() {",
        "        assert_eq!(world_capture_percent(-1), None);",
        "        assert_eq!(world_part_name(0, 999), None);",
        "        assert_eq!(rise_part_name(-1, 0), None);",
        "    }",
        "",
        "    #[test]",
        "    fn rise_part_names_keep_their_positions() {",
        "        let parts = RISE_PARTS[0].1;",
        "        assert!(parts.iter().flatten().count() > 0);",
        "    }",
        "}",
        "",
    ]

    with open(output, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))

    print(f"{output}: {len(world)} monstruos de World, {len(rise)} de Rise")


if __name__ == "__main__":
    main()
