# LaTUI Customizability & Theming System — Design Plan

> **Status:** Planning  
> **Version Target:** v2.0.0  
> **Author:** Architecture review + design by Antigravity  
> **Date:** 2026-03-20

---

## 1. Executive Summary

LaTUI currently has no real customizability surface. The `Theme` struct in `src/ui/theme.rs` is a stub with only a `name: String` field, and all visual decisions in `src/ui/renderer.rs` are hardcoded inline — colors, borders, symbols, layout heights, padding. The config system (`src/config/settings.rs`) is purely functional (which desktop dirs to index, which apps to skip) and has zero influence on visual output.

This document defines a **production-grade theming and customizability system** inspired by Rofi's rasi/CSS approach, adapted for a TUI-first Rust app. The result will allow users to configure *every visual and layout aspect* of LaTUI via a single `~/.config/latui/config.toml`, with the possibility of shipping named preset themes.

The key insight: **in ratatui, CSS does not exist**. Instead of CSS, we replicate CSS's expressive power through a structured TOML theme/config block that maps directly to ratatui's `Style`, `Block`, `Layout`, and widget primitives. Think of it as a "CSS for the terminal."

---

## 2. Current State Analysis

### 2.1 What is hardcoded today

| Location | What is hardcoded |
|---|---|
| `renderer.rs:57` | Navbar title `"LaTUI - Multi-Mode Launcher"` |
| `renderer.rs:56` | `Borders::ALL` on navbar |
| `renderer.rs:60–64` | Tab fg = White, selected tab fg = Yellow + BOLD |
| `renderer.rs:78` | Search prompt symbol `> ` |
| `renderer.rs:79` | `Borders::ALL` on search bar |
| `renderer.rs:80` | Search bar text fg = White |
| `renderer.rs:105` | `Borders::ALL` on results list |
| `renderer.rs:107–110` | Selected item bg = DarkGray + BOLD |
| `renderer.rs:111` | Selection prefix symbol `>> ` |
| `renderer.rs:153–156` | Inline icon renderer uses same DarkGray + BOLD / White |
| `renderer.rs:202` | `>> ` vs `   ` for inline icon view selection marker |
| `renderer.rs:22–26` | Layout heights: navbar = 3, search = 3, results = Min(1) — no margin/padding |
| `theme.rs` | Entire struct is a stub — only `name: String` |
| `state.rs` | `theme: Theme` field is present but never used in rendering |

### 2.2 What already exists (reuse opportunity)

- XDG base dirs via `xdg` crate → already used for `~/.config/latui/` path resolution.
- `serde` + `toml` crates → already used for deserialization of `UserSettings`. Extend naturally.
- `ratatui::style::{Color, Modifier, Style}` → already imported everywhere.
- `src/config/settings.rs` → clean pattern to follow for new config structs.

---

## 3. High-Level Architecture

```
~/.config/latui/
├── config.toml          ← main config: general + [theme.*] + [modes.*]
└── themes/
    ├── dark.toml        ← named preset (ships with latui)
    ├── light.toml       ← named preset
    ├── catppuccin.toml  ← community theme
    └── my-custom.toml   ← user's own theme
```

### 3.1 Loading flow

```
main.rs
  └── load_user_settings()         ← extended to load full UserConfig
        ├── [general] → GeneralConfig
        ├── [theme]   → inline ThemeConfig  OR  load themes/<name>.toml
        ├── [navbar]  → NavbarConfig
        ├── [search]  → SearchConfig
        ├── [results] → ResultsConfig
        └── [modes.*] → per-mode config (already exists)
              ↓
        AppConfig (unified top-level struct)
              ↓
        AppState.config: AppConfig    (replaces dead `theme: Theme`)
              ↓
        renderer::draw(frame, app)   ← reads app.config everywhere
```

### 3.2 Layer separation

| Layer | Responsibility | File |
|---|---|---|
| **Config structs** | Typed TOML deserialization, default values | `src/config/theme.rs` (new) |
| **Config loader** | XDG path resolution, fallback chain, parse errors | `src/config/settings.rs` (extend) |
| **Style resolver** | Convert config structs → ratatui `Style`/`Block` | `src/ui/style_resolver.rs` (new) |
| **Renderer** | Consume resolved styles, never hardcode visuals | `src/ui/renderer.rs` (refactor) |
| **AppState** | Hold `AppConfig`, pass to renderer | `src/app/state.rs` (extend) |
| **Preset themes** | Bundled TOML theme files | `assets/themes/` (new) |

