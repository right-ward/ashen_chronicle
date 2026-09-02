//! Frontend-neutral interaction events.
//!
//! Terminal-specific keyboard input is translated into these semantic events
//! before game and screen flows consume it. A graphical frontend can provide
//! equivalent events without exposing keyboard or crossterm details upstream.

use crossterm::event::KeyCode;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputEvent {
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Confirm,
    Cancel,
    Tab,
    Character(char),
    Backspace,
    Delete,
    Other,
}

pub(crate) fn read() -> io::Result<InputEvent> {
    Ok(from_key(crate::ui::read_key()?))
}

fn from_key(key: KeyCode) -> InputEvent {
    match key {
        KeyCode::Up | KeyCode::Char('k') => InputEvent::Up,
        KeyCode::Down | KeyCode::Char('j') => InputEvent::Down,
        KeyCode::Home => InputEvent::Home,
        KeyCode::End => InputEvent::End,
        KeyCode::PageUp => InputEvent::PageUp,
        KeyCode::PageDown => InputEvent::PageDown,
        KeyCode::Enter => InputEvent::Confirm,
        KeyCode::Esc => InputEvent::Cancel,
        KeyCode::Tab => InputEvent::Tab,
        KeyCode::Backspace => InputEvent::Backspace,
        KeyCode::Delete => InputEvent::Delete,
        KeyCode::Char(c) => InputEvent::Character(c),
        _ => InputEvent::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::{from_key, InputEvent};
    use crossterm::event::KeyCode;

    #[test]
    fn terminal_keys_map_to_semantic_events() {
        assert_eq!(from_key(KeyCode::Up), InputEvent::Up);
        assert_eq!(from_key(KeyCode::Char('k')), InputEvent::Up);
        assert_eq!(from_key(KeyCode::Down), InputEvent::Down);
        assert_eq!(from_key(KeyCode::Char('j')), InputEvent::Down);
        assert_eq!(from_key(KeyCode::Enter), InputEvent::Confirm);
        assert_eq!(from_key(KeyCode::Esc), InputEvent::Cancel);
        assert_eq!(from_key(KeyCode::Char('x')), InputEvent::Character('x'));
        assert_eq!(from_key(KeyCode::Backspace), InputEvent::Backspace);
    }
}
