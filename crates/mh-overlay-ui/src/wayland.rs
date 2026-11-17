//! Panel como superficie de la capa `overlay` de Wayland.
//!
//! En una sesión Wayland el juego suele ser una ventana XWayland que pide fullscreen, y el
//! compositor la dibuja por encima de cualquier ventana X11: ni `override-redirect` ni
//! `_NET_WM_STATE_ABOVE` sirven para colarse encima. La capa `overlay` de `wlr-layer-shell` sí,
//! porque es la misma que usan las notificaciones del escritorio para aparecer sobre un juego a
//! pantalla completa.
//!
//! El lienzo se sube por `wl_shm`: el panel es un HUD pequeño que se repinta unas pocas veces
//! por segundo, así que compartir memoria es más simple que montar EGL y no exige que el
//! compositor nos dé un contexto acelerado. El lienzo ya viene compuesto sobre negro y sus
//! píxeles son opacos, así que `ARGB8888` los toma tal cual.

use std::fs::File;
use std::os::fd::{AsFd, AsRawFd, FromRawFd};
use std::ptr::NonNull;

use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{
    wl_buffer, wl_compositor::WlCompositor, wl_output, wl_output::WlOutput, wl_region::WlRegion,
    wl_registry::WlRegistry, wl_shm::Format, wl_shm::WlShm, wl_shm_pool::WlShmPool,
    wl_surface::WlSurface,
};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{
        Anchor, Event as LayerEvent, KeyboardInteractivity, ZwlrLayerSurfaceV1,
    },
};

use crate::OverlayError;
use crate::canvas::Canvas;

/// Búferes en vuelo: uno se escribe mientras el compositor lee el otro.
const BUFFERS: usize = 2;

/// Mismo radio que usa el recorte SHAPE de la ventana X11, para que el panel se vea igual.
const CORNER_RADIUS: f32 = 10.0;

/// Superficie del panel ya conectada al compositor.
pub struct WaylandOverlay {
    connection: Connection,
    queue: EventQueue<Events>,
    events: Events,
    surface: WlSurface,
    layer_surface: ZwlrLayerSurfaceV1,
    shm: WlShm,
    /// Se conserva solo para seguir recibiendo el tamaño del monitor: si el proxy se suelta,
    /// el cliente lo destruye y deja de llegar el modo de la pantalla.
    _output: Option<WlOutput>,
    empty_region: WlRegion,
    pool: Option<WlShmPool>,
    buffers: Vec<wl_buffer::WlBuffer>,
    mapping: Option<Mapping>,
    size: (u32, u32),
    stride: usize,
}

/// Eventos del compositor que nos importan; el resto se descarta.
#[derive(Default)]
struct Events {
    configure: Option<(u32, u32, u32)>,
    closed: bool,
    released: [bool; BUFFERS],
    output_size: Option<(u32, u32)>,
}

/// Memoria compartida donde vive el lienzo.
struct Mapping {
    pointer: NonNull<u8>,
    length: usize,
}

impl Drop for Mapping {
    fn drop(&mut self) {
        // SAFETY: el rango vino de `mmap` con este mismo puntero y longitud en `resize`.
        unsafe {
            libc::munmap(self.pointer.as_ptr().cast(), self.length);
        }
    }
}

