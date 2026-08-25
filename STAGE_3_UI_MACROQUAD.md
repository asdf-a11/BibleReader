# Stage 3: Interactive UI, Rendering & Macroquad Integration (Rust)

## Objective
Build the responsive front-end application using Macroquad. Implement screen navigation (Book List -> Chapter Grid -> Reader), Cyrillic custom font rendering, word layout engine, mouse touch selection with 1–2px padding highlights, and the translation popup drawer.

---

## Technical Constraints & Environment
- **Framework**: `macroquad` (0.4+).
- **Font**: Custom Cyrillic `.ttf` dynamic font loading (`load_ttf_font`).
- **Target OS**: Windows, Linux, WASM.
- **Architecture**: Decoupled immediate-mode UI with layout geometry classes/helpers that can be headless unit-tested without an active window or GPU context.

---

## Session Breakdown & Implementation Steps

### Session 3.1: State Machine & Font System
1. **State Machine**:
   ```rust
   pub enum AppState {
       BookSelection,
       ChapterSelection { book_id: String },
       Reader { book_id: String, chapter: u32 },
   }
   ```
2. **Font & Theme Context**:
   - Load custom Cyrillic TTF font at boot.
   - Color Constants:
     - Unseen: `Cyan / Light Blue` (`#00E5FF` with opacity)
     - Seen: `Goldish Yellow` (`#FFD54F`)
     - Known: `No Highlighting` (Default text color)
     - Background / UI Panel colors.

### Session 3.2: Page Layout Engines & UI Views
1. **Screen 1: Book Selection View**:
   - Centered underlined title: `"Books"`.
   - Scrollable list of available Bible books.
2. **Screen 2: Chapter Selection View**:
   - Centered title: `"Chapters"`.
   - Grid layout of numbered chapter buttons from left-to-right, top-to-bottom.
3. **Screen 3: Bible Reader & Token Wrapping Engine**:
   - Compute line wrapping for words and punctuation based on screen width.
   - For each word token, compute its bounding box `Rect { x, y, width, height }`.
   - Apply 1–2px extra padding to word bounding boxes for visual highlight background rectangles and mouse interaction detection.
   - Punctuation is rendered inline but excluded from bounding box click targets and highlight background rects.

### Session 3.3: Translation Popup Drawer & Word Control Buttons
1. **Drawer Sizing**:
   - Takes up lower 1/3 (large screen) to 1/2 (small screen) of the window height.
2. **Drawer Contents**:
   - Selected Russian word title.
   - Instant translation listing (1–3 English closest words).
   - 3 Status Control Buttons: `[Unseen]`, `[Seen]`, `[Known]`.
3. **Interaction Rules**:
   - Clicking a status button immediately updates the word's status, triggers persistence via Stage 2 storage driver, and updates the highlight color.
   - Default behavior upon clicking an `Unseen` word: set status to `Seen` immediately unless the user explicitly clicks `Unseen`.

---

## Required Unit Tests (`src/ui/layout_tests.rs`)

Write headless layout tests that do not require an active OpenGL/Macroquad context.

### Key Test Cases:
1. **`test_responsive_drawer_height()`**:
   - Test screen size `800x600` -> drawer height is ~200px (1/3).
   - Test small screen `400x300` -> drawer height is ~150px (1/2).
2. **`test_word_bounding_box_padding()`**:
   - Input word dimensions `width: 40px, height: 20px` at `(x: 10, y: 10)`.
   - Assert highlight rect applies 1–2px padding (`x: 8–9, y: 8–9, width: 42–44, height: 22–24`).
3. **`test_word_wrapping_line_breaks()`**:
   - Supply sequence of word widths and max screen width (e.g. 200px).
   - Verify layout engine correctly breaks words onto line 2 when exceeding width bounds.
4. **`test_click_hit_testing()`**:
   - Given a list of token bounding boxes, test mouse position `(12, 12)` hits the correct word token.
   - Test mouse position over punctuation token returns `None` (not clickable).
5. **`test_button_layout_in_drawer()`**:
   - Validate status buttons (`Unseen`, `Seen`, `Known`) sit inside the visible screen bounds of the popup drawer regardless of window size.

---

## Agent Verification Checklist
- [ ] Verify text renders with custom Cyrillic font on all screens without rendering question marks or missing glyph boxes.
- [ ] Verify clicking words highlights them with 1-2px padding without overlapping adjacent line text.
- [ ] Confirm WebAssembly build (`cargo build --target wasm32-unknown-unknown`) compiles without missing UI/rendering symbols.
