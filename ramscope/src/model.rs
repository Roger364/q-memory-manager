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

/// Nature d'un gain : mesuré avant/après, estimé à partir d'une mesure, ou
/// indisponible faute de mesure.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum GainNature {
    #[default]
    Unavailable,
    Estimated,
    Measured,
}

impl GainNature {
    pub fn label(self) -> &'static str {
        match self {
            GainNature::Measured => "mesuré",
            GainNature::Estimated => "estimé",
            GainNature::Unavailable => "non chiffrable",
        }
    }
}

/// Portée d'un gain : mémoire privée, working set, part d'un hôte partagé,
/// RAM d'un hôte entier, ou mélange de plusieurs portées.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum GainScope {
    #[default]
    Private,
    WorkingSet,
    HostPart,
    Host,
    System,
    Mixed,
}

/// Mesure de gain typée : jamais un entier nu présenté comme une vérité.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Gain {
    pub bytes: u64,
    pub nature: GainNature,
    pub scope: GainScope,
    /// Vrai si ce gain peut être additionné à ceux d'éléments distincts.
    pub additive: bool,
}

impl Gain {
    pub fn unavailable() -> Self {
        Gain::default()
    }

    pub fn estimated(bytes: u64, scope: GainScope) -> Self {
        if bytes == 0 {
            return Gain::unavailable();
        }
        Gain {
            bytes,
            nature: GainNature::Estimated,
            scope,
            additive: matches!(scope, GainScope::Private | GainScope::WorkingSet),
        }
    }

    /// Un zéro mesuré reste une mesure : il ne doit pas être confondu avec
    /// l'absence de mesure.
    pub fn measured(bytes: u64, scope: GainScope) -> Self {
        Gain {
            bytes,
            nature: GainNature::Measured,
            scope,
            additive: matches!(scope, GainScope::Private | GainScope::WorkingSet),
        }
    }

