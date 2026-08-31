use crossterm::cursor;
use crossterm::execute;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Spacing};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::{Block, LineGauge, Paragraph, Wrap};
use std::io;

const ACTIONS: [&str; 3] = ["Attack", "Guard", "Flee"];

#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_action(
    player_name: &str,
    player_hp: i32,
    player_max_hp: i32,
    player_condition: Option<&str>,
    enemy_name: &str,
    enemy_hp: i32,
    enemy_max_hp: i32,
    enemy_power: i32,
    location_name: &str,
    turn: u32,
    events: &[String],
) -> io::Result<usize> {
    let mut selected = 0usize;
    loop {
        render(
            player_name,
            player_hp,
            player_max_hp,
            player_condition,
            enemy_name,
            enemy_hp,
            enemy_max_hp,
            enemy_power,
            location_name,
            turn,
            events,
            Some(selected),
            None,
        )?;

        match crate::ui::read_key()? {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                selected = selected.checked_sub(1).unwrap_or(ACTIONS.len() - 1);
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                selected = (selected + 1) % ACTIONS.len();
            }
            crossterm::event::KeyCode::Home => selected = 0,
            crossterm::event::KeyCode::End => selected = ACTIONS.len() - 1,
            crossterm::event::KeyCode::Char(c) if ('1'..='3').contains(&c) => {
                return Ok(c.to_digit(10).unwrap() as usize - 1);
            }
            crossterm::event::KeyCode::Enter => return Ok(selected),
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn show_result(
    player_name: &str,
    player_hp: i32,
    player_max_hp: i32,
    player_condition: Option<&str>,
    enemy_name: &str,
    enemy_hp: i32,
    enemy_max_hp: i32,
    enemy_power: i32,
    location_name: &str,
    turn: u32,
    events: &[String],
    result_title: &str,
    result_note: &str,
) -> io::Result<()> {
    render(
        player_name,
        player_hp,
        player_max_hp,
        player_condition,
        enemy_name,
        enemy_hp,
        enemy_max_hp,
        enemy_power,
        location_name,
        turn,
        events,
        None,
        Some((result_title, result_note)),
    )
}

pub(crate) fn wait_for_key() -> io::Result<()> {
    let _ = crate::ui::read_key()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render(
    player_name: &str,
    player_hp: i32,
    player_max_hp: i32,
    player_condition: Option<&str>,
    enemy_name: &str,
    enemy_hp: i32,
    enemy_max_hp: i32,
    enemy_power: i32,
    location_name: &str,
    turn: u32,
    events: &[String],
    selected_action: Option<usize>,
    result: Option<(&str, &str)>,
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
            player_name,
            player_hp,
            player_max_hp,
            player_condition,
            Color::Indexed(124),
        );
        render_actor(
            frame,
            actors[1],
            "Enemy",
            enemy_name,
            enemy_hp,
            enemy_max_hp,
            Some(&format!("Power: {enemy_power}")),
            Color::Indexed(90),
        );

        let event_title = format!("Events — {location_name} — Turn {turn}");
        let visible = root[1].height.saturating_sub(2) as usize;
        let lines = if events.is_empty() {
            vec!["The encounter begins.".to_string()]
        } else {
            events
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

        let action_lines = if let Some((title, note)) = result {
            vec![
                title.to_string(),
                String::new(),
                note.to_string(),
                String::new(),
                "Press any key to continue...".to_string(),
            ]
        } else {
            ACTIONS
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
