use tui::widgets::{TableState};

// Tabs
pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }

    pub fn select(&mut self, index: usize) {
        self.index = index;
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

// Table
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<'a, T> StatefulTable<T> {
    pub fn new() -> Self {
        StatefulTable {
            state: TableState::default(),
            items: vec![],
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        return match self.state.selected() {
            None => None,
            Some(index) => Some(&self.items[index]),
        };
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
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
            None => 0,
        };

        self.state.select(Some(i));
        self.selected_item()
    }
}