use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};

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
        Block, BorderType, Borders, ListState, Paragraph, Tabs,
    },
    Terminal,
};

extern crate linux_embedded_hal as hal;
extern crate max3010x;
extern crate ratatui;

use max3010x::{Max3010x, Led, SampleAveraging};

const DB_PATH: &str = "./data/db.json";

mod render_tabs;
mod menus;

use render_tabs::{render_map, render_stat, render_data, render_inv, read_db, get_map_data, Item,};
use menus::{MenuItem, StatSubMenu, InvSubMenu, DataSubMenu};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
	

	let coords: [f64; 2] = [44.817028, -89.736273];

	let mut map_data: Option<String> = None;
	let mut start_time = Instant::now();
    let one_minute = Duration::from_secs(60);

	if map_data.is_none() || start_time.elapsed() >= one_minute {
		start_time = Instant::now();
        map_data = Some(get_map_data(coords));
		println!("getting map data: {:?}", map_data)
    }

	let stat_submenu_titles = vec!["GENERAL", "STATUS", "SETTINGS"];
	let inv_submenu_titles = vec!["WEAPONS", "APPAREL", "AID", "MISC", "JUNK", "MODS", "AMMO"];
	let data_submenu_titles = vec!["QUESTS", "WORKSHOPS", "STATS"];

	let mut active_stat_submenu = StatSubMenu::General;
	let mut active_inv_submenu = InvSubMenu::Weapons;
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
						InvSubMenu::Weapons => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Weapons"),
						InvSubMenu::Apparel => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Apparel"),
						InvSubMenu::Aid => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Aid"),
						InvSubMenu::Misc => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Misc"),
						InvSubMenu::Junk => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Junk"),
						InvSubMenu::Mods => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Mods"),
						InvSubMenu::Ammo => draw_filtered_inventory(rect, adjusted_chunks[2], &mut inv_list_state, "Ammo"),
					},
					MenuItem::Data => match active_data_submenu {
						DataSubMenu::Quests => rect.render_widget(render_data_quests(), adjusted_chunks[2]),
						DataSubMenu::Workshops => rect.render_widget(render_data_workshops(), adjusted_chunks[2]),
						DataSubMenu::Stats => rect.render_widget(render_data_stats(), adjusted_chunks[2]),
					},
					_ => {}
				}


				rect.render_widget(copyright, adjusted_chunks[3]);
			} else {
				// fallback layout with no submenu
				rect.render_widget(tabs, chunks[0]);
				match active_menu_item {
					MenuItem::Map => rect.render_widget(render_map(map_data.clone()), chunks[1]),
					MenuItem::Radio => rect.render_widget(render_stat(), chunks[1]),
					_ => {}
				}
				rect.render_widget(copyright, chunks[2]);
			}
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
					ratatui::restore();
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
								InvSubMenu::Weapons => InvSubMenu::Ammo,
								InvSubMenu::Apparel => InvSubMenu::Weapons,
								InvSubMenu::Aid => InvSubMenu::Apparel,
								InvSubMenu::Misc => InvSubMenu::Aid,
								InvSubMenu::Junk => InvSubMenu::Misc,
								InvSubMenu::Mods => InvSubMenu::Junk,
								InvSubMenu::Ammo => InvSubMenu::Mods,
							};
						}
						MenuItem::Data => {
							active_data_submenu = match active_data_submenu {
								DataSubMenu::Quests => DataSubMenu::Stats,
								DataSubMenu::Workshops => DataSubMenu::Quests,
								DataSubMenu::Stats => DataSubMenu::Workshops,
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
								InvSubMenu::Weapons => InvSubMenu::Apparel,
								InvSubMenu::Apparel => InvSubMenu::Aid,
								InvSubMenu::Aid => InvSubMenu::Misc,
								InvSubMenu::Misc => InvSubMenu::Junk,
								InvSubMenu::Junk => InvSubMenu::Mods,
								InvSubMenu::Mods => InvSubMenu::Ammo,
								InvSubMenu::Ammo => InvSubMenu::Weapons,
							};
						}
						MenuItem::Data => {
							active_data_submenu = match active_data_submenu {
								DataSubMenu::Quests => DataSubMenu::Workshops,
								DataSubMenu::Workshops => DataSubMenu::Stats,
								DataSubMenu::Stats => DataSubMenu::Quests,
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

fn draw_filtered_inventory<'a>(
    rect: &mut tui::Frame<'a, CrosstermBackend<std::io::Stdout>>,
    area: tui::layout::Rect,
    inv_list_state: &mut ListState,
    category: &str,
) {
    let inv_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(area);

    let (left, right) = render_inv(inv_list_state, category);
    rect.render_stateful_widget(left, inv_chunks[0], inv_list_state);
    rect.render_widget(right, inv_chunks[1]);
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
		details: rng.sample_iter(Alphanumeric).take(20).collect(),
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

fn render_data_quests<'a>() -> Paragraph<'a> {
    Paragraph::new("Quests").block(Block::default().title("DATA"))
}
fn render_data_workshops<'a>() -> Paragraph<'a> {
    Paragraph::new("Workshops").block(Block::default().title("DATA"))
}
fn render_data_stats<'a>() -> Paragraph<'a> {
    Paragraph::new("Stats").block(Block::default().title("DATA"))
}
