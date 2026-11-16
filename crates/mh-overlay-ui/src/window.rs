//! Ventana X11 sin decorar: siempre encima, sin robar el foco y atravesable por el ratón.
//!
//! Bajo Proton el juego es una ventana XWayland, así que el panel es una ventana X11 más. La
//! forma (`SHAPE`) recorta las esquinas redondeadas y, con el mismo mecanismo, deja la región
//! de entrada vacía para que los clics lleguen al juego.

use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{
    AtomEnum, ClipOrdering, ConnectionExt as _, CreateWindowAux, EventMask, Gcontext, ImageFormat,
    MapState, PropMode, Rectangle, StackMode, Window, WindowClass,
};
use x11rb::rust_connection::RustConnection;
// `change_property32`/`change_property8` y `extension_information` vienen del wrapper.
use x11rb::wrapper::ConnectionExt as _;

use crate::OverlayError;
use crate::canvas::{Canvas, Rect, rounded_rect_coverage};

const WINDOW_TYPE_UTILITY: &str = "_NET_WM_WINDOW_TYPE_UTILITY";
const NET_WM_WINDOW_TYPE: &str = "_NET_WM_WINDOW_TYPE";
const NET_WM_STATE: &str = "_NET_WM_STATE";
const NET_WM_STATE_ABOVE: &str = "_NET_WM_STATE_ABOVE";
const NET_WM_STATE_SKIP_TASKBAR: &str = "_NET_WM_STATE_SKIP_TASKBAR";
const NET_WM_STATE_SKIP_PAGER: &str = "_NET_WM_STATE_SKIP_PAGER";
const NET_WM_NAME: &str = "_NET_WM_NAME";
const UTF8_STRING: &str = "UTF8_STRING";

/// Ventana del panel, ya conectada al servidor X.
pub struct OverlayWindow {
    connection: RustConnection,
    window: Window,
    gc: Gcontext,
    screen: usize,
    size: (u32, u32),
    /// Qué forma tiene aplicada ahora mismo, para no repetir la petición en cada fotograma.
    shape: ShapeState,
    shaped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShapeState {
    rounded: bool,
    click_through: bool,
}

impl OverlayWindow {
    pub fn open(title: &str) -> Result<Self, OverlayError> {
        let (connection, screen) =
            x11rb::connect(None).map_err(|source| OverlayError::Connect { source })?;
        let root = connection.setup().roots[screen].root;
        let depth = connection.setup().roots[screen].root_depth;
        let shaped = connection
            .extension_information(shape::X11_EXTENSION_NAME)
            .map_err(|source| OverlayError::X11 { source })?
            .is_some();

        let window = connection
            .generate_id()
            .map_err(|source| OverlayError::X11Id { source })?;

        connection
            .create_window(
                depth,
                window,
                root,
                0,
                0,
                1,
                1,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new()
                    .override_redirect(1)
                    // Sin fondo propio: el panel se sube entero con PutImage.
                    .background_pixel(0)
                    .event_mask(EventMask::EXPOSURE),
            )
            .map_err(|source| OverlayError::X11 { source })?;

        let atoms = Atoms::intern(&connection)?;

        connection
            .change_property32(
                PropMode::REPLACE,
                window,
                atoms.window_type,
                AtomEnum::ATOM,
                &[atoms.utility],
            )
            .map_err(|source| OverlayError::X11 { source })?;

        connection
            .change_property32(
                PropMode::REPLACE,
                window,
                atoms.state,
                AtomEnum::ATOM,
                &[atoms.above, atoms.skip_taskbar, atoms.skip_pager],
            )
            .map_err(|source| OverlayError::X11 { source })?;

        connection
            .change_property8(
                PropMode::REPLACE,
                window,
                atoms.name,
                atoms.utf8_string,
                title.as_bytes(),
            )
            .map_err(|source| OverlayError::X11 { source })?;

        // También el nombre clásico: `xprop`/`xdotool` buscan WM_NAME, no _NET_WM_NAME.
        connection
            .change_property8(
                PropMode::REPLACE,
                window,
                AtomEnum::WM_NAME,
                AtomEnum::STRING,
                title.as_bytes(),
            )
            .map_err(|source| OverlayError::X11 { source })?;

        let gc = connection
            .generate_id()
            .map_err(|source| OverlayError::X11Id { source })?;
        connection
            .create_gc(gc, window, &Default::default())
            .map_err(|source| OverlayError::X11 { source })?;

        connection
            .map_window(window)
            .map_err(|source| OverlayError::X11 { source })?;
        connection
            .flush()
            .map_err(|source| OverlayError::X11 { source })?;

        Ok(Self {
            connection,
            window,
            gc,
            screen,
            size: (1, 1),
            shape: ShapeState {
                rounded: false,
                click_through: false,
            },
            shaped,
        })
    }