    /// Additionne un gain à un total. Un gain indisponible n'ajoute rien.
    ///
    /// Un zéro mesuré est une observation valide : il peut initialiser un
    /// total vide ou confirmer une mesure, mais il ne transforme pas un total
    /// estimé en mesure. Un zéro estimé reste indisponible.
    pub fn merge(&mut self, other: &Gain) {
        if other.nature == GainNature::Unavailable {
            return;
        }
        if other.nature == GainNature::Estimated && other.bytes == 0 {
            return;
        }
        if self.nature == GainNature::Unavailable {
            *self = *other;
            self.additive = false;
            return;
        }
        if other.bytes == 0 {
            return;
        }
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.nature = if self.nature == GainNature::Measured && other.nature == GainNature::Measured
        {
            GainNature::Measured
        } else {
            GainNature::Estimated
        };
        if self.scope != other.scope {
            self.scope = GainScope::Mixed;
        }
        self.additive = false;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Advice {
    pub level: Level,
    pub label: String,
    pub reason: String,
    /// Provenance de la règle nommée ; vide pour un conseil de repli.
    #[serde(default)]
    pub source: String,
    /// Date de revue de la règle, ou `non datée` quand elle n'a pas eu lieu.
    #[serde(default)]
    pub reviewed: String,
    /// Action conseillée (None = ne rien faire).
    pub action: Option<Action>,
    /// Verrou de sécurité, distinct de la gravité : un élément verrouillé
    /// peut être `Keep` ou `System` sans être vital.
    #[serde(default)]
    pub locked: bool,
    /// Gain récupérable tout de suite, avec sa nature et sa portée.
    pub gain: Gain,
    /// Gain récupérable au prochain démarrage, jamais compté dans `gain`.
    pub gain_next_boot: Gain,
}

impl Advice {
    pub fn new(level: Level, reason: &str) -> Self {
        Advice {
            level,
            label: level.label().to_string(),
            reason: reason.to_string(),
            source: String::new(),
            reviewed: String::new(),
            action: None,
            locked: false,
            gain: Gain::unavailable(),
            gain_next_boot: Gain::unavailable(),
        }
    }
    pub fn with_action(mut self, a: Action) -> Self {
        self.action = Some(a);
        self
    }
    /// Reprend la provenance de la règle qui a produit ce conseil.
    pub fn with_provenance(mut self, source: &str, reviewed: &str) -> Self {
        self.source = source.to_string();
        self.reviewed = reviewed.to_string();
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
    Service {
        name: String,
        start: StartType,
        /// État constaté lors de l'analyse ; `None` pour une action de restauration.
        #[serde(default)]
        from: Option<StartType>,
    },
    /// Activer / désactiver une entrée de démarrage (mécanisme Gestionnaire des tâches).
    Startup {
        id: String,
        enabled: bool,
        /// État constaté lors de l'analyse ; `None` pour une action de restauration.
        #[serde(default)]
        was_enabled: Option<bool>,
    },
    /// Activer / désactiver une tâche planifiée.
    Task {
        path: String,
        enabled: bool,
        /// État constaté lors de l'analyse ; `None` pour une action de restauration.
        #[serde(default)]
        was_enabled: Option<bool>,
    },
    /// Terminer un processus (irréversible, hors rollback).
    Kill {
        pid: u32,
        name: String,
        /// Chemin de l'image au moment de l'analyse (identité de la cible).
        #[serde(default)]
        image_path: String,
        /// Heure de création FILETIME (100 ns depuis 1601) ; 0 = inconnue.
        #[serde(default)]
        created_at: u64,
    },
    /// Arrêter un service maintenant (libère la RAM immédiatement).
    StopService { name: String },
    /// Démarrer un service (utilisé par la restauration).
    StartService { name: String },
}

impl Action {
    pub fn describe(&self) -> String {
        match self {
            Action::Service { name, start, .. } => {
                format!("Service « {name} » → {}", start.label())
            }
            Action::Startup { id, enabled, .. } => format!(
                "Démarrage « {} » → {}",
                id.rsplit('|').next().unwrap_or(id),
                if *enabled { "activé" } else { "désactivé" }
            ),
            Action::Task { path, enabled, .. } => format!(
                "Tâche « {path} » → {}",
                if *enabled { "activée" } else { "désactivée" }
            ),
            Action::Kill { pid, name, .. } => format!("Terminer {name} (PID {pid})"),
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
    /// Heure de création FILETIME (100 ns depuis 1601) ; 0 si indisponible.
    #[serde(default)]
    pub created_at: u64,
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
    /// Vrai si cette session autorise une demande d'élévation depuis
    /// l'interface. Faux en mode `--no-admin`, en démonstration et dans une
    /// instance déjà élevée.
    pub can_elevate: bool,
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
    pub plan_gain: Gain,
    pub plan_gain_next_boot: Gain,
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
pub struct MemoryDelta {
    pub available_before: u64,
    pub available_after: u64,
    /// Gain mesuré après l'application ; zéro si la mémoire n'a pas augmenté.
    pub gain: Gain,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplyResponse {
    pub results: Vec<ActionResult>,
    pub rollback_id: Option<String>,
    /// Mesure disponible/après uniquement quand un arrêt immédiat a été demandé.
    #[serde(default)]
    pub memory: Option<MemoryDelta>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_zero_mesure_survit_au_merge() {
        let mut total = Gain::unavailable();
        total.merge(&Gain::measured(0, GainScope::Private));
        assert_eq!(total.nature, GainNature::Measured);
        assert_eq!(total.bytes, 0);

        let mut mesure = Gain::measured(0, GainScope::Private);
        mesure.merge(&Gain::measured(4 * 1024 * 1024, GainScope::Private));
        assert_eq!(mesure.nature, GainNature::Measured);
        assert_eq!(mesure.bytes, 4 * 1024 * 1024);
    }

    #[test]
    fn un_zero_estime_reste_indisponible() {
        let zero = Gain::estimated(0, GainScope::Private);
        assert_eq!(zero.nature, GainNature::Unavailable);
        assert_eq!(zero.bytes, 0);

        let mut total = Gain::estimated(10, GainScope::Private);
        total.merge(&zero);
        assert_eq!(total.nature, GainNature::Estimated);
        assert_eq!(total.bytes, 10);
    }
}
