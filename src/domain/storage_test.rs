use std::collections::HashMap;

use crate::domain::storage::{desktop, StateFile};
use crate::domain::WordStatus;

fn sample_states() -> HashMap<String, WordStatus> {
    let mut map = HashMap::new();
    map.insert("привет".to_string(), WordStatus::Seen);
    map.insert("бог".to_string(), WordStatus::Known);
    map.insert("небо".to_string(), WordStatus::Unseen);
    map
}

#[test]
fn test_state_file_serialization_roundtrip() {
    let states = sample_states();
    let file = StateFile::from_states(&states);
    let json = file.encode().unwrap();
    let decoded = StateFile::decode(&json).unwrap();
    let decoded_map = decoded.into_map();

    assert_eq!(decoded_map.len(), 3);
    assert_eq!(decoded_map.get("привет"), Some(&WordStatus::Seen));
    assert_eq!(decoded_map.get("бог"), Some(&WordStatus::Known));
    assert_eq!(decoded_map.get("небо"), Some(&WordStatus::Unseen));
}

#[test]
fn test_state_file_decode_rejects_garbage() {
    assert!(StateFile::decode("not json").is_err());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_missing_file_falls_back_to_empty() {
    let path = std::env::temp_dir().join("bible_reader_missing_states.json");
    let path = path.to_str().unwrap().to_string();
    let _ = std::fs::remove_file(&path);

    let loaded = desktop::load_from(&path);
    assert!(loaded.is_empty(), "a missing file must yield an empty state map");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_save_then_load_roundtrip_persists_state() {
    let path = std::env::temp_dir().join("bible_reader_roundtrip_states.json");
    let path = path.to_str().unwrap().to_string();
    let _ = std::fs::remove_file(&path);

    let states = sample_states();
    desktop::save_to(&path, &states).unwrap();
    let loaded = desktop::load_from(&path);

    assert_eq!(loaded, states);
    let _ = std::fs::remove_file(&path);
}