//! Couche « plateforme » : choisit la vraie collecte Windows ou la démonstration.
use crate::model::{Action, RollbackEntry, Snapshot};
use std::sync::Mutex;

pub struct Backend {
    demo: bool,
    state: Mutex<crate::demo::DemoState>,
}

impl Backend {
    pub fn new(force_demo: bool) -> Backend {
        Backend {
            demo: force_demo || !cfg!(windows),
            state: Mutex::new(crate::demo::DemoState::default()),
        }
    }

    pub fn is_demo(&self) -> bool {
        self.demo
    }

    pub fn snapshot(&self) -> Snapshot {
        if self.demo {
            self.state.lock().unwrap().snapshot()
        } else {
            #[cfg(windows)]
            {
                crate::win::collect()
            }
            #[cfg(not(windows))]
            {
                self.state.lock().unwrap().snapshot()
            }
        }
    }

    pub fn apply(&self, actions: &[Action]) -> Vec<RollbackEntry> {
        if self.demo {
            return self.state.lock().unwrap().apply(actions);
        }
        #[cfg(windows)]
        {
            crate::win::apply(actions)
        }
        #[cfg(not(windows))]
        {
            self.state.lock().unwrap().apply(actions)
        }
    }

    /// Mémoire disponible maintenant, mesurée sur la machine réelle ou dans
    /// l'état simulé. Sert au delta avant/après d'un arrêt immédiat.
    pub fn available_memory(&self) -> u64 {
        if self.demo {
            return self.state.lock().unwrap().snapshot().memory.available;
        }
        #[cfg(windows)]
        {
            crate::win::memory_available()
        }
        #[cfg(not(windows))]
        {
            self.state.lock().unwrap().snapshot().memory.available
        }
    }

    pub fn is_admin(&self) -> bool {
        if self.demo {
            return true;
        }
        #[cfg(windows)]
        {
            crate::win::is_admin()
        }
        #[cfg(not(windows))]
        {
            true
        }
    }

    pub fn elevate(&self) -> bool {
        #[cfg(windows)]
        {
            crate::win::elevate()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }
}
