//! Estado de un monstruo, independiente del juego del que venga.

/// Vida de un componente (el monstruo entero, o una parte concreta).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
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
    /// Vida real de la parte tal como la guarda el juego. En World es independiente del
    /// progreso de rompimiento, que se escala a los golpes del siguiente tramo.
    pub health: Option<Health>,
    /// Rompimientos hechos y totales, cuando la parte se rompe por golpes acumulados. El
    /// porcentaje de `breakable` ya los incluye; esto deja el marcador "1/2" en la barra.
    pub breaks: Option<(u32, u32)>,
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

/// Corona de tamaño que el juego registra para un monstruo.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Crown {
    /// Tamaño normal: sin corona.
    #[default]
    None,
    /// Corona miniatura.
    Mini,
    /// Corona plateada.
    Silver,
    /// Corona dorada.
    Gold,
}

impl Crown {
    /// Clasifica un multiplicador de tamaño con los umbrales del monstruo.
    ///
    /// Un tamaño justo en el umbral ya cuenta, igual que en HunterPie: `>=` para las coronas
    /// grandes y `<=` para la miniatura.
    pub fn of(multiplier: f32, thresholds: CrownThresholds) -> Self {
        if !multiplier.is_finite() {
            return Self::None;
        }

        if thresholds.gold.is_some_and(|gold| multiplier >= gold) {
            Self::Gold
        } else if multiplier >= thresholds.silver {
            Self::Silver
        } else if thresholds.mini.is_some_and(|mini| multiplier <= mini) {
            Self::Mini
        } else {
            Self::None
        }
    }

    /// Etiqueta legible de la corona; `None` cuando no hay ninguna.
    pub fn label(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Mini => Some("corona miniatura"),
            Self::Silver => Some("corona plateada"),
            Self::Gold => Some("corona dorada"),
        }
    }
}

/// Umbrales de corona de un monstruo, como multiplicador del tamaño base.
///
/// Rise deja sin umbral la corona que ese monstruo no puede alcanzar (por ejemplo, Elder
/// Dragons sin talla miniatura), así que `mini` y `gold` son opcionales.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrownThresholds {
    pub mini: Option<f32>,
    pub silver: f32,
    pub gold: Option<f32>,
}

/// Arma equipada, en el orden en que el juego numera los tipos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Greatsword,
    SwordAndShield,
    DualBlades,
    Longsword,
    Hammer,
    HuntingHorn,
    Lance,
    Gunlance,
    SwitchAxe,
    ChargeBlade,
    InsectGlaive,
    LightBowgun,
    HeavyBowgun,
    Bow,
}

impl WeaponKind {
    /// Tipo de arma tal como lo guarda el juego (`byte` al principio del bloque de arma).
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Self::Greatsword,
            1 => Self::SwordAndShield,
            2 => Self::DualBlades,
            3 => Self::Longsword,
            4 => Self::Hammer,
            5 => Self::HuntingHorn,
            6 => Self::Lance,
            7 => Self::Gunlance,
            8 => Self::SwitchAxe,
            9 => Self::ChargeBlade,
            10 => Self::InsectGlaive,
            11 => Self::LightBowgun,
            12 => Self::HeavyBowgun,
            13 => Self::Bow,
            _ => return None,
        })
    }

    /// Tipo de arma tal como lo reporta Rise (`WEAPON_ADDRESS` + `0x8C`): usa otro orden que
    /// World, así que no comparte [`Self::from_id`].
    pub fn from_rise_id(id: i32) -> Option<Self> {
        Some(match id {
            0 => Self::Greatsword,
            1 => Self::SwitchAxe,
            2 => Self::Longsword,
            3 => Self::LightBowgun,
            4 => Self::HeavyBowgun,
            5 => Self::Hammer,
            6 => Self::Gunlance,
            7 => Self::Lance,
            8 => Self::SwordAndShield,
            9 => Self::DualBlades,
            10 => Self::HuntingHorn,
            11 => Self::ChargeBlade,
            12 => Self::InsectGlaive,
            13 => Self::Bow,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Greatsword => "Espadón",
            Self::SwordAndShield => "Espada y escudo",
            Self::DualBlades => "Espadas dobles",
            Self::Longsword => "Espada larga",
            Self::Hammer => "Martillo",
            Self::HuntingHorn => "Cornamusa",
            Self::Lance => "Lanza",
            Self::Gunlance => "Lanza pistola",
            Self::SwitchAxe => "Hacha espada",
            Self::ChargeBlade => "Espada cargada",
            Self::InsectGlaive => "Glaive insecto",
            Self::LightBowgun => "Ballesta ligera",
            Self::HeavyBowgun => "Ballesta pesada",
            Self::Bow => "Arco",
        }
    }

    /// Solo las armas con filo lo gastan; las de proyectiles no.
    pub fn has_sharpness(self) -> bool {
        !matches!(self, Self::LightBowgun | Self::HeavyBowgun | Self::Bow)
    }
}

