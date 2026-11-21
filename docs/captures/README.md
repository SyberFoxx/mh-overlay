# Capturas para el README

Esta carpeta guarda las capturas reales que enlaza el README. Todavía no hay ninguna: las que
se usan ahora son el banner y el título de `docs/`. Cuando estén, se insertan en la sección
**Capturas** del README con estos nombres.

## Qué hace falta

| Archivo | Encuadre | Qué tiene que verse |
| --- | --- | --- |
| `panel-hunt.png` | Panel sobre el juego en una cacería normal | Vida del monstruo, partes con barra, estados y arma con filo |
| `panel-parts.png` | El mismo panel con una parte rota | Una fila **Roto** con la barra vacía y, si hay, un estado con segundos |
| `hub-diagnostics.png` | Hub, pestaña **Diagnóstico** | Juego detectado, pid, base, mapa y resultado del CRC |
| `hub-settings.png` | Hub, **Ajustes** o editor visual | Barras, posición y tamaño del panel |
| `hub-preview.png` | Hub, vista previa | La misma disposición que el panel real |
| `tray.png` | Bandeja del sistema | Icono y menú de mh-overlay |

## Cómo tomarlas

- **Wayland (Hyprland, Sway, river):** `grim docs/captures/panel-hunt.png` captura el monitor
  entero, incluidas las capas `overlay`. Para recortar, `grim -g "$(slurp)" archivo.png`.
- **X11:** `maim -s archivo.png` o `spectacle -r`.
- **GNOME:** `gnome-screenshot -a` para una zona; el panel X11 se captura como una ventana más.
- **Hub:** la ventana es normal; basta con la captura de la ventana.

Guarda PNG sin escalar (el README los muestra a 900 px de ancho). Si la captura pesa más de
~1 MB, recórtala al panel o reduce a 1600 px de ancho; no la conviertas a JPEG para texto.

## Cuando estén

Con los archivos en esta carpeta, la sección del README queda así:

```markdown
## Capturas

| Panel en cacería | Hub, diagnóstico |
| --- | --- |
| ![Panel durante una cacería](docs/captures/panel-hunt.png) | ![Diagnóstico del hub](docs/captures/hub-diagnostics.png) |

![Partes y estados](docs/captures/panel-parts.png)
```

Las imágenes también entran en el AppImage: `scripts/appimage.sh` copia `docs/` a
`usr/share/doc/mh-overlay/docs/`, así que el README que va dentro no las verá rotas si se añaden
antes de empaquetar.