    /// Tamaño de la pantalla donde vive la ventana.
    pub fn screen_size(&self) -> (u32, u32) {
        let screen = &self.connection.setup().roots[self.screen];
        (
            u32::from(screen.width_in_pixels),
            u32::from(screen.height_in_pixels),
        )
    }

    /// Sube el lienzo a la ventana, la recoloca y aplica la forma.
    pub fn present(
        &mut self,
        canvas: &Canvas,
        origin: (i32, i32),
        rounded: bool,
        click_through: bool,
    ) -> Result<(), OverlayError> {
        let size = (canvas.width(), canvas.height());
        let resize = size != self.size;

        if resize {
            self.connection
                .configure_window(
                    self.window,
                    &x11rb::protocol::xproto::ConfigureWindowAux::new()
                        .width(u32::from(u16::try_from(size.0).unwrap_or(u16::MAX)))
                        .height(u32::from(u16::try_from(size.1).unwrap_or(u16::MAX))),
                )
                .map_err(|source| OverlayError::X11 { source })?;
            self.size = size;
        }

        self.connection
            .configure_window(
                self.window,
                &x11rb::protocol::xproto::ConfigureWindowAux::new()
                    .x(origin.0)
                    .y(origin.1)
                    .stack_mode(StackMode::ABOVE),
            )
            .map_err(|source| OverlayError::X11 { source })?;

        let shape_state = ShapeState {
            rounded,
            click_through,
        };

        if resize || shape_state != self.shape {
            self.apply_shape(canvas.width(), canvas.height(), shape_state)?;
            self.shape = shape_state;
        }

        self.connection
            .put_image(
                ImageFormat::Z_PIXMAP,
                self.window,
                self.gc,
                u16::try_from(size.0).unwrap_or(u16::MAX),
                u16::try_from(size.1).unwrap_or(u16::MAX),
                0,
                0,
                0,
                self.connection.setup().roots[self.screen].root_depth,
                &canvas.to_pixmap_bytes(),
            )
            .map_err(|source| OverlayError::X11 { source })?;

        self.connection
            .map_window(self.window)
            .map_err(|source| OverlayError::X11 { source })?;
        self.connection
            .flush()
            .map_err(|source| OverlayError::X11 { source })?;

        Ok(())
    }

    /// `true` cuando el servidor ya tiene la ventana mapeada y visible.
    pub fn is_viewable(&self) -> Result<bool, OverlayError> {
        let attributes = self
            .connection
            .get_window_attributes(self.window)
            .map_err(|source| OverlayError::X11 { source })?
            .reply()
            .map_err(|source| OverlayError::X11Reply { source })?;

        Ok(attributes.map_state == MapState::VIEWABLE)
    }

    /// Deja la ventana fuera de la pantalla sin cerrar la conexión.
    pub fn hide(&mut self) -> Result<(), OverlayError> {
        self.connection
            .unmap_window(self.window)
            .map_err(|source| OverlayError::X11 { source })?;
        self.connection
            .flush()
            .map_err(|source| OverlayError::X11 { source })?;

        Ok(())
    }

