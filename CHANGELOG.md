# Changelog

Todos los cambios visibles para quien usa `mh-overlay`. El workflow de release usa la sección
de la versión etiquetada como cuerpo de la release de GitHub y falla si no existe.

## [0.3.0] - 2026-11-20

### Añadido

- Pestaña **Novedades** en el hub: versión instalada, última publicada, notas de la release
  desde GitHub, enlace a la release y botón para actualizar o reintentar la consulta.
- Editor visual de barras: arrastra cada placa sobre una pantalla esquemática; se imanta a la
  rejilla de 10 px, al centro y a los bordes. `monster.N` mueve vida, partes y estados juntos.
- Presets de disposición: dos integrados (compacto y monstruo a la derecha), guardar la
  disposición actual con nombre, aplicar y eliminar. Se guardan en `[[overlay.presets]]`.
- `--release-notes` en la CLI: imprime la etiqueta, versión, fecha y cuerpo de la última release.

### Cambiado

- Las releases ya no usan las notas automáticas de GitHub: el workflow exige una sección en
  `CHANGELOG.md` para la etiqueta y publica ese texto.
- El workflow de release comprueba que la etiqueta `v*` coincide con la versión de `Cargo.toml`
  antes de compilar o publicar nada.

## [0.2.0] - 2026-11-20

### Añadido

- Modo de barras sueltas: una placa por barra, con posición y orientación propias.
- `monster.N` agrupa la vida del monstruo con sus partes y estados; `direction` elige columna
  debajo o a la derecha.
- Medidor de espíritu como espada, con color por nivel.
- Umbrales de rompimiento y vida de las partes en números.

## [0.1.1] - 2026-11-19

### Arreglado

- Cierre con Super+Q en Wayland.
- Integración de escritorio y metadatos del AppImage.

## [0.1.0] - 2026-11-19

### Añadido

- AppImage autoactualizable y tarball con checksums.
- Hub gráfico, bandeja de sistema y lectores de Monster Hunter World y Rise.
