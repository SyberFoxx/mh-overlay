use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::MemoryError;

/// Parsed `.map` file: static addresses and pointer offsets for one game version.
///
/// The format is line oriented and comes from HunterPie (Apache-2.0, see `NOTICE`):
///
/// ```text
/// # comment
/// Address MONSTERS_ADDRESS 0x0F6ED1B8
/// Offset  MONSTER_LIST_OFFSETS 0x80,0x10,0x0
/// Instruction CRC_ORIGINAL_FUNC_1A 0x0F,0x31,0x48
/// ```
#[derive(Debug, Clone, Default)]
pub struct AddressMap {
    game: String,
    version: String,
    base: u64,
    addresses: HashMap<String, u64>,
    offsets: HashMap<String, Vec<i32>>,
    instructions: HashMap<String, Vec<u8>>,
}

impl AddressMap {
    /// Loads a `.map` file, taking the game name and version from its file name.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)?;
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        let (game, version) = split_file_stem(stem);

        Self::parse(&text, game, version)
    }

    pub fn parse(
        text: &str,
        game: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<Self, MemoryError> {
        let mut map = Self {
            game: game.into(),
            version: version.into(),
            ..Self::default()
        };

        for (index, raw_line) in text.lines().enumerate() {
            let line = raw_line.trim_start_matches('\u{feff}').trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split_whitespace();
            let keyword = fields.next().unwrap_or_default();
            let key = fields.next().unwrap_or_default();
            let raw_value = fields.next().unwrap_or_default();
            let line_number = index + 1;

            if key.is_empty() || raw_value.is_empty() {
                return Err(MemoryError::InvalidMapLine {
                    line: line_number,
                    reason: format!("se esperaban tres campos, llegó: {line}"),
                });
            }

            match keyword {
                "Address" => {
                    let address =
                        parse_hex_u64(raw_value).ok_or_else(|| MemoryError::InvalidMapLine {
                            line: line_number,
                            reason: format!("'{raw_value}' no es una dirección hexadecimal"),
                        })?;
                    map.addresses.insert(key.to_string(), address);
                }
                "Offset" => {
                    let offsets = parse_hex_i32_list(raw_value).ok_or_else(|| {
                        MemoryError::InvalidMapLine {
                            line: line_number,
                            reason: format!("'{raw_value}' no es una lista de offsets"),
                        }
                    })?;
                    map.offsets.insert(key.to_string(), offsets);
                }
                "Instruction" => {
                    let bytes = parse_hex_u8_list(raw_value).ok_or_else(|| {
                        MemoryError::InvalidMapLine {
                            line: line_number,
                            reason: format!("'{raw_value}' no es una lista de bytes"),
                        }
                    })?;
                    map.instructions.insert(key.to_string(), bytes);
                }
                other => {
                    return Err(MemoryError::InvalidMapLine {
                        line: line_number,
                        reason: format!("palabra clave desconocida '{other}'"),
                    });
                }
            }
        }

        Ok(map)
    }

    /// Picks the newest `.map` of `game` in `directory`, comparing version components numerically.
    pub fn newest_for_game(
        game: &str,
        directory: impl AsRef<Path>,
    ) -> Result<PathBuf, MemoryError> {
        let directory = directory.as_ref();
        let prefix = format!("{game}.");

        let newest = fs::read_dir(directory)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|extension| extension == "map")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with(&prefix))
            })
            .max_by_key(|path| {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(version_key)
                    .unwrap_or_default()
            });

        newest.ok_or_else(|| MemoryError::NoMapsForGame {
            game: game.to_string(),
            directory: directory.display().to_string(),
        })
    }

    /// Module base the `Address` entries are relative to.
    pub fn set_base(&mut self, base: u64) {
        self.base = base;
    }

    pub fn base(&self) -> u64 {
        self.base
    }

    pub fn game(&self) -> &str {
        &self.game
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// Raw static address, without the module base applied.
    pub fn address(&self, key: &str) -> Result<u64, MemoryError> {
        self.addresses
            .get(key)
            .copied()
            .ok_or_else(|| MemoryError::MissingKey {
                key: key.to_string(),
            })
    }

    /// `module base + static address`: where the key actually lives in this process.
    pub fn absolute(&self, key: &str) -> Result<u64, MemoryError> {
        Ok(self.base + self.address(key)?)
    }

    pub fn offsets(&self, key: &str) -> Result<&[i32], MemoryError> {
        self.offsets
            .get(key)
            .map(Vec::as_slice)
            .ok_or_else(|| MemoryError::MissingKey {
                key: key.to_string(),
            })
    }

    pub fn instruction(&self, key: &str) -> Result<&[u8], MemoryError> {
        self.instructions
            .get(key)
            .map(Vec::as_slice)
            .ok_or_else(|| MemoryError::MissingKey {
                key: key.to_string(),
            })
    }
}

