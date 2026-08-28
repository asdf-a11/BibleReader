#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    BookSelection,
    ChapterSelection { book_id: String },
    Reader { book_id: String, chapter: u32 },
    /// Settings overlay; `return_to` restores the screen it was opened from.
    Settings { return_to: Box<AppState> },
}