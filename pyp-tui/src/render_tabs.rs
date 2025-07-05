use reverse_geocoder::{ReverseGeocoder};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use thiserror::Error;
use tui::{

    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap
    },
};

const DB_PATH: &str = "./data/db.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub details: String,
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
