use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Niveaux de verdict
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// Vital pour Windows : ne JAMAIS toucher (blocage serveur).
    Critical,
    /// Composant Windows normal, à laisser tel quel.
    System,
    /// Utile (pilote, sécurité, usage quotidien) : à garder.
    Keep,
    /// Dépend de ton usage : passage en manuel possible.
    Optional,
    /// Recommandé à alléger (démarrage auto superflu, télémétrie...).
    Trim,
    /// Inutile dans la quasi-totalité des cas.
    Junk,
    /// Inconnu : ni conseillé ni déconseillé.
    Unknown,
}

impl Level {
    pub fn rank(self) -> u8 {
        match self {
            Level::Junk => 0,
            Level::Trim => 1,
            Level::Optional => 2,
            Level::Unknown => 3,
            Level::Keep => 4,
            Level::System => 5,
            Level::Critical => 6,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Level::Critical => "Vital",
            Level::System => "Système",
            Level::Keep => "À garder",
            Level::Optional => "Selon usage",
            Level::Trim => "À alléger",
            Level::Junk => "Superflu",
            Level::Unknown => "Non répertorié",
        }
    }
    /// Sélectionné par défaut dans le plan d'optimisation ?
    pub fn preselected(self) -> bool {
        matches!(self, Level::Junk | Level::Trim)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Advice {
    pub level: Level,
    pub label: String,
    pub reason: String,
    /// Action conseillée (None = ne rien faire).
    pub action: Option<Action>,
    /// Octets de RAM réellement récupérables tout de suite (estimation).
    pub gain: u64,
    /// Octets qui seront libérés au prochain démarrage (non comptés dans gain).
    pub gain_next_boot: u64,
}

impl Advice {
    pub fn new(level: Level, reason: &str) -> Self {
        Advice {
            level,
            label: level.label().to_string(),
            reason: reason.to_string(),
            action: None,
            gain: 0,
            gain_next_boot: 0,
        }
    }
    pub fn with_action(mut self, a: Action) -> Self {
        self.action = Some(a);
        self
    }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StartType {
    Auto,
    Delayed,
    Manual,
    Disabled,
}

impl StartType {
    pub fn label(self) -> &'static str {
        match self {
            StartType::Auto => "Automatique",
            StartType::Delayed => "Auto (différé)",
            StartType::Manual => "Manuel",
            StartType::Disabled => "Désactivé",
        }
    }
    #[allow(dead_code)]
    pub fn code(self) -> u32 {
        match self {
            StartType::Auto | StartType::Delayed => 2,
            StartType::Manual => 3,
            StartType::Disabled => 4,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    /// Changer le type de démarrage d'un service.
    Service { name: String, start: StartType },
    /// Activer / désactiver une entrée de démarrage (mécanisme Gestionnaire des tâches).
    Startup { id: String, enabled: bool },
    /// Activer / désactiver une tâche planifiée.
    Task { path: String, enabled: bool },
    /// Terminer un processus (irréversible, hors rollback).
    Kill { pid: u32, name: String },
    /// Arrêter un service maintenant (libère la RAM immédiatement).
    StopService { name: String },
    /// Démarrer un service (utilisé par la restauration).
    StartService { name: String },
}

impl Action {
    pub fn describe(&self) -> String {
        match self {
            Action::Service { name, start } => {
                format!("Service « {name} » → {}", start.label())
            }
            Action::Startup { id, enabled } => format!(
                "Démarrage « {} » → {}",
                id.rsplit('|').next().unwrap_or(id),
                if *enabled { "activé" } else { "désactivé" }
            ),
            Action::Task { path, enabled } => format!(
                "Tâche « {path} » → {}",
                if *enabled { "activée" } else { "désactivée" }
            ),
            Action::Kill { pid, name } => format!("Terminer {name} (PID {pid})"),
            Action::StopService { name } => format!("Arrêter le service « {name} » maintenant"),
            Action::StartService { name } => format!("Redémarrer le service « {name} »"),
        }
    }
    #[allow(dead_code)]
    pub fn reversible(&self) -> bool {
        !matches!(self, Action::Kill { .. })
    }
}

// ---------------------------------------------------------------------------
// Éléments collectés
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub description: String,
    pub company: String,
    pub path: String,
    /// Mémoire totale vue par le Gestionnaire des tâches (working set).
    pub working_set: u64,
    /// Mémoire privée : celle réellement libérée si on ferme le processus.
    pub private: u64,
    pub threads: u32,
    pub has_window: bool,
    /// Services hébergés (svchost / taskhost).
    pub services: Vec<String>,
    pub category: String,
    pub advice: Advice,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceInfo {
    pub name: String,
    pub display: String,
    pub description: String,
    pub running: bool,
    pub state: String,
    pub start: StartType,
    pub pid: u32,
    pub path: String,
    pub dll: String,
    pub account: String,
    /// RAM du processus hôte.
    pub host_ram: u64,
    /// Nombre de services partageant le même hôte.
    pub host_shared: u32,
    /// Part de RAM attribuée à ce service.
    pub ram: u64,
    pub protected: bool,
    pub advice: Advice,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StartupItem {
    /// Identifiant technique unique : "hkcu_run|Nom", "folder|fichier.lnk"...
    pub id: String,
    pub name: String,
    pub command: String,
    pub location: String,
    pub enabled: bool,
    /// RAM du processus correspondant s'il tourne actuellement.
    pub ram: u64,
    pub advice: Advice,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskItem {
    pub path: String,
    pub name: String,
    pub enabled: bool,
    pub triggers: Vec<String>,
    pub command: String,
    pub author: String,
    pub description: String,
    pub advice: Advice,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub load: u32,
    pub commit_used: u64,
    pub commit_limit: u64,
    pub kernel: u64,
    pub cache: u64,
    pub compressed: u64,
    pub sum_private: u64,
    pub sum_working_set: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meta {
    pub computer: String,
    pub user: String,
    pub os: String,
    pub uptime_secs: u64,
    pub is_admin: bool,
    pub demo: bool,
    pub collected_at: String,
    pub duration_ms: u64,
    pub version: String,
    pub notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Snapshot {
    pub meta: Meta,
    pub memory: MemoryInfo,
    pub processes: Vec<ProcessInfo>,
    pub services: Vec<ServiceInfo>,
    pub startup: Vec<StartupItem>,
    pub tasks: Vec<TaskItem>,
    pub plan_gain: u64,
    pub plan_gain_next_boot: u64,
    pub counts: Counts,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Counts {
    pub processes: usize,
    pub services: usize,
    pub services_auto: usize,
    pub services_running: usize,
    pub startup_enabled: usize,
    pub tasks_enabled: usize,
    pub actionable: usize,
}

// ---------------------------------------------------------------------------
// Rollback
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RollbackEntry {
    pub label: String,
    pub done: Action,
    pub undo: Option<Action>,
    pub ok: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RollbackPoint {
    pub id: String,
    pub created_at: String,
    pub label: String,
    pub entries: Vec<RollbackEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionResult {
    pub label: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplyResponse {
    pub results: Vec<ActionResult>,
    pub rollback_id: Option<String>,
}
