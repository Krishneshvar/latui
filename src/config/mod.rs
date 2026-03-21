//! Configuration parsing, theming, and user settings.
//!
//! The `config` module is responsible for loading the `config.toml`,
//! managing layout settings (`margin`, `navbar_height`), and applying aesthetics
//! via the theme system.
//!
//! - `settings`: Primary structures defining the configuration schema.
//! - `theme`: Defines styling parameters and layout bounds.
//! - `keywords`: Contains logic to map terms like "browser" to actual apps like "firefox".

pub mod keywords;
pub mod loader;
pub mod settings;
pub mod theme;
