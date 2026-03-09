mod app;
mod core;
mod matcher;
mod ui;
mod modes;

use std::io;

use app::state::AppState;
use core::{action::Action, item::Item};
use matcher::fuzzy::FuzzyMatcher;
use modes::apps::load_apps;

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

    let items = load_apps();

    let mut app = AppState::new(items);
    let mut matcher = FuzzyMatcher::new();

    loop {
        terminal.draw(|f| ui::renderer::draw(f, &mut app))?;

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
                    app.next();
                }

                KeyCode::Up => {
                    app.previous();
                }

                KeyCode::Enter => {
                    if let Some(i) = app.list_state.selected() {
                        if let Some(item) = app.filtered_items.get(i) {
                            match &item.action {
                                Action::Launch(cmd) => {
                                    std::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(cmd)
                                        .spawn()
                                        .ok();
                                }

                                Action::Command(cmd) => {
                                    std::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(cmd)
                                        .spawn()
                                        .ok();
                                }
                            }
                        }
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

    app.reset_selection();
}
