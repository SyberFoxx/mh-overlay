# Roadmap hacia 1.0.0

Estado: 21 de noviembre de 2026. Esta es la lista viva de lo que falta para publicar
`mh-overlay` 1.0.0 como una app estable; el [README](README.md) cuenta lo que ya funciona.

La 1.0 no es «la versión con más funciones», sino la primera que se puede recomendar sin
advertencias: los números de los dos juegos verificados en partida real, instalación y
actualización probadas, y sin sorpresas en el panel ni en el hub.

## Cómo se mantiene

- Cada entrada describe un resultado, no una técnica concreta.
- Al terminarla, se mueve a **Hecho** con la versión o el commit, y se actualiza el README
  si cambia lo que el usuario ve.
- **Ideas** no está aprobado: no se implementa nada de ahí sin pedirlo.

## Criterio de 1.0.0

- [ ] Rise: una cacería completa (vida, partes, estados, arma, buffs) con los números
      cuadrando contra la pantalla del juego durante 30 minutos seguidos.
- [ ] World: lo mismo, en una máquina con el juego instalado.
- [ ] Partes rotas, estados y filo con el comportamiento que se ve en partida (ya corregido,
      pendiente de confirmar en una cacería con la 0.5.3).
- [ ] Instalación limpia del AppImage y del tarball en una máquina o VM sin el proyecto.
- [ ] Autoactualización entre dos releases publicadas, no contra un servidor local.
- [ ] Permiso de memoria concedido desde el hub con polkit, sin abrir una terminal.
- [ ] 30 minutos de juego sin panics, sin lecturas basura y con el diagnóstico limpio.
- [ ] README, capturas, CHANGELOG y artefactos de la release alineados con la etiqueta.
- [ ] Los `.map` soportados están enumerados y probados; ningún juego lee con un mapa que
      no corresponda a su build.

## Lo que falta, por orden de riesgo

### 1. Cerrar la validación en juego (bloquea la 1.0)

- [ ] Rise: cacería completa con la 0.5.3. Confirmar que el filo baja al atacar, que una
      parte agotada muestra **Roto** y que los estados enseñan segundos o lo que falta.
- [ ] Rise: romper una parte con el panel abierto y comprobar que no hay saltos ni un
      «roto» prematuro cuando el hielo regenera la vida de la parte.
- [ ] Confirmar el salto observado en la cabeza de Lunagaron al romperse (2.052 → 4.104 de
      máximo): si es la segunda fase del juego, mostrarlo como tal; si no, es una lectura mal
      atribuida.
- [ ] Rise: una cacería con dos monstruos, captura y cambio de zona.
- [ ] World: repetir todo lo anterior en una máquina con el juego.
- [ ] Probar al menos dos builds o mapas por juego, no solo el más nuevo.
- [ ] Repetir en X11 y en Wayland, y con una sesión de 30 minutos.

**Bloqueo:** World necesita una máquina con el juego instalado. Es la única prueba que no se
puede sustituir con tests; hasta pasarla, el README debe seguir distinguiendo «escrito» de
«probado».

### 2. Panel y hub en Wayland

- [x] El panel ya no cierra la conexión al abrirse sin tamaño (0.5.2).
- [ ] Reiniciar el hub con la 0.5.3 y confirmar que «Abrir panel» deja la ventana en pantalla
      en Hyprland, Sway y un escritorio GNOME (sin layer-shell).
- [ ] El panel vuelve tras ocultarlo y abrirlo varias veces, y al cambiar de monitor.
- [ ] Comprobar que la vista previa del hub y el panel real muestran lo mismo.

**Por qué:** el fallo que quedaba era de ciclo de vida de la superficie, y el arreglo solo
está verificado por tests; falta verlo con el compositor real.

### 3. Permiso de memoria sin terminal

- [ ] Probar «Conceder acceso» con polkit desde una sesión limpia.
- [ ] Decidir si se ofrece restaurar `kernel.yama.ptrace_scope=1` al terminar.
- [ ] Documentar el flujo y sus límites en el README y en el hub.

### 4. Distribución y actualización

- [ ] Autoactualización real entre `v0.5.2` y `v0.5.3` (descarga, reemplazo en sitio,
      reinicio y configuración intacta).
- [ ] Verificar que la release lleva README, LICENSE, NOTICE, AppStream, `.desktop`, iconos
      y las imágenes que enlaza el README.
- [ ] Firmar los artefactos (minisign o GPG) además del SHA-256.
- [ ] Publicar también para `aarch64` si hay usuarios en ARM.
- [ ] Revisar que un AppImage sin `CAP_SYS_PTRACE` explica qué hacer en vez de fallar en
      silencio.

### 5. Calidad y mantenimiento

- [ ] Tests del hub: cierre, bandeja y actualización (navegación, atajos y fondo ya están
      cubiertos sin ventana).
