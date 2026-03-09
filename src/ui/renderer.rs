use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(size);

    let input = Paragraph::new(format!("> {}", app.query))
        .block(Block::default().borders(Borders::ALL).title("latui"));

    frame.render_widget(input, layout[0]);

    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .map(|i| ListItem::new(i.title.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, layout[1], &mut app.list_state);
}
