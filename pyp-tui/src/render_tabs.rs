use reverse_geocoder::{ReverseGeocoder};
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

use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap
    },
};
use crate::kb;
use kb::{show_virtual_keyboard, centered_rect};
const DB_PATH: &str = "./data/db.json";
use ratatui::widgets;
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
    let map_text = format!("Latitude: {}\nLongitude: {}\nAddress: {}, {}, {}, {}\nTime Zone: {:?}", coordinates[0], coordinates[1], search_result.record.name, search_result.record.admin1, search_result.record.admin2, search_result.record.cc, spatial_time.tzid);
    return map_text;
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
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Pyp-Boy",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'i' to access inventory, 'a' to add random new items and 'd' to delete the currently selected item.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

pub fn render_inv<'a>(
    inv_list_state: &ListState,
    category_filter: &'a str,
) -> (List<'a>, Paragraph<'a>) {    
    let invs = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(category_filter)
        .border_type(BorderType::Plain);

    let inv_list = read_db().expect("can fetch item list");

    // 🔍 Filter by category
    let filtered_items: Vec<Item> = inv_list
        .into_iter()
        .filter(|item| item.category.eq_ignore_ascii_case(category_filter))
        .collect();

    let mut items: Vec<_> = filtered_items
        .iter()
        .map(|item| {
            ListItem::new(Spans::from(vec![Span::styled(
                item.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    items.push(ListItem::new(Spans::from(vec![Span::styled(
    "+ Add New",
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD),
        )])));
    // ⚠️ Handle empty case gracefully
    let selected_item = filtered_items
        .get(inv_list_state.selected().unwrap_or(0))
        .cloned()
        .unwrap_or(Item {
            id: 0,
            name: "ERR".into(),
            details: "ERR".into(),
            quantity: 1,
            category: category_filter.into(),
            created_at: chrono::Utc::now(),
        });

    let list = List::new(items).block(invs).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    // 🧾 Render details as a paragraph
    let mut detail_lines = vec![
        Spans::from(vec![Span::styled(
            format!("Name: {}", selected_item.name),
            Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(format!("Created: {}", selected_item.created_at))]),
        Spans::from(""),
        Spans::from(vec![Span::styled("Details:", Style::default().fg(Color::LightBlue))]),
    ];

    // Add the multi-line detail text as a final `Spans`
    detail_lines.push(Spans::from(selected_item.details));

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
    let quantity = show_quantity_selector(&mut terminal)
        .map_err(|e| Error::ReadDBError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
    let random_item = Item {
        id: rng.gen_range(0, 9999999),
        name,
        details,
        quantity,
        category,
        created_at: Utc::now(),
    };

    parsed.push(random_item);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}
pub fn show_category_selector<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<String> {
    let categories = vec!["Weapons", "Apparel", "Aid", "Misc", "Junk", "Mods", "Ammo"];
    let mut state = widgets::ListState::default();
    state.select(Some(0)); // Select the first item by default

    loop {
        terminal.clear();
        terminal.draw(|f| {
            let size = centered_rect(70, 50, f.area()); // Use .area() instead of .size() as per recommendation
            let items: Vec<widgets::ListItem> = categories
                .iter()
                .map(|c| widgets::ListItem::new(*c))
                .collect();

            let list = widgets::List::new(items)
                .block(widgets::Block::default().borders(widgets::Borders::ALL).title("Select Category"))
                .highlight_style(
                    ratatui::style::Style::default()
                        .bg(ratatui::style::Color::LightBlue)
                        .fg(ratatui::style::Color::Black)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                );

            // Pass 'list' by value, and the mutable reference to 'state'
            f.render_stateful_widget(list, size, &mut state);
        })?;

        // Handle events for navigation and selection
        if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? {
            match key.code {
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => 0, // Wrap around to the last item, or stay at 0
                    };
                    state.select(Some(i));
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let i = match state.selected() {
                        Some(i) if i < categories.len() - 1 => i + 1,
                        _ => categories.len() - 1, // Wrap around to the first item, or stay at the last
                    };
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        return Ok(categories[i].to_string()); // Return the selected category
                    }
                }
                KeyCode::Esc => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Category selection cancelled")); // Allow escaping
                }
                _ => {} // Ignore other keys
            }
        }
    }
}
pub fn show_quantity_selector<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<u32> {
    // Generate a vector of strings "0" through "99"
    let categories: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    let mut state = widgets::ListState::default();
    state.select(Some(0)); // Select the first item by default

    loop {
        terminal.clear();
        terminal.draw(|f| {
            let size = centered_rect(70, 50, f.area()); // Use .area() instead of .size() as per recommendation
            let items: Vec<widgets::ListItem> = categories
                .iter()
                .map(|c| widgets::ListItem::new(c.as_str())) // Use .as_str() for ListItem
                .collect();

            let list = widgets::List::new(items)
                .block(widgets::Block::default().borders(widgets::Borders::ALL).title("Select Quantity")) // Changed title
                .highlight_style(
                    ratatui::style::Style::default()
                        .bg(ratatui::style::Color::LightBlue)
                        .fg(ratatui::style::Color::Black)
                        .add_modifier(ratatui::prelude::Modifier::BOLD),
                );

            // Pass 'list' by value, and the mutable reference to 'state'
            f.render_stateful_widget(list, size, &mut state);
        })?;

        // Handle events for navigation and selection
        if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? {
            match key.code {
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => categories.len() - 1, // Wrap around to the last item
                    };
                    state.select(Some(i));
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let i = match state.selected() {
                        Some(i) if i < categories.len() - 1 => i + 1,
                        _ => 0, // Wrap around to the first item
                    };
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(selected_index) = state.selected() {
                        // Parse the selected string into a u32
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
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Pyp-Boy",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'i' to access inventory, 'a' to add random new items and 'd' to delete the currently selected item.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}
pub fn read_db() -> Result<Vec<Item>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Item> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}
