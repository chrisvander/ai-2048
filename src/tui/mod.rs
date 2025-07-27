use crate::agent::expectimax::Expectimax;
use crate::agent::random::{RandomAgent, RandomTree, RandomTreeMetric};
use crate::agent::user::UserAgent;
use crate::agent::TuiAgent;
use crate::game::*;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::sync::RwLock;
use std::thread::JoinHandle;
use std::{error::Error, io, sync::Arc, thread, time::Duration};
use tui::widgets::Widget;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

mod board;
mod menu;

static TICK_RATE: Duration = Duration::from_millis(50);

pub enum Screen {
    Menu {
        state: ListState,
        menu: List<'static>,
    },
    Train(JoinHandle<()>),
    // join handle for multithreading if needed
    Game(JoinHandle<()>, Arc<RwLock<Box<dyn TuiAgent + Sync + Send>>>),
}

impl Default for Screen {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Screen::Menu {
            state,
            menu: menu::MENU.clone(),
        }
    }
}

#[derive(Default)]
pub struct App {
    screen: Screen,
}

fn get_train_text() -> impl Widget {
    let block = Block::default().title("Training").borders(Borders::ALL);
    let text = vec![
        Spans::from("Training in progress..."),
        Spans::from("Press q to exit"),
    ];
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    paragraph
}

fn get_game_text<'a>(game: &Game, mut agent_spans: Vec<Spans<'a>>) -> impl Widget + 'a {
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
    text.append(&mut agent_spans);
    text.append(&mut vec![Spans::from(""), Spans::from("Press q to exit")]);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    paragraph
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(20)].as_ref())
        .split(f.size());

    match &mut app.screen {
        Screen::Menu { state, menu } => {
            f.render_stateful_widget(menu::get_menu(menu), chunks[0], state);
            f.render_widget(menu::get_menu_text(), chunks[1]);
        }
        Screen::Train(_) => f.render_widget(get_train_text(), chunks[0]),
        Screen::Game(_, game_sim) => {
            let agent = game_sim.read().unwrap();
            let game = agent.get_game();
            board::render_board(f, &game, chunks[0]);
            f.render_widget(get_game_text(&game, agent.messages()), chunks[1]);
        }
    }
}

pub enum IntAction {
    Continue,
    Exit,
}

pub enum MenuItem {
    Play(Box<dyn TuiAgent + Sync + Send>),
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
                    if sel < menu::MENU_ITEMS.len() - 1 {
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
                            1000,
                            RandomTreeMetric::AvgMoves,
                            true,
                        ))),
                        Some(4) => MenuItem::Play(Box::new(Expectimax::new(game))),
                        _ => panic!(),
                    };

                    let MenuItem::Play(agent) = item else {
                        return match item {
                            MenuItem::Exit => Ok(IntAction::Exit),
                            _ => Ok(IntAction::Continue),
                        };
                    };

                    let agent = Arc::new(RwLock::new(agent));
                    let local_agent = agent.clone();
                    let t = thread::spawn(move || {
                        while !agent.read().unwrap().get_game().game_over() {
                            agent.write().unwrap().make_move();
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
            let Event::Key(key_event) = event else {
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
            let Event::Key(key_event) = event else {
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

fn tui_interaction_loop<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
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

pub fn start() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear()?;

    tui_interaction_loop(&mut terminal)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
