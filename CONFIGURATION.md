# Configuration and Theming Guide

LaTUI is designed to be highly customizable. By default, it looks for its configuration file at `~/.config/latui/config.toml`. The application utilizes a powerful "theme cascade"—allowing you to load bundled themes (like `dark` or `light`), user-defined theme files, or inline overrides.

This guide details the complete structure of `config.toml`.

## Table of Contents
1. [General Settings](#general-settings)
2. [Layout Configuration](#layout-configuration)
3. [Navbar Settings](#navbar-settings)
4. [Search Settings](#search-settings)
5. [Results Settings](#results-settings)
6. [Modes Configuration](#modes-configuration)
7. [Theming System](#theming-system)

---

## General Settings
The `[general]` block defines global application behavior.

```toml
[general]
default_mode = "apps"    # The mode LaTUI starts in (e.g., "apps", "files", "run")
max_results = 50         # The maximum number of search results to display
theme = "dark"           # The base theme to load ("dark", "light", or a custom path/name)
full_background = None   # Optional hex color for the terminal's global background
```

## Layout Configuration
The `[layout]` block controls spacing and the visual hierarchy of LaTUI.

```toml
[layout]
margin = [0, 0, 0, 0]      # Top, Right, Bottom, Left margins relative to the terminal edge
navbar_height = 3          # Height of the mode switching tabs block
search_height = 3          # Height of the input search block
results_min = 1            # Minimum height required to render the results panel
navbar_position = "top"    # Can be "top" or "bottom"
```

## Navbar Settings
The `[navbar]` block configures the mode tabs and the application title.

```toml
[navbar]
title = "LaTUI"
title_alignment = "left"   # "left", "center", or "right"
visible = true             # Toggle visibility of the entire navbar

[navbar.border]
visible = true
style = "rounded"          # "rounded", "plain", "double", "thick", "none"
color = "#7aa2f7"          # Optional border hex color

[navbar.style]
background = "#1a1b26"     # Optional background hex color
foreground = "#c0caf5"     # Optional text hex color

[navbar.tabs]
active_fg = "#7aa2f7"
active_bg = None
active_modifier = ["Bold"] # Text modifiers: "Bold", "Italic", "Dim", "Underlined", "Reversed"
inactive_fg = "#565f89"
show_icons = true
```

## Search Settings
The `[search]` block controls the text input field.

```toml
[search]
visible = true
prompt_symbol = "> "       # The character prefix before user input
placeholder = "Search..."  # Optional ghost text shown when input is empty
prompt_fg = "#9ece6a"      # Hex color specifically for the prompt symbol

# Includes nested [search.border] and [search.style] identically to navbar
```

## Results Settings
The `[results]` block styles the list of matching items and their previews.

```toml
[results]
visible = true
title = "Results ({count})" # Shown above the list. {count} is dynamically replaced
show_count = false
item_display = "icon_name_desc" # "name", "name_desc", "icon_name", "icon_name_desc"
icon_visible = true
show_scrollbar = true
item_padding = [0, 1]       # Vertical, Horizontal padding for list items

[results.selected]
background = "#283457"
foreground = None
modifier = ["Bold"]
symbol = ">> "             # Prefix for the currently selected item
symbol_visible = true

[results.scrollbar]
thumb_symbol = "█"
track_symbol = "░"
```

## Modes Configuration
The `[modes]` block lets you customize specific backend strategies.

### Apps Mode
Configures how `.desktop` files and application launching behave.

```toml
[modes.apps]
desktop_dirs = ["~/.local/share/applications", "/usr/share/applications"]
include = []               # Explicitly include paths
exclude = []               # Exclude strings matching these names
skip_terminal_apps = false # Hide apps that require `Terminal=true`

[modes.apps.icons]
enabled = true
theme = "Papirus-Dark"     # System icon theme to look up fallback SVGs/PNGs
size = 24
scale = 1
prefer_svg = false
render_mode = "thumbnail"  # "thumbnail" (image protocol) or "icon_name" text
fallback = "📦"           # Fallback emoji if no icon is found
```

### Custom Modes
You can define your own menus and scripts easily.

```toml
[modes.custom.my_power_menu]
name = "Power"
icon = "⏻"
description = "Session management"
list_cmd = "echo -e 'Shutdown\nReboot\nSuspend'"
exec_cmd = "my-power-script {}"
stays_open = false
```

## Theming System
LaTUI resolves themes in the following order:
1. Try loading from `~/.config/latui/themes/<theme_name>.toml`
2. Try loading from an absolute path if provided.
3. Fallback to bundled themes (`"dark"`, `"light"`).

You can heavily override any property from the base theme by specifying it in your main `config.toml`. For example, setting `theme = "dark"` but providing a custom `[results.selected]` background will merge your override on top of the bundled dark theme.
