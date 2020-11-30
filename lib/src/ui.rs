use std::borrow::Cow;

#[derive(Debug)]
pub enum UiEvent {
    DeprecationWarning(Cow<'static, str>),
}

pub trait VutUiHandler {
    fn event(&mut self, e: &UiEvent);
}
