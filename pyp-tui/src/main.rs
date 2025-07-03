use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

use osm_geo_mapper::{
    geo_types, interface, features
};

extern crate linux_embedded_hal as hal;
extern crate max3010x;
use max3010x::{Max3010x, Led, SampleAveraging};
extern crate osm_geo_mapper;
const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Item {
    id: usize,
    name: String,
    category: String,
    created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
	Stat,
    Inv,
    Data,
	Map,
	Radio,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Stat => 0,
            MenuItem::Inv => 1,
			MenuItem::Data => 2,
            MenuItem::Map => 3,
			MenuItem::Radio => 4,
        }
    }
}
#[derive(Copy, Clone, Debug)]
enum StatSubMenu {
    General,
    Status,
    Settings,
}

impl From<StatSubMenu> for usize {
    fn from(input: StatSubMenu) -> usize {
        match input {
            StatSubMenu::General => 0,
            StatSubMenu::Status => 1,
            StatSubMenu::Settings => 2,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum InvSubMenu {
    Items,
    Aid,
    Weapons,
}

impl From<InvSubMenu> for usize {
    fn from(input: InvSubMenu) -> usize {
        match input {
            InvSubMenu::Items => 0,
            InvSubMenu::Aid => 1,
            InvSubMenu::Weapons => 2,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum DataSubMenu {
    Quests,
    Logs,
    Notes,
}

impl From<DataSubMenu> for usize {
    fn from(input: DataSubMenu) -> usize {
        match input {
            DataSubMenu::Quests => 0,
            DataSubMenu::Logs => 1,
            DataSubMenu::Notes => 2,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mapper_result = interface::OSMGeoMapper::from_address("ottawa canada".to_string(), Some(20));
	let mut geo_features = vec![];

	if let Ok(mapper) = mapper_result {
		let data = mapper.data_structure.read().unwrap();
		for (_coord, features) in data.iter() {
			for feature in features {
				geo_features.push(format!("{:?}", feature));
			}
		}
	}

	let stat_submenu_titles = vec!["GENERAL", "STATUS", "SETTINGS"];
	let inv_submenu_titles = vec!["ITEMS", "AID", "WEAPONS"];
	let data_submenu_titles = vec!["QUESTS", "LOGS", "NOTES"];

	let mut active_stat_submenu = StatSubMenu::General;
	let mut active_inv_submenu = InvSubMenu::Items;
	let mut active_data_submenu = DataSubMenu::Quests;


    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["STAT", "INV", "DATA", "MAP", "RADIO"];
    let mut active_menu_item = MenuItem::Stat;
    let mut inv_list_state = ListState::default();
    inv_list_state.select(Some(0));

	/* let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();

	let mut sensor = Max3010x::new_max30102(dev);
	let mut sensor = sensor.into_heart_rate().unwrap();
	sensor.set_sample_averaging(SampleAveraging::Sa4).unwrap();
	sensor.set_pulse_amplitude(Led::All, 15).unwrap();
	sensor.enable_fifo_rollover().unwrap();

	let mut data = [0; 3];
	let samples_read = sensor.read_fifo(&mut data).unwrap();

	// get the I2C device back
  	let dev = sensor.destroy(); */

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);
			


            let copyright = Paragraph::new("COPYRIGHT 2075 ROBCO(R)")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("COPYRIGHT")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();
			
            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("STAT").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

			let (submenu_spans, active_index) = match active_menu_item {
				MenuItem::Stat => (
					stat_submenu_titles
						.iter()
						.map(|t| {
							let (first, rest) = t.split_at(1);
							Spans::from(vec![
								Span::styled(first, Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED)),
								Span::styled(rest, Style::default().fg(Color::White)),
							])
						})
						.collect::<Vec<Spans>>(),
					active_stat_submenu.into(),
				),
				MenuItem::Inv => (
					inv_submenu_titles
						.iter()
						.map(|t| {
							let (first, rest) = t.split_at(1);
							Spans::from(vec![
								Span::styled(first, Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED)),
								Span::styled(rest, Style::default().fg(Color::White)),
							])
						})
						.collect::<Vec<Spans>>(),
					active_inv_submenu.into(),
				),
				MenuItem::Data => (
					data_submenu_titles
						.iter()
						.map(|t| {
							let (first, rest) = t.split_at(1);
							Spans::from(vec![
								Span::styled(first, Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED)),
								Span::styled(rest, Style::default().fg(Color::White)),
							])
						})
						.collect::<Vec<Spans>>(),
					active_data_submenu.into(),
				),
				_ => (vec![], 0),
			};

			let show_secondary_menu = !submenu_spans.is_empty();


			if show_secondary_menu {
				let secondary_tabs = Tabs::new(submenu_spans)
					.select(active_index)
					.block(Block::default().title("SUBMENU").borders(Borders::ALL))
					.style(Style::default().fg(Color::White))
					.highlight_style(Style::default().fg(Color::Green))
					.divider(Span::raw("|"));

				// shrink chunks to make room for submenu
				let adjusted_chunks = Layout::default()
					.direction(Direction::Vertical)
					.margin(2)
					.constraints(
						[
							Constraint::Length(3),
							Constraint::Length(3),
							Constraint::Min(1),
							Constraint::Length(3),
						]
						.as_ref(),
					)
					.split(size);

				rect.render_widget(tabs, adjusted_chunks[0]);
				rect.render_widget(secondary_tabs, adjusted_chunks[1]);

				match active_menu_item {
					MenuItem::Stat => match active_stat_submenu {
						StatSubMenu::General => rect.render_widget(render_stat_general(), adjusted_chunks[2]),
						StatSubMenu::Status => rect.render_widget(render_stat_status(), adjusted_chunks[2]),
						StatSubMenu::Settings => rect.render_widget(render_stat_settings(), adjusted_chunks[2]),
					},
					MenuItem::Inv => match active_inv_submenu {
						InvSubMenu::Items => {
							let inv_chunks = Layout::default()
								.direction(Direction::Horizontal)
								.constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
								.split(adjusted_chunks[2]);
							let (left, right) = render_inv(&inv_list_state);
							rect.render_stateful_widget(left, inv_chunks[0], &mut inv_list_state);
							rect.render_widget(right, inv_chunks[1]);
						}
						InvSubMenu::Aid => rect.render_widget(render_inv_aid(), adjusted_chunks[2]),
						InvSubMenu::Weapons => rect.render_widget(render_inv_weapons(), adjusted_chunks[2]),
					},
					MenuItem::Data => match active_data_submenu {
						DataSubMenu::Quests => rect.render_widget(render_data_quests(), adjusted_chunks[2]),
						DataSubMenu::Logs => rect.render_widget(render_data_logs(), adjusted_chunks[2]),
						DataSubMenu::Notes => rect.render_widget(render_data_notes(), adjusted_chunks[2]),
					},
					_ => {}
				}


				rect.render_widget(copyright, adjusted_chunks[3]);
			} else {
				// fallback layout with no submenu
				rect.render_widget(tabs, chunks[0]);
				match active_menu_item {
					MenuItem::Map => rect.render_widget(render_map(&geo_features), chunks[1]),
					MenuItem::Radio => rect.render_widget(render_stat(), chunks[1]),
					_ => {}
				}
				rect.render_widget(copyright, chunks[2]);
			}

            //rect.render_widget(tabs, chunks[0]);
			
            /* match active_menu_item {
                MenuItem::Stat => rect.render_widget(render_stat(), chunks[1]),
                MenuItem::Inv => {
                    let inv_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_inv(&inv_list_state);
                    rect.render_stateful_widget(left, inv_chunks[0], &mut inv_list_state);
                    rect.render_widget(right, inv_chunks[1]);
                },
				MenuItem::Data => rect.render_widget(render_stat(), chunks[1]),
				MenuItem::Map => rect.render_widget(render_stat(), chunks[1]),
				MenuItem::Radio => rect.render_widget(render_stat(), chunks[1]),
            }
            //rect.render_widget(copyright, chunks[2]); */
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('s') => active_menu_item = MenuItem::Stat,
                KeyCode::Char('i') => active_menu_item = MenuItem::Inv,
                KeyCode::Char('d') => active_menu_item = MenuItem::Data,
                KeyCode::Char('m') => active_menu_item = MenuItem::Map,
				KeyCode::Char('r') => active_menu_item = MenuItem::Radio,

                KeyCode::Down => {
                    if let Some(selected) = inv_list_state.selected() {
                        let amount_items = read_db().expect("can fetch item list").len();
                        if selected >= amount_items - 1 {
                            inv_list_state.select(Some(0));
                        } else {
                            inv_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = inv_list_state.selected() {
                        let amount_items = read_db().expect("can fetch item list").len();
                        if selected > 0 {
                            inv_list_state.select(Some(selected - 1));
                        } else {
                            inv_list_state.select(Some(amount_items - 1));
                        }
                    }
                }
				KeyCode::Left => {
					match active_menu_item {
						MenuItem::Stat => {
							active_stat_submenu = match active_stat_submenu {
								StatSubMenu::General => StatSubMenu::Settings,
								StatSubMenu::Status => StatSubMenu::General,
								StatSubMenu::Settings => StatSubMenu::Status,
							};
						}
						MenuItem::Inv => {
							active_inv_submenu = match active_inv_submenu {
								InvSubMenu::Items => InvSubMenu::Weapons,
								InvSubMenu::Aid => InvSubMenu::Items,
								InvSubMenu::Weapons => InvSubMenu::Aid,
							};
						}
						MenuItem::Data => {
							active_data_submenu = match active_data_submenu {
								DataSubMenu::Quests => DataSubMenu::Notes,
								DataSubMenu::Logs => DataSubMenu::Quests,
								DataSubMenu::Notes => DataSubMenu::Logs,
							};
						}
						_ => {}
					}
				}
				KeyCode::Right => {
					match active_menu_item {
						MenuItem::Stat => {
							active_stat_submenu = match active_stat_submenu {
								StatSubMenu::General => StatSubMenu::Status,
								StatSubMenu::Status => StatSubMenu::Settings,
								StatSubMenu::Settings => StatSubMenu::General,
							};
						}
						MenuItem::Inv => {
							active_inv_submenu = match active_inv_submenu {
								InvSubMenu::Items => InvSubMenu::Aid,
								InvSubMenu::Aid => InvSubMenu::Weapons,
								InvSubMenu::Weapons => InvSubMenu::Items,
							};
						}
						MenuItem::Data => {
							active_data_submenu = match active_data_submenu {
								DataSubMenu::Quests => DataSubMenu::Logs,
								DataSubMenu::Logs => DataSubMenu::Notes,
								DataSubMenu::Notes => DataSubMenu::Quests,
							};
						}
						_ => {}
					}
				}

                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_stat<'a>() -> Paragraph<'a> {
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

fn render_inv<'a>(inv_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let invs = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Items")
        .border_type(BorderType::Plain);

    let inv_list = read_db().expect("can fetch item list");
    let items: Vec<_> = inv_list
        .iter()
        .map(|item| {
            ListItem::new(Spans::from(vec![Span::styled(
                item.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_item = inv_list
        .get(
            inv_list_state
                .selected()
                .expect("there is always a selected item"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(invs).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let item_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_item.id.to_string())),
        Cell::from(Span::raw(selected_item.name)),
        Cell::from(Span::raw(selected_item.category)),
        Cell::from(Span::raw(selected_item.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, item_detail)
}

fn render_data<'a>() -> Paragraph<'a> {
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
fn render_map<'a>(geo_features: &'a Vec<String>) -> Paragraph<'a> {
    let display_lines: Vec<Spans> = geo_features
        .iter()
        .map(|line| Spans::from(Span::raw(line.clone())))
        .collect();

    Paragraph::new(display_lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Map")
                .border_type(BorderType::Plain),
        )
}

fn read_db() -> Result<Vec<Item>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Item> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_item_to_db() -> Result<Vec<Item>, Error> {
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Item> = serde_json::from_str(&db_content)?;
    let category = match rng.gen_range(0, 1) {
        0 => "Food",
        _ => "Industrial",
    };

    let random_item = Item {
        id: rng.gen_range(0, 9999999),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        category: category.to_owned(),
        created_at: Utc::now(),
    };

    parsed.push(random_item);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_item_at_index(inv_list_state: &mut ListState) -> Result<(), Error> {
    if let Some(selected) = inv_list_state.selected() {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Item> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
        let amount_items = read_db().expect("can fetch item list").len();
        if selected > 0 {
            inv_list_state.select(Some(selected - 1));
        } else {
            inv_list_state.select(Some(0));
        }
    }
    Ok(())
}

fn render_stat_general<'a>() -> Paragraph<'a> {
    Paragraph::new("General").block(Block::default().title("STAT"))
}
fn render_stat_status<'a>() -> Paragraph<'a> {
    Paragraph::new("Status").block(Block::default().title("STAT"))
}
fn render_stat_settings<'a>() -> Paragraph<'a> {
    Paragraph::new("Settings").block(Block::default().title("STAT"))
}

fn render_inv_aid<'a>() -> Paragraph<'a> {
    Paragraph::new("Aid").block(Block::default().title("INVENTORY"))
}
fn render_inv_weapons<'a>() -> Paragraph<'a> {
    Paragraph::new("Weapons").block(Block::default().title("INVENTORY"))
}

fn render_data_quests<'a>() -> Paragraph<'a> {
    Paragraph::new("Quests").block(Block::default().title("DATA"))
}
fn render_data_logs<'a>() -> Paragraph<'a> {
    Paragraph::new("Logs").block(Block::default().title("DATA"))
}
fn render_data_notes<'a>() -> Paragraph<'a> {
    Paragraph::new("Notes").block(Block::default().title("DATA"))
}
