use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::WordStatus;

const STATE_FILENAME: &str = "word_states.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub states: Vec<(String, String)>,
}

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop {
    use super::*;
    use std::fs;

    pub fn load_states() -> Result<HashMap<String, WordStatus>, String> {
        match fs::read_to_string(STATE_FILENAME) {
            Ok(content) => {
                let file: StateFile = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse state file: {}", e))?;
                Ok(file.states
                    .into_iter()
                    .map(|(word, status_str)| (word, WordStatus::from_str(&status_str)))
                    .collect())
            }
            Err(_) => Ok(HashMap::new()),
        }
    }

    pub fn save_states(states: &HashMap<String, WordStatus>) -> Result<(), String> {
        let file = StateFile {
            states: states.iter()
                .map(|(word, status)| (word.clone(), status.to_str().to_string()))
                .collect(),
        };
        
        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize states: {}", e))?;
        
        fs::write(STATE_FILENAME, content)
            .map_err(|e| format!("Failed to write state file: {}", e))?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use quad_storage::STORAGE;

    pub fn load_states() -> HashMap<String, WordStatus> {
        match STORAGE.lock().expect("Failed to acquire storage lock").get(STATE_FILENAME) {
            Some(content) => {
                match serde_json::from_str::<StateFile>(&content) {
                    Ok(file) => file.states
                        .into_iter()
                        .map(|(word, status_str)| (word, WordStatus::from_str(&status_str)))
                        .collect(),
                    Err(_) => HashMap::new(),
                }
            }
            None => HashMap::new(),
        }
    }

    pub fn save_states(states: &HashMap<String, WordStatus>) -> Result<(), String> {
        let file = StateFile {
            states: states.iter()
                .map(|(word, status)| (word.clone(), status.to_str().to_string()))
                .collect(),
        };
        
        let content = serde_json::to_string(&file)
            .map_err(|e| format!("Failed to serialize states: {}", e))?;
        
        STORAGE.lock().expect("Failed to acquire storage lock").set(STATE_FILENAME, &content);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::load_states;

#[cfg(target_arch = "wasm32")]
pub use wasm::load_states;
