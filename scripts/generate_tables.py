#!/usr/bin/env python3
"""Genera crates/mh-overlay-games/src/tables.rs desde los datos de HunterPie.

Uso:
    python3 scripts/generate_tables.py rise.xml world.xml es-es.xml tablas.rs

Los XML son de HunterPie: `MonsterData.xml` (Apache-2.0) describe qué parte y qué estado
tiene cada id, y `es-es.xml` del repositorio de localización (MIT) trae los nombres que el
juego muestra en castellano. El script falla si una parte no tiene nombre: una tabla con
etiquetas a medias es peor que no tener nombres, porque el usuario no puede distinguir
"sin nombre" de "nombre equivocado". Después de generarlo, pasa `cargo fmt --all`: el
formato final del archivo lo decide rustfmt.
"""

import sys
import xml.etree.ElementTree as ET

# Claves que HunterPie deja sin nombre a propósito: no son una parte física.
#
# `PART_KNOCKDOWN` es el contador de derribos del monstruo, no un sitio del cuerpo: World lo
# marca como cortable y por eso aparecía como una parte "Noqueo" que nunca se rompe.
NON_PARTS = {"PART_UNKNOWN", "PART_TO_BE_MAPPED", "PART_KNOCKDOWN"}

# Estados de una misma parte, para completar las claves que la localización no trae.
VARIANTS = {
    "MUD": "lodo",
    "ROCK": "roca",
    "ICE": "hielo",
    "SNOW": "nieve",
    "GOLD": "dorado",
}


