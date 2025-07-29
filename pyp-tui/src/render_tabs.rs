use reverse_geocoder::ReverseGeocoder;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;
use ratatui::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs;
use thiserror::Error;
use rand::{distributions::Alphanumeric, prelude::*};

use crate::kb;
use kb::{show_virtual_keyboard, centered_rect};

const DB_PATH: &str = "./data/db.json";

use ratatui::widgets::{
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap
};
use ratatui::layout::{Alignment, Constraint};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Line, Text};

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub details: String,
    pub quantity: u32,
    pub category: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

pub fn get_map_data<'a>(coordinates: [f64; 2]) -> String {
    let geocoder = ReverseGeocoder::new();
    let spatial_time = spatialtime::osm::lookup(coordinates[0], coordinates[1]).unwrap();
    let coord_tuple: (f64, f64) = (coordinates[0], coordinates[1]);
    let search_result = geocoder.search(coord_tuple);
    format!(
        "Latitude: {}\nLongitude: {}\nAddress: {}, {}, {}, {}\nTime Zone: {:?}",
        coordinates[0],
        coordinates[1],
        search_result.record.name,
        search_result.record.admin1,
        search_result.record.admin2,
        search_result.record.cc,
        spatial_time.tzid
    )
}

pub fn render_map<'a>(map_text: Option<String>) -> Paragraph<'a> {
    let bold_style = Style::default().add_modifier(Modifier::BOLD);
    let text_content: std::borrow::Cow<'a, str> = match map_text {
        Some(s) => std::borrow::Cow::Owned(s),
        None => std::borrow::Cow::Borrowed("Map data not available"),
    };

    let map_text_finished = Text::styled(text_content, bold_style);

    Paragraph::new(map_text_finished)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Map")
                .border_type(BorderType::Plain),
        )
}

pub fn render_stat<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Welcome")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("to")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Pyp-Boy", Style::default().fg(Color::LightBlue))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'i' to access inventory, 'a' to add random new items and 'd' to delete the currently selected item.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    )
}

pub fn render_inv<'a>(mut inv_list_state: &ListState, category_filter: &'a str) -> (List<'a>, Paragraph<'a>) {
    let invs = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(category_filter)
        .border_type(BorderType::Plain);

    let inv_list = read_db().expect("can fetch item list");

    let mut filtered_items: Vec<Item> = inv_list
        .into_iter()
        .filter(|item| item.category.eq_ignore_ascii_case(category_filter))
        .collect();

    filtered_items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let mut items: Vec<_> = filtered_items
        .iter()
        .map(|item| {
            ListItem::new(Line::from(vec![Span::styled(
                item.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    items.push(ListItem::new(Line::from(vec![Span::styled(
        "+ Add New",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )])));

    let selected_item = filtered_items
        .get(inv_list_state.selected().unwrap_or(0))
        .cloned()
        .unwrap_or(Item {
            id: 0,
            name: "ERR".into(),
            details: "ERR".into(),
            quantity: 0,
            category: category_filter.into(),
            created_at: chrono::Utc::now(),
        });

    let list = List::new(items).block(invs).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let detail_lines = vec![
        Line::from(vec![Span::styled(
            format!("Name: {}", selected_item.name),
            Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(format!("Created: {}", selected_item.created_at))]),
        Line::from(""),
        Line::from(vec![Span::styled("Details:", Style::default().fg(Color::LightBlue))]),
        Line::from(vec![Span::raw(format!("{}", selected_item.details))]),
        Line::from(""),
        Line::from(vec![Span::styled("Quantity:", Style::default().fg(Color::LightBlue))]),
        Line::from(vec![Span::raw(format!("{}", selected_item.quantity))])
    ];

    let paragraph = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .title("Item Detail")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .wrap(Wrap { trim: true });

    (list, paragraph)
}


pub fn add_item_to_db() -> Result<Vec<Item>, Error> {
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Item> = serde_json::from_str(&db_content)?;

    let name = show_virtual_keyboard(&mut terminal, "Item Name")
        .map_err(|e| Error::ReadDBError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
    let category = show_category_selector(&mut terminal)
        .map_err(|e| Error::ReadDBError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
    let details = show_virtual_keyboard(&mut terminal, "Item Details")
        .map_err(|e| Error::ReadDBError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
    let quantity = show_quantity_selector(&mut terminal, 0)
        .map_err(|e| Error::ReadDBError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
    let new_item = Item {
        id: rng.gen_range(0, 9999999),
        name,
        details,
        quantity,
        category,
        created_at: Utc::now(),
    };

    parsed.push(new_item);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

pub fn show_category_selector<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<String> {
    let categories = vec!["Weapons", "Apparel", "Aid", "Misc", "Junk", "Mods", "Ammo"];
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        terminal.clear();
        terminal.draw(|f| {
            let size = centered_rect(70, 50, f.area());
            let items: Vec<ListItem> = categories.iter().map(|c| ListItem::new(*c)).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Category"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightBlue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(list, size, &mut state);
        })?;

        if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? {
            match key.code {
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => 0,
                    };
                    state.select(Some(i));
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let i = match state.selected() {
                        Some(i) if i < categories.len() - 1 => i + 1,
                        _ => categories.len() - 1,
                    };
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        return Ok(categories[i].to_string());
                    }
                }
                KeyCode::Esc => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Category selection cancelled"));
                }
                _ => {}
            }
        }
    }
}

pub fn show_quantity_selector<B: Backend>(terminal: &mut Terminal<B>, initial_quantity: u32) -> io::Result<u32> {
    let categories: Vec<String> = (0..101).map(|i| i.to_string()).collect();
    let mut state = ListState::default();
    state.select(Some(initial_quantity as usize));

    loop {
        terminal.clear();
        terminal.draw(|f| {
            let size = centered_rect(70, 50, f.area());
            let items: Vec<ListItem> = categories.iter().map(|c| ListItem::new(c.as_str())).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Quantity"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightBlue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(list, size, &mut state);
        })?;

        if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? {
            match key.code {
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => categories.len() - 1,
                    };
                    state.select(Some(i));
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let i = match state.selected() {
                        Some(i) if i < categories.len() - 1 => i + 1,
                        _ => 0,
                    };
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(selected_index) = state.selected() {
                        let quantity_str = categories[selected_index].clone();
                        let quantity: u32 = quantity_str.parse().map_err(|e| {
                            io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse quantity: {}", e))
                        })?;
                        return Ok(quantity);
                    }
                }
                KeyCode::Esc => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Selection cancelled"));
                }
                _ => {}
            }
        }
    }
}

pub fn render_data<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Welcome")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("to")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Pyp-Boy", Style::default().fg(Color::LightBlue))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'i' to access inventory, 'a' to add random new items and 'd' to delete the currently selected item.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    )
}

pub fn read_db() -> Result<Vec<Item>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Item> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}
