//! Iconos dibujados con el lienzo: una silueta por tipo de arma y una familia por buff.
//!
//! No son imágenes: son trazos sobre `Canvas`, así que escalan con el panel y no añaden
//! archivos ni dependencias. Un icono de arma se reconoce por su contorno (espadón ancho,
//! lanza con escudo, ballesta con cañón) y uno de buff por su familia (ataque, defensa, vida,
//! aguante, elemento...). El color y el temporizador los pone el panel.

use mh_overlay_config::Color;
use mh_overlay_games::WeaponKind;

use crate::canvas::{Canvas, Rect};

/// Familia visual de un buff o estado. Los nombres del juego son muchos; las familias son
/// pocas para que el icono se lea de un vistazo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffGlyph {
    Attack,
    Defense,
    Health,
    Stamina,
    Evasion,
    Sharpness,
    Flame,
    Water,
    Ice,
    Bolt,
    Dragon,
    Poison,
    Bleed,
    Stun,
    Sleep,
    Bubble,
    Song,
    Food,
    Wind,
    Misc,
}

/// Clasifica un nombre de buff por las palabras que usa el juego.
///
/// El orden importa: «Regeneración de filo» es filo y no vida, y «Munición de armadura» es
/// defensa y no ataque, así que los casos concretos van antes que los genéricos.
pub fn buff_glyph(name: &str) -> BuffGlyph {
    let name = name.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|word| name.contains(word));

    if has(&["veneno", "efluvio", "hedor", "mofeta", "azote explosivo"]) {
        BuffGlyph::Poison
    } else if has(&["sangrado", "sangre"]) {
        BuffGlyph::Bleed
    } else if has(&["sueño", "sueno", "somnolencia"]) {
        BuffGlyph::Sleep
    } else if has(&["parálisis", "paralisis"]) {
        BuffGlyph::Bolt
    } else if has(&["aturdimiento", "destello", "flash", "temblor"]) {
        BuffGlyph::Stun
    } else if has(&["burbuja"]) {
        BuffGlyph::Bubble
    } else if has(&["fuego"]) {
        BuffGlyph::Flame
    } else if has(&["agua"]) {
        BuffGlyph::Water
    } else if has(&["hielo", "escarcha", "frío", "frio"]) {
        BuffGlyph::Ice
    } else if has(&["rayo"]) {
        BuffGlyph::Bolt
    } else if has(&["dragón", "dragon"]) {
        BuffGlyph::Dragon
    } else if has(&["filo", "afilado", "pulido"]) {
        BuffGlyph::Sharpness
    } else if has(&[
        "defensa",
        "adamantio",
        "armadura",
        "piel",
        "protección",
        "proteccion",
        "resistencia",
        "tapones",
        "inmunidad",
        "anulado",
        "anulada",
        "anulados",
        "firmeza",
        "coraza",
        "guardia",
    ]) {
        BuffGlyph::Defense
    } else if has(&[
        "vida",
        "regeneración",
        "regeneracion",
        "curación",
        "curacion",
        "recuperación",
        "recuperacion",
        "zumo",
        "lichi",
        "cecina",
        "pez gourmet",
        "natural",
    ]) {
        BuffGlyph::Health
    } else if has(&[
        "aguante",
        "energía",
        "energia",
        "dango cazador",
        "mosca",
        "calamar",
    ]) {
        BuffGlyph::Stamina
    } else if has(&[
        "evasión",
        "evasion",
        "velocidad",
        "deslizante",
        "correr",
        "subidón",
        "subidon",
    ]) {
        BuffGlyph::Evasion
    } else if has(&[
        "ataque",
        "poder",
        "afinidad",
        "crítico",
        "critico",
        "adrenalina",
        "coalescencia",
        "munición demoníaca",
        "municion demoniaca",
        "fortificar",
    ]) {
        BuffGlyph::Attack
    } else if has(&[
        "dango",
        "comida",
        "bebida",
        "mantequilla",
        "semilla",
        "pastilla",
        "polvo",
        "cabeza de nieve",
        "cono de poder",
    ]) {
        BuffGlyph::Food
    } else if has(&[
        "melodía",
        "melodia",
        "barrera sónica",
        "barrera sonica",
        "automejora",
        "extensión de filo",
        "extension de filo",
        "presión del viento",
        "presion del viento",
        "empuje",
    ]) {
        BuffGlyph::Song
    } else if has(&["viento"]) {
        BuffGlyph::Wind
    } else {
        BuffGlyph::Misc
    }
}

