use crate::commands::tui::app::AppData;
use crossterm::event::KeyEvent;

pub(crate) trait EventsListener {
    fn update(&mut self, app_data: &AppData);
    fn reset(&mut self);
    fn focus_gained(&mut self, _: &AppData) {}
    fn focus_lost(&mut self, _: &AppData) {}
    fn on_enter(&mut self, _: &AppData) {}
    fn on_left(&mut self, _: &AppData) {}
    fn on_right(&mut self, _: &AppData) {}
    fn on_up(&mut self, _: &AppData) {}
    fn on_down(&mut self, _: &AppData) {}
    fn on_other(&mut self, _: KeyEvent, _: &AppData) {}
}
