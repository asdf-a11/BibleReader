//! Application state and the (pure) transitions between screens.
//!
//! The views in [`crate::ui`] only ever call the methods on [`App`], which
//! keeps every state change unit testable without a window, a font or a GPU.

use egui::{Pos2, Rect};

use crate::domain::{
    default_store, load_bible_from_rst, load_translations, load_verse_translations, today, Bible,
    StateStore, StatsHistory, Translations, VerseTranslations, WordStateStore, WordStatus,
};
use crate::ui::layout::{
    TextLayout, BIBLE_PATH, MAX_ZOOM, MIN_ZOOM, TRANSLATIONS_PATH, VERSE_TRANSLATIONS_PATH,
    ZOOM_STEP,
};
use crate::ui::state::AppState;

/// The data the app runs on. Kept separate from [`App`] so tests can swap in a
/// tiny bible.
pub struct AppData {
    pub bible: Bible,
    pub translations: Translations,
    pub verse_translations: VerseTranslations,
    /// Set when a data file could not be read; the UI shows it instead of
    /// silently rendering nothing.
    pub load_error: Option<String>,
}

impl AppData {
    /// Load the real bible and translations from disk.
    pub fn load() -> Self {
        let bible = load_bible_from_rst(BIBLE_PATH);
        let translations = load_translations(TRANSLATIONS_PATH);
        let verse_translations = load_verse_translations(VERSE_TRANSLATIONS_PATH);

        let load_error = match (&bible, &translations, &verse_translations) {
            (Err(e), _, _) => Some(e.clone()),
            (_, Err(e), _) => Some(e.clone()),
            (_, _, Err(e)) => Some(e.clone()),
            _ => None,
        };

        AppData {
            bible: bible.unwrap_or_default(),
            translations: translations.unwrap_or_default(),
            verse_translations: verse_translations.unwrap_or_default(),
            load_error,
        }
    }
}

/// Everything a chapter's [`TextLayout`] depends on. Laying a long chapter out
/// measures every word, which is far too slow to redo on every frame while
/// scrolling.
#[derive(Debug, Clone, PartialEq)]
pub struct ReaderLayoutKey {
    pub book_id: String,
    pub chapter: u32,
    pub width: f32,
    pub pixels_per_point: f32,
}

/// Everything that changes while the app runs.
pub struct App {
    pub data: AppData,
    pub state: AppState,
    /// The word whose translation drawer is open, if any.
    pub selected_word: Option<String>,
    /// User zoom, applied with `egui::Context::set_zoom_factor`.
    pub zoom: f32,
    pub word_store: WordStateStore,
    /// Day by day number of known and seen words, shown on the Stats screen.
    pub history: StatsHistory,
    /// Whether [`App::history`] changed since it was last saved.
    history_dirty: bool,
    /// Last persistence error, surfaced in the toolbar.
    pub save_error: Option<String>,
    /// Layout of the chapter currently on screen. The reader keeps it between
    /// frames and only rebuilds it when [`App::reader_layout_key`] changes;
    /// tests (and the GUI tests in particular) assert on the real geometry
    /// through it instead of re-deriving it.
    pub reader_layout: TextLayout,
    /// What [`App::reader_layout`] was built for.
    pub reader_layout_key: Option<ReaderLayoutKey>,
    /// Screen space position of the reader content's top-left corner, i.e. the
    /// origin [`TextLayout`] rects are relative to.
    pub reader_origin: Pos2,
    /// How far the reader is scrolled down, in points.
    pub reader_scroll: f32,
    /// Total height of the reader's scrollable content, including the blank
    /// space reserved below the last line for the translation drawer.
    pub reader_content_height: f32,
    /// The part of the reader that is actually on screen (the scroll area's
    /// viewport, excluding the scroll bar).
    pub reader_viewport: Rect,
    /// Where the translation drawer covers the reader, if it is open.
    pub reader_drawer: Option<Rect>,
    store: Box<dyn StateStore>,
}

impl App {
    /// Build the app with a specific store (used by tests and by
    /// [`crate::run`], which loads from disk).
    pub fn with_store(data: AppData, store: Box<dyn StateStore>) -> Self {
        let word_store = WordStateStore::from_states(store.load());
        let history = store.load_history();
        let mut app = App {
            data,
            state: AppState::BookSelection,
            selected_word: None,
            zoom: 1.0,
            word_store,
            history,
            history_dirty: false,
            save_error: None,
            reader_layout: TextLayout::default(),
            reader_layout_key: None,
            reader_origin: Pos2::ZERO,
            reader_scroll: 0.0,
            reader_content_height: 0.0,
            reader_viewport: Rect::ZERO,
            reader_drawer: None,
            store,
        };
        // Progress saved before the history existed (or edited by hand) starts
        // the history off with today's totals.
        app.record_stats(&today());
        app
    }

    // ----------------------------------------------------------------- screens

    pub fn open_book(&mut self, book_id: &str) {
        self.state = AppState::ChapterSelection {
            book_id: book_id.to_string(),
        };
        self.selected_word = None;
    }

