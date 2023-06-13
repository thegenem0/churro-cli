mod jira;
mod structs;

use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::env;
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

const DB_PATH: &str = "./data/db.json";
const PROJECTS_PATH: &str = "./data/projects.json";

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
struct Pet {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Projects = 0,
    Tickets = 1,
    Help = 2,
}

impl MenuItem {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(input: usize) -> Option<MenuItem> {
        match input {
            0 => Some(MenuItem::Projects),
            1 => Some(MenuItem::Tickets),
            2 => Some(MenuItem::Help),
            _ => None,
        }
    }
}

impl TryFrom<MenuItem> for usize {
    type Error = ();

    fn try_from(input: MenuItem) -> Result<Self, Self::Error> {
        match input {
            MenuItem::Projects => Ok(0),
            MenuItem::Tickets => Ok(1),
            MenuItem::Help => Ok(2),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut logged_in = false;

    if args.iter().any(|arg| arg == "--auth") {
        return Ok(());
    }

    if args.iter().any(|arg| arg == "--login") {
        let (auth_url, code_verifier) = jira::auth_service::build_authorization_url()?;
        println!("Please visit this URL to authorize the app: {}", auth_url);
        println!("Please enter the code you received: ");
        let mut auth_code = String::new();
        std::io::stdin().read_line(&mut auth_code).unwrap();
        auth_code = auth_code.trim().to_string();

        match jira::auth_service::exchange_code_for_token(auth_code, code_verifier.to_string())
            .await
        {
            Ok(auth_code) => {
                logged_in = true;
                println!("User info: {:?}", auth_code)
            }
            Err(e) => panic!("Error: {}", e),
        }
    }

    if !logged_in {
        println!("Please login first with the --login flag");
        return Ok(());
    } else {
        enable_raw_mode().expect("can run in raw mode");
    }

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

    let menu_titles = vec!["Projects", "Tickets", "Help"];
    let mut active_menu_item = MenuItem::Projects;
    let mut selected_pane: Option<MenuItem> = None;
    let mut project_list_state = ListState::default();

    project_list_state.select(Some(0));

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

            let copyright = Paragraph::new("Churro-CLI 2023 - Gergo Nemeth")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
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
                .select(active_menu_item.try_into().unwrap()) // Handle err case
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Projects => {
                    let project_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_projects(&project_list_state);
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                    rect.render_widget(right, project_chunks[1]);
                }
                MenuItem::Tickets => {
                    todo!()
                }
                MenuItem::Help => {}
            }
            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => {
                    if selected_pane.is_none() {
                        if active_menu_item.as_usize() > MenuItem::Projects.as_usize() {
                            let tab_no = active_menu_item.as_usize() - 1;
                            active_menu_item = MenuItem::from_usize(tab_no).unwrap();
                        } else {
                            active_menu_item = MenuItem::Projects;
                        }
                    }
                }
                KeyCode::Char('l') => {
                    if selected_pane.is_none() {
                        if active_menu_item.as_usize() < MenuItem::Help.as_usize() {
                            let tab_no = active_menu_item.as_usize() + 1;
                            active_menu_item = MenuItem::from_usize(tab_no).unwrap();
                        } else {
                            active_menu_item = MenuItem::Help;
                        }
                    }
                }
                KeyCode::Enter => {
                    if selected_pane.is_none() {
                        selected_pane = Some(active_menu_item);
                    }
                }
                KeyCode::Esc => {
                    if selected_pane.is_some() {
                        selected_pane = None;
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_projects<'a>(project_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let projects = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Projects")
        .border_type(BorderType::Plain);

    let project_list = read_projects().expect("can fetch project list");
    let items: Vec<_> = project_list
        .iter()
        .map(|project| {
            ListItem::new(Spans::from(vec![Span::styled(
                project.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_project = project_list
        .get(
            project_list_state
                .selected()
                .expect("there is always a selected project"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(projects).highlight_style(
        Style::default()
            .bg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
    );

    let project_details = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_project.key)),
        Cell::from(Span::raw(selected_project.name)),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Key",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Project Details"),
    )
    .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);
    (list, project_details)
}

fn render_pets<'a>(pet_list_state: &ListState) -> (List<'a>, Table<'a>) {
    todo!()
}

fn read_projects() -> Result<Vec<structs::project::Project>, Error> {
    let db_content = fs::read_to_string(PROJECTS_PATH)?;
    let parsed: structs::project::ProjectResponse = serde_json::from_str(&db_content)?;
    let projects = parsed.values;
    Ok(projects)
}
