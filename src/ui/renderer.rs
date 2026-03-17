use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::state::AppState;

/// Main rendering function for the TUI.
/// Draws the mode tabs, search input, and results list.
pub fn draw(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    // Create layout with mode tabs at the top
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Mode tabs
            Constraint::Length(3),  // Search input
            Constraint::Min(1),     // Results list
        ])
        .split(size);

    // Render mode tabs
    render_mode_tabs(frame, app, layout[0]);
    
    // Render search input
    render_search_input(frame, app, layout[1]);
    
    // Render results list
    render_results_list(frame, app, layout[2]);
}

/// Renders the mode selection tabs at the top of the interface.
fn render_mode_tabs(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let tab_titles = app.mode_registry.get_tab_titles();
    let active_index = app.mode_registry.get_active_index();
    
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("LaTUI - Multi-Mode Launcher")
        )
        .select(active_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        );
    
    frame.render_widget(tabs, area);
}

/// Renders the search input box with the current query.
fn render_search_input(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let mode_name = if let Some(mode) = app.mode_registry.get_active_mode() {
        format!("{} {}", mode.icon(), mode.description())
    } else {
        "Search".to_string()
    };
    
    let input = Paragraph::new(format!("> {}", app.query))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(mode_name)
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, area);
}

/// Renders the list of search results.
fn render_results_list(frame: &mut Frame, app: &mut AppState, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .map(|i| {
            let content = match (&i.icon, &i.description) {
                (Some(icon), Some(desc)) => format!("{} {} - {}", icon, i.title, desc),
                (Some(icon), None) => format!("{} {}", icon, i.title),
                (None, Some(desc)) => format!("{} - {}", i.title, desc),
                (None, None) => i.title.clone(),
            };
            ListItem::new(content)
        })
        .collect();

    let results_count = app.filtered_items.len();
    let title = format!("Results ({})", results_count);
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}
