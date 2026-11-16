//! Panel del overlay: dibujo en un lienzo propio y ventana X11.
//!
//! La separación es deliberada: [`view`] produce una imagen a partir del estado del juego y
//! [`window`] solo la sube a una ventana. Así el diseño del panel se prueba sin servidor X.

pub mod canvas;
pub mod editor;
pub mod text;
pub mod view;
pub mod window;

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
}
