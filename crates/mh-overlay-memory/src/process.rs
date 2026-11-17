use std::fs;
use std::path::Path;

use crate::error::MemoryError;
use crate::memory::Memory;

/// A game process the overlay is attached to, with its module base already resolved.
#[derive(Debug, Clone)]
pub struct Process {
    pid: i32,
    module: String,
    base: u64,
}

impl Process {
    /// Finds the first running process whose executable matches one of `names`.
    ///
    /// Under Proton `/proc/<pid>/exe` points at the Wine loader, so the match is done against
    /// `comm` and against the basename of every `cmdline` argument.
    pub fn find(names: &[&str]) -> Result<Self, MemoryError> {
        let module = names
            .first()
            .copied()
            .ok_or_else(|| MemoryError::ProcessNotFound {
                names: String::new(),
            })?;

        let mut pids = fs::read_dir("/proc")?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().to_str()?.parse::<i32>().ok())
            .collect::<Vec<_>>();
        pids.sort_unstable();

        let mut matched: Option<i32> = None;

        for pid in pids {
            if !process_matches(pid, names) {
                continue;
            }

            matched = Some(pid);

            // Steam/Proton pone el nombre del ejecutable en la línea de comandos de procesos
            // auxiliares que nunca mapean el módulo: un candidato sin base no es definitivo,
            // hay que seguir buscando el que sí lo tiene mapeado.
            let Ok(base) = module_base(pid, module) else {
                continue;
            };

            if base == 0 {
                continue;
            }

            return Ok(Self {
                pid,
                module: module.to_string(),
                base,
            });
        }

        match matched {
            Some(pid) => Err(MemoryError::ModuleBaseNotFound {
                pid,
                module: module.to_string(),
            }),
            None => Err(MemoryError::ProcessNotFound {
                names: names.join(", "),
            }),
        }
    }

    /// Attaches to a known pid and resolves the module base from `/proc/<pid>/maps`.
    pub fn from_pid(pid: i32, module: &str) -> Result<Self, MemoryError> {
        Ok(Self {
            pid,
            module: module.to_string(),
            base: module_base(pid, module)?,
        })
    }

    /// Attaches to a known pid with a base supplied by the caller, for the cases where the
    /// module is mapped without a path in `/proc/<pid>/maps`.
    pub fn with_base(pid: i32, module: impl Into<String>, base: u64) -> Self {
        Self {
            pid,
            module: module.into(),
            base,
        }
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn base(&self) -> u64 {
        self.base
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn memory(&self) -> Memory {
        Memory::new(self.pid)
    }
}

fn process_matches(pid: i32, names: &[&str]) -> bool {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();

    if names
        .iter()
        .any(|name| comm.trim().eq_ignore_ascii_case(name))
    {
        return true;
    }

    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();

    cmdline
        .split(|byte| *byte == 0)
        .filter_map(|argument| std::str::from_utf8(argument).ok())
        .any(|argument| {
            let executable = argument.rsplit(['/', '\\']).next().unwrap_or(argument);
            names
                .iter()
                .any(|name| executable.eq_ignore_ascii_case(name))
        })
}

/// Splits a `/proc/<pid>/maps` line into its address range and its path. The path is the only
/// field that may contain spaces — a game installed under `Monster Hunter World` turns any
/// `split_whitespace` on the line into a path that stops at `Monster`.
fn split_map_line(line: &str) -> Option<(&str, &str)> {
    let mut rest = line;
    let mut range = "";

    for field in 0..6 {
        rest = rest.trim_start();

        if field == 5 {
            return Some((range, rest));
        }

        let end = rest.find(char::is_whitespace)?;

        if field == 0 {
            range = &rest[..end];
        }

        rest = &rest[end..];
    }

    None
}

/// Lowest load address of the mapping backed by `module`, which is the base the `.map`
/// static offsets are relative to.
fn module_base(pid: i32, module: &str) -> Result<u64, MemoryError> {
    let maps = fs::read_to_string(format!("/proc/{pid}/maps"))
        .map_err(|source| MemoryError::MapsUnreadable { pid, source })?;

    let mut lowest: Option<u64> = None;

    for line in maps.lines() {
        let Some((range, path)) = split_map_line(line) else {
            continue;
        };

        let file_name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        if !file_name.eq_ignore_ascii_case(module) {
            continue;
        }

        let Some(start) = range
            .split('-')
            .next()
            .and_then(|start| u64::from_str_radix(start, 16).ok())
        else {
            continue;
        };

        lowest = Some(lowest.map_or(start, |current| current.min(start)));
    }

    lowest.ok_or_else(|| MemoryError::ModuleBaseNotFound {
        pid,
        module: module.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_path_with_spaces_is_read_whole() {
        let line = "140000000-140001000 r--p 00000000 08:11 4487 \
                    /mnt/god500/Games/Monster Hunter World/MonsterHunterWorld.exe";

        let (range, path) = split_map_line(line).unwrap();

        assert_eq!(range, "140000000-140001000");
        assert_eq!(
            Path::new(path).file_name().and_then(|name| name.to_str()),
            Some("MonsterHunterWorld.exe")
        );
    }

    #[test]
    fn the_test_process_can_be_found_by_its_own_name() {
        let pid = std::process::id() as i32;
        let cmdline = fs::read(format!("/proc/{pid}/cmdline")).unwrap();
        let first_argument = cmdline
            .split(|byte| *byte == 0)
            .next()
            .and_then(|argument| std::str::from_utf8(argument).ok())
            .expect("el proceso de test tiene cmdline");
        let name = first_argument.rsplit('/').next().unwrap();

        let found = Process::find(&[name]).unwrap();

        assert_eq!(found.pid(), pid);
    }

    #[test]
    fn an_unknown_process_reports_which_names_it_looked_for() {
        let error = Process::find(&["no-existe-este-proceso-12345"]).unwrap_err();

        assert!(matches!(error, MemoryError::ProcessNotFound { .. }));
    }
}