/// Dibuja el icono del arma en un cuadrado de `size` píxeles con origen en `(x, y)`.
pub fn weapon(canvas: &mut Canvas, kind: WeaponKind, x: f32, y: f32, size: f32, color: Color) {
    let mut pen = Pen::new(canvas, x, y, size, color);

    match kind {
        WeaponKind::Greatsword => {
            pen.line(12.0, 3.0, 12.0, 15.0, 4.6);
            pen.diamond(12.0, 2.4, 2.2);
            pen.line(7.0, 15.5, 17.0, 15.5, 1.8);
            pen.line(12.0, 15.5, 12.0, 22.0, 2.0);
            pen.diamond(12.0, 22.0, 1.4);
        }
        WeaponKind::Longsword => {
            pen.line(8.0, 20.0, 16.0, 3.0, 2.0);
            pen.diamond(16.5, 2.4, 1.8);
            pen.line(11.0, 14.0, 16.5, 16.0, 1.5);
            pen.line(8.0, 20.0, 5.5, 22.5, 1.8);
        }
        WeaponKind::SwordAndShield => {
            pen.line(5.0, 19.0, 15.0, 4.0, 1.8);
            pen.diamond(15.5, 3.2, 1.6);
            pen.line(9.0, 13.5, 14.0, 16.0, 1.4);
            pen.rounded(12.0, 11.0, 9.0, 11.0, 2.0);
        }
        WeaponKind::DualBlades => {
            pen.line(5.0, 21.0, 12.0, 4.0, 1.9);
            pen.line(19.0, 21.0, 12.0, 4.0, 1.9);
            pen.line(8.5, 13.0, 5.5, 15.0, 1.2);
            pen.line(15.5, 13.0, 18.5, 15.0, 1.2);
            pen.diamond(12.0, 3.0, 1.6);
        }
        WeaponKind::Hammer => {
            pen.line(8.0, 21.0, 15.0, 8.0, 2.4);
            pen.line(12.0, 4.0, 21.0, 8.0, 6.0);
            pen.diamond(21.0, 8.0, 2.2);
            pen.line(6.5, 21.5, 5.0, 23.0, 1.6);
        }
        WeaponKind::HuntingHorn => {
            pen.line(5.0, 20.0, 8.0, 11.0, 2.8);
            pen.line(8.0, 11.0, 13.0, 5.0, 2.8);
            pen.line(13.0, 5.0, 19.0, 6.0, 2.8);
            pen.diamond(20.0, 7.0, 3.0);
            pen.line(5.0, 20.0, 3.0, 22.5, 1.8);
        }
        WeaponKind::Lance => {
            pen.line(9.0, 22.0, 16.0, 5.0, 1.8);
            pen.diamond(16.5, 3.4, 2.2);
            pen.rounded(2.5, 9.0, 8.0, 10.0, 2.0);
        }
        WeaponKind::Gunlance => {
            pen.line(8.0, 22.0, 15.0, 6.0, 1.8);
            pen.diamond(16.0, 4.0, 2.0);
            pen.line(11.0, 17.0, 20.0, 12.0, 2.6);
            pen.diamond(21.0, 11.0, 2.0);
            pen.rounded(2.0, 9.0, 8.0, 10.0, 2.0);
        }
        WeaponKind::SwitchAxe => {
            pen.line(6.0, 22.0, 16.0, 8.0, 2.4);
            pen.line(15.0, 9.0, 21.0, 3.0, 4.2);
            pen.line(15.0, 9.0, 22.0, 8.0, 3.0);
            pen.diamond(21.5, 7.0, 1.8);
        }
        WeaponKind::ChargeBlade => {
            pen.rounded(4.0, 7.0, 11.0, 13.0, 2.5);
            pen.line(12.0, 2.0, 12.0, 13.0, 2.0);
            pen.line(8.0, 11.0, 16.0, 11.0, 1.4);
            pen.diamond(20.0, 5.0, 2.2);
            pen.line(20.0, 10.0, 20.0, 14.0, 1.2);
        }
        WeaponKind::InsectGlaive => {
            pen.line(5.0, 22.0, 17.0, 4.0, 1.6);
            pen.diamond(18.0, 2.6, 2.0);
            pen.diamond(8.0, 8.0, 2.8);
            pen.diamond(13.0, 11.0, 2.2);
            pen.line(8.0, 8.0, 11.0, 13.0, 1.0);
            pen.line(13.0, 11.0, 16.0, 15.0, 1.0);
        }
        WeaponKind::LightBowgun => {
            pen.rounded(3.0, 9.0, 12.0, 6.0, 1.8);
            pen.line(14.0, 8.0, 21.0, 8.0, 2.0);
            pen.line(7.0, 14.0, 5.0, 21.0, 2.4);
            pen.diamond(11.0, 7.0, 1.4);
            pen.line(20.0, 6.0, 20.0, 10.0, 1.2);
        }
        WeaponKind::HeavyBowgun => {
            pen.rounded(2.0, 9.0, 14.0, 8.0, 2.0);
            pen.line(15.0, 6.5, 22.0, 6.5, 3.0);
            pen.line(8.0, 16.0, 6.0, 22.0, 2.6);
            pen.line(18.0, 10.0, 21.5, 20.0, 1.4);
            pen.line(18.0, 10.0, 14.5, 20.0, 1.4);
            pen.diamond(12.0, 7.0, 1.6);
        }
        WeaponKind::Bow => {
            pen.line(7.0, 3.0, 13.0, 5.5, 1.8);
            pen.line(13.0, 5.5, 16.5, 12.0, 1.8);
            pen.line(16.5, 12.0, 13.0, 18.5, 1.8);
            pen.line(13.0, 18.5, 7.0, 21.0, 1.8);
            pen.line(7.0, 3.0, 7.0, 21.0, 0.9);
            pen.line(3.0, 12.0, 21.0, 12.0, 1.2);
            pen.diamond(21.5, 12.0, 1.8);
        }
    }
}

