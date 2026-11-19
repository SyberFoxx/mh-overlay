#!/usr/bin/env bash
# Instala, actualiza o desinstala mh-overlay en cualquier distribución de Linux.
#
# Uso normal (sin sudo): deja el programa en los directorios XDG del usuario, enlaza
# ~/.local/bin/mh-overlay y añade la entrada de menú.
#
#   ./install.sh
#
# Para todo el sistema (requiere root o sudo):
#
#   sudo ./install.sh --system
#
# En un prefijo propio (por ejemplo, para probar sin tocar ~/.local):
#
#   ./install.sh --prefix /tmp/mh-overlay
#
# Volver a ejecutarlo sobre una instalación existente la actualiza y conserva
# ~/.config/mh-overlay. `--uninstall` borra programa, lanzador e icono, pero no la configuración
# salvo que se pase `--purge`.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
mode="usuario"
prefix=""
uninstall="no"
purge="no"
menu="sí"
check="no"

usage() {
    cat <<'EOF'
Uso: ./install.sh [opciones]

  --system           instalar para todo el sistema (necesita root; por defecto /usr/local)
  --prefix DIR       instalar bajo DIR (por defecto, los directorios XDG del usuario)
  --uninstall        desinstalar; conserva la configuración
  --purge            con --uninstall, borra también ~/.config/mh-overlay
  --no-menu          no instalar la entrada de menú ni el icono
  --check            solo comprobar el paquete y el entorno, sin instalar
  -h, --help         esta ayuda

Variables: PREFIX (equivale a --prefix), DESTDIR (para empaquetadores), XDG_DATA_HOME,
XDG_BIN_HOME, XDG_CONFIG_HOME.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --system) mode="sistema" ;;
        --prefix)
            shift
            [ $# -gt 0 ] || { echo "error: --prefix necesita un directorio" >&2; exit 2; }
            prefix="$1"
            # Con --system el prefijo solo cambia la raíz (/usr, /opt…), no la disposición.
            [ "$mode" = "sistema" ] || mode="prefijo"
            ;;
        --uninstall) uninstall="sí" ;;
        --purge) purge="sí" ;;
        --no-menu) menu="no" ;;
        --check) check="sí" ;;
        -h|--help) usage; exit 0 ;;
        *)
            echo "error: opción desconocida: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

if [ -n "${PREFIX:-}" ] && [ "$mode" = "usuario" ]; then
    prefix="$PREFIX"
    mode="prefijo"
fi

