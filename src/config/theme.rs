use serde::{Deserialize, Serialize};
use crate::config::settings::ModesSettings;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub navbar: NavbarConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub results: ResultsConfig,
    #[serde(default)]
    pub modes: ModesSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_theme_name")]
    pub theme: String,
    pub full_background: Option<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_mode: default_mode(),
            max_results: default_max_results(),
            theme: default_theme_name(),
            full_background: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutConfig {
    #[serde(default = "default_margin")]
    pub margin: [u16; 4],
    #[serde(default = "default_navbar_height")]
    pub navbar_height: u16,
    #[serde(default = "default_search_height")]
    pub search_height: u16,
    #[serde(default = "default_results_min")]
    pub results_min: u16,
    #[serde(default = "default_navbar_pos")]
    pub navbar_position: NavbarPosition,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            margin: default_margin(),
            navbar_height: default_navbar_height(),
            search_height: default_search_height(),
            results_min: default_results_min(),
            navbar_position: default_navbar_pos(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NavbarPosition {
    #[default]
    Top,
    Bottom,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavbarConfig {
    #[serde(default = "default_navbar_title")]
    pub title: String,
    #[serde(default = "default_title_align")]
    pub title_alignment: TitleAlignment,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub border: BorderConfig,
    #[serde(default)]
    pub style: PanelStyle,
    #[serde(default)]
    pub tabs: TabsConfig,
}

impl Default for NavbarConfig {
    fn default() -> Self {
        Self {
            title: default_navbar_title(),
            title_alignment: default_title_align(),
            visible: default_true(),
            border: BorderConfig::default(),
            style: PanelStyle::default(),
            tabs: TabsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TitleAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorderConfig {
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_border_style")]
    pub style: BorderStyle,
    #[serde(default)]
    pub color: Option<String>,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            visible: default_true(),
            style: default_border_style(),
            color: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BorderStyle {
    #[default]
    Rounded,
    Plain,
    Double,
    Thick,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PanelStyle {
    pub background: Option<String>,
    pub foreground: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TabsConfig {
    #[serde(default = "default_active_fg")]
    pub active_fg: Option<String>,
    #[serde(default)]
    pub active_bg: Option<String>,
    #[serde(default = "default_active_modifier")]
    pub active_modifier: Vec<TextModifier>,
    #[serde(default = "default_inactive_fg")]
    pub inactive_fg: Option<String>,
    #[serde(default)]
    pub inactive_bg: Option<String>,
    #[serde(default = "default_true")]
    pub show_icons: bool,
}

impl Default for TabsConfig {
    fn default() -> Self {
        Self {
            active_fg: default_active_fg(),
            active_bg: None,
            active_modifier: default_active_modifier(),
            inactive_fg: default_inactive_fg(),
            inactive_bg: None,
            show_icons: default_true(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextModifier {
    Bold,
    Italic,
    Dim,
    Underlined,
    Reversed,
    CrossedOut,
    SlowBlink,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchConfig {
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_prompt")]
    pub prompt_symbol: String,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub border: BorderConfig,
    #[serde(default)]
    pub style: PanelStyle,
    #[serde(default)]
    pub prompt_fg: Option<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            visible: default_true(),
            prompt_symbol: default_prompt(),
            placeholder: None,
            border: BorderConfig::default(),
            style: PanelStyle::default(),
            prompt_fg: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultsConfig {
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_results_title")]
    pub title: String,
    #[serde(default)]
    pub show_count: bool,
    #[serde(default = "default_item_display")]
    pub item_display: ItemDisplay,
    #[serde(default = "default_true")]
    pub icon_visible: bool,
    #[serde(default = "default_true")]
    pub show_scrollbar: bool,
    #[serde(default = "default_item_padding")]
    pub item_padding: [u16; 2],
    #[serde(default)]
    pub border: BorderConfig,
    #[serde(default)]
    pub style: PanelStyle,
    #[serde(default)]
    pub selected: SelectedItemStyle,
    #[serde(default)]
    pub scrollbar: ScrollbarConfig,
}

impl Default for ResultsConfig {
    fn default() -> Self {
        Self {
            visible: default_true(),
            title: default_results_title(),
            show_count: false,
            item_display: default_item_display(),
            icon_visible: default_true(),
            show_scrollbar: default_true(),
            item_padding: default_item_padding(),
            border: BorderConfig::default(),
            style: PanelStyle::default(),
            selected: SelectedItemStyle::default(),
            scrollbar: ScrollbarConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemDisplay {
    Name,
    NameDesc,
    IconName,
    #[default]
    IconNameDesc,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectedItemStyle {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
    #[serde(default = "default_selected_modifier")]
    pub modifier: Vec<TextModifier>,
    #[serde(default = "default_selection_symbol")]
    pub symbol: String,
    #[serde(default = "default_true")]
    pub symbol_visible: bool,
}

impl Default for SelectedItemStyle {
    fn default() -> Self {
        Self {
            background: None,
            foreground: None,
            modifier: default_selected_modifier(),
            symbol: default_selection_symbol(),
            symbol_visible: default_true(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScrollbarConfig {
    #[serde(default = "default_thumb")]
    pub thumb_symbol: String,
    #[serde(default = "default_track")]
    pub track_symbol: String,
    #[serde(default)]
    pub color: Option<String>,
}

// Default helper functions for serde
fn default_true() -> bool { true }
fn default_mode() -> String { "apps".to_string() }
fn default_max_results() -> usize { 50 }
fn default_theme_name() -> String { "dark".to_string() }
fn default_margin() -> [u16; 4] { [0, 0, 0, 0] }
fn default_navbar_height() -> u16 { 3 }
fn default_search_height() -> u16 { 3 }
fn default_results_min() -> u16 { 1 }
fn default_navbar_pos() -> NavbarPosition { NavbarPosition::Top }
fn default_navbar_title() -> String { "LaTUI".to_string() }
fn default_title_align() -> TitleAlignment { TitleAlignment::Left }
fn default_border_style() -> BorderStyle { BorderStyle::Rounded }
fn default_active_fg() -> Option<String> { Some("#7aa2f7".to_string()) }
fn default_active_modifier() -> Vec<TextModifier> { vec![TextModifier::Bold] }
fn default_inactive_fg() -> Option<String> { Some("#565f89".to_string()) }
fn default_prompt() -> String { "> ".to_string() }
fn default_results_title() -> String { "Results ({count})".to_string() }
fn default_item_display() -> ItemDisplay { ItemDisplay::IconNameDesc }
fn default_item_padding() -> [u16; 2] { [0, 1] }
fn default_selected_modifier() -> Vec<TextModifier> { vec![TextModifier::Bold] }
fn default_selection_symbol() -> String { ">> ".to_string() }
fn default_thumb() -> String { "█".to_string() }
fn default_track() -> String { "░".to_string() }