/// Dibuja una corona en un cuadrado de `size` píxeles con origen en `(x, y)`.
///
/// `mini` la achica y le baja los picos: la talla mínima se distingue de la grande por la
/// forma, no solo por el color.
pub fn crown(canvas: &mut Canvas, x: f32, y: f32, size: f32, mini: bool, color: Color) {
    let mut pen = Pen::new(canvas, x, y, size, color);
    let (base, top, middle) = if mini {
        (19.0, 11.0, 6.0)
    } else {
        (18.0, 7.0, 3.0)
    };
    let width = if mini { 1.8 } else { 1.6 };

    pen.line(4.0, base, 20.0, base, width + 0.8);
    for (spike, peak) in [(5.0, top), (12.0, middle), (19.0, top)] {
        pen.line(spike, base - 1.0, spike, peak, width);
        pen.diamond(spike, peak - 1.5, if mini { 1.4 } else { 1.8 });
    }
}

/// Dibuja el icono de un buff en un cuadrado de `size` píxeles.
pub fn buff(canvas: &mut Canvas, glyph: BuffGlyph, x: f32, y: f32, size: f32, color: Color) {
    let mut pen = Pen::new(canvas, x, y, size, color);

    match glyph {
        BuffGlyph::Attack => {
            pen.line(11.0, 21.0, 11.0, 6.0, 2.6);
            pen.diamond(11.0, 3.6, 2.0);
            pen.line(6.0, 11.0, 16.0, 11.0, 1.4);
            pen.line(6.0, 16.0, 6.0, 20.0, 1.2);
            pen.line(6.0, 16.0, 3.5, 18.5, 1.2);
            pen.line(6.0, 16.0, 8.5, 18.5, 1.2);
        }
        BuffGlyph::Defense => {
            pen.line(12.0, 3.0, 20.0, 6.0, 2.0);
            pen.line(20.0, 6.0, 19.0, 15.0, 2.0);
            pen.line(19.0, 15.0, 12.0, 21.0, 2.0);
            pen.line(12.0, 21.0, 5.0, 15.0, 2.0);
            pen.line(5.0, 15.0, 4.0, 6.0, 2.0);
            pen.line(4.0, 6.0, 12.0, 3.0, 2.0);
            pen.line(12.0, 7.0, 12.0, 16.0, 1.0);
        }
        BuffGlyph::Health => {
            pen.diamond(7.5, 8.0, 4.2);
            pen.diamond(16.5, 8.0, 4.2);
            pen.line(4.5, 9.0, 12.0, 21.0, 2.2);
            pen.line(19.5, 9.0, 12.0, 21.0, 2.2);
            pen.line(5.5, 7.0, 12.0, 12.0, 1.6);
            pen.line(18.5, 7.0, 12.0, 12.0, 1.6);
        }
        BuffGlyph::Stamina => {
            pen.line(6.0, 21.0, 17.0, 4.0, 2.0);
            pen.diamond(18.0, 3.0, 2.4);
            pen.diamond(9.0, 10.0, 3.2);
            pen.line(9.0, 10.0, 5.0, 16.0, 1.2);
        }
        BuffGlyph::Evasion => {
            pen.line(4.0, 18.0, 14.0, 9.0, 2.4);
            pen.line(10.0, 19.0, 20.0, 10.0, 2.4);
            pen.line(15.0, 20.0, 22.0, 13.0, 1.8);
            pen.diamond(14.0, 9.0, 2.2);
        }
        BuffGlyph::Sharpness => {
            pen.line(6.0, 21.0, 16.0, 5.0, 2.2);
            pen.diamond(16.5, 3.4, 1.9);
            pen.diamond(20.0, 14.0, 2.0);
            pen.line(17.0, 14.0, 23.0, 14.0, 1.0);
            pen.line(20.0, 11.0, 20.0, 17.0, 1.0);
        }
        BuffGlyph::Flame => {
            pen.line(12.0, 2.5, 7.0, 12.0, 2.4);
            pen.line(7.0, 12.0, 12.0, 21.0, 2.4);
            pen.line(12.0, 21.0, 17.0, 12.0, 2.4);
            pen.line(17.0, 12.0, 12.0, 2.5, 2.4);
            pen.line(12.0, 9.0, 10.0, 14.0, 1.6);
            pen.line(10.0, 14.0, 12.0, 18.0, 1.6);
        }
        BuffGlyph::Water => {
            pen.diamond(12.0, 4.0, 3.0);
            pen.line(9.5, 6.0, 6.0, 14.0, 2.2);
            pen.line(6.0, 14.0, 12.0, 21.0, 2.2);
            pen.line(12.0, 21.0, 18.0, 14.0, 2.2);
            pen.line(18.0, 14.0, 14.5, 6.0, 2.2);
            pen.line(10.0, 14.0, 12.0, 17.0, 1.2);
        }
        BuffGlyph::Ice => {
            pen.line(12.0, 2.0, 12.0, 22.0, 1.6);
            pen.line(3.5, 7.0, 20.5, 17.0, 1.6);
            pen.line(20.5, 7.0, 3.5, 17.0, 1.6);
            pen.line(12.0, 2.0, 9.0, 5.0, 1.0);
            pen.line(12.0, 2.0, 15.0, 5.0, 1.0);
            pen.line(12.0, 22.0, 9.0, 19.0, 1.0);
            pen.line(12.0, 22.0, 15.0, 19.0, 1.0);
        }
        BuffGlyph::Bolt => {
            pen.line(14.0, 2.0, 7.0, 12.0, 2.2);
            pen.line(7.0, 12.0, 13.0, 12.0, 2.2);
            pen.line(13.0, 12.0, 9.0, 22.0, 2.2);
            pen.line(9.0, 22.0, 18.0, 9.0, 2.2);
            pen.line(18.0, 9.0, 12.0, 9.0, 2.2);
            pen.line(12.0, 9.0, 14.0, 2.0, 2.2);
        }
        BuffGlyph::Dragon => {
            pen.line(4.0, 20.0, 16.0, 4.0, 2.2);
            pen.line(9.0, 13.0, 20.0, 10.0, 2.0);
            pen.line(13.0, 9.0, 20.0, 15.0, 1.6);
            pen.line(6.0, 17.0, 10.0, 20.0, 1.4);
            pen.diamond(18.0, 4.0, 2.2);
        }
        BuffGlyph::Poison => {
            pen.diamond(12.0, 4.5, 3.2);
            pen.line(9.0, 7.0, 6.0, 14.0, 2.2);
            pen.line(6.0, 14.0, 12.0, 21.0, 2.2);
            pen.line(12.0, 21.0, 18.0, 14.0, 2.2);
            pen.line(18.0, 14.0, 15.0, 7.0, 2.2);
            pen.line(9.0, 12.0, 15.0, 17.0, 1.2);
            pen.line(15.0, 12.0, 9.0, 17.0, 1.2);
        }
        BuffGlyph::Bleed => {
            pen.diamond(12.0, 4.0, 3.0);
            pen.line(9.0, 7.0, 6.0, 14.0, 2.2);
            pen.line(6.0, 14.0, 12.0, 21.0, 2.2);
            pen.line(12.0, 21.0, 18.0, 14.0, 2.2);
            pen.line(18.0, 14.0, 15.0, 7.0, 2.2);
            pen.line(16.0, 3.0, 20.0, 6.0, 1.2);
        }
        BuffGlyph::Stun => {
            pen.line(12.0, 3.0, 12.0, 21.0, 1.6);
            pen.line(4.0, 7.0, 20.0, 17.0, 1.6);
            pen.line(20.0, 7.0, 4.0, 17.0, 1.6);
            pen.diamond(12.0, 12.0, 3.0);
        }
        BuffGlyph::Sleep => {
            pen.line(7.0, 6.0, 17.0, 6.0, 2.0);
            pen.line(17.0, 6.0, 7.0, 17.0, 2.0);
            pen.line(7.0, 17.0, 17.0, 17.0, 2.0);
            pen.diamond(18.0, 4.0, 1.6);
            pen.diamond(6.0, 20.0, 1.4);
        }
        BuffGlyph::Bubble => {
            pen.diamond(9.0, 9.0, 4.0);
            pen.diamond(16.0, 14.0, 3.0);
            pen.diamond(8.0, 17.0, 2.2);
        }
        BuffGlyph::Song => {
            pen.line(14.0, 4.0, 14.0, 16.0, 1.8);
            pen.line(14.0, 4.0, 20.0, 2.5, 1.8);
            pen.line(20.0, 2.5, 20.0, 13.0, 1.8);
            pen.diamond(11.5, 17.0, 3.0);
            pen.diamond(17.5, 14.5, 2.6);
        }
        BuffGlyph::Food => {
            pen.line(8.0, 16.0, 18.0, 5.0, 4.0);
            pen.line(8.0, 16.0, 4.5, 20.0, 2.2);
            pen.diamond(4.0, 20.5, 1.8);
            pen.line(19.0, 4.0, 22.0, 7.0, 1.4);
        }
        BuffGlyph::Wind => {
            pen.line(3.0, 8.0, 16.0, 8.0, 1.6);
            pen.line(8.0, 12.0, 21.0, 12.0, 1.6);
            pen.line(4.0, 16.0, 18.0, 16.0, 1.6);
            pen.line(16.0, 8.0, 19.0, 10.0, 1.2);
            pen.line(21.0, 12.0, 18.0, 14.0, 1.2);
        }
        BuffGlyph::Misc => {
            pen.diamond(12.0, 12.0, 6.5);
            pen.diamond(12.0, 12.0, 2.2);
            pen.line(12.0, 3.0, 12.0, 7.0, 1.0);
            pen.line(12.0, 17.0, 12.0, 21.0, 1.0);
        }
    }
}

