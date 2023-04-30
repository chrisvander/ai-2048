use crate::agent::random::RandomAgent;
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

mod agent;
mod game;

static TICK_RATE: Duration = Duration::from_millis(250);
static MENU_ITEMS: &[&str] = &[
    "Play (Keyboard)",
    "Play (Random)",
    "Play (Expectimax)",
    "Play (RL)",
];

pub enum Screen {
    Menu {
        state: ListState,
        menu: List<'static>,
    },
    Game(Option<Box<dyn Agent>>, Game),
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

fn render_game<B: Backend>(f: &mut Frame<B>, block: Block<'_>, game: &Game, rect: Rect) {
    let game_state = game.get_table();

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default()
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
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
            .style(normal_style.bg(get_color_for_value(*c)))
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
            let text = vec![
                Spans::from("Use arrow keys to navigate"),
                Spans::from("Press q to exit"),
            ];
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        }
        Screen::Game(agent, game) => {
            let game_block = Block::default().title("Game").borders(Borders::ALL);
            render_game(f, game_block, game, chunks[0]);
            let block = Block::default().title("Info").borders(Borders::ALL);
            let game_over_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
            let bold_span = |s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD));
            let text = vec![
                Spans::from(vec![
                    bold_span("Score: "),
                    Span::from(game.get_score().to_string()),
                ]),
                Spans::from(if game.game_over() {
                    Span::styled("Game over.", game_over_style)
                } else {
                    Span::from("")
                }),
                Spans::from(""),
                Spans::from(if let Some(a) = agent {
                    a.log_messages()
                } else {
                    "Use arrow keys to move the tiles".to_string()
                }),
                Spans::from("Press q to exit"),
            ];
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
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

        match &mut app.screen {
            Screen::Menu { state, menu: _ } => {
                if !event::poll(timeout)? {
                    continue;
                }
                let Event::Key(event) = event::read().unwrap() else { continue; };
                match event.code {
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
                    KeyCode::Enter => {
                        let agent: Option<Box<dyn Agent>> = match state.selected() {
                            Some(1) => Some(Box::new(RandomAgent::default())),
                            _ => None,
                        };

                        app.screen = Screen::Game(agent, Game::new());
                    }
                    _ => {}
                };
            }
            Screen::Game(None, game) => {
                let Ok(keyboard_move) = (match event::read()? {
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
            Screen::Game(Some(agent), game) => {
                if event::poll(Duration::ZERO)? || game.game_over() {
                    let Event::Key(event) = event::read()? else { continue };
                    match event.code {
                        KeyCode::Char('q') => {
                            app.screen = Screen::default();
                            continue;
                        }
                        _ => {}
                    };
                }

                agent.make_move(game);
            }
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
