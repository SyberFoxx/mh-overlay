# AGENTS.md

## Context

`mh-overlay` is a Linux-native overlay for Monster Hunter titles. It reads the game
process memory (Proton/Wine) and reports monster health, part damage and ailments.
Written in Rust 2024, workspace with three crates:

| Crate | Path | Responsibility |
| --- | --- | --- |
| `mh-overlay-memory` | `crates/mh-overlay-memory` | Process discovery, `/proc` parsing, `process_vm_readv`, pointer chains, `.map` parser |
| `mh-overlay-games` | `crates/mh-overlay-games` | Per-game readers (`rise.rs`, `world.rs`) and the shared `model` |
| `mh-overlay-cli` | `crates/mh-overlay-cli` | Binary `mh-overlay`, argument parsing, rendering |

Supported games today: Monster Hunter Rise (`MonsterHunterRise.exe`) and
Monster Hunter World (`MonsterHunterWorld.exe`), both through Proton.

## Commands

- Build: `cargo build`
- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Format: `cargo fmt --all`
- Run: `cargo run -p mh-overlay-cli -- --game rise`

## Style and conventions

- `cargo fmt` is non-negotiable; no manual formatting diffs.
- Prefer the standard library. `libc` is the only dependency of the memory crate; a new
  dependency needs a one-line justification in the commit message.
- Errors: one `MemoryError` enum with `thiserror`, variants named after the failure, not the
  syscall. Wrap the OS error in `source`.
- No `unwrap`/`expect` outside tests. No `panic!` for conditions the game can produce
  (unmapped page, null pointer, stale offsets) — those are `Err`.
- `unsafe` blocks carry a comment stating the invariant that makes them sound.
- Comments explain layout facts (`// +0x18 es el HP actual en el componente de vida`), not
  mechanics of the code.
- Public items get one line of `///` when the name alone does not convey the unit or the
  meaning of the raw value.

## Memory model (read this before touching offsets)

Three rules govern every read, and getting one wrong silently returns garbage:

1. `AddressMap::absolute(key)` is `module base + static offset`. The base comes from
   `/proc/<pid>/maps` and changes with ASLR, so it is resolved once per attach.
2. `.map` `Offset` keys describe two *different* chain shapes; they are not
   interchangeable:
   - **deref chain** (`Memory::deref_chain`): read pointer at `addr`, then `addr = ptr + offset`,
     per step. The final value is an address, not a value.
   - **ptr chain** (`Memory::follow_ptr_chain`): `addr = addr + offset`, read pointer there,
     per step.
   HunterPie's `ReadAsync(address, offsets)` maps to the first, `ReadPtrAsync` to the second.
   When porting a read from HunterPie, copy the chain shape from the C# call it came from.
3. Floats are IEEE-754 `f32` and little-endian. Health components store `max` at `+0x18` on
   the component and `current` at `+0x18` of the encoded pointer.

## Work rules

- Never write to game memory. The crate exposes reads only; adding a writer is a product
  decision, not a refactor.
- Offsets belong in `data/maps/`, never hardcoded in Rust, except the per-struct field
  offsets that HunterPie also hardcodes (documented inline with the game version they were
  verified against).
- A new game is one module in `mh-overlay-games` plus one `GameKind` variant. No `match` on
  the game name inside `main`.
- A `.map` file that cannot be parsed is an error, never a silent fallback to another
  version: wrong offsets read plausible garbage.
- Update `README.md` (status table) in the same change that alters what a game can report.
- Commits: conventional prefix (`feat`, `fix`, `docs`, `chore`), one logical change, and the
  message says why.

## Environment facts

- `kernel.yama.ptrace_scope=1` on a default Arch install blocks `process_vm_readv` against a
  non-descendant process. The binary needs `CAP_SYS_PTRACE` (`setcap`) or the sysctl
  lowered. This is the most common "attach works but every read fails" cause.
- Under Proton the game is a normal Linux process; `/proc/<pid>/exe` points at the Wine
  loader, so process discovery matches on `/proc/<pid>/comm` and on the basename of every
  `cmdline` argument.
- Games under Proton are XWayland windows. Under a Wayland compositor that window is composed
  above any other X11 window when fullscreen, so the panel has two backends: an X11
  `override-redirect` window by default and a `wlr-layer-shell` surface on the `overlay` layer
  when `WAYLAND_DISPLAY` is set. The layer surface is created without an output so the
  compositor places it on the active monitor, and compositors without layer-shell fall back
  to X11. GNOME does not implement layer-shell, so there is no overlay layer to use there.
