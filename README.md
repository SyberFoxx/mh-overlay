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
| Overlay gráfico (ventana X11) | ✅ escrito | ✅ escrito |
| Configuración en caliente | ✅ probado | ✅ probado |
| Editor de ajustes | ✅ probado | ✅ probado |

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
./target/release/mh-overlay --overlay --game rise
```

```
-g, --game <JUEGO>      rise o world (por defecto, el del archivo de configuración)
    --pid <PID>         pid explícito, si hay más de una instancia corriendo
    --map <ARCHIVO>     address map concreto; por defecto, el más nuevo que coincida con el juego
    --base <HEX>        base del módulo, si /proc no la expone
    --overlay           dibujar el panel encima del juego en una ventana X11
    --settings          abrir el editor de ajustes (no necesita el juego)
    --position <POS>    top-left, top-right, bottom-left, bottom-right o center
    --config <ARCHIVO>  configuración a usar (por defecto ~/.config/mh-overlay/config.toml)
    --init-config       escribir la configuración documentada y salir
    --watch             refresco continuo en lugar de una sola lectura
    --interval <MS>     periodo de refresco; por defecto, el de la configuración
    --list-maps         muestra los address maps disponibles
```

Sin `--overlay` la ejecución imprime una línea por monstruo, una por parte y una por estado
activo; el README abre con un ejemplo de esa salida.

```bash
sudo setcap cap_sys_ptrace+ep ./target/release/mh-overlay
./target/release/mh-overlay --game world --watch --interval 250
```

### El panel

```bash
./target/release/mh-overlay --overlay --game rise
```

El panel es una ventana X11 sin decorar (`override-redirect`), siempre por encima de la
ventana del juego, sin robar el foco y con los clics atravesándola. Por monstruo muestra el
nombre o el id, la vida en porcentaje, una barra con la marca del umbral de captura y, debajo,
las barras finas de partes (azul, rompible; ámbar, cortable) y de estados activos, cada uno con
su color. El redondeo de las esquinas y la región de entrada vacía usan la extensión `SHAPE`;
si el servidor no la tiene, el panel sale rectangular y sigue funcionando.

El tamaño se deriva del contenido y de `overlay.scale`; la posición, de `overlay.position` y
los márgenes. Con `overlay.hide_on_empty` desaparece cuando no hay monstruos grandes en la zona.

Para ver el diseño sin abrir el juego:

```bash
cargo run -p mh-overlay-ui --example preview
```

### Configuración

La primera vez que se ejecuta `--overlay` o `--watch` se escribe una plantilla comentada en
`$XDG_CONFIG_HOME/mh-overlay/config.toml` (o en la ruta de `--config`). El archivo se relee al
guardarlo: cambiar un color o la posición con el juego abierto se aplica en el siguiente
refresco, sin reiniciar.

Los flags de la CLI ganan al archivo, y el archivo gana a los valores por defecto. Un valor sin
sentido (una escala de 12, un juego que no existe, un color mal escrito) es un error con el
campo culpable: al arrancar aborta, y en caliente se descarta el archivo nuevo y se sigue con el
anterior, porque cerrar el overlay en mitad de una cacería es peor que quedarse con el último
estado bueno.

```toml
[overlay]
position = "top-left"
margin_x = 24
margin_y = 24
scale = 1.0
max_monsters = 4
show_parts = true
show_ailments = true
click_through = true
hide_on_empty = true

[style]
panel = "#0b0d12e6"
health = "#7fd47f"
low_health = "#e2565c"
low_health_percent = 25.0
capture = "#e6b45e"
part = "#4f8dd6"
severable = "#d69a4f"
```

Los colores admiten `#rgb`, `#rrggbb` y `#rrggbbaa`. El alfa se compone sobre negro antes de
subir la imagen, así que el panel se ve igual con o sin compositor.

### Editor de ajustes

```bash
./target/release/mh-overlay --settings
```