impl WaylandOverlay {
    /// Conecta con el compositor y crea la superficie en la capa `overlay`.
    pub fn open(title: &str) -> Result<Self, OverlayError> {
        let connection = Connection::connect_to_env()
            .map_err(|error| OverlayError::Wayland(error.to_string()))?;
        let (globals, queue) = registry_queue_init::<Events>(&connection)
            .map_err(|error| OverlayError::Wayland(error.to_string()))?;
        let handle = queue.handle();

        let compositor: WlCompositor = globals
            .bind(&handle, 4..=6, ())
            .map_err(|_| OverlayError::Wayland("el compositor no ofrece wl_compositor".into()))?;
        let shm: WlShm = globals
            .bind(&handle, 1..=1, ())
            .map_err(|_| OverlayError::Wayland("el compositor no ofrece wl_shm".into()))?;
        let layer_shell: ZwlrLayerShellV1 = globals
            .bind(&handle, 1..=4, ())
            .map_err(|_| OverlayError::Wayland("el compositor no ofrece wlr-layer-shell".into()))?;

        // El output solo se enlaza para conocer el tamaño de la pantalla, que es lo que sitúa
        // el panel cuando `position` lo ancla abajo o a la derecha. La superficie no se ata a
        // ningún output concreto: ver el comentario de `get_layer_surface`.
        let output = globals.bind::<WlOutput, _, _>(&handle, 1..=4, ()).ok();

        let surface = compositor.create_surface(&handle, ());
        let empty_region = compositor.create_region(&handle, ());

        // Sin output: el compositor coloca la capa en el monitor activo (Hyprland, el que tiene
        // el foco), que al lanzar el overlay es el del juego. Atarla a un output fijo la mandaría
        // siempre al mismo monitor, que con dos pantallas puede no ser el del juego.
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None,
            Layer::Overlay,
            title.to_string(),
            &handle,
            (),
        );

        layer_surface.set_anchor(Anchor::Top | Anchor::Left);
        // -1 = ninguna zona exclusiva: el panel se dibuja sobre el juego en vez de empujarlo.
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

        surface.set_buffer_scale(1);

        // La superficie no se confirma aquí: un panel anclado arriba a la izquierda no puede
        // medir cero, y el compositor rechaza el commit hasta que `resize` fije un tamaño. Ese
        // primer commit lo hace `present`, que es quien conoce el tamaño del lienzo.
        let events = Events {
            released: [true; BUFFERS],
            ..Events::default()
        };