    pub fn open_chapter(&mut self, chapter: u32) {
        let book_id = match &self.state {
            AppState::ChapterSelection { book_id } => book_id.clone(),
            _ => return,
        };
        self.state = AppState::Reader { book_id, chapter };
        self.selected_word = None;
    }

    pub fn open_settings(&mut self) {
        if matches!(self.state, AppState::Settings { .. }) {
            return;
        }
        let return_to = self.take_return_to();
        self.state = AppState::Settings { return_to };
    }

    pub fn open_stats(&mut self) {
        if matches!(self.state, AppState::Stats { .. }) {
            return;
        }
        let return_to = self.take_return_to();
        self.state = AppState::Stats { return_to };
    }

    /// The screen Settings/Stats should go back to. Switching between the two
    /// keeps the original screen, so Back never bounces between them.
    fn take_return_to(&mut self) -> Box<AppState> {
        match std::mem::replace(&mut self.state, AppState::BookSelection) {
            AppState::Settings { return_to } | AppState::Stats { return_to } => return_to,
            other => Box::new(other),
        }
    }

    /// Back button: reader -> chapters -> books, settings/stats -> previous
    /// screen.
    pub fn go_back(&mut self) {
        self.state = match std::mem::replace(&mut self.state, AppState::BookSelection) {
            AppState::Settings { return_to } | AppState::Stats { return_to } => *return_to,
            AppState::Reader { book_id, .. } => AppState::ChapterSelection { book_id },
            // Already at the top of the list; there is nowhere further to go.
            AppState::ChapterSelection { .. } => AppState::BookSelection,
            other => other,
        };
        self.selected_word = None;
    }

    // ------------------------------------------------------------------- words

    /// Toggle the translation drawer of `word`.
    ///
    /// Opening a word for the first time also marks it as seen, and clicking
    /// the word that is already open closes the drawer again.
    pub fn toggle_word(&mut self, word: &str) {
        if self.selected_word.as_deref() == Some(word) {
            self.selected_word = None;
            return;
        }
        self.word_store.access_word(word);
        self.selected_word = Some(word.to_string());
    }

    pub fn close_drawer(&mut self) {
        self.selected_word = None;
    }

    /// Apply a status from the drawer to the selected word.
    pub fn set_selected_status(&mut self, status: WordStatus) {
        if let Some(word) = self.selected_word.clone() {
            self.word_store.set_status(&word, status);
        }
    }

    pub fn status_of(&self, word: &str) -> WordStatus {
        self.word_store.get_status(word)
    }

    // ------------------------------------------------------------------- stats

    pub fn known_count(&self) -> usize {
        self.word_store.count(WordStatus::Known)
    }

    pub fn seen_count(&self) -> usize {
        self.word_store.count(WordStatus::Seen)
    }

    /// Record the current known and seen totals as those of `date`
    /// (`YYYY-MM-DD`). Totals are recorded rather than increments, so a word
    /// moving between known, seen and unseen in any direction is counted right.
    pub fn record_stats(&mut self, date: &str) {
        let known = self.known_count();
        let seen = self.seen_count();
        let changed_known = self.history.known.record(date, known);
        let changed_seen = self.history.seen.record(date, seen);
        if changed_known || changed_seen {
            self.history_dirty = true;
        }
    }

    // -------------------------------------------------------------------- zoom

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - ZOOM_STEP).max(MIN_ZOOM);
    }

    // -------------------------------------------------------------- persistence

    /// Write the word states and the stats history back to the store if they
    /// changed.
    pub fn persist(&mut self) {
        if self.word_store.is_dirty() {
            self.record_stats(&today());
        }
        if !self.word_store.is_dirty() && !self.history_dirty {
            return;
        }

        let mut result = Ok(());
        if self.word_store.is_dirty() {
            result = self.store.save(&self.word_store.as_map());
            if result.is_ok() {
                self.word_store.clear_dirty();
            }
        }
        if self.history_dirty && result.is_ok() {
            result = self.store.save_history(&self.history);
            if result.is_ok() {
                self.history_dirty = false;
            }
        }
        match result {
            Ok(()) => self.save_error = None,
            Err(e) => self.save_error = Some(e),
        }
    }
}

/// The `eframe` glue: owns an [`App`] and hands it a [`egui::Ui`] each frame.
pub struct AppShell {
    app: App,
}

impl AppShell {
    /// Build the real application: the bible from disk, progress from
    /// `word_states.json`.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        AppShell::with_data(cc, AppData::load(), default_store())
    }

    /// Build an application around explicit data and a store, so the GUI tests
    /// can run against a tiny bible without touching the file system.
    pub fn with_data(
        cc: &eframe::CreationContext<'_>,
        data: AppData,
        store: Box<dyn StateStore>,
    ) -> Self {
        crate::ui::fonts::install(&cc.egui_ctx);
        crate::ui::theme::install(&cc.egui_ctx);
        AppShell {
            app: App::with_store(data, store),
        }
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}

impl eframe::App for AppShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Progress is written out as soon as it changes, so closing the window
        // never loses anything.
        self.app.persist();
        crate::ui::show(ui, &mut self.app);
    }
}
