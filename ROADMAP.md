# Roadmap

Estado: 20 de noviembre de 2026. Esta es la lista viva de lo que falta para dar
`mh-overlay` por terminado; el [README](README.md) cuenta lo que ya funciona.

## Cómo se mantiene

- Cada entrada describe un resultado, no una técnica concreta.
- Al terminarla, se mueve a **Hecho** con la versión o el commit, y se actualiza el README
  si cambia lo que el usuario ve.
- **Ideas** no está aprobado: no se implementa nada de ahí sin pedirlo.

## Criterio de «completo»

- [ ] Una cacería real en World y otra en Rise con vida, partes, estados, arma y buffs correctos.
- [ ] Instalación limpia del AppImage y del tarball en una máquina o VM sin el proyecto.
- [ ] Permiso de memoria concedido desde el hub con polkit, sin abrir una terminal.
- [ ] Actualización real `v0.3.0 → v0.4.0`, reinicio y configuración conservada.
- [ ] 30 minutos de juego sin panics ni lecturas basura, con el diagnóstico limpio.
- [ ] README y release alineados: misma versión, mismos artefactos y mismas capturas.

## Ahora

### 1. Probar la actualización real de `v0.4.0`

- [x] Actualización `v0.2.0 → v0.3.0` probada, reinicio y configuración conservada.
- [x] Subir la versión del workspace a `0.4.0`.
- [x] Publicar las notas desde `CHANGELOG.md` y comprobar la etiqueta contra `Cargo.toml` en CI.
- [ ] Workflow `v0.4.0` y comprobar que publica release, notas y cuatro artefactos.
- [ ] Abrir el AppImage `v0.3.0`, detectar `v0.4.0`, instalar, reiniciar y verificar que la
      configuración sigue intacta.
- [ ] Confirmar que la release lleva el README y el banner nuevos, AppStream y los iconos.

**Por qué:** la autoactualización solo se probó contra un servidor local con una versión
falsa, y los artefactos de `v0.1.0` son anteriores al acabado de distribución y al arreglo
de cierre en Wayland.

### 2. Validar los lectores con el juego real

- [ ] World: una cacería con monstruo, partes rompibles, estados y arma.
- [ ] Rise: lo mismo, incluyendo ataque, afinidad y elemento.
- [ ] Probar al menos dos builds o mapas por juego, no solo el más nuevo.
- [ ] Repetir en X11 y en Wayland, y con una sesión de 30 minutos.

**Bloqueo:** hace falta una máquina con los juegos instalados. Es la única prueba que no se
puede sustituir con tests; hasta pasarla, el README debe seguir diciendo «escrito», no «probado».

### 3. Permiso de memoria sin terminal

- [ ] Probar «Conceder acceso» con polkit desde una sesión limpia.
- [ ] Decidir si se ofrece restaurar `kernel.yama.ptrace_scope=1` al terminar.
- [ ] Documentar el flujo y sus límites en el README y en el hub.

### 4. Reproducibilidad del AppImage

- [ ] Fijar también el runtime `type2` que descarga `appimagetool` (versión y SHA-256), o
      documentar por qué se deja en `continuous`.

`appimagetool` 1.9.1 ya está fijado y verificado; el runtime es la pieza que falta.

## Después

### Producto

- [ ] Exportar diagnóstico: proceso, mapa, CRC, último error y versión en un solo botón.
- [ ] Registro a archivo en `$XDG_STATE_HOME/mh-overlay/` para depurar cierres.
- [ ] Actualización más viva: comprobar periódicamente, saltar una versión y enlazar a las notas.
- [ ] Migración de configuración cuando cambie el esquema del TOML.
- [ ] Restaurar el permiso de memoria desde el hub tras jugar.

### Distribución

- [ ] Publicar también para `aarch64` si hay usuarios en ARM.
- [ ] Firma minisign o GPG de los artefactos, además del SHA-256.
- [ ] Enviar el AppImage a AppImageHub una vez validado en juego.
- [ ] Evaluar un paquete de distro (AUR, Flathub) solo si el proyecto lo pide.
- [ ] Capturas reales del hub y del panel, y un GIF corto para el README.

### Comunidad

- [ ] `CONTRIBUTING.md`, `SECURITY.md` y plantillas de issue/PR.

### Calidad

- [ ] Fixtures de memoria para probar los lectores sin el juego.
- [ ] Tests del hub: cierre, bandeja, actualización y navegación.
- [ ] Prueba de actualización contra GitHub en CI, sin publicar nada.

## Ideas (sin aprobar)

- Soporte de Monster Hunter Wilds.
- API de plugins o temas.
- Traducciones distintas del castellano.
- Telemetría; el proyecto es local-first y no la necesita.

No se implementa nada de esta lista sin hablarlo antes.

## Hecho

- **2026-11-20 · `v0.4.0`**: guía de primera ejecución, editor visual a la resolución real del
  monitor, arrastre sin reconstruir la vista previa en cada frame, caché de rectángulos y
  ajustes responsivos.
- **2026-11-20 · `v0.3.0`**: pestaña **Novedades** con las notas de GitHub, editor visual de
  barras con imán, presets de disposición, `--release-notes`, `CHANGELOG.md` obligatorio y
  comprobación de etiqueta contra `Cargo.toml` antes de publicar.
- **2026-11-20 · `v0.2.0`**: medidor de espíritu como espada, barras sueltas con posición
  propia y grupo de monstruo con partes pegadas, umbrales de rompimiento y vida de partes.
- **2026-11-19 · `v0.1.0`**: AppImage y tarball, checksums, autoactualización en sitio, hub,
  bandeja y lectores de World y Rise.
- **`3299753` / `bb79c24`**: banner, wordmark y README de presentación.
- **`0c6ee3a`**: AppStream, iconos 128/256/512, `appimagetool` fijado y CI de empaquetado.
- **`d04564f`**: cierre con Super+Q en Wayland (pendiente de la próxima release).
