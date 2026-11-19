//! Bandeja del sistema (StatusNotifierItem) para dejar el hub en segundo plano.
//!
//! `ksni` habla el protocolo de bandeja de KDE/freedesktop por D-Bus: no enlaza GTK ni
//! libappindicator, así que funciona igual en Arch, Debian, Fedora o openSUSE con el escritorio
//! que ya trae el usuario. Si no hay bandeja (GNOME sin extensión, sesión sin D-Bus) el hub lo
//! dice y el botón de cerrar cierra de verdad, en vez de esconder la ventana sin retorno.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui;
use ksni::blocking::{Handle, TrayMethods};

use mh_overlay_ui::icons;

/// Avisos entre el hilo de la bandeja y el bucle de la interfaz.
#[derive(Clone, Default)]
pub struct Signals {
    /// El usuario pulsó el icono o «Mostrar»: hay que volver a enseñar la ventana.
    show: Arc<AtomicBool>,
    /// «Salir»: cerrar la ventana y terminar el proceso.
    quit: Arc<AtomicBool>,
    /// «Abrir/Cerrar panel»: el hub abre o mata el proceso del overlay.
    toggle_overlay: Arc<AtomicBool>,
    /// Estado del panel, para que el menú diga «abrir» o «cerrar».
    overlay_running: Arc<AtomicBool>,
    /// Contexto de egui, guardado en el primer fotograma, para despertar una ventana oculta.
    context: Arc<Mutex<Option<egui::Context>>>,
}

impl Signals {
    /// Guarda el contexto para que la bandeja pueda pedir un repintado desde su hilo.
    pub fn attach(&self, context: &egui::Context) {
        if let Ok(mut slot) = self.context.lock() {
            *slot = Some(context.clone());
        }
    }

    pub fn take_show(&self) -> bool {
        self.show.swap(false, Ordering::Relaxed)
    }

    pub fn take_quit(&self) -> bool {
        self.quit.swap(false, Ordering::Relaxed)
    }

    pub fn take_toggle_overlay(&self) -> bool {
        self.toggle_overlay.swap(false, Ordering::Relaxed)
    }

    fn wake(&self) {
        if let Ok(slot) = self.context.lock()
            && let Some(context) = slot.as_ref()
        {
            context.request_repaint();
        }
    }
}

/// Bandeja viva mientras el proceso lo esté. Al soltar el `Handle` se apaga el servicio.
pub struct Tray {
    signals: Signals,
    handle: Handle<Menu>,
}

impl Tray {
    /// Intenta publicar el icono. `None` significa que este escritorio no tiene bandeja.
    pub fn spawn() -> Option<Self> {
        let signals = Signals::default();

        match (Menu {
            signals: signals.clone(),
        })
        .spawn()
        {
            Ok(handle) => Some(Self { signals, handle }),
            Err(error) => {
                eprintln!(
                    "aviso: no hay bandeja de sistema ({error}); cerrar la ventana terminará \
                     mh-overlay"
                );
                None
            }
        }
    }

    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    /// Actualiza la etiqueta del menú tras abrir o cerrar el panel.
    pub fn set_overlay_running(&self, running: bool) {
        self.signals
            .overlay_running
            .store(running, Ordering::Relaxed);
        self.handle.update(|_| {});
    }
}

struct Menu {
    signals: Signals,
}

impl ksni::Tray for Menu {
    fn id(&self) -> String {
        "mh-overlay".to_string()
    }

    fn title(&self) -> String {
        "mh-overlay".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![ksni::Icon {
            width: 32,
            height: 32,
            data: icons::tray_icon(32),
        }]
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let overlay = self.signals.overlay_running.load(Ordering::Relaxed);

        ksni::ToolTip {
            icon_name: "mh-overlay".to_string(),
            icon_pixmap: self.icon_pixmap(),
            title: "mh-overlay".to_string(),
            description: if overlay {
                "Leyendo el juego · panel visible".to_string()
            } else {
                "Hub abierto · panel cerrado".to_string()
            },
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.signals.show.store(true, Ordering::Relaxed);
        self.signals.wake();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::{MenuItem, StandardItem};

        let overlay = self.signals.overlay_running.load(Ordering::Relaxed);

        vec![
            StandardItem {
                label: "Mostrar mh-overlay".to_string(),
                activate: Box::new(|menu: &mut Self| {
                    menu.signals.show.store(true, Ordering::Relaxed);
                    menu.signals.wake();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: if overlay {
                    "Cerrar panel sobre el juego".to_string()
                } else {
                    "Abrir panel sobre el juego".to_string()
                },
                activate: Box::new(|menu: &mut Self| {
                    menu.signals.toggle_overlay.store(true, Ordering::Relaxed);
                    menu.signals.wake();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Salir".to_string(),
                icon_name: "application-exit".to_string(),
                activate: Box::new(|menu: &mut Self| {
                    menu.signals.quit.store(true, Ordering::Relaxed);
                    menu.signals.wake();
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tray_icon_is_argb32_rgba_sized() {
        let icon = icons::tray_icon(32);

        assert_eq!(icon.len(), 32 * 32 * 4);

        let panel = (6 * 32 + 6) * 4;
        assert_eq!(icon[panel], 0xff, "el fondo del icono es opaco");
        assert_eq!(icon[panel + 3], 0x12, "el canal azul va al final");
    }

    #[test]
    fn the_signals_are_one_shot() {
        let signals = Signals::default();
        signals.show.store(true, Ordering::Relaxed);

        assert!(signals.take_show());
        assert!(!signals.take_show());
    }
}