        Ok(Self {
            connection,
            queue,
            events,
            surface,
            layer_surface,
            shm,
            _output: output,
            empty_region,
            pool: None,
            buffers: Vec::new(),
            mapping: None,
            size: (0, 0),
            stride: 0,
        })
    }

    /// Tamaño del monitor en píxeles lógicos.
    ///
    /// Es el del primer `wl_output` del compositor, que es el que se enlaza para esto. Con dos
    /// pantallas de distinto tamaño, anclar el panel a la derecha o abajo lo descoloca si acaba
    /// en la otra, pero arriba a la izquierda siempre cae donde toca.
    pub fn screen_size(&self) -> (u32, u32) {
        self.events.output_size.unwrap_or((1920, 1080))
    }

    /// Sube el lienzo a la superficie y recoloca el panel con los márgenes de `origin`.
    pub fn present(
        &mut self,
        canvas: &Canvas,
        origin: (i32, i32),
        rounded: bool,
        click_through: bool,
    ) -> Result<(), OverlayError> {
        let size = (canvas.width(), canvas.height());

        if size != self.size {
            self.resize(size)?;
        }

        // El panel está anclado arriba a la izquierda, así que su posición se expresa como
        // margen desde esa esquina: vale para cualquiera de las cuatro posiciones porque
        // `origin` ya viene calculado sobre la pantalla completa.
        self.layer_surface
            .set_margin(origin.1.max(0), 0, 0, origin.0.max(0));

        if click_through {
            // Región de entrada vacía: los clics atraviesan el panel y llegan al juego.
            self.surface.set_input_region(Some(&self.empty_region));
        } else {
            self.surface.set_input_region(None);
        }

        let index = self.acquire_buffer()?;
        let stride = self.stride;
        let height = size.1 as usize;

        {
            let mapping = self
                .mapping
                .as_ref()
                .ok_or_else(|| OverlayError::Wayland("falta la memoria compartida".into()))?;
            let offset = index * stride * height;
            // SAFETY: cada búfer ocupa `stride * height` bytes dentro del mapeo, y `index` es
            // menor que `BUFFERS`, así que el rango cae dentro de la memoria reservada.
            let pixels = unsafe {
                std::slice::from_raw_parts_mut(
                    mapping.pointer.as_ptr().add(offset).cast::<u32>(),
                    size.0 as usize * height,
                )
            };

            pixels.copy_from_slice(canvas.pixels());

            if rounded {
                Self::round_corners(pixels, size.0 as usize, height, CORNER_RADIUS);
            }
        }

        self.surface.attach(Some(&self.buffers[index]), 0, 0);
        self.surface.damage_buffer(
            0,
            0,
            i32::try_from(size.0).unwrap_or(i32::MAX),
            i32::try_from(size.1).unwrap_or(i32::MAX),
        );
        self.surface.commit();

        self.connection
            .flush()
            .map_err(|error| OverlayError::Wayland(error.to_string()))
    }

    /// Oculta el panel: en una capa, quitar el contenido es lo que la deja vacía.
    pub fn hide(&mut self) -> Result<(), OverlayError> {
        self.surface.attach(None, 0, 0);
        self.surface.commit();
        self.connection
            .flush()
            .map_err(|error| OverlayError::Wayland(error.to_string()))
    }

    /// Devuelve un búfer que el compositor ya soltó y lo marca como ocupado.
    fn acquire_buffer(&mut self) -> Result<usize, OverlayError> {
        if let Some(index) = (0..BUFFERS).find(|index| self.events.released[*index]) {
            self.events.released[index] = false;
            return Ok(index);
        }

        // Todos adjuntos: despachar hasta que el compositor libere alguno.
        self.queue
            .roundtrip(&mut self.events)
            .map_err(|error| OverlayError::Wayland(error.to_string()))?;

        let index = (0..BUFFERS)
            .find(|index| self.events.released[*index])
            .unwrap_or(0);
        self.events.released[index] = false;

        Ok(index)
    }

    /// Vacía las esquinas de `pixels`.
    ///
    /// En X11 el recorte lo hace SHAPE; una superficie de Wayland no se recorta, así que el
    /// redondeo hay que ponerlo en los píxeles: alfa cero en la parte de fuera del arco y el
    /// compositor hace el resto.
    fn round_corners(pixels: &mut [u32], width: usize, height: usize, radius: f32) {
        let limit = radius * radius;
        let band = radius.floor() as usize;

        for y in 0..band {
            for x in 0..band {
                let dx = radius - x as f32 - 0.5;
                let dy = radius - y as f32 - 0.5;

                if dx * dx + dy * dy <= limit {
                    continue;
                }

                for (column, row) in [
                    (x, y),
                    (width - 1 - x, y),
                    (x, height - 1 - y),
                    (width - 1 - x, height - 1 - y),
                ] {
                    if let Some(pixel) = pixels.get_mut(row * width + column) {
                        *pixel = 0;
                    }
                }
            }
        }
    }

    fn resize(&mut self, size: (u32, u32)) -> Result<(), OverlayError> {
        if size.0 == 0 || size.1 == 0 {
            return Ok(());
        }

        let width = i32::try_from(size.0).unwrap_or(i32::MAX);
        let height = i32::try_from(size.1).unwrap_or(i32::MAX);
        let stride = width * 4;
        let length = stride as usize * height as usize;
        let total = length * BUFFERS;

        self.destroy_buffers();

        let file = shared_memory(total)?;
        let mapping = Mapping::new(&file, total)?;

        let pool = self
            .shm
            .create_pool(file.as_fd(), total as i32, &self.queue.handle(), ());

        let buffers = (0..BUFFERS)
            .map(|index| {
                pool.create_buffer(
                    (index * length) as i32,
                    width,
                    height,
                    stride,
                    Format::Argb8888,
                    &self.queue.handle(),
                    index,
                )
            })
            .collect();

        self.pool = Some(pool);
        self.buffers = buffers;
        self.mapping = Some(mapping);
        self.size = size;
        self.stride = stride as usize;
        self.events.released = [true; BUFFERS];

        self.layer_surface.set_size(size.0, size.1);
        self.surface.commit();

        // Cada cambio de tamaño se confirma con un configure nuevo que hay que reconocer.
        for _ in 0..8 {
            self.queue
                .roundtrip(&mut self.events)
                .map_err(|error| OverlayError::Wayland(error.to_string()))?;

            if let Some((serial, _, _)) = self.events.configure.take() {
                self.layer_surface.ack_configure(serial);
                break;
            }
        }

        Ok(())
    }

    fn destroy_buffers(&mut self) {
        for buffer in self.buffers.drain(..) {
            buffer.destroy();
        }

        if let Some(pool) = self.pool.take() {
            pool.destroy();
        }

        self.mapping = None;
    }
}

