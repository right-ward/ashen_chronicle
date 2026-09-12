//! Compatibility façade for gameplay code that still reports presentation events.
//!
//! The shipped frontend is Bevy. These functions deliberately contain no
//! terminal rendering or input handling; migrated Bevy systems own presentation
//! and semantic input directly.

use crate::presentation::ScreenView;
use std::sync::atomic::{AtomicBool, Ordering};

static KEY_LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_menu_screen(_title: impl Into<String>, _subtitle: Option<String>, _art: Option<String>) {
}

pub fn line(_text: &str) {}

pub fn clear_log() {}

pub fn diagnostic(_text: &str) {}

pub fn prompt(_message: &str) -> std::io::Result<String> {
    Ok(String::new())
}

pub fn pause() {}

pub fn narrate(_message: &str) {}

pub fn choose_from_list(
    _title: &str,
    _options: &[String],
    _zero_label: Option<&str>,
) -> std::io::Result<Option<usize>> {
    Ok(None)
}

pub(crate) fn set_key_logging(enabled: bool) {
    KEY_LOGGING_ENABLED.store(enabled, Ordering::Relaxed);
}

pub(crate) fn key_logging_enabled() -> bool {
    KEY_LOGGING_ENABLED.load(Ordering::Relaxed)
}