---

## 4. Config Schema

This is the full proposed TOML schema a user can place in `~/.config/latui/config.toml`.

### 4.1 `[general]`

```toml
[general]
default_mode   = "apps"    # which mode to open with
max_results    = 50        # max items shown in results list
theme          = "dark"    # name of preset, or "inline" to use [theme.*] blocks below
```

### 4.2 `[layout]` — Overall app geometry

```toml
[layout]
# Outer padding from terminal edges (top, right, bottom, left) in terminal cells
margin = [0, 0, 0, 0]

# Heights in terminal rows (0 = auto/disabled)
navbar_height  = 3
search_height  = 3
results_min    = 1         # Min(n) for the results Constraint

# Navbar position: "top" | "bottom"
navbar_position = "top"
```

> **Why TOML margin tuples instead of CSS shorthand:** TOML has no CSS shorthand syntax. Explicit arrays `[top, right, bottom, left]` are self-documenting and trivially deserialized to a `[u16; 4]`.

### 4.3 `[navbar]`

```toml
[navbar]
title             = "LaTUI"        # Text shown in the navbar block title
title_alignment   = "left"         # "left" | "center" | "right"
visible           = true           # Show/hide the entire navbar

[navbar.border]
visible           = true
style             = "rounded"      # "plain" | "rounded" | "double" | "thick" | "none"
color             = "#c0caf5"      # hex RGB or named ratatui color
width             = 1              # logical border weight (1 = normal, 2 = thick)

[navbar.style]
background        = "#1a1b26"
foreground        = "#c0caf5"

[navbar.tabs]
active_fg         = "#7aa2f7"
active_bg         = ""             # empty = transparent
active_modifier   = ["bold"]       # array of: "bold" | "italic" | "dim" | "underlined"
inactive_fg       = "#565f89"
inactive_bg       = ""
show_icons        = true           # show mode icons in tabs
```

### 4.4 `[search]`

```toml
[search]
visible           = true
prompt_symbol     = "> "           # prefix shown before typed query
placeholder       = "Type to search..."

[search.border]
visible           = true
style             = "rounded"
color             = "#3b4261"

[search.style]
background        = "#1a1b26"
foreground        = "#c0caf5"
prompt_fg         = "#7aa2f7"      # color of the prompt symbol
```

### 4.5 `[results]`

```toml
[results]
visible           = true
title             = "Results ({count})"   # {count} is a placeholder substituted at runtime
show_count        = true

# What to show per item:
# "name"           → only the item name/title
# "name_desc"      → name + " - " + description
# "icon_name"      → icon + name
# "icon_name_desc" → icon + name + description (default)
item_display      = "icon_name_desc"

icon_visible      = true           # show text emoji/icon column
show_scrollbar    = true           # show a scrollbar indicator (ratatui Scrollbar widget)

item_padding      = [0, 1]         # [vertical, horizontal] padding per item row (in cells)

[results.border]
visible           = true
style             = "rounded"
color             = "#3b4261"

[results.style]
background        = "#1a1b26"
foreground        = "#c0caf5"

[results.selected]
background        = "#283457"
foreground        = "#c0caf5"
modifier          = ["bold"]
symbol            = ">> "          # prefix on selected item
symbol_visible    = true

[results.normal]
modifier          = []

[results.scrollbar]
thumb_symbol      = "█"
track_symbol      = "░"
color             = "#3b4261"
```

### 4.6 Named Preset Themes (`assets/themes/dark.toml`)

A preset theme file uses exactly the same schema as the `[navbar]`, `[search]`, `[results]` blocks above, but lives in a separate file:

```toml
# assets/themes/dark.toml
[navbar.style]
background = "#1a1b26"
foreground = "#c0caf5"
# ... rest of fields
```

When `general.theme = "dark"`, the loader reads `assets/themes/dark.toml` (or from `~/.config/latui/themes/dark.toml` if the user places their own), and uses it as the base. Any inline `[navbar.*]` blocks in `config.toml` **override** the preset values (cascade semantics).

---

## 5. Rust Implementation Plan

### 5.1 New files

#### `src/config/theme.rs`

Defines all typed config structs. Every field has a `#[serde(default = "...")]` annotation so a minimal config is fully valid.

