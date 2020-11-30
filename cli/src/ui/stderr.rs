use vut::ui::*;

pub struct StderrUiHandler;

impl StderrUiHandler {
    pub fn new() -> Self {
        Self
    }
}

impl VutUiHandler for StderrUiHandler {
    fn event(&mut self, e: &UiEvent) {
        match e {
            UiEvent::DeprecationWarning(w) => eprintln!("DEPRECATION WARNING: {}", w),
        }
    }
}
