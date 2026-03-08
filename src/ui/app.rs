use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::browser::{ContentRenderer, Fetcher, Form, HtmlParser};
use crate::reader::ArticleExtractor;
use crate::storage::Database;
use crate::{Article, Feed, FeedItem, Theme, ViewMode};

#[derive(Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub content: String,
    pub links: Vec<(String, String)>,
    pub reader_content: Option<Article>,
    #[allow(dead_code)]
    timestamp: Instant,
}

#[derive(Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub content: String,
    pub reader_content: Option<Article>,
    pub links: Vec<(String, String)>,
    pub forms: Vec<Form>,
    pub selected_link: usize,
    pub scroll_offset: u16,
    pub total_lines: u16,
    pub loading: bool,
    pub history_stack: Vec<HistoryEntry>,
    pub history_index: isize,
}

impl Tab {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: String::new(),
            url: String::new(),
            content: String::new(),
            reader_content: None,
            links: Vec::new(),
            forms: Vec::new(),
            selected_link: 0,
            scroll_offset: 0,
            total_lines: 0,
            loading: false,
            history_stack: Vec::new(),
            history_index: -1,
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index < (self.history_stack.len() as isize - 1)
    }
}

pub struct App {
    pub theme: Theme,
    pub view_mode: ViewMode,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    
    pub current_url: String,
    pub page_title: String,
    pub page_content: String,
    pub reader_content: Option<Article>,
    pub links: Vec<(String, String)>,
    pub forms: Vec<Form>,
    pub selected_link: usize,
    pub scroll_offset: u16,
    pub total_lines: u16,
    
    pub feeds: Vec<Feed>,
    pub selected_feed: usize,
    pub feed_items: Vec<FeedItem>,
    pub selected_item: usize,
    
    pub bookmarks: Vec<(String, String, String, Option<String>)>,
    pub selected_bookmark: usize,
    
    pub history_stack: Vec<HistoryEntry>,
    pub history_index: isize,
    pub history_snapshots: Vec<HistoryEntry>,
    
    pub show_help: bool,
    pub show_url_input: bool,
    pub show_search: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub selected_search_result: usize,
    pub url_input: String,
    pub status_message: Option<String>,
    pub loading: bool,
    pub error_message: Option<String>,
    
    pub db: Arc<Mutex<Database>>,
    pub fetcher: Arc<Fetcher>,
    pub renderer: Arc<Mutex<ContentRenderer>>,
    
    pub width: u16,
    pub height: u16,
}

impl App {
    pub fn new(theme: Theme, insecure: bool) -> Self {
        let db = Database::new().expect("Failed to initialize database");
        let fetcher = Fetcher::new_with_verify(!insecure).expect("Failed to create fetcher");
        let renderer = ContentRenderer::new(80, theme);

        let mut initial_tab = Tab::new(0);
        initial_tab.title = "New Tab".to_string();

        Self {
            theme,
            view_mode: ViewMode::Browser,
            tabs: vec![initial_tab],
            active_tab: 0,
            current_url: String::new(),
            page_title: String::new(),
            page_content: String::new(),
            reader_content: None,
            links: Vec::new(),
            forms: Vec::new(),
            selected_link: 0,
            scroll_offset: 0,
            total_lines: 0,
            feeds: Vec::new(),
            selected_feed: 0,
            feed_items: Vec::new(),
            selected_item: 0,
            bookmarks: Vec::new(),
            selected_bookmark: 0,
            history_stack: Vec::new(),
            history_index: -1,
            history_snapshots: Vec::new(),
            show_help: false,
            show_url_input: false,
            show_search: false,
            search_query: String::new(),
            search_results: Vec::new(),
            selected_search_result: 0,
            url_input: String::new(),
            status_message: Some("Welcome to ReadFlow! Press 'o' to open a URL".to_string()),
            loading: false,
            error_message: None,
            db: Arc::new(Mutex::new(db)),
            fetcher: Arc::new(fetcher),
            renderer: Arc::new(Mutex::new(renderer)),
            width: 80,
            height: 24,
        }
    }