- [ ] Prueba de actualización contra GitHub en CI, sin publicar nada.
- [ ] Un test por lector contra la fixture de cada `.map` soportado, para detectar un cambio
      de offsets al añadir una versión.
- [ ] Revisar los `unwrap`/`expect` que queden y convertir en `Err` los que el juego pueda
      provocar.
- [ ] Repasar avisos y mensajes para que un fallo de lectura diga qué mapa o permiso falta.

### 6. Documentación de la 1.0

- [ ] Capturas reales del hub y del panel, y un GIF corto para el README.
- [ ] Página de solución de problemas: ptrace, Proton, REFramework, mapas y Wayland.
- [ ] Enumerar en el README las builds y mapas probados, con fecha.
- [ ] Notas de la 1.0 con lo que se congela (formato del TOML, CLI y `.map`) y lo que no.

## Después de la 1.0

- Compatibilidad con nuevas builds del juego sin cambiar el binario (mapas en `data/maps/`).
- Actualización más viva: comprobar periódicamente, saltar una versión y enlazar a las notas.
- Restaurar el permiso de memoria desde el hub tras jugar.
- Enviar el AppImage a AppImageHub una vez validado en juego.
- Evaluar un paquete de distro (AUR, Flathub) solo si el proyecto lo pide.

## Ideas (sin aprobar)

- Soporte de Monster Hunter Wilds.
- API de plugins o temas.
- Traducciones distintas del castellano.
- Telemetría; el proyecto es local-first y no la necesita.

No se implementa nada de esta lista sin hablarlo antes.

## Hecho

- **2026-11-21 · 0.5.4**: observado en una cacería real: el filo se muestra en golpes del
  color actual y del siguiente; las partes repetidas se numeran; se elimina el buff fantasma
  de «Llamada de spiribird» y las filas de estado dejan de parpadear con residuos mínimos.
- **2026-11-21 · 0.5.3**: el filo del arma ya baja con los golpes (el relleno era
  `inicio_del_color + unidades`, que se clavaba al llenar un color); las partes agotadas
  muestran **Roto** con la barra vacía en el panel y en `--print`; los estados elementales
  (rayo, hielo, agua, fuego, draco) dejan de mostrarse como porcentaje y los estados reales
  enseñan los segundos restantes o cuánta acumulación falta.
- **2026-11-21 · `v0.5.2`**: el panel moría al abrirse en compositores wlroots; ahora recibe
  tamaño antes del primer commit y el hub explica la espera. El aviso de CRC nombra a
  REFramework/Fluffy.
- **2026-11-21 · `v0.5.1`**: mapa inexistente ya no es un fallo en el menú; `.bak` del TOML;
  CONTRIBUTING, SECURITY y plantillas; Dependabot; runtime `type2` y `appimagetool` fijados
  por etiqueta y SHA-256; CI con `--doctor`; imágenes del README dentro del paquete.
- **2026-11-21 · `v0.5.0`**: release con cuatro artefactos verificados; actualización
  `0.4.0 → 0.5.0` en sitio y configuración conservada; rediseño del hub (tema
  automático/oscuro/claro, fondo ambiental), atajos, diagnóstico copiable, `--doctor`,
  registro con pánico, `config_version` y tamaño de ventana recordado.
- **2026-11-21 · `858a125`, `fa04653`, `57a1917`, `91d398b`**: robustez 0.5.0: registro en
  `$XDG_STATE_HOME` con rotación, hook de pánico, `--doctor`, `config_version = 1` con
  migración y rechazo de esquemas futuros, y pruebas sin ventana.
- **2026-11-21 · `58b4c7f`**: fixtures de memoria; un test por juego recorre vida, partes,
  estados, captura, corona, jugador y cacería sin ptrace ni juego abierto.
- **2026-11-21 · `e2ae463`**: detección por el módulo mapeado y `--ps`.
- **2026-11-20 · `v0.4.0`**: guía de primera ejecución, editor visual a la resolución real,
  arrastre sin reconstruir la vista previa, caché de rectángulos y ajustes responsivos.
- **2026-11-20 · `v0.3.0`**: pestaña **Novedades**, editor visual de barras con imán,
  presets de disposición, `--release-notes`, `CHANGELOG.md` obligatorio y comprobación de
  etiqueta contra `Cargo.toml`.
- **2026-11-20 · `v0.2.0`**: espíritu como espada, barras sueltas con posición propia,
  grupo de monstruo con partes pegadas, umbrales de rompimiento y vida de partes.
- **2026-11-19 · `v0.1.0`**: AppImage y tarball, checksums, autoactualización en sitio, hub,
  bandeja y lectores de World y Rise.
- **`3299753` / `bb79c24`**: banner, wordmark y README de presentación.
- **`0c6ee3a`**: AppStream, iconos 128/256/512, `appimagetool` fijado y CI de empaquetado.
- **`d04564f`**: cierre con Super+Q en Wayland.
