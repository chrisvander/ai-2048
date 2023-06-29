use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::game::Game;

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

fn get_table_cell(c: &u32) -> Cell<'_> {
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

pub fn render_board<B: Backend>(f: &mut Frame<B>, game: &Game, rect: Rect) {
    let block = Block::default().title("Game").borders(Borders::ALL);
    let game_state = game.get_table();
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows = game_state.iter().map(|row| {
        let row = row.iter().map(get_table_cell);
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
