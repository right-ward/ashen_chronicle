//! Frontend-neutral interaction events.
//!
//! Graphical frontends translate their native input into these semantic events
//! before game and screen flows consume them.

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
    OpenSecondaryNavigation,
    Tab,
    Character(char),
    Backspace,
    Delete,
}