# Un `--prefix` relativo se resuelve contra el directorio actual; uno absoluto se respeta.
case "$prefix" in
    "") ;;
    /*) ;;
    *) prefix="$(cd "$prefix" 2>/dev/null && pwd || printf '%s/%s' "$PWD" "$prefix")" ;;
esac

destdir=${DESTDIR:-}

if [ "$mode" = "sistema" ]; then
    prefix=${prefix:-/usr/local}
    app="$destdir$prefix/lib/mh-overlay"
    bindir="$destdir$prefix/bin"
    desktop_dir="$destdir$prefix/share/applications"
    icon_root="$destdir$prefix/share/icons/hicolor"
    metainfo_dir="$destdir$prefix/share/metainfo"
    config_path="${XDG_CONFIG_HOME:-${HOME:-/root}/.config}/mh-overlay"
elif [ "$mode" = "prefijo" ]; then
    app="$destdir$prefix/share/mh-overlay"
    bindir="$destdir$prefix/bin"
    desktop_dir="$destdir$prefix/share/applications"
    icon_root="$destdir$prefix/share/icons/hicolor"
    metainfo_dir="$destdir$prefix/share/metainfo"
    config_path="${XDG_CONFIG_HOME:-${HOME:-/root}/.config}/mh-overlay"
else
    data_home=${XDG_DATA_HOME:-${HOME:-/root}/.local/share}
    bin_home=${XDG_BIN_HOME:-${HOME:-/root}/.local/bin}
    app="$destdir$data_home/mh-overlay"
    bindir="$destdir$bin_home"
    desktop_dir="$destdir$data_home/applications"
    icon_root="$destdir$data_home/icons/hicolor"
    metainfo_dir="$destdir$data_home/metainfo"
    config_path="${XDG_CONFIG_HOME:-${HOME:-/root}/.config}/mh-overlay"
fi

say() { printf '%s\n' "$*"; }
warn() { printf 'aviso: %s\n' "$*" >&2; }

if [ "$check" = "sí" ]; then
    status=0

    say "mh-overlay · comprobación"
    say "  paquete:  $here"

    if [ -x "$here/mh-overlay" ]; then
        say "  binario:  $here/mh-overlay ($("$here/mh-overlay" --version 2>/dev/null || echo 'no ejecuta'))"
    else
        warn "falta $here/mh-overlay o no es ejecutable"
        status=1
    fi

    if [ -d "$here/data/maps" ]; then
        maps=$(find "$here/data/maps" -maxdepth 1 -name '*.map' | wc -l)
        say "  mapas:    $maps archivos"
    else
        warn "falta $here/data/maps"
        status=1
    fi

    libc_version=$(ldd --version 2>&1 | head -1 || true)

    if printf '%s' "$libc_version" | grep -qi musl; then
        warn "este binario es para glibc; en Alpine/musl hay que compilar desde el código"
        status=1
    else
        say "  libc:     ${libc_version:-no se pudo consultar}"
    fi

    if command -v ldconfig >/dev/null 2>&1; then
        if ldconfig -p 2>/dev/null | grep -qE 'libEGL\.so|libGL\.so'; then
            say "  OpenGL:   presente (el hub lo necesita; el panel no)"
        else
            warn "no se encontró libEGL/libGL: el hub no abrirá; el panel con --overlay sí"
        fi
    fi

    session=${XDG_SESSION_TYPE:-}
    if [ -n "${WAYLAND_DISPLAY:-}" ] || [ "$session" = "wayland" ]; then
        say "  sesión:   Wayland (el panel usa layer-shell si el compositor lo ofrece)"
    elif [ -n "${DISPLAY:-}" ]; then
        say "  sesión:   X11"
    else
        warn "sin DISPLAY ni WAYLAND_DISPLAY: no hay sesión gráfica que comprobar"
    fi

    if [ "$mode" = "sistema" ]; then
        target="$app/mh-overlay"
    else
        target="$app/mh-overlay"
    fi

    if command -v getcap >/dev/null 2>&1 && [ -x "$target" ]; then
        if getcap "$target" | grep -q cap_sys_ptrace; then
            say "  ptrace:   cap_sys_ptrace ya aplicada"
        else
            warn "sin cap_sys_ptrace: aplica «sudo setcap cap_sys_ptrace+ep $target» o baja kernel.yama.ptrace_scope"
        fi
    elif [ -z "${MH_OVERLAY_MAPS:-}" ]; then
        say "  ptrace:   recuerda aplicar cap_sys_ptrace al binario instalado"
    fi

    exit "$status"
fi

if [ "$mode" = "sistema" ] && [ "$(id -u)" -ne 0 ] && [ -z "$destdir" ]; then
    echo "error: --system escribe fuera de tu home; ejecútalo con sudo." >&2
    echo "       para una instalación de usuario, ejecuta ./install.sh sin --system." >&2
    exit 1
fi

remove_file() {
    if [ -e "$1" ] || [ -L "$1" ]; then
        rm -f "$1"
        say "  borrado $1"
    fi
}

if [ "$uninstall" = "sí" ]; then
    say "desinstalando mh-overlay"
    remove_file "$bindir/mh-overlay"
    remove_file "$desktop_dir/io.github.syberfoxx.mh_overlay.desktop"
    remove_file "$desktop_dir/mh-overlay.desktop"
    remove_file "$icon_root/scalable/apps/mh-overlay.svg"
    remove_file "$icon_root/128x128/apps/mh-overlay.png"
    remove_file "$icon_root/256x256/apps/mh-overlay.png"
    remove_file "$icon_root/512x512/apps/mh-overlay.png"
    remove_file "$metainfo_dir/io.github.syberfoxx.mh_overlay.appdata.xml"

    if [ -d "$app" ]; then
        rm -rf "$app"
        say "  borrado $app"
    fi

    if [ "$purge" = "sí" ] && [ -d "$config_path" ]; then
        rm -rf "$config_path"
        say "  borrada la configuración $config_path"
    elif [ -d "$config_path" ]; then
        say "  configuración conservada en $config_path (usa --purge para borrarla)"
    fi

    say
    say "listo. Si el menú no se actualiza, cierra la sesión o ejecuta:"
    say "  update-desktop-database \"$desktop_dir\" 2>/dev/null || true"
    exit 0
fi

if [ ! -x "$here/mh-overlay" ]; then
    echo "error: no encuentro $here/mh-overlay; descomprime el paquete completo." >&2
    exit 1
fi

if [ ! -d "$here/data/maps" ]; then
    echo "error: no encuentro $here/data/maps junto al instalador." >&2
    exit 1
fi

if [ -x "$app/mh-overlay" ]; then
    say "actualizando la instalación de $app"
fi

install -d "$app" "$bindir"
if [ "$menu" = "sí" ]; then
    install -d "$desktop_dir" "$metainfo_dir" \
        "$icon_root/scalable/apps" \
        "$icon_root/128x128/apps" "$icon_root/256x256/apps" "$icon_root/512x512/apps"
fi

# Reemplazar el archivo pierde las capacidades que puso `setcap`, así que se miran antes.
capabilities=""
if command -v getcap >/dev/null 2>&1 && [ -x "$app/mh-overlay" ]; then
    capabilities=$(getcap "$app/mh-overlay" || true)
fi

# El binario se copia a un temporal y se renombra: una actualización interrumpida no deja un
# ejecutable truncado.
install -m755 "$here/mh-overlay" "$app/mh-overlay.new"
mv -f "$app/mh-overlay.new" "$app/mh-overlay"
rm -rf "$app/data"
cp -r "$here/data" "$app/data"

# Los mapas se buscan junto al binario: el enlace de bin/ no vale porque `current_exe`
# resuelve el destino real.
ln -sf "$app/mh-overlay" "$bindir/mh-overlay"

if [ "$menu" = "sí" ]; then
    # El .desktop se genera con la ruta real de este prefijo.
    sed "s|@APP@|$app|g" "$here/io.github.syberfoxx.mh_overlay.desktop" \
        >"$desktop_dir/io.github.syberfoxx.mh_overlay.desktop"
    chmod 644 "$desktop_dir/io.github.syberfoxx.mh_overlay.desktop"
    # Instalaciones anteriores usaban el nombre corto: no dejar dos entradas de menú.
    rm -f "$desktop_dir/mh-overlay.desktop"

    if [ -f "$here/mh-overlay.svg" ]; then
        install -m644 "$here/mh-overlay.svg" "$icon_root/scalable/apps/mh-overlay.svg"
    fi

    for size in 128 256 512; do
        source_icon="$here/mh-overlay-$size.png"
        [ -f "$source_icon" ] || continue
        install -m644 "$source_icon" "$icon_root/${size}x${size}/apps/mh-overlay.png"
    done

    if [ -f "$here/io.github.syberfoxx.mh_overlay.appdata.xml" ]; then
        install -m644 "$here/io.github.syberfoxx.mh_overlay.appdata.xml" \
            "$metainfo_dir/io.github.syberfoxx.mh_overlay.appdata.xml"
    fi
fi

# Refrescar las cachés del escritorio si las herramientas existen; nunca es un error.
if [ "$menu" = "sí" ] && [ -z "$destdir" ]; then
    command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database "$desktop_dir" >/dev/null 2>&1 || true
    command -v gtk-update-icon-cache >/dev/null 2>&1 \
        && gtk-update-icon-cache -q "$icon_root" >/dev/null 2>&1 || true
fi

version=$("$app/mh-overlay" --version 2>/dev/null || echo "versión desconocida")
say "instalado $version en $app"
say "  binario:  $bindir/mh-overlay"
[ "$menu" = "sí" ] && say "  menú:     $desktop_dir/io.github.syberfoxx.mh_overlay.desktop"
say

if [ -n "$capabilities" ] && printf '%s' "$capabilities" | grep -q cap_sys_ptrace; then
    if setcap cap_sys_ptrace+ep "$app/mh-overlay" 2>/dev/null; then
        say "capacidades de lectura de memoria restauradas"
    else
        say "el binario nuevo perdió las capacidades; vuelve a aplicarlas una vez:"
        say "  sudo setcap cap_sys_ptrace+ep $app/mh-overlay"
    fi
else
    say "para leer la memoria del juego, una sola vez:"
    say "  sudo setcap cap_sys_ptrace+ep $app/mh-overlay"
    say "  (o baja kernel.yama.ptrace_scope mientras juegas)"
fi

say
say "abrir:      $bindir/mh-overlay            (hub gráfico; también desde el menú)"
say "panel:      $bindir/mh-overlay --overlay --game world"
say "consola:    $bindir/mh-overlay --print --game rise"
say "desinstalar:$here/install.sh --uninstall"
