use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Table, Row, Clear},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

// Utility: Centered area calculation
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// Keyboard popup function using ratatui 0.29 conventions
pub fn show_virtual_keyboard(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, kb_title: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = String::new();
    let keyboard_layout = vec![
        vec!['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'],
        vec!['K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T'],
        vec!['U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3'],
        vec!['4', '5', '6', '7', '8', '9', ' ', '←', '<', ' '],
    ];



    let mut cursor_pos = (0, 0);
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| {
            f.render_widget(Clear, f.size());
            let area = centered_rect(70, 50, f.size());
            
            let rows: Vec<Row> = keyboard_layout
                .iter()
                .enumerate()
                .map(|(y, row)| {
                    Row::new(
                        row.iter()
                            .enumerate()
                            .map(|(x, &ch)| {
                                if (y, x) == cursor_pos {
                                    format!("[{}]", ch)
                                } else {
                                    format!(" {} ", ch)
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            let max_cols = keyboard_layout.iter().map(|row| row.len()).max().unwrap_or(1);
            let widths = vec![Constraint::Length(4); max_cols];

            let table = Table::new(rows, widths)
                .block(Block::default().title("Keyboard").borders(Borders::ALL));
            
            f.render_widget(table, area);

            let input_area = Rect {
                x: area.x,
                y: area.y.saturating_sub(3),
                width: area.width,
                height: 3,
            };

            let preview = Paragraph::new(format!("Input: {}", input))
                .block(Block::default().borders(Borders::ALL).title(kb_title));
            f.render_widget(preview, input_area);
        })?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => {
                        if cursor_pos.1 > 0 {
                            cursor_pos.1 -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if cursor_pos.1 < keyboard_layout[0].len() - 1 {
                            cursor_pos.1 += 1;
                        }
                    }
                    KeyCode::Up => {
                        if cursor_pos.0 > 0 {
                            cursor_pos.0 -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if cursor_pos.0 < keyboard_layout.len() - 1 {
                            cursor_pos.0 += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let ch = keyboard_layout[cursor_pos.0][cursor_pos.1];
                        match ch {
                            '<' => return Ok(input),
                            '←' => {
                                input.pop();
                            }
                            _ => input.push(ch),
                        }
                    }

                    KeyCode::Esc => {
                        return Ok(input);
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            terminal.draw(|f| {
                let area = centered_rect(70, 50, f.size());
                f.render_widget(Clear, area);
                // draw table and preview here
            })?;
            last_tick = Instant::now();
        }
    }
}
