use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Table, Row, Clear},
};
use crossterm::{
    event::{self, Event, KeyCode},
};
use std::io;
use std::time::{Duration, Instant};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn show_virtual_keyboard(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    kb_title: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = String::new();
    let keyboard_layout = vec![
        vec!['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'],
        vec!['K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T'],
        vec!['U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3'],
        vec!['4', '5', '6', '7', '8', '9', ' ', '←', '<', ' '], 
    ];

    let mut cursor_pos = (0, 0);

    loop {
        terminal.clear();
        terminal.draw(|f| {
            
            let area = centered_rect(70, 50, f.area());
            f.render_widget(Clear, area);
            
            let rows: Vec<Row> = keyboard_layout
                .iter()
                .enumerate()
                .map(|(y, row)| {
                    Row::new(
                        row.iter()
                            .enumerate()
                            .map(|(x, &ch)| {
                                let content = if ch == '←' {
                                    "Bksp".to_string()
                                } else {
                                    ch.to_string()
                                };

                                if (y, x) == cursor_pos {
                                    Span::styled(format!("[{}]", content), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                                } else {
                                    Span::raw(format!(" {} ", content))
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();

            let max_cols = keyboard_layout.iter().map(|row| row.len()).max().unwrap_or(1);
            let widths: Vec<Constraint> = vec![Constraint::Length(6); max_cols];

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

        if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? {
                match key.code {
                    KeyCode::Char('A') | KeyCode::Char('a') => {
                        if cursor_pos.1 > 0 {
                            cursor_pos.1 -= 1;
                        }
                    }
                    KeyCode::Char('D') | KeyCode::Char('d') => {
                        if cursor_pos.1 < keyboard_layout[cursor_pos.0].len() - 1 {
                            cursor_pos.1 += 1;
                        }
                    }
                    KeyCode::Char('W') | KeyCode::Char('w') => {
                        if cursor_pos.0 > 0 {
                            cursor_pos.0 -= 1;
                            
                            if cursor_pos.1 >= keyboard_layout[cursor_pos.0].len() {
                                cursor_pos.1 = keyboard_layout[cursor_pos.0].len() - 1;
                            }
                        }
                    }
                    KeyCode::Char('S') | KeyCode::Char('s') => {
                        if cursor_pos.0 < keyboard_layout.len() - 1 {
                            cursor_pos.0 += 1;
                           
                            if cursor_pos.1 >= keyboard_layout[cursor_pos.0].len() {
                                cursor_pos.1 = keyboard_layout[cursor_pos.0].len() - 1;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        let ch = keyboard_layout[cursor_pos.0][cursor_pos.1];
                        match ch {
                            '←' => {
                                input.pop();
                            }
                            '<' => return Ok(input),
                            ' ' if cursor_pos == (3, 8) => {
                                return Ok(input);
                            }
                            _ => input.push(ch),
                        }
                    }
                    KeyCode::Char(c) => {
                        if c.is_ascii_alphanumeric() || c == ' ' {
                            input.push(c);
                        }
                    }
                    _ => {}
                }
            
        }
    }
}