use crate::agent::{random::RandomAgent, Agent};
use crate::game::*;

use agent::random::{RandomTree, RandomTreeMetric};
use agent::rl::{RLAgent, RLAgentTrained, STORE_PATH};
use agent::user::UserAgent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rurel::strategy::explore::RandomExploration;
use rurel::strategy::learn::QLearning;
use rurel::strategy::terminate::FixedIterations;
use rurel::AgentTrainer;
use std::fs::{File};
use std::io::Write;
use std::sync::RwLock;
use std::thread::JoinHandle;
use std::{error::Error, io, sync::Arc, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

mod agent;
mod game;

static TICK_RATE: Duration = Duration::from_millis(50);
static MENU_ITEMS: &[&str] = &[
    "Play (Keyboard)",
    "Solve (Random)",
    "Solve (Tree Search, Max Score)",
    "Solve (Tree Search, Max Moves)",
    "Train (RL)",
    "Solve (RL)",
    "Solve (Expectimax)",
];

pub enum Screen {
    Menu {
        state: ListState,
        menu: List<'static>,
    },
    Train(JoinHandle<()>),
    // join handle for multithreading if needed
    Game(JoinHandle<()>, Arc<RwLock<Box<dyn Agent + Sync + Send>>>),
}

pub struct App {
    screen: Screen,
}

impl Default for Screen {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        let menu = List::new(
            MENU_ITEMS
                .iter()
                .map(|s| ListItem::new(*s))
                .collect::<Vec<ListItem<'static>>>(),
        );

        Screen::Menu { state, menu }
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            screen: Screen::default(),
        }
    }
}

fn get_color_for_value(v: u32) -> Color {
    match v {
        2 => Color::Rgb(238, 228, 218),
        4 => Color::Rgb(237, 224, 200),
        8 => Color::Rgb(242, 177, 121),
        16 => Color::Rgb(245, 149, 99),
        32 => Color::Rgb(246, 124, 95),
        64 => Color::Rgb(246, 94, 59),
        128 => Color::Rgb(237, 207, 114),
        256 => Color::Rgb(237, 204, 97),
        512 => Color::Rgb(237, 200, 80),
        1024 => Color::Rgb(237, 197, 63),
        2048 => Color::Rgb(237, 194, 46),
        4096 => Color::Rgb(173, 183, 119),
        8192 => Color::Rgb(170, 183, 102),
        16384 => Color::Rgb(166, 183, 85),
        32768 => Color::Rgb(163, 183, 68),
        65536 => Color::Rgb(160, 183, 51),
        _ => Color::DarkGray,
    }
}

fn render_table_cell(c: &u32) -> Cell<'_> {
    let cstr = if *c == 1 {
        String::from("")
    } else {
        c.to_string()
    };

    let cell_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(get_color_for_value(*c));

    let front_padding = " ".repeat(5 - (cstr.len() / 2));
    let cell_body = ["\n\n", front_padding.as_str(), cstr.as_str(), "\n\n"].join("");

    Cell::from(Text::from(cell_body)).style(cell_style)
}

fn render_game<B: Backend>(f: &mut Frame<B>, block: Block<'_>, game: &Game, rect: Rect) {
    let game_state = game.get_table();
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows = game_state.iter().map(|row| {
        let row = row.iter().map(render_table_cell);
        Row::new(row).height(5).bottom_margin(1)
    });
    let t = Table::new(rows)
        .block(block)
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(11),
        ]);

    f.render_widget(t, rect);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(20)].as_ref())
        .split(f.size());

    match &mut app.screen {
        Screen::Menu { state, menu } => {
            let block = Block::default().title("Menu").borders(Borders::ALL);
            let list = menu
                .clone()
                .block(block)
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], state);

            let block = Block::default().title("Info").borders(Borders::ALL);
            let text = vec![
                Spans::from("Use arrow keys to navigate"),
                Spans::from("Press q to exit"),
            ];
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        }
        Screen::Train(_) => {
            let block = Block::default().title("Training").borders(Borders::ALL);
            let text = vec![
                Spans::from("Training in progress..."),
                Spans::from("Press q to exit"),
            ];
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[0]);
        }
        Screen::Game(_, game_sim) => {
            let agent = game_sim.read().unwrap();
            let game = agent.get_game();
            let game_block = Block::default().title("Game").borders(Borders::ALL);
            render_game(f, game_block, &game, chunks[0]);
            let block = Block::default().title("Info").borders(Borders::ALL);
            let game_over_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
            let bold_span = |s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD));
            let mut text = vec![
                Spans::from(vec![
                    bold_span("Score: "),
                    Span::from(game.get_score().to_string()),
                ]),
                Spans::from(vec![
                    bold_span("Moves: "),
                    Span::from(game.get_num_moves().to_string()),
                ]),
                Spans::from(if game.game_over() {
                    Span::styled("Game over.", game_over_style)
                } else {
                    Span::from("")
                }),
                Spans::from(""),
            ];
            text.append(&mut agent.messages());
            text.append(&mut vec![Spans::from(""), Spans::from("Press q to exit")]);
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        }
    }
}

