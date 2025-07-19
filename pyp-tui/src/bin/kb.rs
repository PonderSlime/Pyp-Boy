use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Table, Row},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input_preview = String::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Layout: preview + button
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let preview = Paragraph::new(format!("Input: {}", input_preview))
                .block(Block::default().borders(Borders::ALL).title("Preview"));
            f.render_widget(preview, chunks[0]);

            let button_area = centered_rect(30, 5, chunks[1]);
            let button = Paragraph::new("Open Keyboard [Enter/K]")
                .block(Block::default().borders(Borders::ALL).title("Action"));
            f.render_widget(button, button_area);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('k') | KeyCode::Enter => {
                        input_preview = show_virtual_keyboard(&mut terminal, "Test")?;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

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
fn show_virtual_keyboard(
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

    loop {
        terminal.draw(|f| {
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

        if event::poll(Duration::from_millis(100))? {
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
    }
}
