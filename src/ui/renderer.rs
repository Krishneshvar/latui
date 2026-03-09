use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::AppState;

pub fn draw(frame: &mut Frame, app: &AppState) {
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
        .enumerate()
        .map(|(i, item)| {
            if i == app.selected {
                ListItem::new(format!("> {}", item.title))
            } else {
                ListItem::new(item.title.clone())
            }
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    frame.render_widget(list, layout[1]);
}
