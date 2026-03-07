use crate::{Article, Theme};

pub struct ContentRenderer {
    terminal_width: u16,
    theme: Theme,
}

impl ContentRenderer {
    pub fn new(terminal_width: u16, theme: Theme) -> Self {
        Self {
            terminal_width,
            theme,
        }
    }

    pub fn set_width(&mut self, width: u16) {
        self.terminal_width = width;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn render_page(&self, title: &str, content: &str, url: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let width = self.terminal_width as usize;

        let header = self.render_header(url);
        lines.extend(header);

        let title_block = self.render_title_block(title);
        lines.extend(title_block);

        let text_lines = self.wrap_text(content, width.saturating_sub(4));
        for line in text_lines {
            lines.push(format!("  {}", line));
        }

        lines
    }

    pub fn render_article(&self, article: &Article) -> Vec<String> {
        let mut lines = Vec::new();
        let width = self.terminal_width as usize;

        let header = self.render_header(&article.url);
        lines.extend(header);

        lines.push(String::new());
        lines.push(self.colorize(&format!("  {}", article.title), "bold"));

        if let Some(author) = &article.author {
            lines.push(self.colorize(&format!("  By {}", author), "dim"));
        }

        if let Some(date) = &article.published_at {
            let date_str = date.format("%Y-%m-%d").to_string();
            lines.push(self.colorize(&format!("  Published: {}", date_str), "dim"));
        }

        lines.push(self.colorize(
            &format!("  Reading time: {} min", article.reading_time),
            "dim",
        ));
        lines.push(String::new());

        let content_lines = self.wrap_text(&article.content, width.saturating_sub(4));
        for line in content_lines {
            lines.push(format!("  {}", line));
        }

        lines
    }

    pub fn render_links(&self, links: &[(String, String)], current_idx: usize) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push(self.colorize("Links:", "bold"));
        lines.push(String::new());

        for (i, (href, text)) in links.iter().enumerate() {
            let display_text = if text.len() > 50 {
                format!("{}...", &text[..47])
            } else {
                text.clone()
            };

            if i == current_idx {
                lines.push(self.colorize(&format!("  > {}", display_text), "reverse"));
                lines.push(self.colorize(&format!("    {}", href), "dim"));
            } else {
                lines.push(format!("    {}", display_text));
            }
        }

        lines
    }

    pub fn render_progress_bar(&self, current: u32, total: u32) -> String {
        let width = 30;
        let filled = if total > 0 {
            ((current as f32 / total as f32) * width as f32) as u32
        } else {
            0
        };

        let bar: String = (0..width)
            .map(|i| {
                if i < filled {
                    "█"
                } else if i == filled && current < total {
                    "▌"
                } else {
                    "░"
                }
            })
            .collect();

        let percentage = if total > 0 {
            (current as f32 / total as f32 * 100.0) as u32
        } else {
            0
        };

        format!("[{}] {}%", bar, percentage)
    }

    fn render_header(&self, url: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let width = self.terminal_width as usize;

        let separator = "─".repeat(width);
        lines.push(self.colorize(&separator, "dim"));

        let truncated_url = if url.len() > width.saturating_sub(4) {
            format!("...{}", &url[width.saturating_sub(7)..])
        } else {
            url.to_string()
        };

        let title = " ReadFlow ";
        let remaining = width.saturating_sub(truncated_url.len() + title.len() + 2);

        lines.push(self.colorize(title, "bold") + &truncated_url + &" ".repeat(remaining));
        lines.push(self.colorize(&separator, "dim"));

        lines
    }

    fn render_title_block(&self, title: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let width = self.terminal_width as usize;

        lines.push(String::new());

        let wrapped = self.wrap_text(title, width.saturating_sub(4));
        let max_len = wrapped.iter().map(|s| s.len()).max().unwrap_or(0);

        let border = "┌".to_string() + &"─".repeat(max_len + 2) + "┐";
        lines.push(self.colorize(&border, "dim"));

        for line in wrapped {
            let padding = max_len.saturating_sub(line.len());
            let border_l = "│ ";
            let border_r = " │";
            lines.push(
                self.colorize(border_l, "dim")
                    + &line
                    + &" ".repeat(padding)
                    + &self.colorize(&border_r, "dim"),
            );
        }

        let border = "└".to_string() + &"─".repeat(max_len + 2) + "┘";
        lines.push(self.colorize(&border, "dim"));
        lines.push(String::new());

        lines
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();

        for paragraph in text.split('\n') {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                lines.push(String::new());
                continue;
            }

            let words: Vec<&str> = paragraph.split_whitespace().collect();
            let mut current_line = String::new();

            for word in words {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }

            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }

        lines
    }

    fn colorize(&self, text: &str, style: &str) -> String {
        let ansi = match style {
            "bold" => "\x1b[1m",
            "dim" => "\x1b[2m",
            "italic" => "\x1b[3m",
            "underline" => "\x1b[4m",
            "reverse" => "\x1b[7m",
            "strike" => "\x1b[9m",
            "reset" => "\x1b[0m",
            _ => "\x1b[0m",
        };

        let fg = match self.theme {
            Theme::Light => "30",
            Theme::Dark => "37",
            Theme::Sepia => "30",
        };

        let bg = match self.theme {
            Theme::Light => "47",
            Theme::Dark => "40",
            Theme::Sepia => "43",
        };

        let (fg_code, bg_code) = if style == "reverse" {
            (bg, fg)
        } else {
            (fg, "")
        };

        let bg_str = if bg_code.is_empty() {
            String::new()
        } else {
            format!(";{}", bg_code)
        };

        if style == "reset" {
            format!("{}{}", ansi, "\x1b[0m")
        } else {
            format!("\x1b[{}{}{}m{}\x1b[0m", fg_code, bg_str, ansi, text)
        }
    }

    pub fn estimate_reading_time(text: &str) -> u32 {
        let word_count = text.split_whitespace().count();
        let wpm = 200;
        (word_count as f32 / wpm as f32).ceil() as u32
    }

    pub fn render_help() -> Vec<String> {
        vec![
            String::new(),
            "  Navigation".to_string(),
            "  ──────────".to_string(),
            "  j/k, Down/Up  Scroll down/up".to_string(),
            "  g/G           Go to top/bottom".to_string(),
            "  h/l           Go back/forward in history".to_string(),
            "  o             Open URL".to_string(),
            "  Enter         Follow link".to_string(),
            "  Tab           Switch panel".to_string(),
            String::new(),
            "  Reader Mode".to_string(),
            "  ──────────".to_string(),
            "  r             Toggle reader mode".to_string(),
            "  t             Cycle theme".to_string(),
            "  b             Add bookmark".to_string(),
            "  s             Save article".to_string(),
            String::new(),
            "  Feeds".to_string(),
            "  ─────".to_string(),
            "  f             Open feeds".to_string(),
            "  a             Add feed".to_string(),
            "  R             Refresh feeds".to_string(),
            String::new(),
            "  Other".to_string(),
            "  ─────".to_string(),
            "  ?             Show help".to_string(),
            "  q             Quit".to_string(),
            "  Ctrl+c        Quit".to_string(),
        ]
    }
}
