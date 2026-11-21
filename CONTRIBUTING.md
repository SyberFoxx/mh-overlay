# Cómo contribuir

Gracias por mirar el proyecto. Este documento dice lo que no se ve en el código: qué se
considera un cambio aceptable, cómo se prueba sin tener el juego y qué se espera de un commit.

## Antes de empezar

- Rust **stable** (edición 2024). El proyecto compila con la toolchain del día; `rust-toolchain`
  no lo fija porque el CI usa `stable`.
- Dependencias de sistema para el hub: `libx11-dev`, `libxkbcommon-dev`, `libwayland-dev`,
  `libgl1-mesa-dev`, `libxi-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxinerama-dev` y
  `pkg-config`. El panel y la consola no necesitan OpenGL.
- **No hace falta** el juego para compilar, pasar los tests ni trabajar en el hub: los lectores
  se prueban contra una imagen de memoria escrita a mano (`crates/mh-overlay-games/src/fixtures.rs`).

## Los cuatro comandos

```bash
cargo build
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

El CI ejecuta los tres últimos; una PR que los deja en rojo no se revisa. `cargo fmt` es
obligatorio: no se aceptan diffs de formato a mano.

Para ver el hub sin instalar nada:

```bash
cargo run -p mh-overlay-cli -- --hub
```

Y para leer sin ventana, con el juego abierto:

```bash
cargo run -p mh-overlay-cli -- --watch --game rise
```

## Mapa del proyecto

| Crate | Qué vive ahí |
| --- | --- |
| `mh-overlay-memory` | `/proc`, `process_vm_readv`, cadenas de punteros y el parser de `.map`. `libc` es su única dependencia. |
| `mh-overlay-games` | Lectores por juego (`rise.rs`, `world.rs`), el modelo compartido y las fixtures de memoria. |
| `mh-overlay-config` | TOML documentado, validación, migración de esquema y rutas XDG. |
| `mh-overlay-ui` | Lienzo, tipografía y la superficie X11/Wayland del panel. |
| `mh-overlay-app` | Hub de escritorio: diagnóstico, ajustes, vista previa y tema. |
| `mh-overlay-cli` | El binario `mh-overlay`, argumentos y salida de consola. |

## Reglas que no se negocian

- **Nunca se escribe en la memoria del juego.** El crate solo expone lecturas. Añadir un writer
  es una decisión de producto, no un refactor, y una PR que escriba memoria se rechaza.
- **Los offsets viven en `data/maps/`, no en Rust.** La excepción son los offsets de campo que
  HunterPie también tiene fijos, documentados con la versión del juego en la que se verificaron.
- **Un `.map` que no se puede parsear es un error.** Nunca hay un fallback silencioso a otra
  versión: offsets equivocados devuelven basura creíble, y eso es peor que fallar.
- **Sin `unwrap`/`expect` fuera de tests.** Tampoco `panic!` por condiciones que el juego puede
  producir (página sin mapear, puntero nulo, offset viejo): eso es `Err`.
- **Cada `unsafe` lleva un comentario** con la invariante que lo hace sonido.
- **Una dependencia nueva necesita justificación** en el mensaje del commit. Lo que se pueda
  hacer con la biblioteca estándar, se hace con la biblioteca estándar.
- **El comentario explica el layout, no el código**: `// +0x18 es el HP actual`, no
  `// suma uno`.

## Añadir un juego

1. Un módulo en `crates/mh-overlay-games/src/` con su lector, como `rise.rs` o `world.rs`.
2. Una variante en `GameKind` (`mh-overlay-games`) y otra en el `GameKind` de
   `mh-overlay-config`, con su `module()` y su nombre visible.
3. El `.map` en `data/maps/`, con el nombre que usa HunterPie para esa versión.
4. Una fixture y su test en `fixtures.rs`: vida, partes, estados, jugador, cacería y arma,
   leídos de bytes escritos a mano. Sin juego y sin ptrace.

No hay ningún `match` sobre el nombre del juego en `main`: si tu cambio lo necesita, la
abstracción está en el sitio equivocado.

## Cambios de configuración

El TOML lleva `config_version`. Añadir una sección o una clave **con valor por defecto** no sube
la versión: no cambia el significado de nada. Subirla es para cuando un archivo viejo necesita
un cambio real para seguir queriendo decir lo mismo, y entonces la migración se escribe en
`Config::migrate` con su comentario y su test. El archivo del usuario se reescribe al guardar,
así que un cambio que no migre se pierde en silencio.

## Tests

- `cargo test -p mh-overlay-games` si tocas un lector; `-p mh-overlay-config` si tocas el TOML.
- Los tests no abren ventanas ni tocan el juego. La interfaz se prueba con `egui::Context` sin
  ventana (`run_pass`/`run_ui`), como en `theme.rs`.
- Un fallo se arregla con un test que lo reproduzca; no se arregla bajando el listón de otro.

## Commits y PR

- Prefijo convencional: `feat`, `fix`, `perf`, `docs`, `test`, `chore`.
- Un cambio lógico por commit, y el mensaje dice **por qué**, no solo qué se tocó.
- `README.md` se actualiza en el mismo cambio que altera lo que ve el usuario. `CHANGELOG.md`
  en la sección `Unreleased`. `ROADMAP.md` mueve la entrada a **Hecho** con la versión o el
  commit.
- Si el cambio se puede explicar en una frase, la PR puede ser esa frase y la lista de
  comprobaciones.

## Reportar un fallo

Abre un issue con la plantilla. Antes, pega la salida de:

```bash
mh-overlay --doctor
```

Ese informe ya incluye sesión, permisos, configuración, mapas, procesos y la ruta del registro
(`$XDG_STATE_HOME/mh-overlay/hub.log`), así que casi nunca hace falta más. No adjuntes volcados
de memoria del juego ni tu `config.toml` entero si lleva rutas personales.