/// Dibuja un cordóptero en un cuadrado de `size` píxeles con origen en `(x, y)`.
///
/// La silueta lo distingue de una barra: cuerpo alargado con cabeza y tórax, dos pares de alas
/// abiertas y el cable curvo que suelta por la cola.
pub fn wirebug(canvas: &mut Canvas, x: f32, y: f32, size: f32, color: Color) {
    let mut pen = Pen::new(canvas, x, y, size, color);

    // Cable fino: sale de la cola y cae hacia la izquierda.
    pen.line(11.5, 19.4, 6.2, 22.6, 1.2);
    pen.line(6.2, 22.6, 2.4, 20.8, 1.1);

    // Cuerpo y cabeza.
    pen.rounded(10.6, 7.0, 2.8, 12.0, 1.4);
    pen.diamond(12.0, 5.2, 1.8);

    // Antenas.
    pen.line(11.3, 4.6, 9.6, 2.0, 0.8);
    pen.line(12.7, 4.6, 14.4, 2.0, 0.8);

    // Alas plegadas que se abren un poco a los costados del abdomen.
    pen.line(10.8, 9.4, 8.0, 17.6, 1.8);
    pen.line(13.2, 9.4, 16.0, 17.6, 1.8);

    // Tres pares de patas.
    for (from, to) in [
        ((10.6, 9.4), (6.2, 7.0)),
        ((13.4, 9.4), (17.8, 7.0)),
        ((10.4, 12.0), (5.5, 12.5)),
        ((13.6, 12.0), (18.5, 12.5)),
        ((10.6, 15.2), (6.8, 18.4)),
        ((13.4, 15.2), (17.2, 18.4)),
    ] {
        pen.line(from.0, from.1, to.0, to.1, 0.9);
    }
}

