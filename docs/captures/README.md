# Capturas para el README

Capturas reales del hub, tomadas con `grim` sobre Hyprland con la ventana a 1880×944.

| Archivo | Qué se ve |
| --- | --- |
| `hub-diagnostico.png` | Pestaña **Diagnóstico**: juego, proceso, mapa y resumen de lectura |
| `hub-novedades.png` | Pestaña **Novedades**: notas de la última release con viñetas |
| `hub-ajustes.png` | Pestaña **Ajustes** con la vista previa del panel al lado |

## Qué falta

| Archivo | Encuadre | Por qué no está |
| --- | --- | --- |
| `panel-hunt.png` | Panel sobre el juego en una cacería | Hace falta el juego corriendo bajo Proton |
| `panel-parts.png` | El mismo panel con una parte rota | Lo mismo |
| `tray.png` | Bandeja del sistema | Necesita una sesión con StatusNotifierItem |

## Cómo tomarlas

- **Wayland (Hyprland, Sway, river):** `grim docs/captures/panel-hunt.png` captura el monitor
  entero, incluidas las capas `overlay`. Para recortar, `grim -g "$(slurp)" archivo.png`.
- **X11:** `maim -s archivo.png` o `spectacle -r`.
- **GNOME:** `gnome-screenshot -a` para una zona; el panel X11 se captura como una ventana más.
- **Hub:** es una ventana normal; basta con capturarla por pid:

  ```bash
  grim -g "$(hyprctl -j clients | jq -r '.[] | select(.title|test("hub")) | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')" docs/captures/hub-diagnostico.png
  ```

Guarda PNG sin escalar (el README los muestra a 900 px de ancho). Si pesa más de ~1 MB, recorta o
reduce a 1600 px de ancho; no lo conviertas a JPEG, el texto fino se ensucia.

## Dentro del paquete

`scripts/package.sh` copia `docs/banner.jpeg` y `docs/title.png` al tarball, que son las que
enlaza el README que viaja dentro. Si se añaden capturas al cuerpo del README, hay que sumarlas
ahí para que no aparezcan rotas.
