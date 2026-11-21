# Solución de problemas

Lo que falla casi siempre es el permiso de memoria, el mapa elegido o la superficie del panel,
en ese orden. `mh-overlay --doctor` reúne en un texto la sesión, el permiso, la configuración,
los mapas y el proceso, y es lo primero que hay que mirar.

## «Se adjunta, pero toda lectura falla»

Es el fallo más común en Arch y derivados. `kernel.yama.ptrace_scope=1` impide que un proceso
lea la memoria de otro que no es su hijo, aunque se ejecute como el mismo usuario.

Comprueba el diagnóstico:

```bash
mh-overlay --doctor
```

Tres salidas posibles y su arreglo:

- **`CAP_SYS_PTRACE efectiva: true`** — el binario ya lleva la capacidad; el fallo está en otra
  parte.
- **`CAP_SYS_PTRACE efectiva: false`** con `ptrace_scope: 1` — aplica la capacidad al binario
  instalado (no al AppImage, que es de solo lectura):

  ```bash
  sudo setcap cap_sys_ptrace+ep "$(command -v mh-overlay)"
  ```

  O baja el sysctl mientras juegas:

  ```bash
  sudo sysctl -w kernel.yama.ptrace_scope=0
  ```

  El hub ofrece **Conceder acceso**, que hace lo mismo con la ventana de polkit. El valor vuelve
  a 1 al reiniciar salvo que se cambie en `/etc/sysctl.d/`.

- **`AppImage`** — la capacidad se aplica al archivo que se ejecuta y dentro de la imagen es de
  solo lectura. Usa el sysctl o instala el tarball y aplica `setcap` al binario.

Que el juego corra bajo Proton no cambia nada: para el kernel es un proceso Linux normal.

## «No encuentra el proceso»

1. `mh-overlay --ps` lista los procesos que mencionan o mapean el `.exe`, con su pid, su línea
   de comandos y si tienen el módulo. Si el juego aparece ahí, usa `--pid <pid>`.
2. Proton puede lanzar el juego con una línea de comandos que no nombra el ejecutable; por eso
   la detección también mira el módulo mapeado.
3. Un lanzador de terceros (Heroic, Lutris) puede arrancar el juego en otro contenedor. Busca el
   proceso desde la misma sesión en la que corre el juego.
4. `--game rise` o `--game world` salta la detección por nombre de ejecutable y evita elegir el
   juego equivocado cuando hay dos abiertos.

## «El CRC no coincide»

El aviso `CRC ... no coincide` significa que el `.map` elegido es de otra build. La app sigue
leyendo con el más nuevo, pero los números pueden ser basura plausible.

- Si usas **REFramework** o **Fluffy Manager**, ya modifican el ejecutable en memoria: el CRC
  original no puede coincidir y el aviso es esperado. Compara los números con la pantalla del
  juego antes de fiarte.
- Si el juego se actualizó y no hay mapa nuevo, fuerza el mapa de la build anterior con
  `--map RUTA`. Sirve para salir del paso, no para olvidarse del aviso.
- Un mapa nuevo se añade a `data/maps/` con el número de versión del ejecutable en el nombre; no
  hay que recompilar.

## «No aparecen monstruos»

- En el menú, la aldea o una zona sin monstruos grandes, la lista está vacía a propósito. El
  panel no inventa filas.
- Los mapas de **Rise anteriores a 16.0.2.0** guardan el estado de la cacería con claves
  separadas (`QUEST_TIMER_OFFSETS`, `QUEST_STATUS_OFFSETS`) que el lector no usa todavía: con
  uno forzado se ven jugador y arma, pero no la lista de monstruos. El mapa más nuevo es el que
  trae `QUEST_OFFSETS`.
- Los mapas de **World anteriores a la build 421652** no definen `MONSTER_LIST_ADDRESS`; no hay
  lista que leer y el binario lo dice en vez de inventar direcciones.
- Con `overlay.monsters = "target"` y un mapa sin objetivo, el panel enseña la zona entera; si
  tampoco hay objetivo marcado en el juego, hace lo mismo para no quedarse vacío.

## «El panel no se ve sobre el juego»

- **Wayland**: el panel usa `wlr-layer-shell` en la capa `overlay`. Hyprland, Sway y river lo
  ofrecen; GNOME no lo implementa y ahí cae a una ventana X11, que en pantalla completa puede
  quedar debajo del juego. Mira la salida de stderr y `--doctor`.
- **X11**: el panel es una ventana `override-redirect`; algunos compositores la colocan debajo
  de una ventana fullscreen. Prueba ventana sin bordes en el juego.
- El panel se crea sin output fijo, así que aparece en el monitor activo al abrirlo. Con dos
  pantallas de distinto tamaño, anclar a la derecha o abajo usa el tamaño del primer monitor.
- Si el panel se abre y se cierra solo, revisa el registro en
  `$XDG_STATE_HOME/mh-overlay/` (por defecto `~/.local/state/mh-overlay/`).

## «La actualización no reemplaza el AppImage»

- El actualizador comprueba el SHA-256 de la release y escribe en el mismo archivo. Si está en
  una carpeta de solo lectura, lo dice y no toca nada.
- La configuración vive en `~/.config/mh-overlay` y se conserva; si algo se corrompe, el TOML
  anterior queda como `.bak`.
- Con el tarball, la actualización la hace el gestor de paquetes o `install.sh`; el binario
  avisa de que hay una versión nueva pero no se reemplaza a sí mismo.
- Las releases publican `*.sha256`. Si además hay `*.minisig`, la firma se verifica con la clave
  pública que acompaña a la release:

  ```bash
  minisign -Vm mh-overlay-<versión>-linux-x86_64.AppImage -P <clave-pública>
  ```

## «Un número no cuadra»

1. Abre `mh-overlay --print` junto al juego y compara la misma fila con la pantalla.
2. Si el número es plausible pero estable y equivocado, el sospechoso es el `.map` o un
   desplazamiento fijo; la capa de memoria ya está probada con tests.
3. Si el valor salta entre lecturas, mira si el juego está en pausa, en un menú o cambiando de
   zona: los contenedores se limpian a medias y el lector descarta los punteros ilegibles.
4. Para un informe, adjunta la salida de `--doctor`, el nombre del mapa, el monstruo y las dos
   lecturas (la del panel y la de la pantalla).
