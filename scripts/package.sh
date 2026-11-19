#!/usr/bin/env bash
# Empaqueta el ejecutable para Linux: binario, mapas de direcciones, configuración de ejemplo,
# lanzador y un instalador que lo deja en ~/.local sin tocar el sistema.
#
# El paquete se arma en dist/ y se puede copiar a otra máquina con la misma arquitectura: el
# binario solo depende de glibc y de las bibliotecas gráficas que ya trae cualquier escritorio.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
machine=$(uname -m)
name="mh-overlay-$version-linux-$machine"
stage="dist/$name"

echo "compilando $version para $machine"
cargo build --release --locked

echo "preparando $stage"
rm -rf "$stage"
mkdir -p "$stage/data"

install -m755 target/release/mh-overlay "$stage/mh-overlay"
cp -r data/maps "$stage/data/maps"
install -m644 README.md LICENSE NOTICE "$stage/"
install -m755 packaging/install.sh "$stage/install.sh"
install -m644 packaging/mh-overlay.desktop \
    "$stage/io.github.syberfoxx.mh_overlay.desktop"
install -m644 packaging/mh-overlay.svg "$stage/mh-overlay.svg"
install -m644 packaging/mh-overlay-128.png "$stage/mh-overlay-128.png"
install -m644 packaging/mh-overlay.png "$stage/mh-overlay-256.png"
install -m644 packaging/mh-overlay-512.png "$stage/mh-overlay-512.png"
install -m644 packaging/io.github.syberfoxx.mh_overlay.appdata.xml \
    "$stage/io.github.syberfoxx.mh_overlay.appdata.xml"

# La plantilla se genera con el propio binario para que no se separe nunca de los valores por
# defecto del código.
config_home=$(mktemp -d)
env -u XDG_CONFIG_HOME HOME="$config_home" "$stage/mh-overlay" --init-config >/dev/null
install -m644 "$config_home/.config/mh-overlay/config.toml" "$stage/config.example.toml"
rm -rf "$config_home"

tar -C dist -czf "dist/$name.tar.gz" "$name"
sha256sum "dist/$name.tar.gz" | sed "s|dist/||" >"dist/$name.tar.gz.sha256"

echo
echo "dist/$name.tar.gz"
cat "dist/$name.tar.gz.sha256"
echo
echo "en la máquina destino: tar xzf $name.tar.gz && cd $name && ./install.sh --check"
echo "después:                ./install.sh   (usuario)   o   sudo ./install.sh --system"