```rust
use serde::{Deserialize, Serialize};

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
    pub modes: ModesSettings,   // existing, just re-routed
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_theme_name")]
    pub theme: String, // "dark" | "light" | "inline"
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutConfig {
    #[serde(default = "default_margin")]
    pub margin: [u16; 4],          // top, right, bottom, left
    #[serde(default = "default_navbar_height")]
    pub navbar_height: u16,        // in rows
    #[serde(default = "default_search_height")]
    pub search_height: u16,
    #[serde(default = "default_results_min")]
    pub results_min: u16,
    #[serde(default = "default_navbar_pos")]
    pub navbar_position: NavbarPosition,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NavbarPosition { #[default] Top, Bottom }

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorderConfig {
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_border_style")]
    pub style: BorderStyle,
    #[serde(default)]
    pub color: Option<String>,  // hex or named
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BorderStyle { #[default] Rounded, Plain, Double, Thick, None }

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextModifier { Bold, Italic, Dim, Underlined, Reversed, CrossedOut, SlowBlink }

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
    pub item_padding: [u16; 2],    // [vertical, horizontal]
    #[serde(default)]
    pub border: BorderConfig,
    #[serde(default)]
    pub style: PanelStyle,
    #[serde(default)]
    pub selected: SelectedItemStyle,
    #[serde(default)]
    pub scrollbar: ScrollbarConfig,
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScrollbarConfig {
    #[serde(default = "default_thumb")]
    pub thumb_symbol: String,
    #[serde(default = "default_track")]
    pub track_symbol: String,
    #[serde(default)]
    pub color: Option<String>,
}
```

#### `src/ui/style_resolver.rs`

Translates `AppConfig` → ratatui primitives. This is a **pure function layer** — no side effects, no widget rendering.

```rust
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType};
use crate::config::theme::{BorderConfig, BorderStyle, PanelStyle, TextModifier};

/// Parse a hex string "#rrggbb" or named color into ratatui Color
pub fn parse_color(s: &str) -> Color {
    let s = s.trim();
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
        return Color::Rgb(r, g, b);
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

pub fn resolve_borders(border: &BorderConfig) -> Borders {
    if !border.visible { Borders::NONE } else { Borders::ALL }
}

pub fn resolve_border_type(border: &BorderConfig) -> BorderType {
    match border.style {
        BorderStyle::Plain   => BorderType::Plain,
        BorderStyle::Rounded => BorderType::Rounded,
        BorderStyle::Double  => BorderType::Double,
        BorderStyle::Thick   => BorderType::Thick,
        BorderStyle::None    => BorderType::Plain,
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
```

### 5.2 Files modified

#### `src/config/settings.rs`

- Remove `UserSettings` (renamed to `AppConfig` in `theme.rs`), or keep `ModesSettings` inside `AppConfig`.
- Extend `load_user_settings()` to return `AppConfig`.
- Add preset theme loading: if `config.general.theme != "inline"`, try loading `~/.config/latui/themes/<name>.toml` first, then fall back to bundled `assets/themes/<name>.toml` (embedded with `include_str!`).
- Implement **cascade merge**: preset values → user overrides. A custom `merge` method on each config struct handles this cleanly without re-inventing serde.

```rust
// Pseudo-code for cascade merge
impl NavbarConfig {
    pub fn merge_over(self, base: NavbarConfig) -> NavbarConfig {
        NavbarConfig {
            title: if self.title != default_navbar_title() { self.title } else { base.title },
            // ...
        }
    }
}
```

> **Alternative:** Use `Option<T>` for every field so `None` means "inherit from preset." Slightly more complex typing but cleaner merge semantics. Recommend this for production.

#### `src/app/state.rs`

```rust
use crate::config::theme::AppConfig;

pub struct AppState {
    // existing fields...
    pub config: AppConfig,    // replaces `pub theme: Theme`
}
```

`AppConfig` is passed by reference into `renderer::draw`. No cloning on each frame.

#### `src/ui/renderer.rs`

Full refactor of `render_mode_tabs`, `render_search_input`, `render_results_list`, and `render_apps_results_list_with_inline_icons`:

- Replace every `Style::default().fg(Color::Yellow)` etc. with calls to `style_resolver::*`.
- Replace `Block::default().borders(Borders::ALL).title(...)` with `style_resolver::resolve_block(...)`.
- Replace `highlight_symbol(">> ")` with `app.config.results.selected.symbol.as_str()`.
- Apply `LayoutConfig` margin/heights to the `Layout::default().constraints(...)` call.
- Add `Scrollbar` widget render driven by `results.show_scrollbar`.
- Handle `navbar_position = "bottom"` by reversing the layout constraint order.

