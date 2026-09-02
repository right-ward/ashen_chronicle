use crossterm::cursor;
use crossterm::execute;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Spacing};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::{Block, LineGauge, Paragraph, Wrap};
use std::io;

use crate::presentation::{CombatResultView, CombatView};

pub(crate) fn choose_action(view: &CombatView) -> io::Result<usize> {
    let mut selected = 0usize;
    loop {
        render(view, Some(selected), None)?;

        match crate::ui::read_key()? {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                selected = selected.checked_sub(1).unwrap_or(view.actions.len() - 1);
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                selected = (selected + 1) % view.actions.len();
            }
            crossterm::event::KeyCode::Home => selected = 0,
            crossterm::event::KeyCode::End => selected = view.actions.len() - 1,
            crossterm::event::KeyCode::Char(c) if c.is_ascii_digit() => {
                let action = c.to_digit(10).unwrap() as usize;
                if action > 0 && action <= view.actions.len() {
                    return Ok(action - 1);
                }
            }
            crossterm::event::KeyCode::Enter => return Ok(selected),
            _ => {}
        }
    }
}

pub(crate) fn show_result(view: &CombatResultView) -> io::Result<()> {
    render(&view.combat, None, Some(view))
}

pub(crate) fn wait_for_key() -> io::Result<()> {
    let _ = crate::ui::read_key()?;
    Ok(())
}

fn render(
    view: &CombatView,
    selected_action: Option<usize>,
    result: Option<&CombatResultView>,
) -> io::Result<()> {
    crate::ui::draw_combat_screen(|frame, area| {
        let margin = if area.width <= 112 || area.height <= 36 {
            1
        } else {
            2
        };
        let outer = Rect {
            x: area.x.saturating_add(margin),
            y: area.y.saturating_add(margin),
            width: area.width.saturating_sub(margin.saturating_mul(2)),
            height: area.height.saturating_sub(margin.saturating_mul(2)),
        };
        if outer.width < 4 || outer.height < 8 {
            return;
        }

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Percentage(34),
                Constraint::Percentage(24),
            ])
            .spacing(Spacing::Overlap(1))
            .split(outer);

        let actors = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(Spacing::Overlap(1))
            .split(root[0]);

        render_actor(
            frame,
            actors[0],
            "Player",
            &view.character.display_name(),
            view.character.hp,
            view.character.max_hp,
            view.player_condition.as_deref(),
            Color::Indexed(124),
        );
        let enemy_detail = format!("Power: {}", view.enemy_power);
        render_actor(
            frame,
            actors[1],
            "Enemy",
            &view.enemy.name,
            view.enemy.current_hp,
            view.enemy.max_hp,
            Some(&enemy_detail),
            Color::Indexed(90),
        );

        let event_title = format!("Events — {} — Turn {}", view.location_name, view.turn);
        let visible = root[1].height.saturating_sub(2) as usize;
        let lines = if view.events.is_empty() {
            vec!["The encounter begins.".to_string()]
        } else {
            view.events
                .iter()
                .rev()
                .take(visible.max(1))
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        };
        frame.render_widget(
            Paragraph::new(lines.join("\n"))
                .block(
                    Block::default()
                        .borders(ratatui::widgets::Borders::ALL)
                        .title(event_title)
                        .merge_borders(MergeStrategy::Exact),
                )
                .wrap(Wrap { trim: false }),
            root[1],
        );

        let action_lines = if let Some(result) = result {
            vec![
                result.result_title.clone(),
                String::new(),
                result.result_note.clone(),
                String::new(),
                "Press any key to continue...".to_string(),
            ]
        } else {
            view.actions
                .iter()
                .enumerate()
                .map(|(index, action)| {
                    let marker = if selected_action == Some(index) {
                        '▶'
                    } else {
                        ' '
                    };
                    format!("{marker} {}. {action}", index + 1)
                })
                .chain([String::new(), "↑ ↓ / j k  Enter: choose".to_string()])
                .collect()
        };
        frame.render_widget(
            Paragraph::new(action_lines.join("\n"))
                .block(
                    Block::default()
                        .borders(ratatui::widgets::Borders::ALL)
                        .title(if result.is_some() {
                            "Result"
                        } else {
                            "Actions"
                        })
                        .merge_borders(MergeStrategy::Exact),
                )
                .wrap(Wrap { trim: true }),
            root[2],
        );
    })?;
    execute!(std::io::stdout(), cursor::Hide)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_actor(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    role: &str,
    name: &str,
    hp: i32,
    max_hp: i32,
    detail: Option<&str>,
    fill: Color,
) {
    let block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title(format!("{role}: {name}"))
        .merge_borders(MergeStrategy::Exact);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }
    let gauge_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let maximum = max_hp.max(1);
    let current = hp.clamp(0, maximum);
    let gauge = LineGauge::default()
        .ratio(current as f64 / maximum as f64)
        .label(format!("HP {current}/{maximum}"))
        .filled_symbol("█")
        .unfilled_symbol("░")
        .filled_style(Style::default().fg(fill).add_modifier(Modifier::BOLD))
        .unfilled_style(Style::default().fg(Color::Gray));
    frame.render_widget(gauge, gauge_area);

    let text_area = Rect {
        y: inner.y.saturating_add(1),
        height: inner.height.saturating_sub(1),
        ..inner
    };
    if let Some(detail) = detail {
        if text_area.height > 0 {
            frame.render_widget(
                Paragraph::new(detail.to_string()).wrap(Wrap { trim: true }),
                text_area,
            );
        }
    }
}
