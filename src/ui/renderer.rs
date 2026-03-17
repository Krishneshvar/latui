use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};
use ratatui_image::StatefulImage;

use crate::app::state::AppState;
use crate::core::item::Item;
use freedesktop_desktop_entry::DesktopEntry;
use std::path::{Path, PathBuf};

/// Main rendering function for the TUI.
/// Draws the mode tabs, search input, and results list.
pub fn draw(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    // Create layout with mode tabs at the top
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Mode tabs
            Constraint::Length(3), // Search input
            Constraint::Min(1),    // Results list
        ])
        .split(size);

    // Render mode tabs
    render_mode_tabs(frame, app, layout[0]);

    // Render search input
    render_search_input(frame, app, layout[1]);

    let is_apps_mode = app
        .mode_registry
        .get_active_mode()
        .map(|mode| mode.name() == "apps")
        .unwrap_or(false);

    if is_apps_mode && app.image_support.is_some() {
        render_apps_results_list_with_inline_icons(frame, app, layout[2]);
    } else {
        render_results_list(frame, app, layout[2]);
    }
}

/// Renders the mode selection tabs at the top of the interface.
fn render_mode_tabs(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let tab_titles = app.mode_registry.get_tab_titles();
    let active_index = app.mode_registry.get_active_index();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("LaTUI - Multi-Mode Launcher"),
        )
        .select(active_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
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
        .block(Block::default().borders(Borders::ALL).title(mode_name))
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
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_apps_results_list_with_inline_icons(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let results_count = app.filtered_items.len();
    let title = format!("Results ({})", results_count);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if app.filtered_items.is_empty() {
        app.list_state.select(None);
        return;
    }

    let item_count = app.filtered_items.len();
    let selected = app
        .list_state
        .selected()
        .unwrap_or(0)
        .min(item_count.saturating_sub(1));
    app.list_state.select(Some(selected));

    let viewport_rows = inner.height as usize;
    let mut offset = app.list_state.offset().min(item_count.saturating_sub(1));
    if selected < offset {
        offset = selected;
    } else if selected >= offset.saturating_add(viewport_rows) {
        offset = selected + 1 - viewport_rows;
    }
    let max_offset = item_count.saturating_sub(viewport_rows);
    if offset > max_offset {
        offset = max_offset;
    }
    *app.list_state.offset_mut() = offset;

    let highlight_style = Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);
    let prefix_width: u16 = 3; // `>> `
    let icon_width: u16 = 4;

    for row in 0..inner.height {
        let index = offset + row as usize;
        if index >= item_count {
            break;
        }

        let item = app.filtered_items[index].clone();
        let is_selected = index == selected;
        let style = if is_selected {
            highlight_style
        } else {
            normal_style
        };

        let row_rect = Rect::new(inner.x, inner.y + row, inner.width, 1);
        let prefix_rect = Rect::new(
            row_rect.x,
            row_rect.y,
            row_rect.width.min(prefix_width),
            row_rect.height,
        );
        let icon_rect = Rect::new(
            prefix_rect.x + prefix_rect.width,
            row_rect.y,
            row_rect
                .width
                .saturating_sub(prefix_rect.width)
                .min(icon_width),
            row_rect.height,
        );
        let text_rect = Rect::new(
            icon_rect.x + icon_rect.width,
            row_rect.y,
            row_rect
                .width
                .saturating_sub(prefix_rect.width + icon_rect.width),
            row_rect.height,
        );

        {
            let buf = frame.buffer_mut();
            buf.set_style(row_rect, style);
            let prefix = if is_selected { ">> " } else { "   " };
            buf.set_stringn(
                prefix_rect.x,
                prefix_rect.y,
                prefix,
                prefix_rect.width as usize,
                style,
            );

            if text_rect.width > 0 {
                let text = match &item.description {
                    Some(desc) => format!("{} - {}", item.title, desc),
                    None => item.title.clone(),
                };
                buf.set_stringn(
                    text_rect.x,
                    text_rect.y,
                    text,
                    text_rect.width as usize,
                    style,
                );
            }
        }

        let rendered_image = render_inline_icon_image(frame, app, icon_rect, &item);
        if !rendered_image && icon_rect.width > 0 {
            let fallback = "⚙️";
            let buf = frame.buffer_mut();
            buf.set_stringn(
                icon_rect.x,
                icon_rect.y,
                fallback,
                icon_rect.width as usize,
                style,
            );
        }
    }
}

fn render_inline_icon_image(
    frame: &mut Frame,
    app: &mut AppState,
    icon_rect: Rect,
    item: &Item,
) -> bool {
    if icon_rect.width == 0 || icon_rect.height == 0 {
        return false;
    }

    let Some(icon_path) = resolve_desktop_icon_path(app, item) else {
        return false;
    };

    let cache_key = icon_path.to_string_lossy().to_string();

    if !app.icon_preview_protocols.contains_key(&cache_key)
        && !app.failed_icon_paths.contains(&cache_key)
    {
        let picker = app
            .image_support
            .as_ref()
            .map(|support| support.picker.clone());
        if let Some(picker) = picker {
            match image::ImageReader::open(&icon_path)
                .ok()
                .and_then(|r| r.with_guessed_format().ok())
                .and_then(|r| r.decode().ok())
            {
                Some(decoded) => {
                    app.icon_preview_protocols
                        .insert(cache_key.clone(), picker.new_resize_protocol(decoded));
                }
                None => {
                    app.failed_icon_paths.insert(cache_key.clone());
                }
            }
        }
    }

    if let Some(protocol) = app.icon_preview_protocols.get_mut(&cache_key) {
        frame.render_stateful_widget(StatefulImage::default(), icon_rect, protocol);
        true
    } else {
        false
    }
}

fn resolve_desktop_icon_path(app: &mut AppState, item: &Item) -> Option<PathBuf> {
    if let Some(cached) = app.desktop_icon_path_cache.get(&item.id) {
        return cached.clone();
    }

    let desktop_path = PathBuf::from(&item.id);
    let resolved = resolve_desktop_icon_path_impl(&desktop_path);
    app.desktop_icon_path_cache
        .insert(item.id.clone(), resolved.clone());
    resolved
}

fn resolve_desktop_icon_path_impl(desktop_path: &Path) -> Option<PathBuf> {
    if !desktop_path.exists() {
        return None;
    }

    let entry = DesktopEntry::from_path(desktop_path, None::<&[&str]>).ok()?;
    let icon_name = entry.icon()?.trim();
    if icon_name.is_empty() {
        return None;
    }

    let direct = Path::new(icon_name);
    if direct.is_absolute() && direct.exists() {
        return Some(direct.to_path_buf());
    }

    let theme = std::env::var("LATUI_ICON_THEME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(freedesktop_icons::default_theme_gtk)
        .unwrap_or_else(|| "hicolor".to_string());

    freedesktop_icons::lookup(icon_name)
        .with_size(96)
        .with_scale(1)
        .with_theme(&theme)
        .with_cache()
        .find()
}
