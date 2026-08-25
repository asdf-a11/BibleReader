# Stage 2: Core Domain Logic & Cross-Platform Persistence (Rust)

## Objective
Implement pure, UI-decoupled Rust domain models for Bible content, user vocabulary state management, and cross-platform JSON persistence supporting both Desktop (`std::fs`) and WebAssembly (`quad-storage` / `localStorage`).

---

## Technical Constraints & Environment
- **Language**: Rust 2021 Edition.
- **Dependencies**: `serde`, `serde_json`, `quad-storage` (for WASM target), `macroquad` (minimal usage for WASM platform detection if required).
- **Target Compatibility**: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `wasm32-unknown-unknown`.
- **Zero Macroquad GUI logic in this stage**: Keep domain logic pure and 100% unit testable natively.

---

## Session Breakdown & Implementation Steps

### Session 2.1: Data Models & Deserialization Engine
1. **Verse Raw Struct**:
   ```rust
   #[derive(Debug, Clone, Deserialize, Serialize)]
   pub struct RawVerse {
       pub chapter: u32,
       pub verse: u32,
       pub text: String,
       pub translation_id: String,
       pub book_id: String,
       pub book_name: String,
   }
   ```
2. **Structured Bible Hierarchy**:
   - Define `Book`, `Chapter`, `Verse`, and `Token` structs.
   - `Token` enum: `Word { original: String, clean_key: String }` or `Punctuation(String)`.
3. **Translation & State Models**:
   - `enum WordStatus { Unseen, Seen, Known }`
   - Struct `TranslationEntry { word: String, closest_words: Vec<String> }`
   - Struct `WordStateStore` with an internal HashMap `HashMap<String, WordStatus>`.

### Session 2.2: Text Tokenization Engine for Bible Reader
1. Create a `TextTokenizer` struct to process verse strings into rendered sequences of `Token::Word` and `Token::Punctuation`.
2. Example: `"В начале,"` -> `[Token::Word("В", "в"), Token::Punctuation(" "), Token::Word("начале", "начале"), Token::Punctuation(",")]`.

### Session 2.3: Cross-Platform Persistence Abstraction
1. Define a `StorageDriver` trait / conditional module:
   - On Desktop (`#[cfg(not(target_arch = "wasm32"))]`): Use standard file I/O (`std::fs::read_to_string`, `std::fs::write`).
   - On WASM (`#[cfg(target_arch = "wasm32")]`): Use `quad_storage::STORAGE`.
2. Save state format:
   ```json
   {
     "states": [
       ["привет", "seen"],
       ["начале", "known"]
     ]
   }
   ```
3. Automatic status transition rule:
   - When a word with status `Unseen` is clicked/opened, it automatically transitions to `Seen` unless explicitly overridden to `Unseen` by user input.

---

## Required Unit Tests (`src/domain/tests.rs`)

### Key Test Cases:
1. **`test_parse_raw_json_lines()`**:
   - Pass multi-line JSON string (`rst.json` sample) and verify successful parsing into `Vec<RawVerse>`.
2. **`test_bible_hierarchy_building()`**:
   - Verify grouping raw verses into `Book` -> `Chapter` -> `Verse` maps correctly without missing chapters.
3. **`test_tokenizer_word_punctuation_split()`**:
   - Input `"В начале, сотворил Бог..."`
   - Assert correct sequence of `Token::Word` and `Token::Punctuation`. Verify punctuation does not leak into `clean_key`.
4. **`test_word_status_transition_rules()`**:
   - Test default status of unrecorded word is `Unseen`.
   - Test rule: accessing/selecting an `Unseen` word updates it to `Seen`.
   - Test setting status explicitly to `Known` or forcing back to `Unseen`.
5. **`test_storage_serialization_roundtrip()`**:
   - Mock state store with sample words, serialize to JSON string, deserialize back, and verify state equality.
6. **`test_missing_file_graceful_fallback()`**:
   - Ensure app defaults to an empty state map without panicking if state file does not exist.

---

## Agent Verification Checklist
- [ ] Run `cargo test --target x86_64-unknown-linux-gnu` (or native desktop target) -> All unit tests pass.
- [ ] Run `cargo check --target wasm32-unknown-unknown` -> Ensure cross-platform storage abstraction compiles cleanly without `std::fs` errors on WASM.
