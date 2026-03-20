use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType};
use crate::config::theme::{BorderConfig, BorderStyle, PanelStyle, TextModifier};

/// Parse a hex string "#rrggbb" or named color into ratatui Color
pub fn parse_color(s: &str) -> Color {
    let s = s.trim();
    if s.starts_with('#') && (s.len() == 7 || s.len() == 4) {
        if s.len() == 7 {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
            return Color::Rgb(r, g, b);
        }
        // #rgb shorthand
        let r_v = u8::from_str_radix(&s[1..2], 16).unwrap_or(15);
        let g_v = u8::from_str_radix(&s[2..3], 16).unwrap_or(15);
        let b_v = u8::from_str_radix(&s[3..4], 16).unwrap_or(15);
        return Color::Rgb(r_v * 17, g_v * 17, b_v * 17);
    }
    
    // fallback to named colors
    match s.to_lowercase().as_str() {
        "black"        => Color::Black,
        "red"          => Color::Red,
        "green"        => Color::Green,
        "yellow"       => Color::Yellow,
        "blue"         => Color::Blue,
        "magenta"      => Color::Magenta,
        "cyan"         => Color::Cyan,
        "white"        => Color::White,
        "darkgray"     => Color::DarkGray,
        "gray"         => Color::Gray,
        "lightred"     => Color::LightRed,
        "lightgreen"   => Color::LightGreen,
        "lightyellow"  => Color::LightYellow,
        "lightblue"    => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan"    => Color::LightCyan,
        _              => Color::Reset,
    }
}

pub fn resolve_modifier(modifiers: &[TextModifier]) -> Modifier {
    modifiers.iter().fold(Modifier::empty(), |acc, m| {
        acc | match m {
            TextModifier::Bold        => Modifier::BOLD,
            TextModifier::Italic      => Modifier::ITALIC,
            TextModifier::Dim         => Modifier::DIM,
            TextModifier::Underlined  => Modifier::UNDERLINED,
            TextModifier::Reversed    => Modifier::REVERSED,
            TextModifier::CrossedOut  => Modifier::CROSSED_OUT,
            TextModifier::SlowBlink   => Modifier::SLOW_BLINK,
        }
    })
}

pub fn resolve_style(panel: &PanelStyle) -> Style {
    let mut s = Style::default();
    if let Some(ref fg) = panel.foreground { s = s.fg(parse_color(fg)); }
    if let Some(ref bg) = panel.background { s = s.bg(parse_color(bg)); }
    s
}

pub const fn resolve_borders(border: &BorderConfig) -> Borders {
    if border.visible { Borders::ALL } else { Borders::NONE }
}

pub const fn resolve_border_type(border: &BorderConfig) -> BorderType {
    match border.style {
        BorderStyle::Plain | BorderStyle::None => BorderType::Plain,
        BorderStyle::Rounded => BorderType::Rounded,
        BorderStyle::Double  => BorderType::Double,
        BorderStyle::Thick   => BorderType::Thick,
    }
}

pub fn resolve_block<'a>(title: &'a str, border: &BorderConfig, style: &PanelStyle) -> Block<'a> {
    let borders = resolve_borders(border);
    let border_type = resolve_border_type(border);
    let mut block = Block::default()
        .borders(borders)
        .border_type(border_type)
        .style(resolve_style(style));
    
    if let Some(ref color) = border.color {
        block = block.border_style(Style::default().fg(parse_color(color)));
    }
    
    if !title.is_empty() {
        block = block.title(title);
    }
    
    block
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#ffffff"), Color::Rgb(255, 255, 255));
        assert_eq!(parse_color("#000000"), Color::Rgb(0, 0, 0));
        assert_eq!(parse_color("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(parse_color("#fff"), Color::Rgb(255, 255, 255));
        assert_eq!(parse_color("#000"), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red"), Color::Red);
        assert_eq!(parse_color("Blue"), Color::Blue);
        assert_eq!(parse_color("unknown"), Color::Reset);
    }
}