/// Colores del filo, de peor a mejor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharpnessLevel {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    White,
    Purple,
}

impl SharpnessLevel {
    pub const ALL: [Self; 7] = [
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::White,
        Self::Purple,
    ];

    pub fn from_index(index: i32) -> Option<Self> {
        usize::try_from(index)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Red => "rojo",
            Self::Orange => "naranja",
            Self::Yellow => "amarillo",
            Self::Green => "verde",
            Self::Blue => "azul",
            Self::White => "blanco",
            Self::Purple => "púrpura",
        }
    }
}

/// Filo del arma: cuánto mide cada color y en qué punto del total está el arma.
///
/// `lengths` son las longitudes que el juego da a cada color en esta arma concreta, en el
/// mismo orden que [`SharpnessLevel::ALL`]; el índice `level` es el color más alto que el arma
/// alcanza.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sharpness {
    pub lengths: [u16; 7],
    pub level: SharpnessLevel,
    pub max_level: SharpnessLevel,
    pub current: f32,
}

impl Sharpness {
    pub fn total(&self) -> f32 {
        self.lengths.iter().map(|length| f32::from(*length)).sum()
    }

    /// Longitud acumulada hasta el principio del color indicado.
    pub fn start_of(&self, level: SharpnessLevel) -> f32 {
        let position = SharpnessLevel::ALL
            .iter()
            .position(|candidate| *candidate == level)
            .unwrap_or(0);

        self.lengths
            .iter()
            .take(position)
            .map(|length| f32::from(*length))
            .sum()
    }

    /// Fracción de 0 a 1 en la que está el filo actual.
    pub fn fraction(&self) -> f32 {
        let total = self.total();
        if total <= 0.0 {
            return 0.0;
        }

        ((self.start_of(self.level) + self.current) / total).clamp(0.0, 1.0)
    }
}

/// Esencias del glaive insecto que el jugador lleva activas.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Extracts {
    pub attack: bool,
    pub speed: bool,
    pub defense: bool,
    pub heal: bool,
}

impl Extracts {
    /// Etiqueta corta con las esencias activas, en el orden en que el juego las muestra.
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.attack {
            parts.push("rojo");
        }
        if self.speed {
            parts.push("blanco");
        }
        if self.defense {
            parts.push("naranja");
        }
        if self.heal {
            parts.push("verde");
        }

        if parts.is_empty() {
            "sin esencias".to_string()
        } else {
            parts.join(" + ")
        }
    }
}

/// Medidores propios de cada clase de arma. Cada juego y cada arma rellenan el suyo; las
/// armas que no tienen un medidor extra (espada y escudo, lanza, martillo...) se quedan sin él.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponGauge {
    /// Espada larga: medidor de espíritu y el temporizador del nivel alcanzado.
    Spirit {
        buildup: f32,
        level: u8,
        timer: f32,
        max_timer: f32,
        regen: f32,
    },
    /// Espada cargada: frascos, carga para llenarlos y los tres buffs de la hoja.
    ChargeBlade {
        phials: u8,
        max_phials: u8,
        charge: f32,
        shield: f32,
        sword: f32,
        axe: f32,
    },
    /// Hacha espada: carga de la hoja, carga amped y su temporizador.
    SwitchAxe {
        buildup: f32,
        charge: f32,
        timer: f32,
    },
    /// Espadas dobles: modo demonio, modo archidemonio y su medidor.
    DualBlades {
        demon: bool,
        arch_demon: bool,
        buildup: f32,
    },
    /// Glaive insecto: esencias, tiempos de cada color y aguante del kinsecto.
    InsectGlaive {
        extracts: Extracts,
        attack: f32,
        speed: f32,
        defense: f32,
        stamina: f32,
        max_stamina: f32,
    },
}

