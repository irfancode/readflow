use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;

use super::app::App;
use crate::browser::ContentRenderer;

pub fn run_app(mut app: App) -> Result<()> {
    let backend = CrosstermBackend::new(io::stderr());

    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        let size = terminal.size()?;
        app.update_size(size.width, size.height);

        terminal.draw(|f| {
            render_app(f, &mut app);
        })?;

        if let Ok(event) = crossterm::event::read() {
            if let crossterm::event::Event::Key(key) = event {
                if !app.handle_key(key) {
                    break;
                }
            }

            if let crossterm::event::Event::Resize(w, h) = event {
                app.update_size(w, h);
            }
        }
    }

    Ok(())
}

fn render_app(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_status(f, app, chunks[2]);

    if app.show_help {
        render_help_popup(f, app);
    }

    if app.show_url_input {
        render_url_input(f, app);
    }

    if app.show_search {
        render_search_input(f, app);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let mode_indicator = match app.view_mode {
        crate::ViewMode::Browser => {
            if app.loading {
                " 🔄 Loading..."
            } else {
                ""
            }
        }
        crate::ViewMode::Reader => " 📖 Reader",
        crate::ViewMode::Feeds => " 📰 Feeds",
        crate::ViewMode::Bookmarks => " 🔖 Bookmarks",
        crate::ViewMode::Settings => " ⚙️ Settings",
    };

    let title = format!(
        " ReadFlow{}{} ",
        mode_indicator,
        if app.view_mode != crate::ViewMode::Browser {
            ""
        } else {
            " - Browser"
        }
    );

    let url_display = if app.current_url.len() > area.width as usize - title.len() - 5 {
        format!(
            "...{}",
            &app.current_url[app.current_url.len() - (area.width as usize - title.len() - 8)..]
        )
    } else {
        app.current_url.clone()
    };

    let title_with_url = format!("{}{}", title, url_display);

    let block = Block::default()
        .title(Line::from(Span::styled(
            title_with_url,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(get_theme_style(app.theme, false));

    let paragraph = Paragraph::new("")
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)));

    f.render_widget(paragraph, area);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(get_theme_style(app.theme, false));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    match app.view_mode {
        crate::ViewMode::Browser | crate::ViewMode::Reader => {
            render_browser_content(f, app, inner_area);
        }
        crate::ViewMode::Feeds => {
            render_feeds_content(f, app, inner_area);
        }
        crate::ViewMode::Bookmarks => {
            render_bookmarks_content(f, app, inner_area);
        }
        crate::ViewMode::Settings => {
            render_settings_content(f, app, inner_area);
        }
    }
}

fn render_browser_content(f: &mut Frame, app: &mut App, area: Rect) {
    let renderer = ContentRenderer::new(area.width, app.theme);

    if app.loading {
        let loading_text = vec![Line::from(Span::styled(
            "  Loading...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];
        let paragraph = Paragraph::new(loading_text)
            .block(Block::default())
            .style(Style::default().fg(get_text_color(app.theme)));
        f.render_widget(paragraph, area);
        return;
    }

    let content = if app.view_mode == crate::ViewMode::Reader {
        if let Some(article) = &app.reader_content {
            renderer.render_article(article)
        } else {
            vec![String::from(
                "No article loaded. Press 'r' to try reader mode or navigate to a page.",
            )]
        }
    } else if app.page_content.is_empty() && app.current_url.is_empty() {
        render_welcome_screen(app, area)
    } else {
        renderer.render_page(&app.page_title, &app.page_content, &app.current_url)
    };

    let max_scroll = content.len().saturating_sub(area.height as usize);
    let scroll = app.scroll_offset.min(max_scroll as u16) as usize;

    let scrollable_content: Vec<Line> = content
        .iter()
        .skip(scroll)
        .take(area.height as usize)
        .enumerate()
        .map(|(idx, s)| {
            if app.show_search && !app.search_results.is_empty() {
                let line_num = scroll + idx;
                if app.search_results.contains(&line_num) {
                    Line::from(Span::styled(
                        s.clone(),
                        Style::default().bg(Color::Yellow).fg(Color::Black),
                    ))
                } else {
                    Line::from(Span::raw(s.clone()))
                }
            } else {
                Line::from(Span::raw(s.clone()))
            }
        })
        .collect();

    let paragraph = Paragraph::new(scrollable_content)
        .block(Block::default())
        .style(Style::default().fg(get_text_color(app.theme)))
        .scroll((app.scroll_offset, 0));

    f.render_widget(paragraph, area);

    if app.view_mode == crate::ViewMode::Browser && !app.links.is_empty() {
        let links_height = 5u16.min(((app.links.len() + 1).min(3)) as u16);
        let links_section = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(links_height),
            area.width,
            links_height,
        );

        let links_text: Vec<Line> = app
            .links
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, (_href, text))| {
                let display = if text.len() > 40 {
                    format!("{}...", &text[..37])
                } else {
                    text.clone()
                };

                if i == app.selected_link {
                    Line::from(Span::styled(
                        format!(" ▶ {}", display),
                        Style::default()
                            .bg(get_accent_color(app.theme))
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::raw(format!("   {}", display)))
                }
            })
            .collect();

        let links_block = Block::default()
            .title(" Links ")
            .borders(Borders::ALL)
            .border_style(get_theme_style(app.theme, false));

        let links_par = Paragraph::new(links_text)
            .block(links_block)
            .style(Style::default().fg(get_text_color(app.theme)));

        f.render_widget(links_par, links_section);
    }
}

fn render_welcome_screen(_app: &App, area: Rect) -> Vec<String> {
    let width = area.width as usize;
    let mut lines = Vec::new();

    let welcome = r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   ███████╗ ██████╗ ██████╗  ██████╗  █████╗ ██████╗     ║
║   ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔══██╗██╔══██╗    ║
║   ███████╗██║   ██║██████╔╝██║  ███╗███████║██████╔╝    ║
║   ╚════██║██║   ██║██╔══██╗██║   ██║██╔══██║██╔══██╗    ║
║   ███████║╚██████╔╝██║  ██║╚██████╔╝██║  ██║██║  ██║    ║
║   ╚══════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝    ║
║                      v0.1.0                              ║
║                                                           ║
║   A modern TUI browser for all platforms                 ║
║                                                           ║
╠═══════════════════════════════════════════════════════════╣
║  Quick Start:                                             ║
║                                                           ║
║    Press 'o' to open a URL                                ║
║    Press '?' for help                                     ║
║    Press 't' to change theme                              ║
║    Press 'f' to view feeds                               ║
║    Press 'Ctrl+b' for bookmarks                           ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"#
    .trim();

    for line in welcome.lines() {
        let trimmed = line.trim_end();
        let chars: Vec<char> = trimmed.chars().collect();
        if chars.len() > width {
            lines.push(chars[..width].iter().collect());
        } else {
            lines.push(trimmed.to_string());
        }
    }

    lines
}

fn render_feeds_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" 📰 Feeds ")
        .borders(Borders::ALL)
        .border_style(get_theme_style(app.theme, false));

    let content = if app.feeds.is_empty() {
        vec![
            Line::from(Span::styled(
                "  No feeds added.",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  Press 'a' to add a feed URL")),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  Press 'h' to return to browser",
                Style::default().fg(Color::Cyan),
            )),
        ]
    } else {
        app.feeds
            .iter()
            .enumerate()
            .map(|(i, feed)| {
                let prefix = if i == app.selected_feed { "▶" } else { " " };
                Line::from(Span::raw(format!(" {} {}", prefix, feed.title)))
            })
            .collect()
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)));

    f.render_widget(paragraph, area);
}

