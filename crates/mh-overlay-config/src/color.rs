//! Color en formato `#rrggbb` o `#rrggbbaa`, tal como se escribe en el TOML.

use std::fmt;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Color RGBA con componentes de 0 a 255; el alfa 255 es opaco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// El mismo color con otro alfa, para paneles translúcidos.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Píxel ARGB32 tal como lo espera un visual TrueColor de X11 en little-endian.
    pub fn to_argb(self) -> u32 {
        (u32::from(self.a) << 24)
            | (u32::from(self.r) << 16)
            | (u32::from(self.g) << 8)
            | u32::from(self.b)
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let digits = value
            .strip_prefix('#')
            .ok_or_else(|| format!("'{value}' no empieza por '#'"))?;

        let component = |index: usize| -> Result<u8, String> {
            u8::from_str_radix(&digits[index..index + 2], 16)
                .map_err(|_| format!("'{value}' no es hexadecimal"))
        };

        match digits.len() {
            6 => Ok(Self::rgb(component(0)?, component(2)?, component(4)?)),
            8 => Ok(Self {
                r: component(0)?,
                g: component(2)?,
                b: component(4)?,
                a: component(6)?,
            }),
            // `#rgb` se expande como en CSS: cada dígito se duplica.
            3 => {
                let expanded = digits
                    .chars()
                    .filter_map(|digit| digit.to_digit(16))
                    .map(|digit| (digit * 17) as u8)
                    .collect::<Vec<_>>();
                Ok(Self::rgb(expanded[0], expanded[1], expanded[2]))
            }
            _ => Err(format!(
                "'{value}' debe ser #rgb, #rrggbb o #rrggbbaa, no {} dígitos",
                digits.len()
            )),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(formatter, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(
                formatter,
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ColorVisitor;

        impl Visitor<'_> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("un color como \"#rrggbb\" o \"#rrggbbaa\"")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Color, E> {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(ColorVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_digit_hex_is_opaque() {
        assert_eq!(
            "#1a2b3c".parse::<Color>().unwrap(),
            Color::rgb(0x1a, 0x2b, 0x3c)
        );
    }

    #[test]
    fn eight_digit_hex_keeps_alpha() {
        assert_eq!("#00000080".parse::<Color>().unwrap().a, 0x80);
    }

    #[test]
    fn three_digit_hex_expands_each_digit() {
        assert_eq!(
            "#f80".parse::<Color>().unwrap(),
            Color::rgb(0xff, 0x88, 0x00)
        );
    }

    #[test]
    fn a_color_round_trips_through_its_text_form() {
        for text in ["#102030", "#10203040", "#abc"] {
            let color = text.parse::<Color>().unwrap();
            assert_eq!(color.to_string().parse::<Color>().unwrap(), color);
        }
    }

    #[test]
    fn a_missing_hash_or_wrong_length_is_an_error() {
        assert!("102030".parse::<Color>().is_err());
        assert!("#12345".parse::<Color>().is_err());
        assert!("#gggggg".parse::<Color>().is_err());
    }
}