    fn apply_shape(&self, width: u32, height: u32, state: ShapeState) -> Result<(), OverlayError> {
        if !self.shaped {
            return Ok(());
        }

        let radius = if state.rounded { 10.0 } else { 0.0 };

        self.connection
            .shape_rectangles(
                shape::SO::SET,
                shape::SK::BOUNDING,
                ClipOrdering::UNSORTED,
                self.window,
                0,
                0,
                &rounded_rectangles(Rect::new(0, 0, width, height), radius),
            )
            .map_err(|source| OverlayError::X11 { source })?;

        let input = if state.click_through {
            Vec::new()
        } else {
            vec![Rectangle {
                x: 0,
                y: 0,
                width: u16::try_from(width).unwrap_or(u16::MAX),
                height: u16::try_from(height).unwrap_or(u16::MAX),
            }]
        };

        self.connection
            .shape_rectangles(
                shape::SO::SET,
                shape::SK::INPUT,
                ClipOrdering::UNSORTED,
                self.window,
                0,
                0,
                &input,
            )
            .map_err(|source| OverlayError::X11 { source })?;

        Ok(())
    }
}

/// Rectángulos de una scanline por fila: así el borde redondeado lo recorta el servidor X.
fn rounded_rectangles(rect: Rect, radius: f32) -> Vec<Rectangle> {
    let mut rectangles = Vec::with_capacity(rect.height as usize);

    for y in rect.y..rect.bottom() {
        let mut span: Option<(i32, i32)> = None;

        for x in rect.x..rect.right() {
            let inside = rounded_rect_coverage(x, y, rect, radius) >= 0.5;

            match (inside, span) {
                (true, None) => span = Some((x, x + 1)),
                (true, Some((start, _))) => span = Some((start, x + 1)),
                (false, Some((start, end))) => {
                    rectangles.push(Rectangle {
                        x: start as i16,
                        y: y as i16,
                        width: (end - start) as u16,
                        height: 1,
                    });
                    span = None;
                }
                (false, None) => {}
            }
        }

        if let Some((start, end)) = span {
            rectangles.push(Rectangle {
                x: start as i16,
                y: y as i16,
                width: (end - start) as u16,
                height: 1,
            });
        }
    }

    rectangles
}

struct Atoms {
    window_type: u32,
    utility: u32,
    state: u32,
    above: u32,
    skip_taskbar: u32,
    skip_pager: u32,
    name: u32,
    utf8_string: u32,
}

impl Atoms {
    fn intern(connection: &RustConnection) -> Result<Self, OverlayError> {
        let intern = |name: &str| -> Result<u32, OverlayError> {
            connection
                .intern_atom(false, name.as_bytes())
                .map_err(|source| OverlayError::X11 { source })?
                .reply()
                .map_err(|source| OverlayError::X11Reply { source })
                .map(|reply| reply.atom)
        };

        Ok(Self {
            window_type: intern(NET_WM_WINDOW_TYPE)?,
            utility: intern(WINDOW_TYPE_UTILITY)?,
            state: intern(NET_WM_STATE)?,
            above: intern(NET_WM_STATE_ABOVE)?,
            skip_taskbar: intern(NET_WM_STATE_SKIP_TASKBAR)?,
            skip_pager: intern(NET_WM_STATE_SKIP_PAGER)?,
            name: intern(NET_WM_NAME)?,
            utf8_string: intern(UTF8_STRING)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_shape_is_one_rectangle_per_row() {
        let rectangles = rounded_rectangles(Rect::new(0, 0, 10, 4), 0.0);

        assert_eq!(rectangles.len(), 4);
        assert_eq!(rectangles[0].width, 10);
    }

    #[test]
    fn rounded_rows_are_narrower_at_the_top_than_in_the_middle() {
        let rectangles = rounded_rectangles(Rect::new(0, 0, 40, 40), 10.0);
        let top = rectangles.first().unwrap();
        let middle = rectangles.get(20).unwrap();

        assert!(top.width < middle.width);
        assert!(middle.width >= 38);
    }
}