fn render_bookmarks_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" 🔖 Bookmarks ")
        .borders(Borders::ALL)
        .border_style(get_theme_style(app.theme, false));

    let content = if app.bookmarks.is_empty() {
        vec![
            Line::from(Span::styled(
                "  No bookmarks yet.",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  Press 'b' on any page to bookmark it")),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  Press 'h' to return to browser",
                Style::default().fg(Color::Cyan),
            )),
        ]
    } else {
        app.bookmarks
            .iter()
            .enumerate()
            .map(|(i, (_, title, _url, _))| {
                let prefix = if i == app.selected_bookmark {
                    "▶"
                } else {
                    " "
                };
                let title = if title.len() > 50 {
                    format!("{}...", &title[..47])
                } else {
                    title.clone()
                };
                Line::from(Span::raw(format!(" {} {}", prefix, title)))
            })
            .collect()
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)));

    f.render_widget(paragraph, area);
}

fn render_settings_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" ⚙️ Settings ")
        .borders(Borders::ALL)
        .border_style(get_theme_style(app.theme, false));

    let theme_name = match app.theme {
        crate::Theme::Light => "Light",
        crate::Theme::Dark => "Dark",
        crate::Theme::Sepia => "Sepia",
    };

    let (history_count, can_go_back, can_go_forward) = app.get_history_info();

    let content = vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  Theme Settings:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(format!(
            "    Current: {}  (press 't' to cycle)",
            theme_name
        ))),
        Line::from(Span::raw(format!(
            "    Background: {}",
            app.theme.background_color()
        ))),
        Line::from(Span::raw(format!("    Text: {}", app.theme.text_color()))),
        Line::from(Span::raw(format!(
            "    Accent: {}",
            app.theme.accent_color()
        ))),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(format!("    History: {} pages", history_count))),
        Line::from(Span::raw(format!(
            "    Can go back: {}",
            if can_go_back { "Yes (h)" } else { "No" }
        ))),
        Line::from(Span::raw(format!(
            "    Can go forward: {}",
            if can_go_forward { "Yes (l)" } else { "No" }
        ))),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  Press 'h' to return to browser",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)));

    f.render_widget(paragraph, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let mut status_parts = Vec::new();

    if let Some(ref error) = app.error_message {
        status_parts.push(Span::styled(
            format!(" ✗ {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    } else if let Some(ref msg) = app.status_message {
        status_parts.push(Span::raw(format!(" {}", msg)));
    }

    if app.view_mode == crate::ViewMode::Browser && app.links.len() > 0 {
        status_parts.push(Span::raw(format!(
            " | Links: {}/{} ",
            app.selected_link + 1,
            app.links.len()
        )));
    }

    if app.show_search && !app.search_results.is_empty() {
        status_parts.push(Span::raw(format!(
            " | Search: {}/{} (n/N) ",
            app.selected_search_result + 1,
            app.search_results.len()
        )));
    }

    let progress = if app.total_lines > 0 {
        let pct = (app.scroll_offset as f32 / app.total_lines as f32 * 100.0) as u32;
        format!("{}%", pct)
    } else {
        "0%".to_string()
    };

    let right_status = format!("[{}]", progress);

    let total_width =
        status_parts.iter().map(|s| s.content.len()).sum::<usize>() + right_status.len() + 2;
    let padding = (area.width as usize).saturating_sub(total_width);

    status_parts.push(Span::raw(" ".repeat(padding)));
    status_parts.push(Span::styled(
        right_status,
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(status_parts);

    let paragraph = Paragraph::new(line).style(
        Style::default()
            .fg(get_text_color(app.theme))
            .bg(get_bg_color(app.theme)),
    );

    f.render_widget(paragraph, area);
}

fn render_help_popup(f: &mut Frame, app: &App) {
    let area = Rect::new(
        f.area().width / 2 - 35,
        f.area().height / 2 - 15,
        f.area().width / 2 + 35,
        f.area().height / 2 + 15,
    );

    let block = Block::default()
        .title(" Help - Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled(
            "  Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("  ─────────────────────────────")),
        Line::from(Span::raw("  j/k, ↑/↓     Scroll down/up")),
        Line::from(Span::raw("  g             Go to top")),
        Line::from(Span::raw("  G             Go to bottom")),
        Line::from(Span::raw("  h             Go back in history")),
        Line::from(Span::raw("  l             Go forward in history")),
        Line::from(Span::raw("  o             Open URL")),
        Line::from(Span::raw(
            "  O             Open URL with current as default",
        )),
        Line::from(Span::raw("  R             Reload page")),
        Line::from(Span::raw("  /             Search page")),
        Line::from(Span::raw("  n/N           Next/previous search result")),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  Links",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("  ─────────────────────────────")),
        Line::from(Span::raw("  Tab           Next link")),
        Line::from(Span::raw("  Shift+Tab     Previous link")),
        Line::from(Span::raw("  n             Next link")),
        Line::from(Span::raw("  N             Previous link")),
        Line::from(Span::raw("  Enter         Follow selected link")),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  Views & Actions",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("  ─────────────────────────────")),
        Line::from(Span::raw("  r             Toggle reader mode")),
        Line::from(Span::raw("  t             Cycle theme")),
        Line::from(Span::raw("  b             Add bookmark")),
        Line::from(Span::raw("  Ctrl+b        View bookmarks")),
        Line::from(Span::raw("  f             View feeds")),
        Line::from(Span::raw("  s             Save article")),
        Line::from(Span::raw("  Ctrl+f        Search page")),
        Line::from(Span::raw("  h             Return to browser")),
        Line::from(Span::raw("")),
        Line::from(Span::raw("  ?             Toggle this help")),
        Line::from(Span::raw("  q             Quit")),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)))
        .scroll((0, 0));

    f.render_widget(paragraph, area);
}

fn render_url_input(f: &mut Frame, app: &App) {
    let area = Rect::new(
        5,
        f.area().height / 2 - 1,
        f.area().width - 5,
        f.area().height / 2 + 2,
    );

    let prompt = if app.url_input.is_empty() {
        " URL: "
    } else {
        " "
    };

    let block = Block::default()
        .title(prompt)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(app.url_input.as_str())
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)));

    f.render_widget(paragraph, area);
    f.set_cursor_position((area.x + 2 + app.url_input.len() as u16, area.y + 1));
}

