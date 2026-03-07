use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use std::path::PathBuf;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{get_data_dir, Article, Feed, FeedItem, Highlight};

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = get_data_dir().join("readflow.db");

        std::fs::create_dir_all(get_data_dir())?;

        let conn = rusqlite::Connection::open(&db_path).context("Failed to open database")?;

        let db = Self { conn };
        db.init_schema()?;

        info!("Database initialized at {:?}", db_path);
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                excerpt TEXT,
                author TEXT,
                published_at TEXT,
                saved_at TEXT NOT NULL,
                is_read INTEGER DEFAULT 0,
                reading_time INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS article_tags (
                article_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (article_id, tag_id),
                FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS highlights (
                id TEXT PRIMARY KEY,
                article_id TEXT NOT NULL,
                text TEXT NOT NULL,
                start_pos INTEGER NOT NULL,
                end_pos INTEGER NOT NULL,
                color TEXT DEFAULT 'yellow',
                note TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                note TEXT,
                tags TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS feeds (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                last_fetched TEXT
            );

            CREATE TABLE IF NOT EXISTS feed_items (
                id TEXT PRIMARY KEY,
                feed_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                content TEXT,
                published_at TEXT,
                is_read INTEGER DEFAULT 0,
                FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT,
                visited_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_articles_url ON articles(url);
            CREATE INDEX IF NOT EXISTS idx_articles_saved ON articles(saved_at);
            CREATE INDEX IF NOT EXISTS idx_feed_items_feed ON feed_items(feed_id);
            CREATE INDEX IF NOT EXISTS idx_history_visited ON history(visited_at);
            ",
            )
            .context("Failed to initialize database schema")?;

        Ok(())
    }

    pub fn save_article(&self, article: &Article) -> Result<()> {
        let published_at = article.published_at.map(|dt| dt.to_rfc3339());

        self.conn.execute(
            "INSERT OR REPLACE INTO articles (id, url, title, content, excerpt, author, published_at, saved_at, is_read, reading_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                article.id,
                article.url,
                article.title,
                article.content,
                article.excerpt,
                article.author,
                published_at,
                article.saved_at.to_rfc3339(),
                article.is_read as i32,
                article.reading_time
            ],
        )?;

        for tag in &article.tags {
            self.add_tag(tag)?;
            let tag_id = self.get_tag_id(tag)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO article_tags (article_id, tag_id) VALUES (?1, ?2)",
                params![article.id, tag_id],
            )?;
        }

        debug!("Saved article: {}", article.title);
        Ok(())
    }

    pub fn get_article(&self, id: &str) -> Result<Option<Article>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, content, excerpt, author, published_at, saved_at, is_read, reading_time FROM articles WHERE id = ?1"
        )?;

        let article = stmt
            .query_row(params![id], |row| {
                let published_at: Option<String> = row.get(6)?;
                let published_at = published_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(Article {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    excerpt: row.get(4)?,
                    author: row.get(5)?,
                    published_at,
                    saved_at: row
                        .get::<_, String>(7)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    tags: Vec::new(),
                    is_read: row.get::<_, i32>(8)? != 0,
                    reading_time: row.get(9)?,
                    highlights: Vec::new(),
                })
            })
            .optional()?;

        if let Some(mut article) = article {
            article.tags = self.get_article_tags(&article.id)?;
            article.highlights = self.get_highlights(&article.id)?;
            Ok(Some(article))
        } else {
            Ok(None)
        }
    }

    pub fn get_article_by_url(&self, url: &str) -> Result<Option<Article>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, content, excerpt, author, published_at, saved_at, is_read, reading_time FROM articles WHERE url = ?1"
        )?;

        let article = stmt
            .query_row(params![url], |row| {
                let published_at: Option<String> = row.get(6)?;
                let published_at = published_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(Article {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    excerpt: row.get(4)?,
                    author: row.get(5)?,
                    published_at,
                    saved_at: row
                        .get::<_, String>(7)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    tags: Vec::new(),
                    is_read: row.get::<_, i32>(8)? != 0,
                    reading_time: row.get(9)?,
                    highlights: Vec::new(),
                })
            })
            .optional()?;

        if let Some(mut article) = article {
            article.tags = self.get_article_tags(&article.id)?;
            article.highlights = self.get_highlights(&article.id)?;
            Ok(Some(article))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_articles(&self) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, content, excerpt, author, published_at, saved_at, is_read, reading_time 
             FROM articles ORDER BY saved_at DESC"
        )?;

        let articles = stmt
            .query_map([], |row| {
                let published_at: Option<String> = row.get(6)?;
                let published_at = published_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(Article {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    excerpt: row.get(4)?,
                    author: row.get(5)?,
                    published_at,
                    saved_at: row
                        .get::<_, String>(7)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    tags: Vec::new(),
                    is_read: row.get::<_, i32>(8)? != 0,
                    reading_time: row.get(9)?,
                    highlights: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    pub fn delete_article(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM articles WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn add_tag(&self, name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            params![Uuid::new_v4().to_string(), name],
        )?;
        Ok(())
    }

    fn get_tag_id(&self, name: &str) -> Result<String> {
        let mut stmt = self.conn.prepare("SELECT id FROM tags WHERE name = ?1")?;
        let id: String = stmt.query_row(params![name], |row| row.get(0))?;
        Ok(id)
    }

    fn get_article_tags(&self, article_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t
             JOIN article_tags at ON t.id = at.tag_id
             WHERE at.article_id = ?1",
        )?;

        let tags = stmt
            .query_map(params![article_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(tags)
    }

    pub fn add_highlight(&self, highlight: &Highlight) -> Result<()> {
        self.conn.execute(
            "INSERT INTO highlights (id, article_id, text, start_pos, end_pos, color, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                highlight.id,
                highlight.article_id,
                highlight.text,
                highlight.start_pos,
                highlight.end_pos,
                highlight.color,
                highlight.note,
                highlight.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_highlights(&self, article_id: &str) -> Result<Vec<Highlight>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, article_id, text, start_pos, end_pos, color, note, created_at 
             FROM highlights WHERE article_id = ?1 ORDER BY start_pos",
        )?;

        let highlights = stmt
            .query_map(params![article_id], |row| {
                Ok(Highlight {
                    id: row.get(0)?,
                    article_id: row.get(1)?,
                    text: row.get(2)?,
                    start_pos: row.get(3)?,
                    end_pos: row.get(4)?,
                    color: row.get(5)?,
                    note: row.get(6)?,
                    created_at: row
                        .get::<_, String>(7)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(highlights)
    }

    pub fn delete_highlight(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM highlights WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_bookmark(
        &self,
        url: &str,
        title: &str,
        note: Option<&str>,
        tags: &[String],
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        self.conn.execute(
            "INSERT INTO bookmarks (id, url, title, note, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                url,
                title,
                note,
                tags.join(","),
                Utc::now().to_rfc3339()
            ],
        )?;

        Ok(id)
    }

    pub fn get_bookmarks(&self) -> Result<Vec<(String, String, String, Option<String>)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, url, title, note FROM bookmarks ORDER BY created_at DESC")?;

        let bookmarks = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bookmarks)
    }

    pub fn delete_bookmark(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_feed(&self, url: &str, title: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        self.conn.execute(
            "INSERT OR IGNORE INTO feeds (id, url, title) VALUES (?1, ?2, ?3)",
            params![id, url, title],
        )?;

        Ok(id)
    }

    pub fn get_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, url, title, last_fetched FROM feeds")?;

        let feeds = stmt
            .query_map([], |row| {
                let last_fetched: Option<String> = row.get(3)?;
                let last_fetched = last_fetched
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(Feed {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    last_fetched,
                    items: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(feeds)
    }

    pub fn delete_feed(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_feed(&self, id: &str, title: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET title = ?1, last_fetched = ?2 WHERE id = ?3",
            params![title, Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn add_feed_item(&self, item: &FeedItem) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO feed_items (id, feed_id, title, url, content, published_at, is_read)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id,
                item.feed_id,
                item.title,
                item.url,
                item.content,
                item.published_at.map(|dt| dt.to_rfc3339()),
                item.is_read as i32
            ],
        )?;
        Ok(())
    }

    pub fn get_feed_items(&self, feed_id: &str) -> Result<Vec<FeedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, feed_id, title, url, content, published_at, is_read 
             FROM feed_items WHERE feed_id = ?1 ORDER BY published_at DESC LIMIT 50",
        )?;

        let items = stmt
            .query_map(params![feed_id], |row| {
                let published_at: Option<String> = row.get(5)?;
                let published_at = published_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(FeedItem {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    published_at,
                    is_read: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn mark_feed_item_read(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feed_items SET is_read = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn add_history(&self, url: &str, title: Option<&str>) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        self.conn.execute(
            "INSERT INTO history (id, url, title, visited_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, url, title, Utc::now().to_rfc3339()],
        )?;

        self.conn.execute(
            "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY visited_at DESC LIMIT 1000)",
            [],
        )?;

        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT url, title, visited_at FROM history ORDER BY visited_at DESC LIMIT ?1",
        )?;

        let history = stmt
            .query_map(params![limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(history)
    }
}