/// Icono de bandeja: un cuadrado redondeado oscuro con la espada del panel, en ARGB32 con
/// orden de red (A, R, G, B), que es lo que espera StatusNotifierItem.
///
/// Las esquinas quedan en negro puro y se vuelven transparentes: el lienzo compone sobre
/// negro, así que ese valor es la marca de «fuera del icono».
pub fn tray_icon(size: u32) -> Vec<u8> {
    let panel = Color::rgb(0x0b, 0x0d, 0x12);
    let accent = Color::rgb(0xe6, 0xb4, 0x5e);
    let mut canvas = Canvas::new(size, size);

    canvas.fill_rounded(Rect::new(0, 0, size, size), size as f32 * 0.22, panel);
    weapon(
        &mut canvas,
        WeaponKind::Greatsword,
        size as f32 * 0.18,
        size as f32 * 0.12,
        size as f32 * 0.76,
        accent,
    );

    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for pixel in canvas.pixels() {
        let alpha = if pixel & 0x00ff_ffff == 0 { 0 } else { 0xff };
        data.push(alpha);
        data.push(((pixel >> 16) & 0xff) as u8);
        data.push(((pixel >> 8) & 0xff) as u8);
        data.push((pixel & 0xff) as u8);
    }

    data
}

/// Trazos en un sistema de coordenadas de 24×24, escalados al tamaño pedido.
struct Pen<'a> {
    canvas: &'a mut Canvas,
    x: f32,
    y: f32,
    scale: f32,
    color: Color,
}

