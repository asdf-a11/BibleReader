//! The screens the application can show.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    BookSelection,
    ChapterSelection {
        book_id: String,
    },
    Reader {
        book_id: String,
        chapter: u32,
    },
    /// Settings screen; `return_to` restores the screen it was opened from.
    Settings {
        return_to: Box<AppState>,
    },
    /// Known/seen totals and their history; `return_to` as for `Settings`.
    Stats {
        return_to: Box<AppState>,
    },
}

impl AppState {
    /// Title shown in the toolbar.
    pub fn title(&self) -> String {
        match self {
            AppState::BookSelection => "Books".to_string(),
            AppState::ChapterSelection { .. } => "Chapters".to_string(),
            AppState::Reader { chapter, .. } => format!("Chapter {}", chapter),
            AppState::Settings { .. } => "Settings".to_string(),
            AppState::Stats { .. } => "Stats".to_string(),
        }
    }
}
