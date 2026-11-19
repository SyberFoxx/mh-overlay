#!/usr/bin/env bash
# Empaqueta mh-overlay como AppImage: un solo archivo ejecutable que lleva el binario, los mapas
# y el lanzador dentro. Al ejecutarse se integra en el menú del usuario y `--update` reemplaza
# ese mismo archivo, así que actualizar no obliga a desinstalar nada.
#
# El AppDir se arma con la disposición estándar (`usr/bin`, `usr/share`). El binario busca los
# mapas por `MH_OVERLAY_MAPS`, que fija `AppRun`, para no depender de la ruta desde la que se
# monte la imagen.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
machine=$(uname -m)
name="mh-overlay-$version-linux-$machine"
appdir="dist/$name.AppDir"
appimage="dist/$name.AppImage"
tool="target/appimage/appimagetool-$machine.AppImage"

if [ "${MH_OVERLAY_SKIP_BUILD:-0}" != "1" ]; then
    echo "compilando $version para $machine"
    cargo build --release --locked
fi

echo "preparando $appdir"
rm -rf "$appdir"
mkdir -p \
    "$appdir/usr/bin" \
    "$appdir/usr/share/mh-overlay/data" \
    "$appdir/usr/share/applications" \
    "$appdir/usr/share/icons/hicolor/scalable/apps" \
    "$appdir/usr/share/icons/hicolor/256x256/apps" \
    "$appdir/usr/share/doc/mh-overlay"

install -m755 target/release/mh-overlay "$appdir/usr/bin/mh-overlay"
cp -r data/maps "$appdir/usr/share/mh-overlay/data/maps"
install -m644 README.md LICENSE NOTICE "$appdir/usr/share/doc/mh-overlay/"
install -m644 packaging/mh-overlay.svg "$appdir/mh-overlay.svg"
install -m644 packaging/mh-overlay.png "$appdir/mh-overlay.png"
install -m644 packaging/mh-overlay.svg "$appdir/usr/share/icons/hicolor/scalable/apps/mh-overlay.svg"
install -m644 packaging/mh-overlay.png \
    "$appdir/usr/share/icons/hicolor/256x256/apps/mh-overlay.png"
install -m644 packaging/mh-overlay-appimage.desktop "$appdir/mh-overlay.desktop"
install -m644 packaging/mh-overlay-appimage.desktop \
    "$appdir/usr/share/applications/mh-overlay.desktop"

# `AppRun` es el punto de entrada que el runtime ejecuta al abrir la imagen. Además de fijar
# los mapas, deja el lanzador del menú apuntando al AppImage actual: si el archivo se mueve o se
# actualiza, la siguiente ejecución corrige la entrada sin que el usuario haga nada.
cat >"$appdir/AppRun" <<'APPRUN'
#!/bin/sh
set -eu

here=$(dirname "$(readlink -f "$0")")
export MH_OVERLAY_MAPS="$here/usr/share/mh-overlay/data/maps"

if [ -n "${APPIMAGE:-}" ] && [ -n "${HOME:-}" ]; then
    data=${XDG_DATA_HOME:-$HOME/.local/share}
    mkdir -p "$data/applications" \
        "$data/icons/hicolor/scalable/apps" \
        "$data/icons/hicolor/256x256/apps"

    desktop="$data/applications/mh-overlay.desktop"
    {
        printf '%s\n' \
            '[Desktop Entry]' \
            'Type=Application' \
            'Version=1.0' \
            'Name=mh-overlay' \
            'GenericName=Monster Hunter Overlay' \
            'Comment=HUD de Monster Hunter: vida, partes, estados, arma y buffs' \
            "Exec=\"$APPIMAGE\"" \
            'Icon=mh-overlay' \
            'Terminal=false' \
            'StartupNotify=true' \
            'Categories=Game;' \
            'Keywords=monster hunter;overlay;hud;world;rise;hunterpie;' \
            'Actions=Overlay;Diagnostics;' \
            '' \
            '[Desktop Action Overlay]' \
            'Name=Abrir panel sobre el juego' \
            "Exec=\"$APPIMAGE\" --overlay" \
            '' \
            '[Desktop Action Diagnostics]' \
            'Name=Diagnóstico y ajustes' \
            "Exec=\"$APPIMAGE\" --hub"
    } >"$desktop.tmp"
    mv "$desktop.tmp" "$desktop"

    install -m644 "$here/mh-overlay.svg" \
        "$data/icons/hicolor/scalable/apps/mh-overlay.svg"
    install -m644 "$here/mh-overlay.png" \
        "$data/icons/hicolor/256x256/apps/mh-overlay.png"
fi

exec "$here/usr/bin/mh-overlay" "$@"
APPRUN
chmod 755 "$appdir/AppRun"

# `appimagetool` es en sí un AppImage. Se cachea en target/ y se ejecuta con
# `--appimage-extract-and-run` para no exigir FUSE en la máquina que empaqueta.
if [ ! -x "$tool" ]; then
    echo "descargando appimagetool para $machine"
    mkdir -p "$(dirname "$tool")"
    curl -fL --retry 3 -o "$tool" \
        "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$machine.AppImage"
    chmod +x "$tool"
fi

echo "generando $appimage"
rm -f "$appimage"
ARCH="$machine" "$tool" --appimage-extract-and-run --no-appstream "$appdir" "$appimage"

sha256sum "$appimage" | sed "s|dist/||" >"$appimage.sha256"

echo
echo "$appimage"
cat "$appimage.sha256"
echo
echo "para probar: ./$appimage --version"
echo "para usar:   chmod +x $appimage && ./$appimage"
