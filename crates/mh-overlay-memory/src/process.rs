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
    /// `comm` and against the basename of every `cmdline` argument. When neither names the game
    /// — Wine cuts `comm` at 15 bytes and a wrapper can hand it a command line that never
    /// mentions the executable — the module being mapped is the fallback, because the game
    /// always maps its own image.
    pub fn find(names: &[&str]) -> Result<Self, MemoryError> {
        if names.is_empty() {
            return Err(MemoryError::ProcessNotFound {
                names: String::new(),
            });
        }

        let pids = process_ids()?;
        let mut matched: Option<(i32, &str)> = None;

        for &pid in &pids {
            let Some(found) = matching_module(pid, names) else {
                continue;
            };

            matched = Some((pid, found));

            // La base se resuelve contra el nombre que coincidió, no contra el primero de la
            // lista: con `[Rise, World]` el proceso encontrado puede ser World.
            // Steam/Proton pone el nombre del ejecutable en la línea de comandos de procesos
            // auxiliares que nunca mapean el módulo: un candidato sin base no es definitivo,
            // hay que seguir buscando el que sí lo tiene mapeado.
            let Ok(base) = module_base(pid, found) else {
                continue;
            };

            if base == 0 {
                continue;
            }

            return Ok(Self {
                pid,
                module: found.to_string(),
                base,
            });
        }

        if let Some(found) = mapped_process(&pids, names) {
            return Ok(found);
        }

        match matched {
            Some((pid, found)) => Err(MemoryError::ModuleBaseNotFound {
                pid,
                module: found.to_string(),
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

/// Bytes the kernel keeps of `/proc/<pid>/comm`: a longer name arrives cut, not shortened.
const COMM_LIMIT: usize = 15;

/// Every pid in `/proc`, lowest first so an older instance wins when two are running.
fn process_ids() -> Result<Vec<i32>, MemoryError> {
    let mut pids = fs::read_dir("/proc")?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str()?.parse::<i32>().ok())
        .collect::<Vec<_>>();
    pids.sort_unstable();
    Ok(pids)
}

/// Whether `candidate` names the executable `name`, accepting the shapes `/proc` produces on
/// its own: a `comm` cut at 15 bytes and a name with or without `.exe`.
fn same_executable(candidate: &str, name: &str) -> bool {
    if candidate.eq_ignore_ascii_case(name) {
        return true;
    }

    // `MonsterHunterRise.exe` llega como `MonsterHunterRi`: el prefijo es el mismo ejecutable.
    if candidate.len() == COMM_LIMIT
        && name.len() > COMM_LIMIT
        && name.as_bytes()[..COMM_LIMIT].eq_ignore_ascii_case(candidate.as_bytes())
    {
        return true;
    }

    without_extension(candidate).eq_ignore_ascii_case(without_extension(name))
}

fn without_extension(executable: &str) -> &str {
    let Some(end) = executable.len().checked_sub(4) else {
        return executable;
    };

    match executable.get(end..) {
        Some(tail) if tail.eq_ignore_ascii_case(".exe") => &executable[..end],
        _ => executable,
    }
}

/// Nombre del módulo que coincide con este proceso, para resolver su base con el correcto.
fn matching_module<'a>(pid: i32, names: &'a [&'a str]) -> Option<&'a str> {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    let comm = comm.trim();

    if let Some(name) = names.iter().find(|name| same_executable(comm, name)) {
        return Some(name);
    }

    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();

    cmdline
        .split(|byte| *byte == 0)
        .filter_map(|argument| std::str::from_utf8(argument).ok())
        .find_map(|argument| {
            let executable = argument.rsplit(['/', '\\']).next().unwrap_or(argument);
            names
                .iter()
                .copied()
                .find(|name| same_executable(executable, name))
        })
}

/// Whether this pid deserves the maps scan even though nothing in its name says it is the game:
/// a Wine/Proton process, whose `/proc/<pid>/exe` is the loader rather than the game, or one
/// whose executable file is already named like a Windows binary.
///
/// Only the fallback pays for this. Scanning every `/proc/<pid>/maps` costs about 25 ms for a
/// desktop, so the filter is what keeps a probe with the game closed cheap.
fn could_host_game(pid: i32) -> bool {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    if comm.trim().to_ascii_lowercase().ends_with(".exe") {
        return true;
    }

    let Ok(executable) = fs::read_link(format!("/proc/{pid}/exe")) else {
        return false;
    };
    let Some(executable) = executable
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
    else {
        return false;
    };

    executable.ends_with(".exe")
        || ["wine", "proton", "reaper", "steam"]
            .iter()
            .any(|loader| executable.contains(loader))
}

/// Second pass for the cases where no process named the executable: the game maps its own
/// image even when Wine renamed the process or hid the path from the command line.
fn mapped_process(pids: &[i32], names: &[&str]) -> Option<Process> {
    for &pid in pids {
        if !could_host_game(pid) {
            continue;
        }

        for name in names {
            let Ok(base) = module_base(pid, name) else {
                continue;
            };

            if base == 0 {
                continue;
            }

            return Some(Process {
                pid,
                module: (*name).to_string(),
                base,
            });
        }
    }

    None
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

        if !same_executable(file_name, module) {
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

/// A process that names or maps one of the game's executables, for troubleshooting the
/// detection when the game is running but `Process::find` does not see it.
#[derive(Debug, Clone)]
pub struct Lookalike {
    pub pid: i32,
    /// `/proc/<pid>/comm`; the kernel cuts it at 15 bytes, which is worth seeing.
    pub comm: String,
    /// `/proc/<pid>/cmdline` joined with spaces.
    pub cmdline: String,
    /// The game module this process maps, with its base, if any.
    pub mapping: Option<(String, u64)>,
}

/// Every process worth looking at when the game is not detected: the ones that mention an
/// executable in their name or command line, and the ones that map it.
///
/// Reads the maps of every readable pid, so it is a diagnostic (`--ps`), not something the
/// normal detection path does.
pub fn lookalikes(names: &[&str]) -> Vec<Lookalike> {
    let Ok(pids) = process_ids() else {
        return Vec::new();
    };

    let mut found = Vec::new();

    for pid in pids {
        let mapping = names.iter().find_map(|name| {
            module_base(pid, name)
                .ok()
                .filter(|base| *base != 0)
                .map(|base| ((*name).to_string(), base))
        });

        if mapping.is_some() {
            found.push(Lookalike {
                pid,
                comm: read_comm(pid),
                cmdline: read_cmdline(pid),
                mapping,
            });
            continue;
        }

        let comm = read_comm(pid);
        let cmdline = read_cmdline(pid);
        let needle = comm.to_ascii_lowercase();
        let arguments = cmdline.to_ascii_lowercase();

        let names_it = names.iter().any(|name| {
            let stem = without_extension(name).to_ascii_lowercase();
            needle.contains(&stem) || arguments.contains(&stem)
        });

        if names_it {
            found.push(Lookalike {
                pid,
                comm,
                cmdline,
                mapping: None,
            });
        }
    }

    found
}

fn read_comm(pid: i32) -> String {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn read_cmdline(pid: i32) -> String {
    fs::read(format!("/proc/{pid}/cmdline"))
        .unwrap_or_default()
        .split(|byte| *byte == 0)
        .filter_map(|argument| std::str::from_utf8(argument).ok())
        .collect::<Vec<_>>()
        .join(" ")
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

    #[test]
    fn a_multi_name_search_resolves_the_base_of_the_name_that_matched() {
        let pid = std::process::id() as i32;
        let cmdline = fs::read(format!("/proc/{pid}/cmdline")).unwrap();
        let first_argument = cmdline
            .split(|byte| *byte == 0)
            .next()
            .and_then(|argument| std::str::from_utf8(argument).ok())
            .expect("el proceso de test tiene cmdline");
        let name = first_argument.rsplit('/').next().unwrap();

        let found = Process::find(&["no-existe-este-proceso-12345", name]).unwrap();

        assert_eq!(found.module(), name);
    }

    #[test]
    fn a_comm_cut_by_the_kernel_is_the_same_executable() {
        assert_eq!("MonsterHunterRi".len(), COMM_LIMIT);

        assert!(same_executable("MonsterHunterRi", "MonsterHunterRise.exe"));
        assert!(same_executable("monsterhunterri", "MonsterHunterRise.exe"));
    }

    #[test]
    fn an_executable_named_without_the_extension_matches() {
        assert!(same_executable(
            "MonsterHunterRise",
            "MonsterHunterRise.exe"
        ));
        assert!(same_executable(
            "MonsterHunterRise.exe",
            "MonsterHunterRise"
        ));
    }

    #[test]
    fn another_executable_or_an_empty_name_does_not_match() {
        assert!(!same_executable(
            "MonsterHunterRi",
            "MonsterHunterWorld.exe"
        ));
        assert!(!same_executable("", "MonsterHunterRise.exe"));
        assert!(!same_executable("exe", "MonsterHunterRise.exe"));
    }

    #[test]
    fn the_report_resolves_the_module_a_process_maps() {
        let pid = std::process::id() as i32;
        let name = own_executable_name();

        let seen = lookalikes(&[&name]);
        let own = seen.iter().find(|lookalike| lookalike.pid == pid);

        assert!(own.is_some_and(|lookalike| {
            lookalike
                .mapping
                .as_ref()
                .is_some_and(|(_, base)| *base > 0)
        }));
    }

    fn own_executable_name() -> String {
        let pid = std::process::id() as i32;
        let cmdline = fs::read(format!("/proc/{pid}/cmdline")).unwrap();
        let first_argument = cmdline
            .split(|byte| *byte == 0)
            .next()
            .and_then(|argument| std::str::from_utf8(argument).ok())
            .expect("el proceso de test tiene cmdline");

        first_argument.rsplit('/').next().unwrap().to_string()
    }
}
