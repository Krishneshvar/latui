use latui::ui::style_resolver;
use ratatui::style::{Color, Modifier};
use latui::config::theme::TextModifier;

#[test]
fn test_parse_color() {
    assert_eq!(style_resolver::parse_color("#ff0000"), Color::Rgb(255, 0, 0));
    assert_eq!(style_resolver::parse_color("#00ff00"), Color::Rgb(0, 255, 0));
    assert_eq!(style_resolver::parse_color("#0000ff"), Color::Rgb(0, 0, 255));
    assert_eq!(style_resolver::parse_color("#fff"), Color::Rgb(255, 255, 255));
    assert_eq!(style_resolver::parse_color("#000"), Color::Rgb(0, 0, 0));
    assert_eq!(style_resolver::parse_color("red"), Color::Red);
    assert_eq!(style_resolver::parse_color("blue"), Color::Blue);
    assert_eq!(style_resolver::parse_color("invalid"), Color::Reset);
}

#[test]
fn test_resolve_modifier() {
    assert_eq!(style_resolver::resolve_modifier(&[TextModifier::Bold]), Modifier::BOLD);
    assert_eq!(style_resolver::resolve_modifier(&[TextModifier::Bold, TextModifier::Italic]), Modifier::BOLD | Modifier::ITALIC);
    assert_eq!(style_resolver::resolve_modifier(&[]), Modifier::empty());
}
