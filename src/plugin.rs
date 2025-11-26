use egui::Ui;

pub trait Plugin: Send {
    fn ui(&mut self, ui: &mut Ui, session_snapshot: &str);
}