fn split_file_stem(stem: &str) -> (String, String) {
    match stem.split_once('.') {
        Some((game, version)) => (game.to_string(), version.to_string()),
        None => (stem.to_string(), String::new()),
    }
}

fn version_key(stem: &str) -> Vec<u64> {
    let (_, version) = split_file_stem(stem);

    version
        .split('.')
        .map(|component| component.parse::<u64>().unwrap_or(0))
        .collect()
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))?;

    u64::from_str_radix(digits, 16).ok()
}

fn parse_hex_i32_list(value: &str) -> Option<Vec<i32>> {
    value
        .split(',')
        .map(|component| {
            let digits = component
                .strip_prefix("0x")
                .or_else(|| component.strip_prefix("0X"))?;
            i32::from_str_radix(digits, 16).ok()
        })
        .collect()
}

fn parse_hex_u8_list(value: &str) -> Option<Vec<u8>> {
    value
        .split(',')
        .map(|component| {
            let digits = component
                .strip_prefix("0x")
                .or_else(|| component.strip_prefix("0X"))?;
            u8::from_str_radix(digits, 16).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\u{feff}# Addresses\nAddress MONSTERS_ADDRESS 0x0F6ED1B8\nAddress LOCKON_ADDRESS 0x0E965B30\n\n# Offsets\nOffset MONSTER_LIST_OFFSETS 0x80,0x10,0x0\nOffset MONSTER_SEVER_HEALTH_COMPONENT_OFFSETS 0x2F8,0x48,0x10,0x28,0x10\n# Instructions\nInstruction CRC_ORIGINAL_FUNC_1A 0x0F,0x31,0x48\n";

    fn sample_map() -> AddressMap {
        let mut map = AddressMap::parse(SAMPLE, "MonsterHunterRise", "16.0.2.0").unwrap();
        map.set_base(0x1_0000_0000);
        map
    }

    #[test]
    fn it_reads_addresses_offsets_and_instructions() {
        let map = sample_map();

        assert_eq!(map.address("MONSTERS_ADDRESS").unwrap(), 0x0F6E_D1B8);
        assert_eq!(
            map.offsets("MONSTER_LIST_OFFSETS").unwrap(),
            &[0x80, 0x10, 0x0]
        );
        assert_eq!(
            map.instruction("CRC_ORIGINAL_FUNC_1A").unwrap(),
            &[0x0F, 0x31, 0x48]
        );
    }

    #[test]
    fn absolute_applies_the_module_base() {
        let map = sample_map();

        assert_eq!(
            map.absolute("MONSTERS_ADDRESS").unwrap(),
            0x1_0000_0000 + 0x0F6E_D1B8
        );
    }

    #[test]
    fn the_bom_does_not_break_the_first_line() {
        let map = sample_map();

        assert_eq!(map.addresses.len(), 2);
    }

    #[test]
    fn a_missing_key_names_the_key() {
        let error = sample_map().offsets("NO_EXISTE").unwrap_err();

        assert!(matches!(error, MemoryError::MissingKey { .. }));
    }

    #[test]
    fn an_unknown_keyword_is_an_error_not_a_skip() {
        let error = AddressMap::parse("Offsetx A 0x1", "g", "v").unwrap_err();

        assert!(matches!(error, MemoryError::InvalidMapLine { .. }));
    }

    #[test]
    fn versions_compare_numerically() {
        assert!(
            version_key("MonsterHunterRise.16.0.2.0") > version_key("MonsterHunterRise.16.0.1.1")
        );
        assert!(
            version_key("MonsterHunterWorld.421810") > version_key("MonsterHunterWorld.410013")
        );
    }
}
