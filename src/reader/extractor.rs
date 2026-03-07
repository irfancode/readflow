use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use tracing::{debug, info};

pub struct ArticleExtractor;

impl ArticleExtractor {
    pub fn extract(html: &str, url: &str) -> Result<ExtractedArticle> {
        let document = Html::parse_document(html);

        let title = Self::extract_title(&document)?;
        let author = Self::extract_author(&document);
        let published_at = Self::extract_published_date(&document);
        let content = Self::extract_content(&document)?;
        let excerpt = Self::extract_excerpt(&document, &content);
        let site_name = Self::extract_site_name(&document);

        let reading_time = Self::calculate_reading_time(&content);

        Ok(ExtractedArticle {
            url: url.to_string(),
            title,
            content,
            excerpt,
            author,
            published_at,
            site_name,
            reading_time,
        })
    }

    fn extract_title(document: &Html) -> Result<String> {
        let selectors = [
            "meta[property='og:title']",
            "meta[name='twitter:title']",
            "title",
            "h1.article-title",
            "h1.post-title",
            "h1.entry-title",
            ".article h1",
            ".post h1",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let title = if selector_str.starts_with("meta") {
                        let content = element.value().attr("content");
                        content.map(|s| s.to_string()).unwrap_or_default()
                    } else {
                        element.text().collect::<String>().trim().to_string()
                    };

                    if !title.is_empty() && title.len() > 5 {
                        return Ok(title);
                    }
                }
            }
        }

        Ok(String::from("Untitled"))
    }

    fn extract_author(document: &Html) -> Option<String> {
        let selectors = [
            "meta[name='author']",
            "meta[property='article:author']",
            "[rel='author']",
            ".author",
            ".byline",
            ".post-author",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let author: String = if selector_str.starts_with("meta") {
                        element.value().attr("content").unwrap_or("").to_string()
                    } else {
                        element.text().collect::<String>().trim().to_string()
                    };

                    if !author.is_empty() && author.len() > 2 {
                        return Some(author);
                    }
                }
            }
        }

        None
    }

    fn extract_published_date(document: &Html) -> Option<chrono::DateTime<chrono::Utc>> {
        let selectors = [
            "meta[property='article:published_time']",
            "meta[name='date']",
            "meta[name='pubdate']",
            "time[datetime]",
            ".published",
            ".post-date",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let date_str: String = if selector_str.starts_with("meta") {
                        element.value().attr("content").unwrap_or("").to_string()
                    } else if selector_str.contains("datetime") {
                        element.value().attr("datetime").unwrap_or("").to_string()
                    } else {
                        element.text().collect::<String>().trim().to_string()
                    };

                    if !date_str.is_empty() {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&date_str) {
                            return Some(dt.with_timezone(&chrono::Utc));
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_content(document: &Html) -> Result<String> {
        let candidates = [
            "article",
            "[role='article']",
            ".article-content",
            ".post-content",
            ".entry-content",
            ".content-body",
            ".article-body",
            ".story-body",
            "main",
            "[role='main']",
            ".main-content",
            "#content",
            ".content",
        ];

        let mut best_content: Option<String> = None;
        let mut best_score = 0;

        for selector_str in candidates {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let html = element.html();
                    let score = Self::score_content(&html);

                    if score > best_score {
                        best_score = score;
                        best_content = Some(html);
                    }
                }
            }
        }

        if best_score > 0 {
            Ok(Self::clean_content(best_content.unwrap()))
        } else {
            let selector = Selector::parse("body").ok();
            if let Some(sel) = selector {
                if let Some(element) = document.select(&sel).next() {
                    return Ok(Self::clean_content(element.html()));
                }
            }
            Ok(String::new())
        }
    }

    fn score_content(html: &str) -> i32 {
        let document = Html::parse_fragment(html);
        let mut score = 0;

        let positive_patterns = [
            "article", "body", "content", "entry", "main", "page", "post", "text", "blog", "story",
            "h1", "h2", "h3", "h4", "h5", "h6", "p",
        ];

        let negative_patterns = [
            "comment",
            "meta",
            "footer",
            "header",
            "nav",
            "sidebar",
            "aside",
            "share",
            "social",
            "related",
            "ad",
            "advertisement",
            "popup",
            "modal",
        ];

        let text = html.to_lowercase();

        for pattern in positive_patterns {
            if let Ok(re) = Regex::new(&format!(r"(?i){}", pattern)) {
                score += re.find_iter(&text).count() as i32;
            }
        }

        for pattern in negative_patterns {
            if let Ok(re) = Regex::new(&format!(r"(?i){}", pattern)) {
                score -= re.find_iter(&text).count() as i32 * 2;
            }
        }

        if let Ok(re) = Regex::new(r"<p[^>]*>") {
            score += re.find_iter(html).count() as i32 * 3;
        }

        score
    }

    fn clean_content(html: String) -> String {
        let document = Html::parse_fragment(&html);

        let remove_selectors = [
            "script",
            "style",
            "noscript",
            "iframe",
            "form",
            "input",
            "button",
            "nav",
            "header",
            "footer",
            "aside",
            "advertisement",
            ".ad",
            ".ads",
            ".social",
            ".share",
            ".comments",
            ".related",
            ".sidebar",
            ".navigation",
            ".menu",
            ".popup",
            ".modal",
        ];

        let mut cleaned = html.clone();

        for selector_str in remove_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let element_html = element.html();
                    cleaned = cleaned.replace(&element_html, "");
                }
            }
        }

        if let Ok(re) = Regex::new(r"(?i)<!--.*?-->") {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        if let Ok(re) = Regex::new(r"(?i)\s+") {
            cleaned = re.replace_all(&cleaned, " ").to_string();
        }

        cleaned.trim().to_string()
    }

    fn extract_excerpt(document: &Html, content: &str) -> String {
        let selectors = [
            "meta[property='og:description']",
            "meta[name='description']",
            "meta[name='twitter:description']",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let desc = element.value().attr("content").unwrap_or("");
                    if !desc.is_empty() {
                        return desc.to_string();
                    }
                }
            }
        }

        let text = Self::html_to_text(content);
        if text.len() > 200 {
            format!("{}...", &text[..197])
        } else {
            text
        }
    }

    fn extract_site_name(document: &Html) -> Option<String> {
        let selectors = [
            "meta[property='og:site_name']",
            "meta[name='application-name']",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let name = element.value().attr("content").unwrap_or("");
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }

        None
    }

    fn html_to_text(html: &str) -> String {
        let document = Html::parse_fragment(html);
        let mut text = String::new();

        fn extract_from_element(element: &scraper::ElementRef, text: &mut String) {
            for child in element.children() {
                if let Some(el) = child.value().as_text() {
                    let content = el.trim();
                    if !content.is_empty() {
                        text.push_str(content);
                        text.push(' ');
                    }
                } else if child.value().is_element() {
                    if let Some(el) = child.value().as_element() {
                        let tag_name = el.name();
                        if tag_name != "script" && tag_name != "style" && tag_name != "noscript" {
                            if let Some(child_el) = scraper::ElementRef::wrap(child) {
                                extract_from_element(&child_el, text);
                            }
                        }
                    }
                }
            }
        }

        if let Some(root) = document.root_element().into() {
            extract_from_element(&root, &mut text);
        }

        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn calculate_reading_time(content: &str) -> u32 {
        let text = Self::html_to_text(content);
        let word_count = text.split_whitespace().count();
        let wpm = 200;
        (word_count as f32 / wpm as f32).ceil() as u32
    }

    pub fn to_plain_text(html: &str) -> String {
        Self::html_to_text(html)
    }

    pub fn to_markdown(html: &str) -> String {
        let document = Html::parse_fragment(html);
        let mut markdown = String::new();

        let selector = Selector::parse("*").ok();
        if let Some(sel) = selector {
            for element in document.select(&sel) {
                let text = element.text().collect::<String>().trim().to_string();

                if text.is_empty() {
                    continue;
                }

                let tag_name = element.value().name();
                match tag_name {
                    "h1" => markdown.push_str(&format!("# {}\n\n", text)),
                    "h2" => markdown.push_str(&format!("## {}\n\n", text)),
                    "h3" => markdown.push_str(&format!("### {}\n\n", text)),
                    "h4" => markdown.push_str(&format!("#### {}\n\n", text)),
                    "h5" => markdown.push_str(&format!("##### {}\n\n", text)),
                    "h6" => markdown.push_str(&format!("###### {}\n\n", text)),
                    "p" => markdown.push_str(&format!("{}\n\n", text)),
                    "li" => markdown.push_str(&format!("- {}\n", text)),
                    "blockquote" => markdown.push_str(&format!("> {}\n\n", text)),
                    "code" => markdown.push_str(&format!("`{}`", text)),
                    "pre" => markdown.push_str(&format!("```\n{}\n```\n\n", text)),
                    "a" => {
                        let href = element.value().attr("href").unwrap_or("");
                        markdown.push_str(&format!("[{}]({})", text, href));
                    }
                    "strong" | "b" => markdown.push_str(&format!("**{}**", text)),
                    "em" | "i" => markdown.push_str(&format!("*{}*", text)),
                    "br" | "hr" => markdown.push_str(&format!("{}\n", text)),
                    _ => markdown.push_str(&format!("{} ", text)),
                }
            }
        }

        markdown
    }
}

#[derive(Debug, Clone)]
pub struct ExtractedArticle {
    pub url: String,
    pub title: String,
    pub content: String,
    pub excerpt: String,
    pub author: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub site_name: Option<String>,
    pub reading_time: u32,
}
