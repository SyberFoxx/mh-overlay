//! Anclaje del panel dentro de la pantalla.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Esquina o centro de la pantalla contra el que se ancla el panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Position {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl FromStr for Position {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "top-left" | "top" => Ok(Self::TopLeft),
            "top-right" => Ok(Self::TopRight),
            "bottom-left" => Ok(Self::BottomLeft),
            "bottom-right" => Ok(Self::BottomRight),
            "center" | "centre" => Ok(Self::Center),
            other => Err(format!(
                "posición desconocida '{other}' (usa: top-left, top-right, bottom-left, \
                 bottom-right, center)"
            )),
        }
    }
}

impl Position {
    /// Coordenada de la esquina superior izquierda del panel.
    ///
    /// Los márgenes se restan del borde derecho o inferior cuando el anclaje es de ese lado.
    pub fn origin(self, screen: (u32, u32), panel: (u32, u32), margin: (i32, i32)) -> (i32, i32) {
        let (screen_width, screen_height) = (screen.0 as i32, screen.1 as i32);
        let (panel_width, panel_height) = (panel.0 as i32, panel.1 as i32);

        match self {
            Self::TopLeft => (margin.0, margin.1),
            Self::TopRight => (screen_width - panel_width - margin.0, margin.1),
            Self::BottomLeft => (margin.0, screen_height - panel_height - margin.1),
            Self::BottomRight => (
                screen_width - panel_width - margin.0,
                screen_height - panel_height - margin.1,
            ),
            Self::Center => (
                (screen_width - panel_width) / 2,
                (screen_height - panel_height) / 2,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_top_right_panel_sits_against_the_right_edge() {
        assert_eq!(
            Position::TopRight.origin((1920, 1080), (300, 120), (16, 16)),
            (1920 - 300 - 16, 16)
        );
    }

    #[test]
    fn a_bottom_left_panel_sits_against_the_bottom_edge() {
        assert_eq!(
            Position::BottomLeft.origin((1920, 1080), (300, 120), (16, 24)),
            (16, 1080 - 120 - 24)
        );
    }

    #[test]
    fn the_names_are_the_ones_the_configuration_accepts() {
        assert_eq!("top-right".parse::<Position>().unwrap(), Position::TopRight);
        assert_eq!("center".parse::<Position>().unwrap(), Position::Center);
        assert!("middle".parse::<Position>().is_err());
    }

    #[test]
    fn a_centered_panel_is_centered() {
        assert_eq!(
            Position::Center.origin((1000, 800), (200, 100), (0, 0)),
            (400, 350)
        );
    }
}
