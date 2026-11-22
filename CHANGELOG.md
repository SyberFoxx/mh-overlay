# Changelog

Todos los cambios visibles para quien usa `mh-overlay`. El workflow de release usa la sección
de la versión etiquetada como cuerpo de la release de GitHub y falla si no existe.

## [0.7.0] - 2026-11-22

### Añadido

- Icono nuevo: una placa dorada con dos cuernos y tres zarpazos, vectorial, en 128, 256 y 512 px
  más el SVG. Se ve igual de bien a 16 px que a 512.
- Dos formas de barra más para el panel, además de la redondeada: **Tramos** (recta, plana y
  partida cada 10 %) y **Cápsula** (muy redondeada, sin marcas). Se eligen en **Ajustes** y se
  aplican a la vida del monstruo y a las barras de partes y estados.

## [0.6.0] - 2026-11-22

### Añadido

- Cuatro fondos para el hub, generados al dibujar y con opacidad propia: **Aurora**, **Brasa**,
  **Niebla** y el ambiente de siempre, además de «sin fondo». Se eligen desde **Ajustes**.
- Tres presets de color del panel —Oscuro, Claro y Alto contraste— que reemplazan `[style]` de
  una vez; los ocho colores siguen editándose uno a uno desde el hub.
- El actualizador verifica la firma minisign de la release en proceso, sin necesidad de tener
  `minisign` instalado. El binario lleva compilada la clave pública del proyecto, así que un
  paquete sin `.minisig` o con una firma que no corresponda no se instala aunque su SHA-256
  cuadre, que es el caso que el SHA-256 solo no cubre: quien puede publicar la release puede
  publicar también su checksum. `--doctor` avisa si la clave compilada falta o no se puede leer.
- La clave pública se adjunta a cada release (`packaging/minisign.pub`) para poder verificar a
  mano lo que el actualizador verifica solo.
- Firma opcional de los artefactos con minisign: si el repositorio define
  `MINISIGN_SECRET_KEY`, la release publica un `.minisig` por AppImage y tarball. Sin ese
  secreto la release sigue saliendo con su SHA-256, igual que antes.
- [Guía de solución de problemas](docs/TROUBLESHOOTING.md): permiso de memoria, proceso que no
  aparece, CRC de otra build, mapas viejos, panel invisible y actualización.
- El README enumera las builds probadas y su fecha, y los mapas que usan el esquema de claves
  que lee el binario.

### Mejorado

- El hub deja de cortar y superponer texto: los ajustes ocupan el ancho completo y la vista
  previa pasa al final cuando la ventana no da para dos columnas, las cifras del diagnóstico son
  cuatro cajas del mismo tamaño, las viñetas de las notas alinean las líneas con el texto y la
  navegación recorta con puntos suspensivos en vez de pisar el borde.
- La release comprueba formato, avisos y tests antes de empaquetar, y verifica cada artefacto
  firmado con la clave del proyecto antes de publicarlo.
- La carpeta temporal de la actualización es aleatoria y privada (0700, en `$XDG_RUNTIME_DIR`
  cuando existe), y si falla el reemplazo de los mapas se restaura la carpeta anterior.
- `--doctor` informa si el binario lleva la clave pública del proyecto y por tanto puede verificar
  la firma de las actualizaciones, o si se quedó en el SHA-256.
- Los tests recorren cada `.map` de Rise y World que tiene las claves del lector, no solo el más
  nuevo: un mapa añadido que pierda `QUEST_OFFSETS` o `MONSTER_LIST_ADDRESS` falla al instante.
- El diagnóstico del AppImage en CI exige que `--doctor` nombre `CAP_SYS_PTRACE`, para que el
  caso «se adjunta pero no lee» nunca quede sin pista.

## [0.5.4] - 2026-11-21

### Corregido

- «Llamada de spiribird» aparecía siempre con 60 s eternos: el lector trataba el cero de un
  estado inactivo como «acaba de empezar». Los estados que guardan el tiempo transcurrido ya
  no se muestran hasta que ese tiempo avanza.
- El agotamiento y otros estados entraban y salían de la lista con residuos de una o dos
  unidades, y con cada aparición cambiaba la altura del panel. Ahora hace falta un 1 % de
  acumulación, o un temporizador activo.

### Mejorado

- El filo se lee en golpes: los que quedan del color actual y los que dura el siguiente
  (`filo 23 · azul · luego blanco 10`), en el panel y en `--print`.
- Las partes con el mismo nombre se numeran para no confundirlas (`Cabeza 1`, `Cabeza 2`,
  `Pata trasera 1`, `Pata trasera 2`).

## [0.5.3] - 2026-11-21

### Corregido

- El filo del arma nunca bajaba: el relleno sumaba el inicio del color a las unidades que
  quedaban, así que al gastar un color entero la barra seguía llena. Ahora es
  `unidades restantes / total`, de modo que el filo se ve bajar con los golpes.
- Una parte agotada mostraba la vida restante o el contador de rompimientos como si aún
  tuviera algo que romper. Ahora pone **Roto** con la barra vacía, tanto en el panel como en
  `--print`, y deja claro que ya no hay vida que leer.
- Los estados elementales (rayo, hielo, agua, fuego, draco) aparecían como porcentaje de
  acumulación. No son estados que el monstruo sufra, así que se quitan de las filas de
  estados; las acumulaciones de daño elemental no se muestran.

### Mejorado

- Los estados que sí importan (veneno, sueño, parálisis, aturdimiento, agotamiento) enseñan
  los segundos que les quedan cuando están activos, o cuánta acumulación falta para
  provocarlos cuando aún no lo están.

## [0.5.2] - 2026-11-21

### Corregido

- El panel moría al abrirse en compositores wlroots (Hyprland, Sway): la capa se confirmaba sin
  tamaño antes del primer fotograma y el compositor cerraba la conexión. Ahora se le da un
  tamaño mínimo y el panel se queda en espera hasta que hay algo que enseñar.
- El hub explica esa espera («el panel aparece al entrar en una cacería») en vez de parecer que
  «Abrir panel» no hace nada, y el panel anota en el registro cuándo se oculta y cuándo vuelve.

## [0.5.1] - 2026-11-21

### Corregido

- Leer los monstruos dejaba de ser un error cuando el juego está en el menú o la zona aún se
  está cargando: la lista no existe todavía y el panel mostraba un fallo en rojo en vez de
  «sin monstruos». En Rise tampoco se escanea la lista si la cacería no está en curso.
- El aviso de CRC desajustado explica también la causa más habitual: un mod como REFramework
  parchea el módulo, así que los CRC de Capcom ya no cuadran aunque la versión sea la correcta.

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
