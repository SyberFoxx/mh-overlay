//! Lectura de memoria de otro proceso Linux y resolución de los address maps de Monster Hunter.
//!
//! Módulo de bajo nivel: sabe *leer* y sabe *dónde* están las cosas, pero no conoce
//! la estructura de ningún juego. Eso vive en `mh-overlay-games`.

pub mod address_map;
pub mod env;
pub mod error;
pub mod memory;
pub mod process;

pub use address_map::{AddressMap, MapValidation, SelectedMap, maps_directory};
pub use error::MemoryError;
pub use memory::{Memory, Reads};
pub use process::{Lookalike, Process, lookalikes};
