mod app;
mod core;
mod matcher;
mod ui;

use std::io;

use app::state::AppState;
use core::{action::Action, item::Item};
use matcher::fuzzy::FuzzyMatcher;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items = vec![
        Item { id: "1".into(), title: "firefox".into(), description: None, score: 0, action: Action::None },
        Item { id: "2".into(), title: "fish".into(), description: None, score: 0, action: Action::None },
        Item { id: "3".into(), title: "files".into(), description: None, score: 0, action: Action::None },
        Item { id: "4".into(), title: "foot".into(), description: None, score: 0, action: Action::None },
    ];

    let mut app = AppState::new(items);
    let mut matcher = FuzzyMatcher::new();

    loop {
        terminal.draw(|f| ui::renderer::draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    app.query.push(c);
                    update_results(&mut app, &mut matcher);
                }

                KeyCode::Backspace => {
                    app.query.pop();
                    update_results(&mut app, &mut matcher);
                }

                KeyCode::Down => {
                    if app.selected + 1 < app.filtered_items.len() {
                        app.selected += 1;
                    }
                }

                KeyCode::Up => {
                    if app.selected > 0 {
                        app.selected -= 1;
                    }
                }

                KeyCode::Enter => {
                    if let Some(item) = app.filtered_items.get(app.selected) {
                        println!("Selected: {}", item.title);
                        break;
                    }
                }

                KeyCode::Esc => break,

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn update_results(app: &mut AppState, matcher: &mut FuzzyMatcher) {
    if app.query.is_empty() {
        app.filtered_items = app.all_items.clone();
        return;
    }

    let titles: Vec<String> = app
        .all_items
        .iter()
        .map(|i| i.title.clone())
        .collect();

    let matches = matcher.filter(&app.query, &titles);

    app.filtered_items = matches
        .iter()
        .map(|(i, _)| app.all_items[*i].clone())
        .collect();

    app.selected = 0;
}