/// Proyectil cargado o disponible en una ballesta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmmoSlot {
    /// Índice dentro de la lista de munición del juego.
    pub index: usize,
    /// Nombre cuando la tabla de munición lo conoce.
    pub name: Option<&'static str>,
    /// Balas cargadas en el arma.
    pub loaded: u32,
    /// Balas que quedan en la bolsa.
    pub total: u32,
    /// Capacidad máxima de la bolsa para este tipo.
    pub max: u32,
}

/// Munición de una ballesta y la que está seleccionada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangedState {
    pub ammo: Vec<AmmoSlot>,
    /// Índice de la munición equipada, si el arma ya está cargada.
    pub equipped: Option<usize>,
}

impl RangedState {
    /// Munición equipada, si el índice apunta a una entrada válida.
    pub fn selected(&self) -> Option<&AmmoSlot> {
        self.equipped
            .and_then(|index| self.ammo.iter().find(|slot| slot.index == index))
    }
}

/// Estado de la lanza pistola: proyectiles normales, cargados y el dragón de fuego.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GunlanceState {
    pub shells: u32,
    pub max_shells: u32,
    pub charged: u32,
    pub max_charged: u32,
    /// Segundos que quedan para poder volver a usar el dragón de fuego.
    pub wyvern_fire: f32,
    pub stake_loaded: bool,
    pub stake_timer: f32,
    pub stake_max: f32,
}

/// Color de una nota de cornamusa, en el orden que usa World.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HornNote {
    Purple,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    LightBlue,
    White,
}

impl HornNote {
    pub fn from_index(index: i32) -> Option<Self> {
        Some(match index {
            0 => Self::Purple,
            1 => Self::Red,
            2 => Self::Orange,
            3 => Self::Yellow,
            4 => Self::Green,
            5 => Self::Blue,
            6 => Self::LightBlue,
            7 => Self::White,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Purple => "púrpura",
            Self::Red => "roja",
            Self::Orange => "naranja",
            Self::Yellow => "amarilla",
            Self::Green => "verde",
            Self::Blue => "azul",
            Self::LightBlue => "azul claro",
            Self::White => "blanca",
        }
    }
}

/// Melodía que la cornamusa puede tocar o que ya está en la cola.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HornSong {
    pub id: u32,
    pub notes: [u8; 4],
    pub length: u8,
}

impl HornSong {
    pub fn notes_label(&self) -> String {
        self.notes
            .iter()
            .take(usize::from(self.length).min(self.notes.len()))
            .map(|note| note.to_string())
            .collect::<Vec<_>>()
            .join("·")
    }
}

/// Cola de notas y melodías de la cornamusa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HornState {
    pub notes: Vec<HornNote>,
    pub songs: Vec<HornSong>,
    pub available: Vec<HornSong>,
    /// Índice de la melodía que se está tocando, si el juego lo expone.
    pub playing: Option<usize>,
}

/// Carga del arco, que es lo que el juego guarda junto al arma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BowState {
    pub charge: f32,
    pub level: u32,
    pub max_level: u32,
}

/// Datos de combate propios de la clase de arma que no caben en [`WeaponGauge`].
///
/// Los campos que el juego no expone en la versión soportada quedan en `None`: es preferible
/// no enseñar un dato a inventarlo.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeaponDetail {
    pub attack: Option<f32>,
    pub affinity: Option<i32>,
    pub element: Option<f32>,
    pub ranged: Option<RangedState>,
    pub gunlance: Option<GunlanceState>,
    pub horn: Option<HornState>,
    pub bow: Option<BowState>,
}

impl WeaponDetail {
    pub fn is_empty(&self) -> bool {
        self.attack.is_none()
            && self.affinity.is_none()
            && self.element.is_none()
            && self.ranged.is_none()
            && self.gunlance.is_none()
            && self.horn.is_none()
            && self.bow.is_none()
    }
}

/// Arma equipada y su estado.
#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    pub kind: WeaponKind,
    pub sharpness: Option<Sharpness>,
    pub gauge: Option<WeaponGauge>,
    pub detail: WeaponDetail,
}

/// Buff o estado activo sobre el jugador, con el tiempo que le queda.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Buff {
    pub name: &'static str,
    pub timer: f32,
    /// Duración máxima vista desde que empezó; crece mientras el buff se renueva.
    pub max_timer: f32,
    /// `true` para los estados negativos (veneno, fuego, sangrado...).
    pub debuff: bool,
    /// `true` para buffs sin temporizador propio (drogas): se muestran como "∞".
    pub infinite: bool,
}