pub enum IntAction {
    Continue,
    Exit,
}

pub enum MenuItem {
    Play(Box<dyn Agent + Sync + Send>),
    Train,
    Exit,
}

fn get_interaction(app: &mut App, timeout: Duration) -> Result<IntAction, io::Error> {
    // each tick, lets see what screen we're at for interaction
    match &mut app.screen {
        // menu
        Screen::Menu { state, menu: _ } => {
            // wait for key event within timeout; if no event, loop
            if !event::poll(timeout)? {
                return Ok(IntAction::Continue);
            }
            let Event::Key(event) = event::read()? else {
                return Ok(IntAction::Continue);
            };
            match event.code {
                KeyCode::Char('q') => return Ok(IntAction::Exit),
                KeyCode::Up => {
                    let Some(sel) = state.selected() else {
                        state.select(Some(0));
                        return Ok(IntAction::Continue);
                    };
                    if sel > 0 {
                        state.select(Some(sel - 1));
                    }
                }
                KeyCode::Down => {
                    let Some(sel) = state.selected() else {
                        state.select(Some(0));
                        return Ok(IntAction::Continue);
                    };
                    if sel < MENU_ITEMS.len() - 1 {
                        state.select(Some(sel + 1));
                    }
                }
                KeyCode::Enter => {
                    let game = Game::new();
                    let item: MenuItem = match state.selected() {
                        Some(0) => MenuItem::Play(Box::new(UserAgent::new(game))),
                        Some(1) => MenuItem::Play(Box::new(RandomAgent::new(game))),
                        Some(2) => MenuItem::Play(Box::new(RandomTree::new(game))),
                        Some(3) => MenuItem::Play(Box::new(RandomTree::new_with(
                            game,
                            RandomTreeMetric::AvgMoves,
                        ))),
                        Some(4) => MenuItem::Train,
                        Some(5) => MenuItem::Play(Box::new(RLAgentTrained::new(game))),
                        _ => panic!(),
                    };

                    let MenuItem::Play(agent) = item else {
                        match item {
                            MenuItem::Train => {
                                let t = thread::spawn(move || {
                                    let mut trainer = AgentTrainer::new();
                                    let mut agent = RLAgent::new(Game::new());
                                    trainer.train(&mut agent, &QLearning::new(0.2, 0.01, 2.), &mut FixedIterations::new(1000000), &RandomExploration::new());
                                    // write out to file
                                    let mut file = File::create(STORE_PATH).unwrap();
                                    let Ok(res) = serde_json::to_string(&trainer.export_learned_values()) else {
                                        return;
                                    };
                                    file.write_all(res.as_bytes()).unwrap();
                                });
                                app.screen = Screen::Train(t);
                            }
                            MenuItem::Exit => {
                                return Ok(IntAction::Exit);
                            }
                            _ => {}
                        };
                        return Ok(IntAction::Continue);
                    };

                    let agent = Arc::new(RwLock::new(agent));
                    let local_agent = agent.clone();
                    let t = thread::spawn(move || {
                        while !agent.read().unwrap().get_game().game_over() {
                            agent.write().unwrap().next_move();
                        }
                    });

                    app.screen = Screen::Game(t, local_agent);
                }
                _ => {}
            };
        }
        Screen::Train(t) => {
            if t.is_finished() {
                return Ok(IntAction::Exit);
            }

            if !event::poll(timeout)? {
                return Ok(IntAction::Continue);
            }
            let event = event::read()?;
            let Event::Key(key_event) =  event else {
                return Ok(IntAction::Continue);
            };
            match key_event.code {
                KeyCode::Char('q') => {
                    return Ok(IntAction::Exit);
                }
                _ => {}
            };
        }
        Screen::Game(_, agent) => {
            if !event::poll(timeout)? {
                return Ok(IntAction::Continue);
            }
            let event = event::read()?;
            let Event::Key(key_event) =  event else {
                return Ok(IntAction::Continue);
            };

            match key_event.code {
                KeyCode::Char('q') => {
                    return Ok(IntAction::Exit);
                }
                _ => {}
            };

            return Ok(agent.write().unwrap().get_input(&event));
        }
    };

    Ok(IntAction::Continue)
}

fn run_tui<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::default();
    let mut last_tick = std::time::Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        match get_interaction(&mut app, timeout)? {
            IntAction::Continue => {}
            IntAction::Exit => match app.screen {
                Screen::Menu { state: _, menu: _ } => break,
                Screen::Game(_, _) | Screen::Train(_) => {
                    app.screen = Screen::default();
                    continue;
                }
            },
        }

        if last_tick.elapsed() >= TICK_RATE {
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear()?;

    run_tui(&mut terminal)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
