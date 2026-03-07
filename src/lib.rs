pub mod browser;
pub mod ui;
pub mod reader;
pub mod storage;
pub mod feeds;
pub mod export;
pub mod ai;

pub use anyhow::Result;
pub use thiserror::Error;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
    Sepia,
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Sepia,
            Theme::Sepia => Theme::Light,
        }
    }

    pub fn background_color(&self) -> &str {
        match self {
            Theme::Light => "#FBF8F3",
            Theme::Dark => "#0D0D0D",
            Theme::Sepia => "#F4ECD8",
        }
    }

    pub fn text_color(&self) -> &str {
        match self {
            Theme::Light => "#2D2D2D",
            Theme::Dark => "#E8E8E8",
            Theme::Sepia => "#5B4636",
        }
    }

    pub fn accent_color(&self) -> &str {
        match self {
            Theme::Light => "#0066CC",
            Theme::Dark => "#4A9EFF",
            Theme::Sepia => "#8B4513",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Browser,
    Reader,
    Feeds,
    Bookmarks,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Article {
    pub id: String,
    pub url: String,
    pub title: String,
    pub content: String,
    pub excerpt: String,
    pub author: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub saved_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub is_read: bool,
    pub reading_time: u32,
    pub highlights: Vec<Highlight>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub id: String,
    pub article_id: String,
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub color: String,
    pub note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Feed {
    pub id: String,
    pub url: String,
    pub title: String,
    pub last_fetched: Option<chrono::DateTime<chrono::Utc>>,
    pub items: Vec<FeedItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeedItem {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_read: bool,
}

pub fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("readflow")
}

pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("readflow")
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("readflow")
}
