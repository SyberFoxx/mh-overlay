#!/usr/bin/env python3
"""Extrae la sección de CHANGELOG.md de una versión para usarla como notas de release."""

import pathlib
import sys


def section(changelog: str, version: str) -> str:
    marker = f"## [{version}]"
    start = changelog.find(marker)

    if start < 0:
        raise SystemExit(f"::error::CHANGELOG.md no tiene una sección para {version}")

    body = changelog[start + len(marker) :]
    body = body.split("\n", 1)[1] if "\n" in body else ""
    next_section = body.find("\n## [")

    if next_section >= 0:
        body = body[:next_section]

    body = body.strip()

    if not body:
        raise SystemExit(f"::error::La sección {version} de CHANGELOG.md está vacía")

    return body


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("uso: release_notes.py <versión> <destino>")

    version, destination = sys.argv[1], pathlib.Path(sys.argv[2])
    body = section(pathlib.Path("CHANGELOG.md").read_text(), version)

    destination.write_text(body + "\n")
    print(f"notas de {version}: {len(body)} caracteres")


if __name__ == "__main__":
    main()