fn render_search_input(f: &mut Frame, app: &App) {
    let area = Rect::new(5, 4, f.area().width - 5, 6);

    let query_display = format!(" Search: {}", app.search_query);

    let block = Block::default()
        .title(query_display.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let status = if app.search_results.is_empty() && !app.search_query.is_empty() {
        "No matches".to_string()
    } else if !app.search_results.is_empty() {
        format!(
            "{}/{}",
            app.selected_search_result + 1,
            app.search_results.len()
        )
    } else {
        "Type to search...".to_string()
    };

    let content = vec![Line::from(Span::raw(status))];

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(get_text_color(app.theme)));

    f.render_widget(paragraph, area);
}

fn get_theme_style(_theme: crate::Theme, selected: bool) -> Style {
    if selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn get_text_color(theme: crate::Theme) -> Color {
    match theme {
        crate::Theme::Light => Color::Black,
        crate::Theme::Dark => Color::White,
        crate::Theme::Sepia => Color::Rgb(91, 70, 54),
    }
}

fn get_bg_color(theme: crate::Theme) -> Color {
    match theme {
        crate::Theme::Light => Color::White,
        crate::Theme::Dark => Color::Black,
        crate::Theme::Sepia => Color::Rgb(244, 236, 216),
    }
}

fn get_accent_color(theme: crate::Theme) -> Color {
    match theme {
        crate::Theme::Light => Color::Blue,
        crate::Theme::Dark => Color::Cyan,
        crate::Theme::Sepia => Color::Rgb(139, 69, 19),
    }
}