    pub fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn new_tab(&mut self) {
        let new_id = self.tabs.len();
        let mut tab = Tab::new(new_id);
        tab.title = "New Tab".to_string();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.sync_from_tab();
        self.status_message = Some(format!("Opened new tab ({} tabs)", self.tabs.len()));
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
            self.sync_from_tab();
            self.status_message = Some(format!("Closed tab ({} tabs remaining)", self.tabs.len()));
        } else {
            self.status_message = Some("Cannot close last tab".to_string());
        }
    }

    pub fn next_tab(&mut self) {
        if self.active_tab < self.tabs.len() - 1 {
            self.active_tab += 1;
            self.sync_from_tab();
        }
    }

    pub fn prev_tab(&mut self) {
        if self.active_tab > 0 {
            self.active_tab -= 1;
            self.sync_from_tab();
        }
    }

    pub fn sync_to_tab(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        tab.url = self.current_url.clone();
        tab.title = self.page_title.clone();
        tab.content = self.page_content.clone();
        tab.reader_content = self.reader_content.clone();
        tab.links = self.links.clone();
        tab.forms = self.forms.clone();
        tab.selected_link = self.selected_link;
        tab.scroll_offset = self.scroll_offset;
        tab.total_lines = self.total_lines;
        tab.loading = self.loading;
    }

    pub fn sync_from_tab(&mut self) {
        let tab = &self.tabs[self.active_tab];
        self.current_url = tab.url.clone();
        self.page_title = tab.title.clone();
        self.page_content = tab.content.clone();
        self.reader_content = tab.reader_content.clone();
        self.links = tab.links.clone();
        self.forms = tab.forms.clone();
        self.selected_link = tab.selected_link;
        self.scroll_offset = tab.scroll_offset;
        self.total_lines = tab.total_lines;
        self.loading = tab.loading;
        self.history_stack = tab.history_stack.clone();
        self.history_index = tab.history_index;
    }

    pub fn get_history_info(&self) -> (usize, bool, bool) {
        let count = self.history_stack.len();
        let can_back = self.history_index > 0;
        let can_forward = self.history_index < (count as isize - 1);
        (count, can_back, can_forward)
    }

    pub fn navigate_to(&mut self, url: &str) -> Result<()> {
        let url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("https://{}", url)
        };

        info!("Navigating to: {}", url);
        self.loading = true;
        self.error_message = None;
        self.status_message = Some(format!("Loading {}...", url));

        let fetcher = self.fetcher.clone();
        let url_clone = url.clone();
        
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let response = rt.block_on(async {
            fetcher.fetch(&url_clone).await
        })?;

        let parsed = HtmlParser::parse(&response.body);
        
        self.current_url = url.clone();
        self.page_title = parsed.title.clone();
        self.links = HtmlParser::extract_links_with_text(&response.body);
        self.forms = parsed.forms;
        self.selected_link = 0;
        
        let reader_article = if let Ok(extracted) = ArticleExtractor::extract(&response.body, &url) {
            let plain_text = ArticleExtractor::to_plain_text(&extracted.content);
            self.page_content = plain_text.clone();
            
            Some(Article {
                id: uuid::Uuid::new_v4().to_string(),
                url: url.clone(),
                title: extracted.title.clone(),
                content: plain_text.clone(),
                excerpt: extracted.excerpt.clone(),
                author: extracted.author.clone(),
                published_at: extracted.published_at,
                saved_at: chrono::Utc::now(),
                tags: Vec::new(),
                is_read: false,
                reading_time: extracted.reading_time,
                highlights: Vec::new(),
            })
        } else {
            self.page_content = HtmlParser::extract_text(&response.body);
            None
        };

        self.reader_content = reader_article.clone();

        let content_lines = self.page_content.lines().count() as u16;
        self.total_lines = content_lines;

        let entry = HistoryEntry {
            url: url.clone(),
            title: self.page_title.clone(),
            content: self.page_content.clone(),
            links: self.links.clone(),
            reader_content: reader_article,
            timestamp: Instant::now(),
        };

        if self.history_index < (self.history_stack.len() as isize - 1) {
            self.history_stack.truncate((self.history_index + 1) as usize);
        }
        self.history_stack.push(entry);
        self.history_index = (self.history_stack.len() - 1) as isize;

        let tab = &mut self.tabs[self.active_tab];
        tab.url = url.clone();
        tab.title = self.page_title.clone();
        tab.content = self.page_content.clone();
        tab.reader_content = self.reader_content.clone();
        tab.links = self.links.clone();
        tab.total_lines = self.total_lines;
        tab.loading = false;
        tab.history_stack = self.history_stack.clone();
        tab.history_index = self.history_index;

        let db = self.db.clone();
        let db_url = url.clone();
        let db_title = self.page_title.clone();
        std::thread::spawn(move || {
            if let Ok(db) = db.try_lock() {
                let _ = db.add_history(&db_url, Some(&db_title));
            }
        });

        self.loading = false;
        self.status_message = Some(format!("Loaded: {}", self.page_title));
        Ok(())
    }

    pub fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let entry = self.history_stack[self.history_index as usize].clone();
            let title = entry.title.clone();
            self.restore_history_entry(&entry);
            self.status_message = Some(format!("Back: {}", title));
        } else {
            self.status_message = Some("No previous page".to_string());
        }
    }

    pub fn go_forward(&mut self) {
        if self.history_index < (self.history_stack.len() as isize - 1) {
            self.history_index += 1;
            let entry = self.history_stack[self.history_index as usize].clone();
            let title = entry.title.clone();
            self.restore_history_entry(&entry);
            self.status_message = Some(format!("Forward: {}", title));
        } else {
            self.status_message = Some("No next page".to_string());
        }
    }

    fn restore_history_entry(&mut self, entry: &HistoryEntry) {
        self.current_url = entry.url.clone();
        self.page_title = entry.title.clone();
        self.page_content = entry.content.clone();
        self.links = entry.links.clone();
        self.reader_content = entry.reader_content.clone();
        self.selected_link = 0;
        self.total_lines = self.page_content.lines().count() as u16;
        self.scroll_offset = 0;
    }

    pub fn toggle_reader_mode(&mut self) {
        if self.view_mode == ViewMode::Reader {
            self.view_mode = ViewMode::Browser;
        } else {
            if self.reader_content.is_some() {
                self.view_mode = ViewMode::Reader;
                self.status_message = Some("Reader mode enabled".to_string());
            } else {
                self.status_message = Some("No article available for reader mode".to_string());
            }
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.status_message = Some(format!("Theme: {:?}", self.theme));
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.total_lines.saturating_sub(self.height / 2) {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_lines.saturating_sub(self.height / 2);
    }

    pub fn next_link(&mut self) {
        if self.selected_link < self.links.len().saturating_sub(1) {
            self.selected_link += 1;
        }
    }

    pub fn prev_link(&mut self) {
        if self.selected_link > 0 {
            self.selected_link -= 1;
        }
    }

    pub fn follow_link(&mut self) {
        if let Some((href, _)) = self.links.get(self.selected_link).cloned() {
            let _ = self.navigate_to(&href);
        }
    }

    pub fn add_bookmark(&mut self) {
        if !self.current_url.is_empty() {
            let db = self.db.clone();
            let url = self.current_url.clone();
            let title = self.page_title.clone();
            
            tokio::spawn(async move {
                if let Ok(db) = db.try_lock() {
                    let _ = db.add_bookmark(&url, &title, None, &[]);
                }
            });
            
            self.status_message = Some("Bookmark added".to_string());
        }
    }

    pub fn save_article(&mut self) {
        if let Some(article) = &self.reader_content {
            let db = self.db.clone();
            let article = article.clone();
            
            tokio::spawn(async move {
                if let Ok(db) = db.try_lock() {
                    let _ = db.save_article(&article);
                }
            });
            
            self.status_message = Some("Article saved".to_string());
        }
    }

    pub fn load_bookmarks(&mut self) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Ok(db) = db.try_lock() {
                let bookmarks = db.get_bookmarks().unwrap_or_default();
                debug!("Loaded {} bookmarks", bookmarks.len());
            }
        });
    }

    pub fn load_feeds(&mut self) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Ok(db) = db.try_lock() {
                let feeds = db.get_feeds().unwrap_or_default();
                debug!("Loaded {} feeds", feeds.len());
            }
        });
    }

    pub fn load_history(&mut self) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Ok(db) = db.try_lock() {
                let history = db.get_history(50).unwrap_or_default();
                debug!("Loaded {} history items", history.len());
            }
        });
    }

    pub fn switch_view(&mut self, mode: ViewMode) {
        self.view_mode = mode;
        self.scroll_offset = 0;
        self.selected_link = 0;
        
        match mode {
            ViewMode::Bookmarks => {
                self.load_bookmarks_data();
                self.status_message = Some("Bookmarks".to_string());
            }
            ViewMode::Feeds => {
                self.load_feeds_data();
                self.status_message = Some("Feeds".to_string());
            }
            ViewMode::Browser => {
                self.status_message = Some("Browser".to_string());
            }
            ViewMode::Settings => {
                self.status_message = Some("Settings".to_string());
            }
            ViewMode::Reader => {
                self.toggle_reader_mode();
            }
        }
    }

    fn load_bookmarks_data(&mut self) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Ok(db) = db.try_lock() {
                let bookmarks = db.get_bookmarks().unwrap_or_default();
                debug!("Loaded {} bookmarks from DB", bookmarks.len());
            }
        });
    }

    fn load_feeds_data(&mut self) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Ok(db) = db.try_lock() {
                let feeds = db.get_feeds().unwrap_or_default();
                debug!("Loaded {} feeds from DB", feeds.len());
            }
        });
    }

    pub fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        
        let content_lines = self.page_content.lines().count() as u16;
        self.total_lines = content_lines;
    }

    pub fn start_search(&mut self) {
        self.show_search = true;
        self.search_query = String::new();
        self.search_results = Vec::new();
        self.selected_search_result = 0;
        self.status_message = Some("Search: type to search page content".to_string());
    }

    pub fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }
        
        let query = self.search_query.to_lowercase();
        self.search_results = self.page_content
            .lines()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains(&query))
            .map(|(idx, _)| idx)
            .collect();
        
        self.selected_search_result = 0;
        
        if !self.search_results.is_empty() {
            self.scroll_offset = self.search_results[0] as u16;
        }
        
        self.status_message = Some(format!("Found {} matches", self.search_results.len()));
    }

    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.selected_search_result = (self.selected_search_result + 1) % self.search_results.len();
            self.scroll_offset = self.search_results[self.selected_search_result] as u16;
        }
    }

    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.selected_search_result = if self.selected_search_result > 0 {
                self.selected_search_result - 1
            } else {
                self.search_results.len() - 1
            };
            self.scroll_offset = self.search_results[self.selected_search_result] as u16;
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        use crossterm::event::KeyModifiers;

        if self.show_search {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('q') => {
                return false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up();
            }
            KeyCode::Char('g') => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.scroll_to_bottom();
                } else {
                    self.scroll_to_top();
                }
            }
            KeyCode::Char('G') => {
                self.scroll_to_bottom();
            }
            KeyCode::Char('h') => {
                self.go_back();
            }
            KeyCode::Char('l') => {
                self.go_forward();
            }
            KeyCode::Char('o') => {
                self.show_url_input = true;
                self.url_input = String::new();
                self.status_message = Some("Enter URL:".to_string());
            }
            KeyCode::Char('O') => {
                self.show_url_input = true;
                self.url_input = self.current_url.clone();
                self.status_message = Some("Edit URL:".to_string());
            }
            KeyCode::Char('/') => {
                self.start_search();
            }
            KeyCode::Enter => {
                if self.show_url_input {
                    if !self.url_input.is_empty() {
                        let url = self.url_input.clone();
                        if let Err(e) = self.navigate_to(&url) {
                            self.error_message = Some(format!("Failed to load: {}", e));
                            self.status_message = Some(format!("Error: {}", e));
                        }
                    }
                    self.show_url_input = false;
                    self.url_input = String::new();
                } else {
                    self.follow_link();
                }
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_link();
                } else {
                    self.next_link();
                }
            }
            KeyCode::Char('r') => {
                self.toggle_reader_mode();
            }
            KeyCode::Char('T') => {
                self.cycle_theme();
            }
            KeyCode::Char('b') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.switch_view(ViewMode::Bookmarks);
                } else {
                    self.add_bookmark();
                }
            }
            KeyCode::Char('s') => {
                self.save_article();
            }
            KeyCode::Char('f') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.start_search();
                } else {
                    self.switch_view(ViewMode::Feeds);
                }
            }
            KeyCode::Char('H') => {
                self.switch_view(ViewMode::Browser);
                self.status_message = Some("Browser view".to_string());
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char('R') => {
                if !self.current_url.is_empty() {
                    let url = self.current_url.clone();
                    if let Err(e) = self.navigate_to(&url) {
                        self.error_message = Some(format!("Failed to reload: {}", e));
                    }
                }
            }
            KeyCode::Char('n') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.new_tab();
                } else if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_link();
                } else {
                    self.next_link();
                }
            }
            KeyCode::Char('N') => {
                self.prev_link();
            }
            KeyCode::Char('w') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.close_tab();
                }
            }
            KeyCode::Char('t') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.new_tab();
                } else {
                    self.cycle_theme();
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let tab_num = c.to_digit(10).unwrap() as usize;
                if tab_num > 0 && tab_num <= self.tabs.len() {
                    self.active_tab = tab_num - 1;
                    self.sync_from_tab();
                    self.status_message = Some(format!("Switched to tab {}", tab_num));
                }
            }
            KeyCode::Char(c) => {
                if self.show_url_input {
                    self.url_input.push(c);
                }
            }
            KeyCode::Backspace => {
                if self.show_url_input {
                    self.url_input.pop();
                }
            }
            KeyCode::Esc => {
                self.show_help = false;
                self.show_url_input = false;
                self.show_search = false;
                self.search_query.clear();
                self.search_results.clear();
            }
            _ => {}
        }

        true
    }

    fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Enter => {
                self.perform_search();
            }
            KeyCode::Char('n') => {
                self.next_search_result();
            }
            KeyCode::Char('N') => {
                self.prev_search_result();
            }
            KeyCode::Char(c) => {
                if c == 'j' {
                    self.next_search_result();
                } else if c == 'k' {
                    self.prev_search_result();
                } else {
                    self.search_query.push(c);
                    self.perform_search();
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.perform_search();
            }
            KeyCode::Esc => {
                self.show_search = false;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Down => {
                self.next_search_result();
            }
            KeyCode::Up => {
                self.prev_search_result();
            }
            _ => {}
        }
        true
    }
}
