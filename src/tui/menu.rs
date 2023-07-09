use once_cell::sync::Lazy;
use tui::{
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

pub static MENU_ITEMS: &[&str] = &[
    "Play (Keyboard)",
    "Solve (Random)",
    "Solve (Tree Search, Max Score)",
    "Solve (Tree Search, Max Moves)",
    "Solve (Expectimax)",
];

pub static MENU: Lazy<List> = Lazy::new(|| {
    List::new(
        MENU_ITEMS
            .iter()
            .map(|s| ListItem::new(*s))
            .collect::<Vec<ListItem<'static>>>(),
    )
});

pub fn get_menu<'a>(menu: &List<'a>) -> impl StatefulWidget<State = ListState> + 'a {
    let block = Block::default().title("Menu").borders(Borders::ALL);
    let list = menu
        .clone()
        .block(block)
        .highlight_style(Style::default().fg(Color::Yellow))
        .highlight_symbol(">> ");
    list
}

pub fn get_menu_text() -> impl Widget {
    let block = Block::default().title("Info").borders(Borders::ALL);
    let text = vec![
        Spans::from("Use arrow keys to navigate"),
        Spans::from("Press q to exit"),
    ];
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    paragraph
}
