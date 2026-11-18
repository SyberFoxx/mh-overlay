#!/usr/bin/env bash
# Instala o actualiza mh-overlay para el usuario actual: el programa y sus mapas en
# ~/.local/share, el ejecutable en ~/.local/bin y una entrada de menú para abrir el hub.
#
# Volver a ejecutarlo sobre una instalación existente la deja en la versión del paquete; la
# configuración de ~/.config/mh-overlay no se toca.
#
# Con PREFIX=/ruta se instala en otro prefijo. No pide sudo: no escribe fuera del prefijo.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
prefix=${PREFIX:-$HOME/.local}
app="$prefix/share/mh-overlay"

if [ -x "$app/mh-overlay" ]; then
    echo "actualizando la instalación de $app"
fi

install -d "$app" "$prefix/bin" "$prefix/share/applications"

# Reemplazar el archivo pierde las capacidades que puso `setcap`, así que se miran antes.
capabilities=""
if command -v getcap >/dev/null && [ -x "$app/mh-overlay" ]; then
    capabilities=$(getcap "$app/mh-overlay" || true)
fi

install -m755 "$here/mh-overlay" "$app/mh-overlay"
rm -rf "$app/data"
cp -r "$here/data" "$app/data"

# Los mapas se buscan junto al binario: el enlace de ~/.local/bin no vale porque `current_exe`
# resuelve el destino real.
ln -sf "$app/mh-overlay" "$prefix/bin/mh-overlay"

# El .desktop se genera aquí para que la ruta del lanzador sea la de este prefijo.
sed "s|@APP@|$app|g" "$here/mh-overlay.desktop" >"$prefix/share/applications/mh-overlay.desktop"
chmod 644 "$prefix/share/applications/mh-overlay.desktop"

echo "instalado $("$app/mh-overlay" --version) en $app"
echo "  binario:  $prefix/bin/mh-overlay"
echo "  menú:     $prefix/share/applications/mh-overlay.desktop"
echo

if [ -n "$capabilities" ] && echo "$capabilities" | grep -q cap_sys_ptrace; then
    if setcap cap_sys_ptrace+ep "$app/mh-overlay" 2>/dev/null; then
        echo "capacidades de lectura de memoria restauradas"
    else
        echo "el binario nuevo perdió las capacidades; vuelve a aplicarlas una vez:"
        echo "  sudo setcap cap_sys_ptrace+ep $app/mh-overlay"
    fi
else
    echo "siguiente paso, una sola vez:"
    echo "  sudo setcap cap_sys_ptrace+ep $app/mh-overlay"
fi

echo
echo "ajustes:  $prefix/bin/mh-overlay --settings"
echo "panel:    $prefix/bin/mh-overlay --overlay --game world"
echo "actualizar más adelante:  $prefix/bin/mh-overlay --update"
