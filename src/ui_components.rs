use ratatui::layout::{Direction, Rect, Spacing};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::{Block, Borders, Clear, LineGauge, Paragraph, Wrap};

pub(crate) fn clear_frame(frame: &mut ratatui::Frame<'_>, area: Rect) {
    frame.render_widget(Clear, area);
}

pub(crate) fn is_compact(area: Rect) -> bool {
    area.width <= 112 || area.height <= 36 || area.width <= area.height.saturating_mul(2)
}

pub(crate) fn bottom_panel_height(area: Rect, compact: bool, content_lines: usize) -> u16 {
    let base_height = if compact { 7 } else { 6 };
    let max_height = if compact {
        area.height.saturating_mul(40) / 100
    } else {
        area.height.saturating_mul(34) / 100
    };
    let desired = content_lines as u16 + 4;
    desired.clamp(base_height, max_height.max(base_height))
}

pub(crate) fn vertical_or_horizontal(compact: bool) -> Direction {
    if compact { Direction::Vertical } else { Direction::Horizontal }
}

pub(crate) fn overlap_spacing() -> Spacing { Spacing::Overlap(1) }

pub(crate) fn render_health_gauge(frame: &mut ratatui::Frame<'_>, area: Rect, label: &str, current: i32, maximum: i32) {
    if area.height == 0 || area.width == 0 { return; }
    let maximum = maximum.max(1);
    let current = current.clamp(0, maximum);
    let gauge = LineGauge::default()
        .ratio(current as f64 / maximum as f64)
        .label(format!("{}: ", label))
        .filled_symbol("█")
        .unfilled_symbol("░")
        .filled_style(Style::default().fg(Color::Indexed(124)))
        .unfilled_style(Style::default().fg(Color::Gray));
    frame.render_widget(gauge, area);
}

pub(crate) fn render_panel(frame: &mut ratatui::Frame<'_>, area: Rect, title: &str, lines: &[String], compact: bool) {
    let content = if lines.is_empty() { vec![String::new()] } else { lines.to_vec() };
    let paragraph = Paragraph::new(content.join("\n"))
        .block(panel_block(title, compact))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

pub(crate) fn render_scrolling_text(frame: &mut ratatui::Frame<'_>, area: Rect, title: &str, lines: &[String], compact: bool) {
    let content = if lines.is_empty() { vec![String::new()] } else { lines.to_vec() };
    let scroll = content.len().saturating_sub(area.height.saturating_sub(2) as usize) as u16;
    let paragraph = Paragraph::new(content.join("\n"))
        .block(panel_block(title, compact))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

pub(crate) fn render_message_panel(frame: &mut ratatui::Frame<'_>, area: Rect, title: &str, lines: &[String], compact: bool) {
    let paragraph = Paragraph::new(lines.join("\n"))
        .block(panel_block(title, compact))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

pub(crate) fn panel_block(title: &str, compact: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(title.to_string())
        .style(border_style(compact))
        .merge_borders(MergeStrategy::Exact)
}

pub(crate) fn border_style(compact: bool) -> Style {
    if compact { Style::default().fg(Color::Gray) }
    else { Style::default().fg(Color::White).add_modifier(Modifier::BOLD) }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;
    use super::{bottom_panel_height, is_compact};

    #[test]
    fn compact_layout_is_selected_for_narrow_or_short_areas() {
        assert!(is_compact(Rect::new(0, 0, 100, 40)));
        assert!(is_compact(Rect::new(0, 0, 120, 30)));
        assert!(!is_compact(Rect::new(0, 0, 140, 40)));
    }

    #[test]
    fn bottom_panel_height_respects_content_and_limits() {
        let area = Rect::new(0, 0, 120, 40);
        assert_eq!(bottom_panel_height(area, false, 0), 6);
        assert_eq!(bottom_panel_height(area, false, 4), 8);
        assert!(bottom_panel_height(area, false, 100) <= 13);
    }
}
