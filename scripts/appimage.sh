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
    "$appdir/usr/share/metainfo" \
    "$appdir/usr/share/icons/hicolor/scalable/apps" \
    "$appdir/usr/share/icons/hicolor/128x128/apps" \
    "$appdir/usr/share/icons/hicolor/256x256/apps" \
    "$appdir/usr/share/icons/hicolor/512x512/apps" \
    "$appdir/usr/share/doc/mh-overlay"

install -m755 target/release/mh-overlay "$appdir/usr/bin/mh-overlay"
cp -r data/maps "$appdir/usr/share/mh-overlay/data/maps"
install -m644 README.md LICENSE NOTICE "$appdir/usr/share/doc/mh-overlay/"
install -m644 packaging/mh-overlay.svg "$appdir/mh-overlay.svg"
install -m644 packaging/mh-overlay.png "$appdir/mh-overlay.png"
install -m644 packaging/mh-overlay.svg "$appdir/usr/share/icons/hicolor/scalable/apps/mh-overlay.svg"
install -m644 packaging/mh-overlay-128.png \
    "$appdir/usr/share/icons/hicolor/128x128/apps/mh-overlay.png"
install -m644 packaging/mh-overlay.png \
    "$appdir/usr/share/icons/hicolor/256x256/apps/mh-overlay.png"
install -m644 packaging/mh-overlay-512.png \
    "$appdir/usr/share/icons/hicolor/512x512/apps/mh-overlay.png"
install -m644 packaging/io.github.syberfoxx.mh_overlay.appdata.xml \
    "$appdir/usr/share/metainfo/io.github.syberfoxx.mh_overlay.appdata.xml"
install -m644 packaging/mh-overlay-appimage.desktop \
    "$appdir/io.github.syberfoxx.mh_overlay.desktop"
install -m644 packaging/mh-overlay-appimage.desktop \
    "$appdir/usr/share/applications/io.github.syberfoxx.mh_overlay.desktop"

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
    mkdir -p "$data/applications" "$data/metainfo" \
        "$data/icons/hicolor/scalable/apps" \
        "$data/icons/hicolor/128x128/apps" \
        "$data/icons/hicolor/256x256/apps" \
        "$data/icons/hicolor/512x512/apps"

    desktop="$data/applications/io.github.syberfoxx.mh_overlay.desktop"
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
    # Instalaciones anteriores usaban el nombre corto: no dejar dos entradas de menú.
    rm -f "$data/applications/mh-overlay.desktop"

    install -m644 "$here/mh-overlay.svg" \
        "$data/icons/hicolor/scalable/apps/mh-overlay.svg"
    install -m644 "$here/usr/share/icons/hicolor/128x128/apps/mh-overlay.png" \
        "$data/icons/hicolor/128x128/apps/mh-overlay.png"
    install -m644 "$here/usr/share/icons/hicolor/256x256/apps/mh-overlay.png" \
        "$data/icons/hicolor/256x256/apps/mh-overlay.png"
    install -m644 "$here/usr/share/icons/hicolor/512x512/apps/mh-overlay.png" \
        "$data/icons/hicolor/512x512/apps/mh-overlay.png"
    install -m644 "$here/usr/share/metainfo/io.github.syberfoxx.mh_overlay.appdata.xml" \
        "$data/metainfo/io.github.syberfoxx.mh_overlay.appdata.xml"
fi

exec "$here/usr/bin/mh-overlay" "$@"
APPRUN
chmod 755 "$appdir/AppRun"

# `appimagetool` es en sí un AppImage. Se fija la versión y se verifica su SHA-256: una
# release no debe cambiar porque upstream reemplazó el binario. Se ejecuta con
# `--appimage-extract-and-run` para no exigir FUSE en la máquina que empaqueta.
appimagetool_version="1.9.1"
case "$machine" in
    x86_64) appimagetool_sha256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0" ;;
    aarch64) appimagetool_sha256="f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158" ;;
    *)
        echo "error: no hay appimagetool verificado para $machine" >&2
        exit 1
        ;;
esac
tool="target/appimage/appimagetool-$appimagetool_version-$machine.AppImage"

tool_ok() {
    printf '%s  %s\n' "$appimagetool_sha256" "$tool" | sha256sum -c --quiet -
}

if ! tool_ok 2>/dev/null; then
    echo "descargando appimagetool $appimagetool_version para $machine"
    mkdir -p "$(dirname "$tool")"
    rm -f "$tool" "$tool.part"
    curl -fL --retry 3 -o "$tool.part" \
        "https://github.com/AppImage/appimagetool/releases/download/$appimagetool_version/appimagetool-$machine.AppImage"
    printf '%s  %s\n' "$appimagetool_sha256" "$tool.part" | sha256sum -c --quiet -
    mv "$tool.part" "$tool"
    chmod +x "$tool"
fi

echo "generando $appimage"
rm -f "$appimage"
VERSION="$version" ARCH="$machine" "$tool" --appimage-extract-and-run "$appdir" "$appimage"

sha256sum "$appimage" | sed "s|dist/||" >"$appimage.sha256"

echo
echo "$appimage"
cat "$appimage.sha256"
echo
echo "para probar: ./$appimage --version"
echo "para usar:   chmod +x $appimage && ./$appimage"
