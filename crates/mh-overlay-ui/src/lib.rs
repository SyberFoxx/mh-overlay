//! Panel del overlay: dibujo en un lienzo propio y una superficie donde mostrarlo.
//!
//! La separación es deliberada: [`view`] produce una imagen a partir del estado del juego y el
//! backend solo la sube a la pantalla. Así el diseño del panel se prueba sin servidor gráfico.
//!
//! Hay dos backends porque el compositor decide quién queda encima. En X11 basta una ventana
//! `override-redirect` ([`window`]); en Wayland el juego fullscreen se compone por encima de
//! cualquier ventana X11, así que hace falta la capa `overlay` de `wlr-layer-shell`
//! ([`wayland`]). [`Panel`] elige según la sesión.

pub mod canvas;
pub mod editor;
pub mod icons;
pub mod text;
pub mod view;
pub mod wayland;
pub mod window;

use canvas::Canvas;
use wayland::WaylandOverlay;
use window::OverlayWindow;

#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("no se pudo conectar al servidor X11: {source}")]
    Connect {
        #[source]
        source: x11rb::errors::ConnectError,
    },
    #[error("error de X11: {source}")]
    X11 {
        #[source]
        source: x11rb::errors::ConnectionError,
    },
    #[error("error de X11: {source}")]
    X11Reply {
        #[source]
        source: x11rb::errors::ReplyError,
    },
    #[error("error de X11 al crear un recurso: {source}")]
    X11Id {
        #[source]
        source: x11rb::errors::ReplyOrIdError,
    },
    #[error("el servidor X11 no tiene la extensión {name}")]
    MissingExtension { name: &'static str },
    #[error("tipografía inválida: {0}")]
    Font(String),
    #[error("error de Wayland: {0}")]
    Wayland(String),
}

/// Superficie del panel, en el backend que corresponda a la sesión actual.
///
/// En Wayland se usa la capa `overlay`: es la única que un compositor dibuja sobre un juego a
/// pantalla completa, y además el panel no necesita recibir teclado ni foco.
pub struct Panel {
    backend: Backend,
}

/// Backend que pide el usuario con `--backend`; `Auto` decide según la sesión.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preference {
    #[default]
    Auto,
    Wayland,
    X11,
}

enum Backend {
    Wayland(WaylandOverlay),
    X11(OverlayWindow),
}

impl Panel {
    /// Abre la superficie del panel eligiendo backend según la sesión.
    pub fn open(title: &str) -> Result<Self, OverlayError> {
        Self::open_with(title, Preference::Auto)
    }

    /// Abre la superficie del panel en el backend pedido.
    ///
    /// Con [`Preference::Auto`] se intenta Wayland primero y, si el compositor no ofrece
    /// layer-shell, se sigue con X11 en lugar de dejar al usuario sin overlay. Ese respaldo no
    /// mantiene el panel sobre un juego a pantalla completa en GNOME: Mutter ignora la posición
    /// de las ventanas X11 y no implementa `wlr-layer-shell`, así que se avisa y se deja la
    /// ventana como panel flotante normal.
    pub fn open_with(title: &str, preference: Preference) -> Result<Self, OverlayError> {
        let wayland_available = std::env::var_os("WAYLAND_DISPLAY").is_some();

        if preference != Preference::X11 && wayland_available {
            match WaylandOverlay::open(title) {
                Ok(overlay) => {
                    return Ok(Self {
                        backend: Backend::Wayland(overlay),
                    });
                }
                Err(error) if preference == Preference::Wayland => return Err(error),
                Err(error) => eprintln!("aviso: {error}; {}", x11_fallback_hint()),
            }
        }

        Ok(Self {
            backend: Backend::X11(OverlayWindow::open(title)?),
        })
    }

    /// Nombre del backend en uso, para que la CLI diga dónde se está dibujando.
    pub fn backend_name(&self) -> &'static str {
        match &self.backend {
            Backend::Wayland(_) => "wayland (capa overlay)",
            Backend::X11(_) => "X11",
        }
    }

    /// Tamaño de la pantalla donde vive el panel.
    pub fn screen_size(&self) -> (u32, u32) {
        match &self.backend {
            Backend::Wayland(overlay) => overlay.screen_size(),
            Backend::X11(window) => window.screen_size(),
        }
    }

    /// Muestra `canvas` en `origin`.
    ///
    /// `rounded` redondea las esquinas: en X11 recortando la ventana con SHAPE y en Wayland
    /// dejando transparentes los píxeles de fuera del arco.
    pub fn present(
        &mut self,
        canvas: &Canvas,
        origin: (i32, i32),
        rounded: bool,
        click_through: bool,
    ) -> Result<(), OverlayError> {
        match &mut self.backend {
            Backend::Wayland(overlay) => overlay.present(canvas, origin, rounded, click_through),
            Backend::X11(window) => window.present(canvas, origin, rounded, click_through),
        }
    }

    /// Oculta el panel hasta el próximo estado que merezca mostrarse.
    pub fn hide(&mut self) -> Result<(), OverlayError> {
        match &mut self.backend {
            Backend::Wayland(overlay) => overlay.hide(),
            Backend::X11(window) => window.hide(),
        }
    }
}

/// Qué supone caer a X11 en una sesión Wayland, según el escritorio.
///
/// GNOME (Mutter) no implementa `wlr-layer-shell` y no deja colocar ventanas X11, así que el
/// panel queda como una ventana flotante que el juego a pantalla completa tapa. El mensaje dice
/// qué hacer en vez de dejar al usuario adivinando por qué no ve nada.
fn x11_fallback_hint() -> String {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    x11_fallback_hint_for(&desktop)
}

/// Variante de [`x11_fallback_hint`] con el escritorio ya leído, para poder probarla.
fn x11_fallback_hint_for(desktop: &str) -> String {
    if desktop.to_ascii_lowercase().contains("gnome") {
        return "GNOME no permite fijar ventanas sobre el juego: el panel queda como ventana \
                flotante normal. Para verlo encima usa la sesión «GNOME on Xorg» (desde el \
                engranaje de la pantalla de acceso), deja el juego en ventana sin bordes, o usa \
                un compositor con layer-shell (KDE, Hyprland, Sway)."
            .to_string();
    }

    "el panel usará X11; si el juego corre a pantalla completa puede taparlo, y en ese caso \
     conviene ponerlo en ventana sin bordes o usar un escritorio con layer-shell"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::x11_fallback_hint_for;

    /// El aviso de GNOME tiene que nombrar la salida concreta, que no es la misma que en X11.
    #[test]
    fn the_gnome_hint_sends_the_user_to_an_xorg_session() {
        let hint = x11_fallback_hint_for("ubuntu:GNOME");

        assert!(hint.contains("GNOME on Xorg"));
        assert!(hint.contains("sin bordes"));
    }

    #[test]
    fn other_desktops_get_the_generic_hint() {
        assert!(!x11_fallback_hint_for("KDE").contains("GNOME"));
    }
}
