use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::WordStatus;

/// Where the word states live on disk.
pub const STATE_FILENAME: &str = "word_states.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub states: Vec<(String, String)>,
}

impl StateFile {
    pub fn from_states(states: &HashMap<String, WordStatus>) -> Self {
        StateFile {
            states: states
                .iter()
                .map(|(word, status)| (word.clone(), status.to_str().to_string()))
                .collect(),
        }
    }

    pub fn encode(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize states: {}", e))
    }

    pub fn decode(content: &str) -> Result<StateFile, String> {
        serde_json::from_str(content).map_err(|e| format!("Failed to parse state file: {}", e))
    }

    pub fn into_map(self) -> HashMap<String, WordStatus> {
        self.states
            .into_iter()
            .map(|(word, status)| (word, WordStatus::from_str(&status)))
            .collect()
    }
}

/// Persistence backend for [`crate::domain::WordStateStore`].
///
/// This is an object-safe trait so the app can be constructed with a real
/// file-backed store (the normal case) or with an in-memory store (tests, and
/// platforms without a writable file system).
pub trait StateStore: Send + Sync {
    /// Load the stored states. A missing or corrupt store yields an empty map
    /// rather than an error: not having progress saved must never stop the
    /// reader from starting.
    fn load(&self) -> HashMap<String, WordStatus>;

    fn save(&self, states: &HashMap<String, WordStatus>) -> Result<(), String>;
}

/// Reads and writes `word_states.json`.
pub struct FileStore {
    path: String,
}

impl FileStore {
    pub fn new(path: impl Into<String>) -> Self {
        FileStore { path: path.into() }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Default for FileStore {
    fn default() -> Self {
        FileStore::new(STATE_FILENAME)
    }
}

impl StateStore for FileStore {
    fn load(&self) -> HashMap<String, WordStatus> {
        match std::fs::read_to_string(&self.path) {
            Ok(content) => StateFile::decode(&content)
                .map(StateFile::into_map)
                .unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    fn save(&self, states: &HashMap<String, WordStatus>) -> Result<(), String> {
        let content = StateFile::from_states(states).encode()?;
        std::fs::write(&self.path, content)
            .map_err(|e| format!("Failed to write '{}': {}", self.path, e))
    }
}

/// Keeps states in RAM only. Used by tests and as the fallback on platforms
/// where the app cannot write to a file (e.g. a browser sandbox).
#[derive(Debug, Default)]
pub struct MemoryStore {
    states: std::sync::Mutex<HashMap<String, WordStatus>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore::default()
    }

    pub fn with_states(states: HashMap<String, WordStatus>) -> Self {
        MemoryStore {
            states: std::sync::Mutex::new(states),
        }
    }
}

impl StateStore for MemoryStore {
    fn load(&self) -> HashMap<String, WordStatus> {
        self.states
            .lock()
            .map(|states| states.clone())
            .unwrap_or_default()
    }

    fn save(&self, states: &HashMap<String, WordStatus>) -> Result<(), String> {
        let mut guard = self
            .states
            .lock()
            .map_err(|_| "Failed to lock the in-memory state store".to_string())?;
        *guard = states.clone();
        Ok(())
    }
}

/// The store used by the real application.
#[cfg(not(target_arch = "wasm32"))]
pub fn default_store() -> Box<dyn StateStore> {
    Box::new(FileStore::default())
}

/// The store used by the real application on platforms without file access.
#[cfg(target_arch = "wasm32")]
pub fn default_store() -> Box<dyn StateStore> {
    Box::new(MemoryStore::new())
}

/// File helpers used by the storage tests and simple call sites.
pub mod desktop {
    use std::collections::HashMap;

    use super::{FileStore, StateStore};
    use crate::domain::models::WordStatus;

    pub fn load_from(path: &str) -> HashMap<String, WordStatus> {
        FileStore::new(path).load()
    }

    pub fn save_to(path: &str, states: &HashMap<String, WordStatus>) -> Result<(), String> {
        FileStore::new(path).save(states)
    }
}
