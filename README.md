# mh-overlay

Overlay nativo para Linux que muestra la vida de los monstruos, el daño acumulado en cada
parte y los estados alterados en **Monster Hunter: World** y **Monster Hunter Rise**,
leyendo la memoria del juego mientras corre bajo Proton.

Es la alternativa a [HunterPie](https://github.com/HunterPie/HunterPie) en Linux: no necesita
Wine para sí mismo, no inyecta DLLs y no escribe ni un byte en el proceso del juego.

```
id 12     HP  74.3%  (16420 / 22100)   captura ≤ 20%
  parte 0    flinch 100.0%  romper  62.4%  cortar 100.0%
  parte 6    flinch  88.1%  romper  41.9%  cortar  41.9%
  veneno  45.2%  (120 / 265)
```

## Estado

| | Rise | World |
| --- | --- | --- |
| Capa de memoria (`process_vm_readv`, cadenas, `.map`) | ✅ probado | ✅ probado |
| Adjuntar al proceso y resolver la base del módulo | ✅ probado | ✅ probado |
| Vida actual / máxima y porcentaje | ✅ escrito | ✅ escrito |
| Umbral de captura | ✅ escrito | ⏳ |
| Partes: flinch / romper / cortar | ✅ escrito | ⚠️ romper y cortar, sin nombres |
| Estados alterados | ✅ escrito | ⏳ |
| Nombres de monstruo y de parte | ⏳ | ⏳ |
| Overlay gráfico | ⏳ | ⏳ |

Leyenda: ✅ hecho · ⚠️ parcial · ⏳ pendiente

> **Sin validar en juego.** El entorno donde se escribió esto no tiene ninguno de los dos
títulos instalados. La capa de memoria, el parser de `.map` y la CLI están probados con tests;
los lectores de Rise y World están escritos contra el layout documentado por HunterPie y la
primera ejecución contra un juego real es lo primero que hay que hacer. Si un número sale
mal, el sospechoso es el `.map` o el desplazamiento fijo, no la capa de memoria.

## Requisitos

- Linux, kernel ≥ 3.2 (`process_vm_readv`).
- Rust 1.85+ para compilar.
- El juego corriendo bajo Proton con **modo ventana sin bordes** (el overlay necesita dibujar
  encima; el fullscreen exclusivo lo impide).

### El permiso que todo el mundo olvida

Por defecto Arch y la mayoría de distros traen `kernel.yama.ptrace_scope = 1`, que prohíbe
leer la memoria de un proceso que no sea hijo tuyo. Steam lanza el juego, no `mh-overlay`,
así que sin esto **todas las lecturas fallan** con `EPERM` aunque el proceso se encuentre.

```bash
# opción A: darle la capacidad al binario, permanente y con alcance mínimo
sudo setcap cap_sys_ptrace+ep ./target/release/mh-overlay

# opción B: bajar la restricción mientras juegas, y devolverla después
sudo sysctl kernel.yama.ptrace_scope=0
```

La opción A es preferible. Ojo: si un lanzador corre con `no_new_privs`, la capacidad se
ignora y hay que usar la B.

## Compilar y usar

```bash
cargo build --release
./target/release/mh-overlay --game rise
```

```
-g, --game <JUEGO>      rise o world (por defecto rise)
    --pid <PID>         pid explícito, si hay más de una instancia corriendo
    --map <ARCHIVO>     address map concreto; por defecto el más nuevo de data/maps
    --base <HEX>        base del módulo, si /proc no la expone
    --watch             refresco continuo en lugar de una sola lectura
    --interval <MS>     periodo de refresco en modo --watch (por defecto 500)
    --list-maps         muestra los address maps disponibles
```

Ejemplo con refresco cada cuarto de segundo:

```bash
sudo setcap cap_sys_ptrace+ep ./target/release/mh-overlay
./target/release/mh-overlay --game world --watch --interval 250
```

Cada ejecución imprime una línea por monstruo, una por parte y una por estado activo; el
README abre con un ejemplo de esa salida.

## Arquitectura

```
crates/
├── mh-overlay-memory   proceso, /proc, process_vm_readv, cadenas de punteros, parser .map
├── mh-overlay-games    rise.rs, world.rs, modelo compartido
└── mh-overlay-cli      binario mh-overlay
data/maps/              66 address maps por versión de juego (ver NOTICE)
```

Tres ideas sostienen todo lo demás:

1. **La base del módulo se resuelve en cada attach.** `/proc/<pid>/maps` da la dirección de
   carga de `MonsterHunterRise.exe`; las direcciones estáticas del `.map` son relativas a ella
   y cambian con ASLR.
2. **Hay dos formas de cadena de punteros y no son intercambiables.** `deref_chain` hace
   `addr = *(addr) + offset`; `follow_ptr_chain` hace `addr = *(addr + offset)`. HunterPie las
   llama `ReadAsync` y `ReadPtrAsync`; usar la equivocada devuelve basura plausible.
3. **Los offsets son datos, no código.** Viven en `data/maps/MonsterHunterRise.<versión>.map`.
   Cuando el juego parchea, se añade un archivo, no se recompila. Un `.map` que no parsea es un
   error, nunca un fallback silencioso.

Para añadir un juego: un módulo en `mh-overlay-games` y una variante en `GameKind`. Nada más.

## Los address maps

Los archivos de `data/maps/` vienen de HunterPie (Apache-2.0, ver `NOTICE`) y son el resultado
de su reverse engineering. Este repositorio no reutiliza su código C#, pero sí esas tablas:
son la parte cara del problema y ya están resueltas para 17 builds de World, 20 de Rise y 30 de
Wilds (Wilds aún sin lector).

Si actualizas un mapa a mano, sube el archivo con el número de versión del ejecutable en el
nombre. `--map` permite probar uno sin tocar los demás.

## Límites conocidos

- **Solo lectura.** No hay escritura ni inyección; el crate no expone un writer a propósito.
- **Wilds no está soportado** todavía, aunque sus mapas están incluidos.
- **Los nombres de monstruo y de parte no están mapeados**: se muestran ids. Las traducciones
  de HunterPie se descargan en runtime y no están en el repositorio.
- **World reporta las partes sin nombre** porque su tabla de partes depende de datos por
  monstruo que aún no se han portado.
- **Wayland nativo**: el overlay futuro será una ventana X11, que funciona sobre juegos de
  Proton (que ya son XWayland). En Wayland puro haría falta `wlr-layer-shell`.
- **Online**: leer memoria no modifica el juego, pero ningún método de este tipo tiene garantía
  del publisher. Úsalo bajo tu criterio.

## Licencia

Apache-2.0. Los datos de `data/maps/` son de HunterPie, también Apache-2.0; los detalles de
atribución están en `NOTICE`.