#### `src/main.rs`

```rust
let config = load_user_config().unwrap_or_default();
app.config = config;
```

#### `assets/themes/` (new directory)

Ship two bundled presets, embedded at compile time:

```rust
// src/ui/bundled_themes.rs
pub const DARK: &str   = include_str!("../../assets/themes/dark.toml");
pub const LIGHT: &str  = include_str!("../../assets/themes/light.toml");
```

---

## 6. Theme Cascade / Override Semantics

This is the key UX principle (mirroring how Rofi's `@theme` import + property overrides work):

```
Priority  (highest → lowest)
──────────────────────────────────────────────────────────
1. Inline [navbar.*], [search.*], [results.*] in config.toml
2. ~/.config/latui/themes/<theme-name>.toml  (user's saved theme)
3. <binary-embedded>/assets/themes/<theme-name>.toml  (bundled preset)
4. Compiled Rust defaults (Default impls)
```

Concretely: a user can do `theme = "catppuccin"` to apply a community theme, and then put `[navbar.style] background = "#ff0000"` in their `config.toml` to override just that one value, with everything else coming from the catppuccin theme file.

**Implementation choice:** Model every leaf _color_ and _string_ config value as `Option<T>`. `None` = "inherit from lower priority." This is clean and composable.

---

## 7. Complete Config Reference (User-Facing)

This is what the user sees in the docs. A well-commented example `config.toml`:

```toml
# ~/.config/latui/config.toml
# Full reference with all supported options and their defaults.

[general]
default_mode = "apps"      # apps | run | files | clipboard | emojis
max_results  = 50
theme        = "dark"      # "dark" | "light" | "inline" | path to a .toml theme file

# ─── Layout ───────────────────────────────────────────────────────────────────
[layout]
margin           = [0, 0, 0, 0]   # outer spacing: [top, right, bottom, left] in terminal cells
navbar_height    = 3
search_height    = 3
results_min      = 1
navbar_position  = "top"          # "top" | "bottom"

# ─── Navbar ───────────────────────────────────────────────────────────────────
[navbar]
title            = "LaTUI"
title_alignment  = "left"         # "left" | "center" | "right"
visible          = true

[navbar.border]
visible  = true
style    = "rounded"              # "plain" | "rounded" | "double" | "thick" | "none"
color    = "#3b4261"

[navbar.style]
background = "#1a1b26"
foreground = "#c0caf5"

[navbar.tabs]
active_fg         = "#7aa2f7"
active_bg         = ""
active_modifier   = ["bold"]
inactive_fg       = "#565f89"
inactive_bg       = ""
show_icons        = true

# ─── Search Bar ───────────────────────────────────────────────────────────────
[search]
visible        = true
prompt_symbol  = "> "
placeholder    = "Type to search..."

[search.border]
visible  = true
style    = "rounded"
color    = "#3b4261"

[search.style]
background = "#1a1b26"
foreground = "#c0caf5"
prompt_fg  = "#7aa2f7"

# ─── Results List ─────────────────────────────────────────────────────────────
[results]
visible        = true
title          = "Results ({count})"
item_display   = "icon_name_desc"   # "name" | "name_desc" | "icon_name" | "icon_name_desc"
icon_visible   = true
show_scrollbar = true
item_padding   = [0, 1]             # [vertical, horizontal]

[results.border]
visible  = true
style    = "rounded"
color    = "#3b4261"

[results.style]
background = "#1a1b26"
foreground = "#c0caf5"

[results.selected]
background     = "#283457"
foreground     = "#c0caf5"
modifier       = ["bold"]
symbol         = ">> "
symbol_visible = true

[results.scrollbar]
thumb_symbol = "█"
track_symbol = "░"
color        = "#565f89"

# ─── Modes ────────────────────────────────────────────────────────────────────
[modes.apps]
desktop_dirs = ["~/.local/share/applications", "/usr/share/applications"]
skip_terminal_apps = false
icons.enabled = true
icons.theme   = ""       # empty = system default
icons.size    = 24
```

---

## 8. Preset Theme Files

Two bundled presets ship with the binary via `include_str!`:

### `assets/themes/dark.toml` (Tokyo Night inspired)

```toml
[navbar.style]
background = "#1a1b26"
foreground = "#c0caf5"

[navbar.border]
style = "rounded"
color = "#3b4261"

[navbar.tabs]
active_fg       = "#7aa2f7"
active_modifier = ["bold"]
inactive_fg     = "#565f89"

[search.style]
background = "#1a1b26"
foreground = "#c0caf5"
prompt_fg  = "#7aa2f7"

[search.border]
style = "rounded"
color = "#3b4261"

[results.style]
background = "#1a1b26"
foreground = "#c0caf5"

[results.border]
style = "rounded"
color = "#3b4261"

[results.selected]
background = "#283457"
foreground = "#c0caf5"
modifier   = ["bold"]
symbol     = ">> "
```

### `assets/themes/light.toml`

```toml
[navbar.style]
background = "#d5d6db"
foreground = "#343b58"

[navbar.border]
style = "rounded"
color = "#9699a3"

[navbar.tabs]
active_fg       = "#2959aa"
active_modifier = ["bold"]
inactive_fg     = "#6c7086"

[search.style]
background = "#d5d6db"
foreground = "#343b58"
prompt_fg  = "#2959aa"

[search.border]
style = "rounded"
color = "#9699a3"

[results.style]
background = "#d5d6db"
foreground = "#343b58"

[results.selected]
background = "#b4c2f0"
foreground = "#343b58"
modifier   = ["bold"]
symbol     = ">> "
```

---

## 9. Config Validation & Error Handling

To meet production-grade standards, the loader must **never silently corrupt state**:

```rust
pub enum ConfigLoadResult {
    Ok(AppConfig),
    WithWarnings(AppConfig, Vec<String>),  // partial parse: use defaults + log warnings
    Fatal(String),                          // completely unreadable file
}
```

- Invalid hex color → warn, use `Color::Reset`.
- Unknown `border.style` value → warn, use `Rounded`.
- Missing theme file referenced by `theme = "foo"` → warn, fall back to `"dark"` preset.
- Malformed TOML → `Fatal`, app starts with all defaults, logs the error path.

The user should **never see a crash due to a config error**. LaTUI degrades gracefully.

---

## 10. CSS vs TOML: The TUI Reality

The user asked if CSS is the only way to achieve Rofi-level theming. Here is the honest answer:

**In a terminal (TUI), CSS does not apply.** Terminals render character cells with foreground color, background color, and text modifiers (bold, italic, etc.). There is no box model, no CSS cascade, no layout engine that understands `padding: 4px`.

What we can do — and this plan does — is **model the same concepts in TOML**, mapped to ratatui primitives:

| CSS concept | TOML equivalent in this plan |
|---|---|
| `color: #hex` | `foreground = "#hex"` |
| `background-color` | `background = "#hex"` |
| `border: 1px solid #hex` | `[*.border] visible=true color="#hex"` |
| `border-radius` | `[*.border] style="rounded"` |
| `padding: 0 4px` | `item_padding = [0, 1]` (in cells) |
| `font-weight: bold` | `modifier = ["bold"]` |
| `display: none` | `visible = false` |
| `@theme` import + overrides | Cascade: preset + inline overrides |
| `::selection` pseudo-element | `[results.selected]` block |
| `::before content: ">>"` | `selected.symbol = ">> "` |

The result is functionally equivalent to Rofi's rasi theming, expressed in an idiomatic TUI/Rust/TOML way.

> **Future option (post-v2.0):** A `latui.css`-like syntax could be parsed into the same `AppConfig` structs using a custom parser. This would allow users familiar with CSS/Rofi to write `navbar { background: #1a1b26; border-style: rounded; }`. This is purely a syntax sugar layer on top of the same underlying types — it does not add any new terminal capabilities.

---

## 11. Implementation Phases

### Phase A — Foundation (required for all else)

1. Create `src/config/theme.rs` with full typed structs and `Default` impls.
2. Extend `src/config/settings.rs` → `load_user_config() -> AppConfig`.
3. Add preset cascade loader (embedded `dark.toml` + `light.toml`).
4. Update `AppState`: replace `theme: Theme` with `config: AppConfig`.
5. Update `main.rs`: load config, inject into `AppState`.

### Phase B — Style Resolver

6. Create `src/ui/style_resolver.rs` with `parse_color`, `resolve_block`, `resolve_modifier`, `resolve_style`.
7. Write unit tests for `parse_color` (hex parsing, named colors, invalid input).

### Phase C — Renderer Refactor

8. Refactor `render_mode_tabs` → consume `config.navbar`.
9. Refactor `render_search_input` → consume `config.search`.
10. Refactor `render_results_list` → consume `config.results`.
11. Refactor `render_apps_results_list_with_inline_icons` → consume `config.results`.
12. Handle `layout.margin` via `Layout::default().horizontal_margin().vertical_margin()` or outer `Block` padding.
13. Handle `navbar_position = "bottom"` layout swap.
14. Add `Scrollbar` widget render (ratatui has a built-in `Scrollbar` widget).

### Phase D — `item_display` & icon visibility

15. Implement `ItemDisplay` enum logic in item formatting (replaces the `match (&i.icon, &i.description)` branches).
16. Implement `icon_visible = false` path (skip icon column entirely).

### Phase E — Validation & UX polish

17. Implement `ConfigLoadResult` with warnings.
18. Log all config warnings via `tracing::warn!`.
19. Add `latui --print-config` subcommand to dump the current resolved `AppConfig` as TOML (invaluable for debugging user configs).
20. Add `latui --theme-list` to list available themes.

### Phase F — Documentation & community

21. Generate `example-config.toml` with every option documented inline.
22. Write `docs/theming.md` — the user-facing guide.
23. Create `assets/themes/catppuccin-mocha.toml` as a third community-contributed example.
24. Update `README.md` with a "Theming" section.

---

## 12. File Map Summary

```
src/
├── config/
│   ├── theme.rs          [NEW]   Full AppConfig type tree
│   ├── settings.rs       [MOD]   load_user_config(), cascade logic
│   ├── loader.rs         [MOD]   minor: load embedded theme by name
│   ├── keywords.rs       [--]    unchanged
│   └── mod.rs            [MOD]   pub mod theme;
├── ui/
│   ├── style_resolver.rs [NEW]   parse_color, resolve_block, resolve_style, resolve_modifier
│   ├── renderer.rs       [MOD]   consume AppConfig via style_resolver, no hardcoded styles
│   ├── theme.rs          [DEL]   stub → removed, replaced by config/theme.rs
│   └── mod.rs            [MOD]   pub mod style_resolver;
├── app/
│   ├── state.rs          [MOD]   replace `theme: Theme` with `config: AppConfig`
│   └── controller.rs     [--]    unchanged
└── main.rs               [MOD]   load AppConfig, inject into AppState

assets/
└── themes/
    ├── dark.toml         [NEW]   bundled preset
    ├── light.toml        [NEW]   bundled preset
    └── catppuccin-mocha.toml [NEW] community example

docs/
├── theming.md            [NEW]   user-facing theming guide
└── customizability-plan.md       this document

tests/
└── config_tests.rs       [NEW]   parse_color, config defaults, cascade merge
```

---

## 13. What This Does NOT Do (Scope Boundaries)

- **Dynamic reload at runtime** (inotify-watch on config file) — out of scope for v2.0. Can be added later via a background thread + channel.
- **GUI theme editor** — out of scope. LaTUI is a TUI app; theme editing is done in a text editor.
- **Per-mode theming** — e.g., different colors in Apps vs Files mode — out of scope for v2.0. The architecture supports it (just add `[modes.apps.results.*]` blocks) but the implementation cost is high. Can be layered in v2.1.
- **CSS parser** — out of scope for v2.0. The TOML schema already achieves the same expressiveness.
- **Transparency / compositor effects** — these depend entirely on the terminal emulator (Kitty, Alacritty, etc.) and cannot be controlled from within a ratatui app. Users configure this in their terminal emulator config.
- **Animations / transitions** — ratatui renders full frames; smooth CSS-like transitions are not possible in a character-cell terminal.

---

## 14. Open Questions for Contributors

1. **Merge semantics:** `Option<T>` per field (cleanest cascade, more boilerplate) vs. `#[serde(default)]` with `PartialEq`-against-default comparison (simpler types, trickier logic). Recommend `Option<T>`.
2. **`include_str!` vs runtime file embedding:** Bundled themes via `include_str!` keep the binary self-contained (no system files needed). Alternatively, install themes to `/usr/share/latui/themes/` in the PKGBUILD. Both can coexist.
3. **Config file location:** Currently `~/.config/latui/config.toml` (XDG config). Should `theme = "path/to/file.toml"` also accept absolute paths for portable dotfiles? Recommended: yes.
4. **`--print-config` output:** Human-readable annotated TOML vs machine-readable JSON? Recommend TOML for consistency with the rest of the project.
