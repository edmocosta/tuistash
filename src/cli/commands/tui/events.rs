use crate::commands::tui::app::AppData;
use crossterm::event::KeyEvent;

pub trait EventsListener {
    fn focus_gained(&mut self, app_data: &AppData);
    fn focus_lost(&mut self, app_data: &AppData);
    fn on_enter(&mut self, app_data: &AppData);
    fn on_left(&mut self, app_data: &AppData);
    fn on_right(&mut self, app_data: &AppData);
    fn on_up(&mut self, app_data: &AppData);
    fn on_down(&mut self, app_data: &AppData);
    fn on_other(&mut self, event: KeyEvent, app_data: &AppData);
}
