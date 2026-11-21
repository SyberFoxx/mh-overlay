//! Hechos del sistema que deciden si una lectura puede funcionar, sin abrir el juego.
//!
//! Están juntos aquí porque son la mitad del diagnóstico del proyecto: la otra mitad es el
//! proceso y el `.map`. `--doctor` los imprime tal cual para que no haya que leer `/proc` a mano.

use std::fs;

/// Bit de `CAP_SYS_PTRACE` en la máscara efectiva; es la que activa `setcap`.
pub const CAP_SYS_PTRACE: u32 = 19;

/// Valor de `kernel.yama.ptrace_scope`; `None` si el kernel no tiene Yama.
pub fn ptrace_scope() -> Option<u8> {
    fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope")
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// `true` si este proceso lleva `CAP_SYS_PTRACE` efectiva.
pub fn has_cap_sys_ptrace() -> bool {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| capability_from_status(&status, CAP_SYS_PTRACE))
        .unwrap_or(false)
}

/// Si el bit `bit` está en la línea `CapEff:` de `/proc/<pid>/status`.
///
/// `CapEff` es una máscara hexadecimal de 64 bits aunque la mayoría de bits sean cero.
fn capability_from_status(status: &str, bit: u32) -> Option<bool> {
    let line = status.lines().find(|line| line.starts_with("CapEff:"))?;
    let value = u64::from_str_radix(line["CapEff:".len()..].trim(), 16).ok()?;

    Some(value & (1_u64 << bit) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ptrace_capability_is_read_from_the_mask() {
        let status = "Name:\tmh-overlay\nCapEff:\t0000000000080000\n";

        assert_eq!(capability_from_status(status, CAP_SYS_PTRACE), Some(true));
    }

    #[test]
    fn a_plain_mask_does_not_have_the_capability() {
        let status = "Name:\tmh-overlay\nCapEff:\t0000000000000000\n";

        assert_eq!(capability_from_status(status, CAP_SYS_PTRACE), Some(false));
    }

    #[test]
    fn another_bit_does_not_count_as_ptrace() {
        // CAP_CHOWN (bit 0), no CAP_SYS_PTRACE.
        let status = "CapEff:\t0000000000000001\n";

        assert_eq!(capability_from_status(status, CAP_SYS_PTRACE), Some(false));
    }

    #[test]
    fn a_status_without_the_mask_is_not_guessed() {
        assert_eq!(
            capability_from_status("Name:\tmh-overlay\n", CAP_SYS_PTRACE),
            None
        );
    }
}
