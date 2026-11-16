use std::io;

/// Everything that can go wrong while attaching to a game and reading its memory.
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("no hay ningún proceso en ejecución que coincida con: {names}")]
    ProcessNotFound { names: String },

    #[error("no se pudo leer /proc/{pid}/maps: {source}")]
    MapsUnreadable {
        pid: i32,
        #[source]
        source: io::Error,
    },

    #[error("no se encontró la base del módulo '{module}' en los mapas del pid {pid}")]
    ModuleBaseNotFound { pid: i32, module: String },

    #[error("lectura de {length} bytes en 0x{address:016X} rechazada: {source}")]
    Read {
        address: u64,
        length: usize,
        #[source]
        source: io::Error,
    },

    #[error("lectura parcial en 0x{address:016X}: se esperaban {expected} bytes, llegaron {read}")]
    ShortRead {
        address: u64,
        expected: usize,
        read: usize,
    },

    #[error("puntero nulo en el paso {step} de la cadena")]
    NullPointer { step: usize },

    #[error("clave '{key}' ausente en el address map")]
    MissingKey { key: String },

    #[error("la clave '{key}' no es un {expected}")]
    WrongKeyType { key: String, expected: &'static str },

    #[error("address map inválido en la línea {line}: {reason}")]
    InvalidMapLine { line: usize, reason: String },

    #[error("no hay address maps de '{game}' en {directory}")]
    NoMapsForGame { game: String, directory: String },

    #[error(transparent)]
    Io(#[from] io::Error),
}
