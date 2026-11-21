# Changelog

Todos los cambios visibles para quien usa `mh-overlay`. El workflow de release usa la sección
de la versión etiquetada como cuerpo de la release de GitHub y falla si no existe.

## [Unreleased]

### Añadido

- Copia de la configuración anterior en `config.toml.bak` cada vez que se guarda, para poder
  volver atrás después de un cambio a medias.
- `CONTRIBUTING.md`, `SECURITY.md`, plantillas de issue y PR, y una política de estabilidad que
  dice qué se congela en 1.0 y qué no.
- Dependabot para las dependencias de Cargo y las acciones del CI.
- El runtime del AppImage se fija por etiqueta y SHA-256, como `appimagetool`: dos builds del
  mismo commit dejan de depender de lo que haya publicado upstream esa semana.

### Corregido

- El tarball y el AppImage incluyen las imágenes que enlaza el README; antes quien abría el
  README del paquete veía dos imágenes rotas.

## [0.5.0] - 2026-11-21

### Añadido

- Rediseño del hub con tokens de color por papel: tarjetas, navegación y controles nuevos, y
  tema **Automático/Oscuro/Claro** que sigue al escritorio y se cambia en caliente desde
  **Ajustes → Interfaz**.
- Fondo ambiental de la ventana, generado al dibujar (dos focos sobre el color base), con
  intensidad configurable y opción **Sin fondo**. No añade imágenes ni dependencias y escala
  con la ventana.
- Atajos en el hub: **Ctrl+1** a **Ctrl+4** cambian de pestaña y **Esc** cierra la guía o la
  confirmación.
- **Copiar diagnóstico** en la pestaña Diagnóstico: versión, juego, proceso, base, mapa, CRC y
  último error en el portapapeles. **Buscar procesos** enseña lo que ve la detección —pid,
  línea de comandos y módulo mapeado— sin abrir una terminal.
- `--doctor`: informe de sesión, permisos, configuración, mapas, procesos y registro, pensado
  para pegarlo en un informe de fallo.
- Registro del hub en `$XDG_STATE_HOME/mh-overlay/hub.log`, con rotación a 256 KiB y, ante un
  pánico, su mensaje y la pila.
- `config_version` en el TOML: un archivo anterior al versionado se migra al leerlo y uno de
  una versión futura se rechaza diciéndolo en vez de fallar por una clave desconocida.
- El hub recuerda el tamaño de su ventana entre sesiones.
- `--ps`, para listar los procesos que mencionan o mapean el ejecutable del juego cuando la
  detección automática no lo encuentra.
- Pruebas de los lectores contra una imagen de memoria por juego, sin juego abierto, y pruebas
  del hub sin ventana: atajos, fondo, estados de validación y tamaño de ventana recordado.

### Cambiado

- La detección del juego ya no depende del nombre del proceso: tolera `comm` truncado a 15
  bytes, la ausencia de `.exe` y busca el módulo mapeado cuando Proton no nombra el ejecutable
  en la línea de comandos.
- Escondido en la bandeja el hub no lee ni repinta (latido de un segundo), y la vista previa
  solo se rehace en Ajustes cuando algo cambia.
- Leer una lista de punteros o de `f32` cuesta una sola llamada al kernel: 32 991 ns a 553 ns
  por array de 64 en la medida del proyecto.

## [0.4.0] - 2026-11-20

### Añadido

- Guía de primera ejecución en el hub: elegir o detectar el juego, explicar y conceder el
  permiso de memoria con polkit, comprobar la conexión y abrir el panel. Se puede saltar y
  repetir desde **Ajustes → Guía**.
- El editor visual usa la resolución real del monitor, con ancho y alto manuales cuando la
  detección de Wayland o de varias pantallas no coincide.

### Cambiado

- Arrastrar una placa ya no reconstruye la vista previa en cada frame: se aplaza hasta soltar
  el ratón y se reutilizan los rectángulos del editor mientras dura el arrastre.
- Cada pestaña conserva su propio scroll y vuelve arriba al abrirse; las filas de controles se
  envuelven en ventanas estrechas y la vista previa ocupa menos.
- Las notas de **Novedades** ya no muestran los asteriscos ni las comillas invertidas del
  markdown.

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
