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

enum Backend {
    Wayland(WaylandOverlay),
    X11(OverlayWindow),
}

impl Panel {
    /// Abre la superficie del panel. Si la sesión es Wayland pero el compositor no ofrece
    /// layer-shell, se sigue con X11 en lugar de dejar al usuario sin overlay.
    pub fn open(title: &str) -> Result<Self, OverlayError> {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            match WaylandOverlay::open(title) {
                Ok(overlay) => {
                    return Ok(Self {
                        backend: Backend::Wayland(overlay),
                    });
                }
                Err(error) => eprintln!("aviso: {error}; el panel usará X11"),
            }
        }

        Ok(Self {
            backend: Backend::X11(OverlayWindow::open(title)?),
        })
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
