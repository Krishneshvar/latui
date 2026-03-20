use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, List, ListItem, Paragraph, Tabs},
};
use ratatui_image::StatefulImage;

use crate::app::state::AppState;
use crate::config::theme::{ItemDisplay, NavbarPosition};
use crate::core::icons;
use crate::core::item::Item;
use crate::ui::components::scrollbar::render_scrollbar;
use crate::ui::style_resolver;
use std::path::PathBuf;

/// Main rendering function for the TUI.
/// Draws the mode tabs, search input, and results list.
pub fn draw(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();
    let config = &app.config;

    // Render full background if configured
    if let Some(ref bg) = config.general.full_background {
        let full_bg_style = Style::default().bg(style_resolver::parse_color(bg));
        frame.render_widget(Block::default().style(full_bg_style), size);
    }

    // Apply margins
    let inner_area = Rect::new(
        size.x + config.layout.margin[3],
        size.y + config.layout.margin[0],
        size.width.saturating_sub(config.layout.margin[1] + config.layout.margin[3]),
        size.height.saturating_sub(config.layout.margin[2] + config.layout.margin[0]),
    );

    let navbar_constraint = if config.navbar.visible {
        Constraint::Length(config.layout.navbar_height)
    } else {
        Constraint::Length(0)
    };

    let search_constraint = if config.search.visible {
        Constraint::Length(config.layout.search_height)
    } else {
        Constraint::Length(0)
    };

    let results_constraint = if config.results.visible {
        Constraint::Min(config.layout.results_min)
    } else {
        Constraint::Length(0)
    };

    let constraints = match config.layout.navbar_position {
        NavbarPosition::Top => [navbar_constraint, search_constraint, results_constraint],
        NavbarPosition::Bottom => [results_constraint, search_constraint, navbar_constraint],
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let (navbar_area, search_area, results_area) = match config.layout.navbar_position {
        NavbarPosition::Top => (layout[0], layout[1], layout[2]),
        NavbarPosition::Bottom => (layout[2], layout[1], layout[0]),
    };

    // Render components if visible
    if config.navbar.visible {
        render_mode_tabs(frame, app, navbar_area);
    }

    if config.search.visible {
        render_search_input(frame, app, search_area);
    }

    if config.results.visible {
        let is_apps_mode = app
            .mode_registry
            .get_active_mode()
            .is_some_and(|mode| mode.name() == "apps");

        if is_apps_mode && app.image_support.is_some() {
            render_apps_results_list_with_inline_icons(frame, app, results_area);
        } else {
            render_results_list(frame, app, results_area);
        }
    }
}

/// Renders the mode selection tabs at the top of the interface.
fn render_mode_tabs(frame: &mut Frame, app: &AppState, area: Rect) {
    let tab_titles = app.mode_registry.get_tab_titles();
    let active_index = app.mode_registry.get_active_index();
    let config = &app.config.navbar;

    let block = style_resolver::resolve_block(&config.title, &config.border, &config.style);

    let active_fg = config.tabs.active_fg.as_deref().unwrap_or_default();
    let active_bg = config.tabs.active_bg.as_deref().unwrap_or_default();

    let tabs = Tabs::new(tab_titles)
        .block(block)
        .select(active_index)
        .style(style_resolver::resolve_style(&config.style))
        .highlight_style(
            Style::default()
                .fg(style_resolver::parse_color(active_fg))
                .bg(style_resolver::parse_color(active_bg))
                .add_modifier(style_resolver::resolve_modifier(&config.tabs.active_modifier)),
        );

    frame.render_widget(tabs, area);
}

/// Renders the search input box with the current query.
fn render_search_input(frame: &mut Frame, app: &AppState, area: Rect) {
    let config = &app.config.search;
    let mode_name = app.mode_registry.get_active_mode().map_or_else(
        || "Search".to_string(),
        |mode| format!("{} {}", mode.icon(), mode.description()),
    );

    let prompt = &config.prompt_symbol;
    let input_text = format!("{}{}", prompt, app.query);
    
    let block = style_resolver::resolve_block(&mode_name, &config.border, &config.style);

    let mut input_style = style_resolver::resolve_style(&config.style);
    if let Some(ref prompt_fg) = config.prompt_fg {
        input_style = input_style.fg(style_resolver::parse_color(prompt_fg));
    }

    let input = Paragraph::new(input_text)
        .block(block)
        .style(input_style);

    frame.render_widget(input, area);
}

/// Renders the list of search results.
fn render_results_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let icon_visible = app.config.results.icon_visible;
    let item_display = app.config.results.item_display.clone();

    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .map(|i| {
            let content = match &item_display {
                ItemDisplay::Name => i.title.clone(),
                ItemDisplay::NameDesc => i
                    .description
                    .as_ref()
                    .map_or_else(|| i.title.clone(), |desc| format!("{} - {}", i.title, desc)),
                ItemDisplay::IconName => match &i.icon {
                    Some(icon) if icon_visible => format!("{} {}", icon, i.title),
                    _ => i.title.clone(),
                },
                ItemDisplay::IconNameDesc => match (&i.icon, &i.description) {
                    (Some(icon), Some(desc)) if icon_visible => format!("{} {} - {}", icon, i.title, desc),
                    (Some(icon), None) if icon_visible => format!("{} {}", icon, i.title),
                    (_, Some(desc)) => format!("{} - {}", i.title, desc),
                    (_, None) => i.title.clone(),
                },
            };
            
            ListItem::new(content).style(style_resolver::resolve_style(&app.config.results.style))
        })
        .collect();

    let config = &app.config.results;
    let results_count = app.filtered_items.len();
    let title = config.title.replace("{count}", &results_count.to_string());

    let block = style_resolver::resolve_block(&title, &config.border, &config.style);

    let bg = config.selected.background.as_deref().unwrap_or_default();
    let fg = config.selected.foreground.as_deref().unwrap_or_default();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(style_resolver::parse_color(bg))
                .fg(style_resolver::parse_color(fg))
                .add_modifier(style_resolver::resolve_modifier(&config.selected.modifier)),
        )
        .highlight_symbol(if config.selected.symbol_visible {
            &config.selected.symbol
        } else {
            ""
        });

    frame.render_stateful_widget(list, area, &mut app.list_state);

    let selected = app.list_state.selected().unwrap_or(0);
    render_scrollbar(frame, app, area, selected, results_count);
}