impl<'a> Pen<'a> {
    fn new(canvas: &'a mut Canvas, x: f32, y: f32, size: f32, color: Color) -> Self {
        Self {
            canvas,
            x,
            y,
            scale: size / 24.0,
            color,
        }
    }

    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        (self.x + x * self.scale, self.y + y * self.scale)
    }

    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32) {
        self.canvas.fill_line(
            self.point(x0, y0),
            self.point(x1, y1),
            (width * self.scale).max(0.7),
            self.color,
        );
    }

    fn diamond(&mut self, x: f32, y: f32, radius: f32) {
        self.canvas
            .fill_diamond(self.point(x, y), radius * self.scale, self.color);
    }

    fn rounded(&mut self, x: f32, y: f32, width: f32, height: f32, radius: f32) {
        let (x, y) = self.point(x, y);
        self.canvas.fill_rounded(
            Rect::new(
                x.round() as i32,
                y.round() as i32,
                (width * self.scale).round().max(1.0) as u32,
                (height * self.scale).round().max(1.0) as u32,
            ),
            radius * self.scale,
            self.color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painted(canvas: &Canvas) -> usize {
        canvas
            .pixels()
            .iter()
            .filter(|pixel| **pixel != 0xff00_0000)
            .count()
    }

    #[test]
    fn every_weapon_type_draws_a_distinct_silhouette() {
        let mut signatures = Vec::new();

        for id in 0..14u8 {
            let kind = WeaponKind::from_id(id).unwrap();
            let mut canvas = Canvas::new(28, 28);
            weapon(&mut canvas, kind, 2.0, 2.0, 24.0, Color::rgb(255, 255, 255));

            assert!(painted(&canvas) > 20, "{kind:?} no dibujó nada");
            signatures.push(canvas.pixels().to_vec());
        }

        for (index, signature) in signatures.iter().enumerate() {
            assert!(
                !signatures[index + 1..].contains(signature),
                "el arma {index} comparte icono con otra"
            );
        }
    }

    #[test]
    fn a_buff_name_lands_in_its_family() {
        assert_eq!(buff_glyph("Semilla del poder"), BuffGlyph::Attack);
        assert_eq!(buff_glyph("Ataque +"), BuffGlyph::Attack);
        assert_eq!(buff_glyph("Veneno"), BuffGlyph::Poison);
        assert_eq!(buff_glyph("Regeneración de filo"), BuffGlyph::Sharpness);
        assert_eq!(buff_glyph("Munición de armadura"), BuffGlyph::Defense);
        assert_eq!(buff_glyph("Sueño"), BuffGlyph::Sleep);
    }

    #[test]
    fn every_known_buff_gets_a_non_empty_icon() {
        for name in mh_overlay_games::buffs::WORLD_BUFFS
            .iter()
            .map(|entry| entry.name)
            .chain(
                mh_overlay_games::buffs::RISE_BUFFS
                    .iter()
                    .map(|entry| entry.name),
            )
        {
            let mut canvas = Canvas::new(20, 20);
            buff(
                &mut canvas,
                buff_glyph(name),
                2.0,
                2.0,
                16.0,
                Color::rgb(255, 255, 255),
            );

            assert!(painted(&canvas) > 8, "«{name}» no dibujó icono");
        }
    }

    #[test]
    fn the_wirebug_glyph_paints_inside_its_box() {
        let mut canvas = Canvas::transparent(60, 60);
        wirebug(&mut canvas, 20.0, 20.0, 16.0, Color::rgb(255, 255, 255));

        let bounds = canvas
            .content_bounds()
            .expect("el glifo tiene que pintar algo");
        assert!(
            bounds.x >= 20 && bounds.y >= 20,
            "el glifo se sale por arriba o por la izquierda: {bounds:?}"
        );
        assert!(
            bounds.right() <= 36 && bounds.bottom() <= 36,
            "el glifo se sale del cuadrado: {bounds:?}"
        );

        let painted = canvas.pixels().iter().filter(|pixel| **pixel != 0).count();
        assert!(painted > 12, "el glifo dibujó muy pocos píxeles: {painted}");
    }

    #[test]
    fn the_tray_icon_has_transparent_corners_and_a_painted_center() {
        let icon = tray_icon(32);
        let alpha = |x: usize, y: usize| icon[(y * 32 + x) * 4];

        assert_eq!(alpha(0, 0), 0);
        assert_eq!(alpha(31, 31), 0);
        assert_eq!(alpha(16, 16), 255);
    }
}
