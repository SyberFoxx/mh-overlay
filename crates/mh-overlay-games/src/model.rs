//! Estado de un monstruo, independiente del juego del que venga.

/// Vida de un componente (el monstruo entero, o una parte concreta).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(current: f32, max: f32) -> Self {
        Self { current, max }
    }

    /// Porcentaje restante, de 0 a 100.
    pub fn percent(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max) * 100.0
        } else {
            0.0
        }
    }

    /// Descarta lecturas corruptas: un float NaN o un `current > max` viene de un offset
    /// desalineado o de una estructura a medio inicializar, no de un monstruo.
    pub fn is_plausible(&self) -> bool {
        self.current.is_finite()
            && self.max.is_finite()
            && self.max > 0.0
            && self.current >= 0.0
            && self.current <= self.max * 1.01
    }
}

/// Una parte rompible o cortable del monstruo.
///
/// Rise expone las tres resistencias en arrays paralelos; World solo la vida de la parte.
#[derive(Debug, Clone, Default)]
pub struct Part {
    /// Índice de la parte en el monstruo, tal como lo reporta el juego.
    pub index: usize,
    /// Nombre corto de la parte cuando la tabla del juego la conoce.
    pub name: Option<&'static str>,
    pub flinch: Option<Health>,
    pub breakable: Option<Health>,
    pub severable: Option<Health>,
}

/// Acumulación de un estado alterado.
#[derive(Debug, Clone)]
pub struct Ailment {
    pub name: &'static str,
    pub buildup: f32,
    pub max_buildup: f32,
    pub timer: f32,
    pub max_timer: f32,
}

impl Ailment {
    /// Estados cuya acumulación no se ha movido no aportan nada a la pantalla.
    pub fn is_active(&self) -> bool {
        self.max_buildup > 0.0 && (self.buildup > 0.0 || self.timer > 0.0)
    }

    pub fn buildup_percent(&self) -> f32 {
        if self.max_buildup > 0.0 {
            (self.buildup / self.max_buildup) * 100.0
        } else {
            0.0
        }
    }
}

/// Vida y aguante del jugador local.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub health: Health,
    pub stamina: Health,
}

impl Player {
    /// Un HUD sin cargar deja ceros y negativos; eso no es un jugador al borde de la muerte.
    pub fn is_plausible(&self) -> bool {
        self.health.is_plausible() && self.health.max > 1.0 && self.stamina.max > 0.0
    }
}

/// Cacería en curso: tiempo restante y vidas del equipo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hunt {
    pub time_left: f32,
    pub deaths: u32,
    pub max_deaths: u32,
}

impl Hunt {
    pub fn is_plausible(&self) -> bool {
        self.time_left.is_finite()
            && self.time_left >= 0.0
            && self.time_left < 60.0 * 60.0 * 24.0
            && self.max_deaths > 0
            && self.deaths <= self.max_deaths
    }

    /// `12:34`; por encima de una hora se añaden las horas.
    pub fn time_label(&self) -> String {
        let total = self.time_left.ceil() as u64;
        let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);

        if hours > 0 {
            format!("{hours}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes}:{seconds:02}")
        }
    }
}

/// Todo lo que el panel puede enseñar de una lectura.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub player: Option<Player>,
    pub hunt: Option<Hunt>,
    pub monsters: Vec<Monster>,
}

impl Snapshot {
    /// Lectura sin jugador ni cacería, para pruebas y vistas previas.
    pub fn of(monsters: Vec<Monster>) -> Self {
        Self {
            player: None,
            hunt: None,
            monsters,
        }
    }
}

/// Un monstruo grande presente en la zona.
#[derive(Debug, Clone)]
pub struct Monster {
    /// Identificador interno del juego. Estable entre parches, a diferencia de las direcciones.
    pub id: i32,
    /// Nombre legible según la localización; en World, si el id no está en la tabla, se
    /// guarda el código `em` del modelo.
    pub name: Option<String>,
    pub health: Health,
    /// Porcentaje de vida por debajo del cual se puede capturar.
    pub capture_percent: Option<f32>,
    pub parts: Vec<Part>,
    pub ailments: Vec<Ailment>,
}

impl Monster {
    pub fn new(id: i32, health: Health) -> Self {
        Self {
            id,
            name: None,
            health,
            capture_percent: None,
            parts: Vec::new(),
            ailments: Vec::new(),
        }
    }

    pub fn is_capturable(&self) -> bool {
        self.capture_percent
            .is_some_and(|threshold| self.health.percent() <= threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_is_zero_without_max_health() {
        assert_eq!(Health::new(10.0, 0.0).percent(), 0.0);
    }

    #[test]
    fn percent_of_half_health_is_fifty() {
        assert!((Health::new(50.0, 100.0).percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_health_above_its_maximum_is_not_plausible() {
        assert!(!Health::new(120.0, 100.0).is_plausible());
        assert!(!Health::new(f32::NAN, 100.0).is_plausible());
        assert!(Health::new(99.0, 100.0).is_plausible());
    }

    #[test]
    fn the_hunt_clock_shows_minutes_and_seconds() {
        let hunt = Hunt {
            time_left: 754.0,
            deaths: 0,
            max_deaths: 3,
        };

        assert_eq!(hunt.time_label(), "12:34");
    }

    #[test]
    fn a_long_hunt_clock_shows_hours() {
        let hunt = Hunt {
            time_left: 3725.0,
            deaths: 1,
            max_deaths: 3,
        };

        assert_eq!(hunt.time_label(), "1:02:05");
    }

    #[test]
    fn an_empty_hud_is_not_a_plausible_player() {
        let empty = Player {
            health: Health::new(0.0, 0.0),
            stamina: Health::new(0.0, 0.0),
        };
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
        };

        assert!(!empty.is_plausible());
        assert!(player.is_plausible());
    }

    #[test]
    fn a_hunt_without_lives_left_to_show_is_not_plausible() {
        let broken = Hunt {
            time_left: 300.0,
            deaths: 4,
            max_deaths: 3,
        };

        assert!(!broken.is_plausible());
    }

    #[test]
    fn capture_is_reported_below_the_threshold() {
        let mut monster = Monster::new(1, Health::new(15.0, 100.0));
        monster.capture_percent = Some(20.0);

        assert!(monster.is_capturable());
    }
}