Una ventana normal, con ratón y teclado, que edita lo que más se toca a mano: posición,
márgenes, escala, monstruos a la vez, refresco y los cuatro interruptores (partes, estados,
clics que atraviesan, ocultar sin monstruos). Cada cambio se guarda en el TOML al instante, así
que un overlay que esté corriendo lo aplica en el siguiente refresco sin reiniciar nada. Con el
ratón se pulsa `-`/`+` o el interruptor; con el teclado, `arriba`/`abajo` eligen fila,
`izquierda`/`derecha` cambian el valor y `Esc` cierra.

Los colores no están en el editor: son cinco selectores hexadecimales que se tocan una vez y se
olvidan, y meterlos en la misma ventana sin un selector de color de verdad habría sido peor que
editarlos en el TOML.

## Arquitectura

```
crates/
├── mh-overlay-memory   proceso, /proc, process_vm_readv, cadenas de punteros, parser .map
├── mh-overlay-games    rise.rs, world.rs, modelo compartido
├── mh-overlay-config   TOML documentado, validación, recarga y ruta XDG
├── mh-overlay-ui       lienzo, tipografía y ventana X11 con SHAPE
└── mh-overlay-cli      binario mh-overlay
data/maps/              66 address maps por versión de juego (ver NOTICE)
```

El dibujo está separado de X a propósito: `mh-overlay-ui::view` produce una imagen a partir
del estado del juego y `window` solo la sube, así que la maquetación se prueba con tests sin
servidor gráfico.

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
son la parte cara del problema y ya están resueltas para 16 builds de World, 19 de Rise y 31 de
Wilds (Wilds aún sin lector).

Un mapa y una build no son intercambiables: usar el archivo más nuevo contra un juego viejo no
falla, devuelve basura plausible. Por eso el binario elige el mapa más nuevo cuyo par
`Address CRC_FUNC_*` + `Instruction CRC_ORIGINAL_*` coincide byte a byte con la memoria del
juego, y avisa por stderr de la versión elegida. Un mapa que no trae CRC no se puede verificar;
`--map` sigue permitiendo forzar uno. Los números de los mapas son hexadecimales aunque les
falte el `0x`, igual que en HunterPie.

Si actualizas un mapa a mano, sube el archivo con el número de versión del ejecutable en el
nombre. `--map` permite probar uno sin tocar los demás.

## Límites conocidos

- **Solo lectura.** No hay escritura ni inyección; el crate no expone un writer a propósito.
- **Wilds no está soportado** todavía, aunque sus mapas están incluidos.
- **Los nombres de monstruo y de parte no están mapeados**: se muestran ids. Las traducciones
  de HunterPie se descargan en runtime y no están en el repositorio.
- **World reporta las partes sin nombre** porque su tabla de partes depende de datos por
  monstruo que aún no se han portado. Las partes cortables solo muestran la barra de corte.
- **World solo expone la lista de monstruos desde la build 421652**; los mapas anteriores no
  definen `MONSTER_LIST_ADDRESS` y el binario lo dice en vez de inventar direcciones.
- **Wayland nativo**: el panel es una ventana X11, que funciona sobre juegos de Proton (que ya
  son XWayland). En Wayland puro haría falta `wlr-layer-shell` y no está soportado.
- **El panel es opaco**: la translucidez del TOML se compone sobre negro en vez de dejar ver el
  juego a través, porque debajo de un juego a pantalla completa no hay compositor garantizado.
- **Los colores se editan solo en el TOML**: el editor cubre posición, tamaño y visibilidad.
- **Online**: leer memoria no modifica el juego, pero ningún método de este tipo tiene garantía
  del publisher. Úsalo bajo tu criterio.

## Licencia

Apache-2.0. Los datos de `data/maps/` son de HunterPie, también Apache-2.0, y la tipografía
Rajdhani es SIL OFL 1.1; los detalles de atribución están en `NOTICE`.