impl Drop for WaylandOverlay {
    fn drop(&mut self) {
        self.destroy_buffers();
        self.empty_region.destroy();
        self.layer_surface.destroy();
        self.surface.destroy();
        let _ = self.connection.flush();
    }
}

impl Mapping {
    fn new(file: &File, length: usize) -> Result<Self, OverlayError> {
        // SAFETY: mapeamos el descriptor que acaba de crear `shared_memory`, del tamaño que se
        // acaba de fijar con `ftruncate`.
        let address = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                length,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };

        if address == libc::MAP_FAILED {
            return Err(OverlayError::Wayland(format!(
                "no se pudo mapear la memoria compartida: {}",
                std::io::Error::last_os_error()
            )));
        }

        let pointer = NonNull::new(address.cast::<u8>())
            .ok_or_else(|| OverlayError::Wayland("mapeo de memoria compartida nulo".into()))?;

        Ok(Self { pointer, length })
    }
}

/// Crea el archivo anónimo que respalda los búferes de `wl_shm`.
fn shared_memory(length: usize) -> Result<File, OverlayError> {
    let name = c"mh-overlay";
    // SAFETY: `name` es una cadena C válida y terminada en nulo.
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };

    if fd < 0 {
        return Err(OverlayError::Wayland(format!(
            "no se pudo crear el archivo de memoria compartida: {}",
            std::io::Error::last_os_error()
        )));
    }

    // SAFETY: `fd` es nuevo, lo acaba de devolver `memfd_create` y no lo tiene nadie más.
    let file = unsafe { File::from_raw_fd(fd) };

    // SAFETY: `fd` sigue abierto a través de `file`.
    if unsafe { libc::ftruncate(fd, length as libc::off_t) } < 0 {
        return Err(OverlayError::Wayland(format!(
            "no se pudo dimensionar la memoria compartida: {}",
            std::io::Error::last_os_error()
        )));
    }

    Ok(file)
}

macro_rules! dispatch_noop {
    ($interface:ty, $data:ty) => {
        impl Dispatch<$interface, $data> for Events {
            fn event(
                _: &mut Self,
                _: &$interface,
                _: <$interface as Proxy>::Event,
                _: &$data,
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
            }
        }
    };
}

dispatch_noop!(WlRegistry, GlobalListContents);
dispatch_noop!(WlCompositor, ());
dispatch_noop!(WlSurface, ());
dispatch_noop!(WlRegion, ());
dispatch_noop!(WlShm, ());
dispatch_noop!(WlShmPool, ());
dispatch_noop!(ZwlrLayerShellV1, ());

impl Dispatch<WlOutput, ()> for Events {
    fn event(
        state: &mut Self,
        _: &WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Mode {
            flags: WEnum::Value(flags),
            width,
            height,
            ..
        } = event
            && flags.contains(wl_output::Mode::Current)
        {
            state.output_size = Some((width as u32, height as u32));
        }
    }
}

impl Dispatch<wl_buffer::WlBuffer, usize> for Events {
    fn event(
        state: &mut Self,
        _: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, wl_buffer::Event::Release) && *index < BUFFERS {
            state.released[*index] = true;
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &ZwlrLayerSurfaceV1,
        event: LayerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            LayerEvent::Configure {
                serial,
                width,
                height,
            } => state.configure = Some((serial, width, height)),
            LayerEvent::Closed => state.closed = true,
            _ => {}
        }
    }
}