fn render_apps_results_list_with_inline_icons(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let results_count = app.filtered_items.len();
    let config_results = &app.config.results;
    let title = config_results.title.replace("{count}", &results_count.to_string());
    
    let block = style_resolver::resolve_block(&title, &config_results.border, &config_results.style);
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

    // Manual scroll offset management:
    // Since we are not using ratatui's List widget for this specialized rendering
    // (needed for inline local image rendering), we must manually handle the viewport offset.
    // ListState manages the selection but we must synchronize its internal offset.
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

    // Cache config values to avoid multiple borrows
    let (highlight_style, normal_style, symbol, symbol_visible, item_display, icon_visible, fallback_icon) = {
        let config = &app.config.results;
        let bg = config.selected.background.as_deref().unwrap_or_default();
        let fg = config.selected.foreground.as_deref().unwrap_or_default();
        
        let highlight = Style::default()
            .bg(style_resolver::parse_color(bg))
            .fg(style_resolver::parse_color(fg))
            .add_modifier(style_resolver::resolve_modifier(&config.selected.modifier));
        
        (
            highlight,
            style_resolver::resolve_style(&config.style),
            config.selected.symbol.clone(),
            config.selected.symbol_visible,
            config.item_display.clone(),
            config.icon_visible,
            app.config.modes.apps.icons.fallback.clone(),
        )
    };
        
    let prefix_width: u16 = if symbol_visible {
        symbol.chars().count() as u16
    } else {
        0
    };
    
    let icon_width: u16 = if icon_visible && matches!(item_display, ItemDisplay::IconName | ItemDisplay::IconNameDesc) {
        4
    } else {
        0
    };

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
            
            if symbol_visible {
                let padding = " ".repeat(symbol.chars().count());
                let prefix = if is_selected { symbol.as_str() } else { padding.as_str() };
                buf.set_stringn(
                    prefix_rect.x,
                    prefix_rect.y,
                    prefix,
                    prefix_rect.width as usize,
                    style,
                );
            }

            if text_rect.width > 0 {
                let text = match &item_display {
                    ItemDisplay::Name | ItemDisplay::IconName => item.title.clone(),
                    ItemDisplay::NameDesc | ItemDisplay::IconNameDesc => match &item.description {
                        Some(desc) => format!("{} - {}", item.title, desc),
                        None => item.title.clone(),
                    },
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
            let buf = frame.buffer_mut();
            buf.set_stringn(
                icon_rect.x,
                icon_rect.y,
                &fallback_icon,
                icon_rect.width as usize,
                style,
            );
        }
    }

    // Render scrollbar
    render_scrollbar(frame, app, area, selected, item_count);
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

        if !app.icon_preview_protocols.contains(&cache_key)
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
                            .put(cache_key.clone(), picker.new_resize_protocol(decoded));
                    }
                    None => {
                        app.failed_icon_paths.insert(cache_key.clone());
                    }
                }
            }
        }

    app.icon_preview_protocols
        .get_mut(&cache_key)
        .is_some_and(|protocol| {
            frame.render_stateful_widget(StatefulImage::default(), icon_rect, protocol);
            true
        })
}

fn resolve_desktop_icon_path(app: &mut AppState, item: &Item) -> Option<PathBuf> {
    if let Some(cached) = app.desktop_icon_path_cache.get(&item.id) {
        return cached.clone();
    }

    let desktop_path = std::path::Path::new(&item.id);
    let resolved = icons::resolve_icon_from_entry(desktop_path);
    app.desktop_icon_path_cache
        .put(item.id.clone(), resolved.clone());
    resolved
}
