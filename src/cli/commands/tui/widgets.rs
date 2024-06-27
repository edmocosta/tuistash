use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::TableState;

pub(crate) const TABLE_HEADER_CELL_STYLE: Style =
    Style::new().fg(Color::Gray).add_modifier(Modifier::BOLD);
pub(crate) const TABLE_HEADER_ROW_STYLE: Style = Style::new().bg(Color::DarkGray);
pub(crate) const TABLE_SELECTED_ROW_STYLE: Style = Style::new().bg(Color::Gray);
pub(crate) const TABLE_SELECTED_ROW_SYMBOL: &str = "▍";

// Tabs
pub struct TabsState {
    pub index: usize,
}

impl TabsState {
    pub fn with_default_index(index: usize) -> Self {
        TabsState { index }
    }

    pub fn new() -> Self {
        TabsState { index: 0 }
    }

    pub fn select(&mut self, index: usize) {
        self.index = index;
    }
}

// Table
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn new() -> Self {
        StatefulTable {
            state: TableState::default(),
            items: Vec::new(),
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        match self.state.selected() {
            None => None,
            Some(index) => Some(&self.items[index]),
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn has_next(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        match self.state.selected() {
            Some(i) => i < self.items.len() - 1,
            None => true,
        }
    }

    pub fn next(&mut self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
        self.selected_item()
    }

    pub fn has_previous(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        match self.state.selected() {
            Some(i) => i > 0,
            None => false,
        }
    }

    pub fn previous(&mut self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.items.len() - 1,
        };

        self.state.select(Some(i));
        self.selected_item()
    }
}
