use ratatui::{
    Frame,
    layout::{Margin, Rect},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use crate::app::state::AppState;

/// Renders a vertical scrollbar for a list area.
pub fn render_scrollbar(frame: &mut Frame, app: &AppState, area: Rect, selected: usize, count: usize) {
    let config = &app.config.results;
    if !config.show_scrollbar || count == 0 {
        return;
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some(&config.scrollbar.track_symbol))
        .thumb_symbol(&config.scrollbar.thumb_symbol);

    let mut scrollbar_state = ScrollbarState::new(count.saturating_sub(1))
        .position(selected);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin { vertical: 1, horizontal: 0 }),
        &mut scrollbar_state,
    );
}