impl Buff {
    /// Fracción de 0 a 1 que le queda al buff, para la barra.
    pub fn fraction(&self) -> f32 {
        if self.infinite {
            return 1.0;
        }
        if self.max_timer <= 0.0 {
            return 0.0;
        }

        (self.timer / self.max_timer).clamp(0.0, 1.0)
    }
}

/// Vida y aguante del jugador local.
#[derive(Debug, Clone, Default)]
pub struct Player {
    pub health: Health,
    pub stamina: Health,
    /// Vida recuperable (la parte roja de la barra); el HUD no siempre la expone.
    pub recoverable: Option<f32>,
    pub weapon: Option<Weapon>,
    /// Buffs y estados activos, ya filtrados y ordenados por tiempo restante.
    pub buffs: Vec<Buff>,
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
    /// Corona de tamaño, si el monstruo tiene tamaños variables y este es excepcional.
    pub crown: Crown,
    /// El juego marca este monstruo como objetivo (fijado en el mapa o de la cacería).
    pub target: bool,
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
            crown: Crown::None,
            target: false,
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
    fn crowns_follow_the_monster_thresholds() {
        let thresholds = CrownThresholds {
            mini: Some(0.9),
            silver: 1.1,
            gold: Some(1.2),
        };

        assert_eq!(Crown::of(1.05, thresholds), Crown::None);
        assert_eq!(Crown::of(1.2, thresholds), Crown::Gold);
        assert_eq!(Crown::of(1.1, thresholds), Crown::Silver);
        assert_eq!(Crown::of(0.9, thresholds), Crown::Mini);
    }

    #[test]
    fn a_monster_without_a_gold_threshold_never_shows_one() {
        let thresholds = CrownThresholds {
            mini: None,
            silver: 1.17,
            gold: None,
        };

        assert_eq!(Crown::of(2.0, thresholds), Crown::Silver);
        assert_eq!(Crown::of(0.5, thresholds), Crown::None);
        assert_eq!(Crown::of(f32::NAN, thresholds), Crown::None);
    }

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
            recoverable: None,
            weapon: None,
            buffs: Vec::new(),
        };
        let player = Player {
            health: Health::new(800.0, 1000.0),
            stamina: Health::new(100.0, 150.0),
            recoverable: None,
            weapon: None,
            buffs: Vec::new(),
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
    fn sharpness_fraction_walks_the_colors() {
        let sharpness = Sharpness {
            lengths: [50, 50, 0, 0, 0, 0, 0],
            level: SharpnessLevel::Orange,
            max_level: SharpnessLevel::Orange,
            current: 25.0,
        };

        assert!((sharpness.fraction() - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn a_sharpness_without_lengths_has_no_position() {
        let sharpness = Sharpness {
            lengths: [0; 7],
            level: SharpnessLevel::Red,
            max_level: SharpnessLevel::Red,
            current: 10.0,
        };

        assert_eq!(sharpness.fraction(), 0.0);
    }

    #[test]
    fn a_buff_bar_shrinks_with_the_timer_and_infinite_ones_stay_full() {
        let buff = Buff {
            name: "Semilla del poder",
            timer: 45.0,
            max_timer: 180.0,
            debuff: false,
            infinite: false,
        };
        assert!((buff.fraction() - 0.25).abs() < f32::EPSILON);

        let drug = Buff {
            infinite: true,
            ..buff
        };
        assert_eq!(drug.fraction(), 1.0);
    }

    #[test]
    fn extracts_read_as_the_colours_the_game_shows() {
        assert_eq!(Extracts::default().label(), "sin esencias");
        assert_eq!(
            Extracts {
                attack: true,
                speed: true,
                defense: false,
                heal: false,
            }
            .label(),
            "rojo + blanco"
        );
    }

    #[test]
    fn every_weapon_type_the_game_sends_has_a_name() {
        for id in 0..14 {
            assert!(WeaponKind::from_id(id).is_some(), "tipo {id} sin nombre");
        }
        assert!(WeaponKind::from_id(14).is_none());
    }

    #[test]
    fn ranged_weapons_do_not_spend_sharpness() {
        assert!(!WeaponKind::Bow.has_sharpness());
        assert!(!WeaponKind::HeavyBowgun.has_sharpness());
        assert!(WeaponKind::Longsword.has_sharpness());
    }

    #[test]
    fn capture_is_reported_below_the_threshold() {
        let mut monster = Monster::new(1, Health::new(15.0, 100.0));
        monster.capture_percent = Some(20.0);

        assert!(monster.is_capturable());
    }
}
