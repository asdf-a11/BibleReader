# Stage 1: Data Preprocessing Pipeline (Python)

## Objective
Build a robust Python preprocessing script that reads `rst.json` (a JSON Lines file containing Bible verses), cleans Russian text, extracts unique normalized words, translates them using Ollama (`gemma4:12b`), and writes structured output files (`translations.json`).

---

## Technical Constraints & Environment
- **Python Version**: 3.10+
- **LLM Engine**: Ollama API (`gemma4:12b`) running locally.
- **Dependencies**: Keep minimal (`ollama`, `pytest`, `pydantic` or standard `json`/`re`).
- **Input File Format**: `rst.json` consists of newline-delimited JSON objects:
  ```json
  {"chapter":1,"verse":1,"text":"В начале сотворил Бог небо и землю.","translation_id":"RST","book_id":"Gen","book_name":"Genesis"}
  ```

---

## Session Breakdown & Implementation Steps

### Session 1.1: Text Tokenization & Normalization Engine
1. **Module Structure**: Create a class `TextProcessor` to encapsulate tokenization.
2. **Cleaning Rules**:
   - Strip punctuation (`. , ! ? : ; " ' - « » — ( )` etc.) from word lookups.
   - Convert words to lowercase (`привет` vs `Привет`).
   - Preserve original punctuation tokens separately for rendering metadata.
3. **Regex Pattern**: Use regex `r'\w+'` with Cyrillic support to extract pure words.

### Session 1.2: Ollama Integration with Retries & Formatting Safeguards
1. **Module Structure**: Create a class `TranslationService` responsible for LLM calls.
2. **Parameters**:
   - Model: `gemma4:12b`
   - Temperature: `0.1` (low temperature to enforce strict output compliance)
   - Prompt Strategy:
     `"In 1-3 words comma separated give me the closest english words to '{word}'. You MUST stick to the format asked. Output ONLY the words."`
3. **Validation & Retry Loop**:
   - Limit retries to 3 attempts per word.
   - Validate response: splitting by comma must produce 1–3 non-empty string items without markdown formatting or extra sentence text.
   - Fallback logic: If all retries fail, default to `["[translation missing]"]` and log the error to `failed_words.log`.

### Session 1.3: Execution & Output Persistence
1. **Module Structure**: Create a class `PipelineRunner` that orchestrates reading `rst.json`, extracting unique words, translating in batch with progress logging, and saving output files.
2. **Output Schemas**:
   - `translations.json`:
     ```json
     {
       "translations": [
         {
           "word": "привет",
           "closestWords": ["hello", "hi", "greetings"]
         }
       ]
     }
     ```
   - `word_states.json` (Initial state default initialization):
     ```json
     {
       "states": [
         ["привет", "unseen"]
       ]
     }
     ```

---

## Required Unit Tests (`test_preprocessing.py`)

Create unit tests using `pytest` with mock objects for network/LLM dependencies.

### Key Test Cases:
1. **`test_tokenize_sentence()`**:
   - Verify `"В начале, сотворил Бог!"` extracts `["в", "начале", "сотворил", "бог"]`.
   - Verify punctuation and Cyrillic character handling.
2. **`test_translation_parser_valid()`**:
   - Test string `"hello, hi, greetings"` correctly parses into `["hello", "hi", "greetings"]`.
3. **`test_translation_parser_formatting_cleanup()`**:
   - Test LLM responses with trailing periods or extra whitespace (e.g. `" hello , hi . "`) are trimmed properly.
4. **`test_ollama_retry_and_fallback()`**:
   - Mock Ollama API to return invalid output 3 times, verifying fallback to `["[translation missing]"]` without crashing the script.
5. **`test_word_states_schema()`**:
   - Verify all unique extracted words are properly initialized to `"unseen"` in `word_states.json`.

---

## Agent Verification Checklist
- [ ] Run `pytest` and ensure 100% pass rate on text processing and mocking logic.
- [ ] Verify `translations.json` and `word_states.json` exist and match valid JSON schemas.
- [ ] Ensure non-Cyrillic characters and numbers are gracefully handled.
