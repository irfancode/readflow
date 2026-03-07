use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::debug;

use crate::FeedItem;

pub struct FeedParser;

impl FeedParser {
    pub async fn parse_feed(url: &str, content: &str) -> Result<FeedParseResult> {
        if content.contains("<rss") {
            Self::parse_rss(content, url)
        } else if content.contains("<feed") {
            Self::parse_atom(content, url)
        } else {
            Err(anyhow::anyhow!("Unknown feed format"))
        }
    }

    fn parse_rss(content: &str, url: &str) -> Result<FeedParseResult> {
        let channel = rss::Channel::read_from(content.as_bytes())?;
        
        let title = channel.title().to_string();
        let items: Vec<FeedItem> = channel.items()
            .iter()
            .enumerate()
            .map(|(_i, item)| {
                let published_at = item.pub_date()
                    .and_then(|s| chrono::DateTime::parse_from_rfc2822(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                FeedItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    feed_id: String::new(),
                    title: item.title().unwrap_or("Untitled").to_string(),
                    url: item.link().unwrap_or("").to_string(),
                    content: item.content().map(String::from),
                    published_at,
                    is_read: false,
                }
            })
            .collect();

        Ok(FeedParseResult {
            title,
            url: url.to_string(),
            items,
        })
    }

    fn parse_atom(content: &str, url: &str) -> Result<FeedParseResult> {
        let feed = atom_syndication::Feed::read_from(content.as_bytes())?;
        
        let title = feed.title().to_string();
        let items: Vec<FeedItem> = feed.entries()
            .iter()
            .enumerate()
            .map(|(_i, entry)| {
                let published_at = entry.published()
                    .or(Some(entry.updated()))
                    .and_then(|dt| chrono::DateTime::parse_from_rfc3339(&dt.to_string()).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                let link = entry.links().first()
                    .map(|l| l.href().to_string())
                    .unwrap_or_default();

                let content: Option<String> = entry.content()
                    .and_then(|c| c.value().map(|s: &str| s.to_string()));

                FeedItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    feed_id: String::new(),
                    title: entry.title().to_string(),
                    url: link,
                    content,
                    published_at,
                    is_read: false,
                }
            })
            .collect();

        Ok(FeedParseResult {
            title,
            url: url.to_string(),
            items,
        })
    }

    pub fn detect_feed_url(html: &str) -> Option<String> {
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse("link[type='application/rss+xml']").ok();
        
        if let Some(sel) = selector {
            if let Some(element) = document.select(&sel).next() {
                return element.value().attr("href").map(String::from);
            }
        }

        let selector = scraper::Selector::parse("link[type='application/atom+xml']").ok();
        if let Some(sel) = selector {
            if let Some(element) = document.select(&sel).next() {
                return element.value().attr("href").map(String::from);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct FeedParseResult {
    pub title: String,
    pub url: String,
    pub items: Vec<FeedItem>,
}
