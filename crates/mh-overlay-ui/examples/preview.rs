//! Vista previa del panel sin abrir el juego.
//!
//! Uso: `cargo run -p mh-overlay-ui --example preview -- panel.ppm`
//!
//! Con un argumento guarda el panel en PPM y no abre ventana; sin argumentos lo muestra unos
//! segundos sobre la pantalla. Sirve para ajustar el diseño sin depender de que haya una
//! partida en marcha.

use std::error::Error;

use mh_overlay_config::Config;
use mh_overlay_games::{Ailment, Health, Monster, Part};
use mh_overlay_ui::editor::Editor;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::{view, window::OverlayWindow};

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::default();
    let text = Text::bundled()?;
    let editor = std::env::args().nth(2).as_deref() == Some("editor");
    let canvas = if editor {
        Editor::new(config.clone()).view(&text, "~/.config/mh-overlay/config.toml")
    } else {
        view::render(&config, &text, &sample(), None)
    };

    let Some(path) = std::env::args().nth(1) else {
        let mut window = OverlayWindow::open("mh-overlay · vista previa")?;
        let origin = config.overlay.position.origin(
            window.screen_size(),
            (canvas.width(), canvas.height()),
            (config.overlay.margin_x, config.overlay.margin_y),
        );

        println!(
            "ventana {}x{} en ({}, {}), visible: {}",
            canvas.width(),
            canvas.height(),
            origin.0,
            origin.1,
            window.is_viewable()?
        );

        for _ in 0..25 {
            window.present(&canvas, origin, true, true)?;
            std::thread::sleep(std::time::Duration::from_millis(80));
        }

        return Ok(());
    };

    write_ppm(&path, &canvas)?;
    println!("{path}: {}x{}", canvas.width(), canvas.height());

    Ok(())
}

fn sample() -> Vec<Monster> {
    let mut rathalos = Monster::new(24, Health::new(11_912.0, 17_440.0));
    rathalos.name = Some("Rathalos".to_string());
    rathalos.capture_percent = Some(20.0);
    rathalos.parts = vec![
        part(24, 0, 90.0),
        part(24, 1, 41.0),
        part(24, 2, 12.0),
        Part {
            index: 5,
            name: mh_overlay_games::tables::world_part_name(24, 5),
            flinch: None,
            breakable: None,
            severable: Some(Health::new(180.0, 240.0)),
        },
    ];
    rathalos.ailments = vec![
        Ailment {
            name: "fuego",
            buildup: 640.0,
            max_buildup: 1000.0,
            timer: 0.0,
            max_timer: 30.0,
        },
        Ailment {
            name: "veneno",
            buildup: 120.0,
            max_buildup: 800.0,
            timer: 0.0,
            max_timer: 40.0,
        },
    ];

    let mut anjanath = Monster::new(9, Health::new(1_860.0, 15_000.0));
    anjanath.name = Some("Anjanath".to_string());
    anjanath.capture_percent = Some(20.0);
    anjanath.parts = vec![part(9, 0, 68.0), part(9, 1, 5.0)];

    vec![rathalos, anjanath]
}

fn part(monster: i32, index: usize, percent: f32) -> Part {
    Part {
        index,
        name: mh_overlay_games::tables::world_part_name(monster, index),
        flinch: None,
        breakable: Some(Health::new(percent, 100.0)),
        severable: None,
    }
}

fn write_ppm(path: &str, canvas: &mh_overlay_ui::canvas::Canvas) -> Result<(), std::io::Error> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;
    write!(file, "P6\n{} {}\n255\n", canvas.width(), canvas.height())?;

    for pixel in canvas.pixels() {
        file.write_all(&[
            ((pixel >> 16) & 0xff) as u8,
            ((pixel >> 8) & 0xff) as u8,
            (pixel & 0xff) as u8,
        ])?;
    }

    Ok(())
}
