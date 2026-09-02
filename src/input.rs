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
    Ok(match crate::ui::read_key()? {
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
    })
}

#[cfg(test)]
mod tests {
    use super::InputEvent;

    #[test]
    fn semantic_events_are_frontend_neutral() {
        let events = [
            InputEvent::Up,
            InputEvent::Down,
            InputEvent::Confirm,
            InputEvent::Cancel,
            InputEvent::Character('x'),
        ];

        assert_eq!(events[0], InputEvent::Up);
        assert_eq!(events[1], InputEvent::Down);
        assert_eq!(events[2], InputEvent::Confirm);
        assert_eq!(events[3], InputEvent::Cancel);
        assert_eq!(events[4], InputEvent::Character('x'));
    }
}
