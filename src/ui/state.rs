#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    BookSelection,
    ChapterSelection { book_id: String },
    Reader { book_id: String, chapter: u32 },
}