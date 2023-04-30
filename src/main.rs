use crate::game::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

mod game;

static TICK_RATE: Duration = Duration::from_millis(250);
static MENU_ITEMS: &[&str] = &["Play (Keyboard)", "Play (Expectimax)", "Play (RL)"];

pub enum Screen {
    Menu {
        state: ListState,
        menu: List<'static>,
    },
    KeyboardGame(Game),
    AIGame(Box<dyn Agent>, Game),
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

fn render_game<B: Backend>(f: &mut Frame<B>, block: Block<'_>, game: &Game, rect: Rect) {
    let game_state = game.get_table();

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::DarkGray);
    let rows = game_state.iter().map(|row| {
        let cells = row.iter().map(|c| {
            let cstr = if *c == 1 {
                String::from("")
            } else {
                c.to_string()
            };

            Cell::from(Text::from(
                [
                    "\n\n",
                    " ".repeat(5 - (cstr.len() / 2)).as_str(),
                    cstr.as_str(),
                    "\n\n",
                ]
                .join(""),
            ))
            .style(normal_style)
        });
        Row::new(cells).height(5).bottom_margin(1)
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
            let paragraph = Paragraph::new("Use arrow keys to change menu items\nPress q to exit")
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        }
        Screen::KeyboardGame(game) => {
            let game_block = Block::default().title("Game").borders(Borders::ALL);
            render_game(f, game_block, game, chunks[0]);
            let block = Block::default().title("Info").borders(Borders::ALL);
            let paragraph = Paragraph::new("Use arrow keys to move the tiles\nPress q to exit")
                .block(block)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[1]);
        }
        Screen::AIGame(agent, game) => {
            let game_block = Block::default().title("Game").borders(Borders::ALL);
            f.render_widget(game_block, chunks[0]);
            let block = Block::default().title("Log").borders(Borders::ALL);
            let paragraph = Paragraph::new(agent.log_messages())
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    terminal.clear()?;
    let mut app = App::default();
    let mut last_tick = std::time::Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        match event::poll(timeout)? {
            true => match &mut app.screen {
                Screen::Menu { state, menu: _ } => {
                    let event = event::read().unwrap();
                    match event {
                        Event::Key(event) => match event.code {
                            KeyCode::Char('q') => break Ok(()),
                            KeyCode::Up => {
                                let Some(sel) = state.selected() else { continue };
                                if sel > 0 {
                                    state.select(Some(sel - 1));
                                }
                            }
                            KeyCode::Down => {
                                let Some(sel) = state.selected() else { continue };
                                if sel < MENU_ITEMS.len() - 1 {
                                    state.select(Some(sel + 1));
                                }
                            }
                            KeyCode::Enter => match state.selected() {
                                Some(0) => {
                                    app.screen = Screen::KeyboardGame(Game::new());
                                }
                                _ => {}
                            },
                            _ => {}
                        },
                        _ => {}
                    }
                }
                Screen::KeyboardGame(game) => {
                    let Ok(keyboard_move) = (match event::read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Char('q') => { app.screen = Screen::default(); continue },
                            KeyCode::Char('w') => Ok(Move::Up),
                            KeyCode::Char('a') => Ok(Move::Left),
                            KeyCode::Char('s') => Ok(Move::Down),
                            KeyCode::Char('d') => Ok(Move::Right),
                            KeyCode::Up => Ok(Move::Up),
                            KeyCode::Left => Ok(Move::Left),
                            KeyCode::Down => Ok(Move::Down),
                            KeyCode::Right => Ok(Move::Right),
                            _ => Err("Invalid key"),
                        },
                        _ => Err("Event not a key"),
                    }) else {
                        continue;
                    };

                    game.update(keyboard_move);
                }
                Screen::AIGame(agent, game) => {
                    agent.make_move(game);
                }
            },
            _ => {}
        };

        if last_tick.elapsed() >= TICK_RATE {
            last_tick = std::time::Instant::now();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    run_app(&mut terminal)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
