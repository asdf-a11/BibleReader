use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawVerse {
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
    pub translation_id: String,
    pub book_id: String,
    pub book_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub name: String,
    pub chapters: HashMap<u32, Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub number: u32,
    pub verses: Vec<Verse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verse {
    pub number: u32,
    pub text: String,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Token {
    Word { original: String, clean_key: String },
    Punctuation(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordStatus {
    Unseen,
    Seen,
    Known,
}

pub struct Bible {
    pub books: HashMap<String, Book>,
    pub book_order: Vec<String>,
}

impl Default for Bible {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Bible {
    fn clone(&self) -> Self {
        Bible {
            books: self.books.clone(),
            book_order: self.book_order.clone(),
        }
    }
}

impl WordStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "unseen" => WordStatus::Unseen,
            "seen" => WordStatus::Seen,
            "known" => WordStatus::Known,
            _ => WordStatus::Unseen,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            WordStatus::Unseen => "unseen",
            WordStatus::Seen => "seen",
            WordStatus::Known => "known",
        }
    }
}
