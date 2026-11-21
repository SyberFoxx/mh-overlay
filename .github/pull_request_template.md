## Qué cambia y por qué

<!-- Una o dos frases. Si no cabe, probablemente son dos PR. -->

## Cómo se probó

<!--
  Tests que añadiste o corriste, y qué cubren. Si no hay test (por ejemplo, un cambio visual),
  dilo y explica cómo lo comprobaste.
-->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Antes de pedir revisión

- [ ] No se escribe en la memoria del juego (el crate no expone writer).
- [ ] Los offsets nuevos están en `data/maps/`, no en Rust.
- [ ] Sin `unwrap`/`expect` fuera de tests; los fallos del juego son `Err`.
- [ ] Dependencias nuevas: ninguna, o justificadas en el commit.
- [ ] `README.md` / `CHANGELOG.md` (`Unreleased`) / `ROADMAP.md` actualizados si cambia lo que ve el usuario.
