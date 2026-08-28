use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::WordStatus;

const STATE_FILENAME: &str = "word_states.json";

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

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop {
    use super::*;
    use std::fs;

    pub fn load_from(path: &str) -> HashMap<String, WordStatus> {
        match fs::read_to_string(path) {
            Ok(content) => StateFile::decode(&content)
                .map(StateFile::into_map)
                .unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    pub fn save_to(path: &str, states: &HashMap<String, WordStatus>) -> Result<(), String> {
        let content = StateFile::from_states(states).encode()?;
        fs::write(path, content).map_err(|e| format!("Failed to write state file: {}", e))
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use quad_storage::STORAGE;

    pub fn load_from(path: &str) -> HashMap<String, WordStatus> {
        match STORAGE.lock().expect("Failed to acquire storage lock").get(path) {
            Some(content) => {
                StateFile::decode(&content).map(StateFile::into_map).unwrap_or_default()
            }
            None => HashMap::new(),
        }
    }

    pub fn save_to(path: &str, states: &HashMap<String, WordStatus>) -> Result<(), String> {
        let content = StateFile::from_states(states).encode()?;
        STORAGE
            .lock()
            .expect("Failed to acquire storage lock")
            .set(path, &content);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_states() -> HashMap<String, WordStatus> {
    desktop::load_from(STATE_FILENAME)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_states(states: &HashMap<String, WordStatus>) -> Result<(), String> {
    desktop::save_to(STATE_FILENAME, states)
}

#[cfg(target_arch = "wasm32")]
pub fn load_states() -> HashMap<String, WordStatus> {
    wasm::load_from(STATE_FILENAME)
}

#[cfg(target_arch = "wasm32")]
pub fn save_states(states: &HashMap<String, WordStatus>) -> Result<(), String> {
    wasm::save_to(STATE_FILENAME, states)
}