def rust_str(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def strings(root, path):
    """El nombre vive en el atributo `String`; el texto del nodo está vacío."""
    return {
        node.get("Id"): node.get("String")
        for node in root.findall(path)
        if node.get("String")
    }


def localized(root, game):
    return {
        int(key): name
        for key, name in strings(root, f"./Monsters/{game}/Monster").items()
    }


def part_names(root):
    return strings(root, "./Monsters/Shared/Part")


def ailment_names(root):
    return strings(root, "./Ailments/Rise/Ailment")


def parse_monsters(path):
    root = ET.parse(path).getroot()
    ailments = [
        (int(ailment.get("Id")), ailment.get("String"))
        for ailment in root.findall(".//Ailments/Ailment")
    ]

    monsters = []
    for monster in root.findall(".//Monster"):
        parts = [
            (
                int(part.get("Id")),
                part.get("String"),
                part.get("IsSeverable") == "True",
                [int(threshold.get("Threshold")) for threshold in part.findall("./Break")],
            )
            for part in monster.findall("./Parts/Part")
        ]
        monsters.append((int(monster.get("Id")), monster.get("Capture"), parts))

    return monsters, ailments


def main():
    rise_path, world_path, locale_path, output = sys.argv[1:5]

    locale = ET.parse(locale_path).getroot()
    parts = part_names(locale)
    ailments = ailment_names(locale)
    rise_names = localized(locale, "Rise")
    world_names = localized(locale, "World")

    rise, _ = parse_monsters(rise_path)
    world, world_ailments = parse_monsters(world_path)

    def name_of(key, origin):
        if key in NON_PARTS:
            return None
        if key in parts:
            return parts[key]

        # Variantes que HunterPie tampoco traduce: "PART_BODY_MUD" es el cuerpo con lodo.
        base, _, variant = key.rpartition("_")
        if base in parts and variant in VARIANTS:
            return f"{parts[base]} ({VARIANTS[variant]})"

        raise SystemExit(f"sin nombre en la localización para {key} ({origin})")

    lines = [
        "//! Tablas derivadas de los datos de HunterPie: `MonsterData.xml` (Apache-2.0) y la",
        "//! localización `es-es.xml` (MIT), ver `NOTICE`.",
        "//!",
        "//! Generado por `scripts/generate_tables.py`; no editar a mano. Rise identifica las",
        "//! partes por su posición en el array (igual que `MHRMonster.Parts[i]`); World las",
        "//! identifica por el índice que reporta el juego (`MHWMonsterPart`).",
        "",
        "/// Parte de World: índice del juego, nombre en castellano y cómo se rompe.",
        "pub struct WorldPart {",
        "    pub index: usize,",
        "    pub name: &'static str,",
        "    /// `IsSeverable`: el juego guarda su vida en la zona de partes cortables.",
        "    pub severable: bool,",
        "    /// Rompimientos que aguanta la parte, en unidades de su propia vida (`Break`).",
        "    pub breaks: &'static [u8],",
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

    for monster_id, _, parts_list in sorted(world, key=lambda entry: entry[0]):
        named = [
            (part_id, name, severable, breaks)
            for part_id, key, severable, breaks in sorted(parts_list)
            if (name := name_of(key, f"World {monster_id}")) is not None
        ]
        if not named:
            continue
        lines.append(f"    ({monster_id}, &[")
        for part_id, name, severable, breaks in named:
            thresholds = ", ".join(str(break_) for break_ in breaks)
            lines.append(
                f"        WorldPart {{ index: {part_id}, name: {rust_str(name)}, "
                f"severable: {str(severable).lower()}, breaks: &[{thresholds}] }},"
            )
        lines.append("    ]),")

    lines += ["];", "", "const RISE_PARTS: &[(i32, &[Option<&'static str>])] = &["]

    for monster_id, _, parts_list in sorted(rise, key=lambda entry: entry[0]):
        lines.append(f"    ({monster_id}, &[")
        for _, key, _, _ in parts_list:
            name = name_of(key, f"Rise {monster_id}")
            lines.append(f"        {('None' if name is None else 'Some(' + rust_str(name) + ')')},")
        lines.append("    ]),")

    lines += [
        "];",
        "",
        "/// Nombres de los monstruos de World por el id que reporta el juego.",
        "const WORLD_MONSTERS: &[(i32, &str)] = &[",
    ]
    for monster_id, name in sorted(world_names.items()):
        lines.append(f"    ({monster_id}, {rust_str(name)}),")

    lines += [
        "];",
        "",
        "/// Nombres de los monstruos de Rise por `MHRMonster.Id`.",
        "const RISE_MONSTERS: &[(i32, &str)] = &[",
    ]
    for monster_id, name in sorted(rise_names.items()):
        lines.append(f"    ({monster_id}, {rust_str(name)}),")

    lines += [
        "];",
        "",
        "/// Estados alterados de World por `MHWMonsterAilmentStructure.Id`.",
        "const WORLD_AILMENTS: &[(i32, &str)] = &[",
    ]

    for ailment_id, key in sorted(world_ailments):
        if key not in ailments:
            # Los ids sin nombre vienen con el propio número como clave; el lector los
            # ignora en vez de mostrar un estado que no puede nombrar.
            continue
        lines.append(f"    ({ailment_id}, {rust_str(ailments[key])}),")

    lines += [
        "];",
        "",
        "fn lookup(table: &[(i32, &'static str)], key: i32) -> Option<&'static str> {",
        "    table",
        "        .binary_search_by_key(&key, |entry| entry.0)",
        "        .ok()",
        "        .map(|index| table[index].1)",
        "}",
        "",
        "/// Umbral de captura de World en porcentaje de vida, o `None` si el id no está en la tabla.",
        "pub fn world_capture_percent(monster: i32) -> Option<f32> {",
        "    WORLD_CAPTURE",
        "        .binary_search_by_key(&monster, |entry| entry.0)",
        "        .ok()",
        "        .map(|index| f32::from(WORLD_CAPTURE[index].1))",
        "}",
        "",
        "/// Partes de World con ese id, en el orden en que el juego las registra.",
        "pub fn world_parts(monster: i32) -> &'static [WorldPart] {",
        "    WORLD_PARTS",
        "        .binary_search_by_key(&monster, |entry| entry.0)",
        "        .ok()",
        "        .map_or(&[], |position| WORLD_PARTS[position].1)",
        "}",
        "",
        "/// Nombre de la parte de World con ese índice, si la tabla lo conoce.",
        "pub fn world_part_name(monster: i32, index: usize) -> Option<&'static str> {",
        "    world_parts(monster)",
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
        "/// Nombre legible del monstruo, si la localización trae ese id.",
        "pub fn world_monster_name(monster: i32) -> Option<&'static str> {",
        "    lookup(WORLD_MONSTERS, monster)",
        "}",
        "",
        "/// Lo mismo para Rise.",
        "pub fn rise_monster_name(monster: i32) -> Option<&'static str> {",
        "    lookup(RISE_MONSTERS, monster)",
        "}",
        "",
        "/// Nombre del estado alterado de World, si el id corresponde a uno conocido.",
        "pub fn world_ailment_name(ailment: i32) -> Option<&'static str> {",
        "    lookup(WORLD_AILMENTS, ailment)",
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
        "    fn known_monsters_have_their_localized_name() {",
        "        assert_eq!(world_monster_name(0), Some(\"Anjanath\"));",
        "        assert_eq!(rise_monster_name(0), Some(\"Rathian\"));",
        "    }",
        "",
        "    #[test]",
        "    fn known_parts_resolve_to_names() {",
        "        assert_eq!(world_part_name(1, 1), Some(\"Cabeza\"));",
        "        assert!(rise_part_name(110, 0).is_some());",
        "    }",
        "",
        "    #[test]",
        "    fn fatalis_only_breaks_the_parts_the_game_breaks() {",
        "        let broken: Vec<_> = world_parts(101)",
        "            .iter()",
        "            .filter(|part| !part.breaks.is_empty())",
        "            .map(|part| part.name)",
        "            .collect();",
        "",
        "        assert_eq!(broken, [\"Cabeza\", \"Pecho\", \"Ala izquierda\", \"Ala derecha\"]);",
        "        assert_eq!(world_parts(101)[1].breaks, [3, 6]);",
        "        assert!(world_parts(101).iter().all(|part| !part.severable));",
        "    }",
        "",
        "    #[test]",
        "    fn unknown_ids_and_positions_are_none() {",
        "        assert_eq!(world_capture_percent(-1), None);",
        "        assert_eq!(world_monster_name(-1), None);",
        "        assert_eq!(rise_monster_name(-1), None);",
        "        assert_eq!(world_part_name(0, 999), None);",
        "        assert_eq!(rise_part_name(-1, 0), None);",
        "    }",
        "",
        "    #[test]",
        "    fn world_ailments_resolve_except_the_ones_the_game_does_not_name() {",
        "        assert_eq!(world_ailment_name(1), Some(\"Veneno\"));",
        "        assert_eq!(world_ailment_name(0), None);",
        "        assert_eq!(world_ailment_name(13), None);",
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

    print(
        f"{output}: {len(world)} monstruos de World, {len(rise)} de Rise, "
        f"{len(world_names)} nombres de World y {len(rise_names)} de Rise"
    )


if __name__ == "__main__":
    main()
