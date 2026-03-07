use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use ratatui::Frame;

pub struct Panel;

impl Panel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}
