//! Vista previa del panel sin abrir el juego.
//!
//! Uso: `cargo run -p mh-overlay-ui --example preview -- panel.ppm`
//!
//! Con un argumento guarda el panel en PPM y no abre ventana; sin argumentos lo muestra unos
//! segundos sobre la pantalla. Sirve para ajustar el diseño sin depender de que haya una
//! partida en marcha.

use std::error::Error;

use mh_overlay_config::Config;
use mh_overlay_games::{
    Ailment, Breaks, Buff, Crown, Health, Hunt, Monster, Part, Player, Sharpness, SharpnessLevel,
    Snapshot, Weapon, WeaponDetail, WeaponGauge, WeaponKind, WirebugRecharge, Wirebugs,
};
use mh_overlay_ui::editor::Editor;
use mh_overlay_ui::text::Text;
use mh_overlay_ui::{view, window::OverlayWindow};

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::default();
    let text = Text::bundled()?;
    let mode = std::env::args().nth(2);
    let editor = mode.as_deref() == Some("editor");
    let free = mode.as_deref() == Some("free");
    let canvas = if editor {
        Editor::new(config.clone()).view(&text, "~/.config/mh-overlay/config.toml")
    } else if free {
        config.overlay.layout = mh_overlay_config::Layout::Free;
        view::render_free(&config, &text, &sample(), None, (960, 720))
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

fn sample() -> Snapshot {
    Snapshot {
        player: Some(Player {
            health: Health::new(712.0, 1000.0),
            stamina: Health::new(126.0, 150.0),
            recoverable: Some(90.0),
            wirebugs: Some(Wirebugs::new(2, 1, Some(0.5), false).with_recharge_slots([
                None,
                Some(WirebugRecharge {
                    seconds: 8.3,
                    fraction: 0.5,
                }),
                None,
                None,
            ])),
            weapon: Some(Weapon {
                kind: WeaponKind::Longsword,
                sharpness: Some(Sharpness {
                    lengths: [40, 30, 30, 30, 30, 20, 0],
                    level: SharpnessLevel::White,
                    max_level: SharpnessLevel::White,
                    current: 10.0,
                }),
                gauge: Some(WeaponGauge::Spirit {
                    buildup: 62.0,
                    level: 2,
                    timer: 98.0,
                    max_timer: 140.0,
                    regen: 0.0,
                }),
                detail: WeaponDetail::default(),
            }),
            buffs: vec![
                Buff {
                    name: "Semilla del poder",
                    timer: 84.0,
                    max_timer: 180.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Protección divina",
                    timer: 37.0,
                    max_timer: 120.0,
                    debuff: false,
                    infinite: false,
                },
                Buff {
                    name: "Veneno",
                    timer: 12.0,
                    max_timer: 20.0,
                    debuff: true,
                    infinite: false,
                },
            ],
        }),
        hunt: Some(Hunt {
            time_left: 12.0 * 60.0 + 34.0,
            deaths: 1,
            max_deaths: 3,
        }),
        monsters: monsters(),
    }
}

fn monsters() -> Vec<Monster> {
    // Los ids son los de World: 1 es Rathalos y 0 Anjanath. Nombres, partes y umbral de
    // captura salen de la tabla, así que la vista previa enseña lo mismo que el juego.
    let mut rathalos = Monster::new(1, Health::new(11_912.0, 17_440.0));
    rathalos.crown = Crown::Gold;
    rathalos.target = true;
    rathalos.name = mh_overlay_games::tables::world_monster_name(1).map(str::to_owned);
    rathalos.capture_percent = mh_overlay_games::tables::world_capture_percent(1);
    rathalos.parts = vec![
        part(1, 0, 90.0),
        part(1, 1, 41.0),
        part(1, 2, 12.0),
        Part {
            index: 5,
            name: mh_overlay_games::tables::world_part_name(1, 5),
            flinch: None,
            breakable: None,
            severable: Some(Health::new(180.0, 240.0)),
            health: Some(Health::new(180.0, 240.0)),
            breaks: None,
            ..Part::default()
        },
    ];
    rathalos.ailments = vec![ailment(9, 640.0, 1000.0), ailment(1, 120.0, 800.0)];

    let mut anjanath = Monster::new(0, Health::new(1_860.0, 15_000.0));
    anjanath.name = mh_overlay_games::tables::world_monster_name(0).map(str::to_owned);
    anjanath.capture_percent = mh_overlay_games::tables::world_capture_percent(0);
    anjanath.parts = vec![part(0, 0, 68.0), part(0, 1, 5.0)];

    vec![rathalos, anjanath]
}

fn ailment(id: i32, buildup: f32, max_buildup: f32) -> Ailment {
    Ailment {
        name: mh_overlay_games::tables::world_ailment_name(id).unwrap_or("estado"),
        buildup,
        max_buildup,
        timer: 0.0,
        max_timer: 30.0,
    }
}

fn part(monster: i32, index: usize, percent: f32) -> Part {
    Part {
        index,
        name: mh_overlay_games::tables::world_part_name(monster, index),
        flinch: None,
        breakable: Some(Health::new(percent, 100.0)),
        severable: None,
        health: Some(Health::new(percent, 100.0)),
        breaks: Some(Breaks::new(1, Some(2), 1, 2)),
        ..Part::default()
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
