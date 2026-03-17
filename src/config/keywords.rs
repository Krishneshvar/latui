use std::collections::HashMap;

/// Manages semantic keyword mappings (e.g., "browser" -> ["firefox", "chrome"])
pub struct KeywordMapper {
    mappings: HashMap<String, Vec<String>>,
}

impl Default for KeywordMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl KeywordMapper {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Load default keywords from embedded TOML
    pub fn with_defaults() -> Self {
        let mut mapper = Self::new();
        mapper.add_mapping("browser".to_string(), vec!["firefox".to_string(), "chrome".to_string(), "brave".to_string(), "chromium".to_string(), "opera".to_string()]);
        mapper.add_mapping("editor".to_string(), vec!["nvim".to_string(), "code".to_string(), "vscode".to_string(), "sublime".to_string(), "emacs".to_string(), "vim".to_string()]);
        mapper.add_mapping("terminal".to_string(), vec!["kitty".to_string(), "alacritty".to_string(), "foot".to_string(), "gnome-terminal".to_string(), "konsole".to_string(), "wezterm".to_string()]);
        mapper.add_mapping("file".to_string(), vec!["thunar".to_string(), "nautilus".to_string(), "dolphin".to_string(), "ranger".to_string(), "nnn".to_string()]);
        mapper
    }



    /// Get all apps matching a keyword
    pub fn get_matches(&self, keyword: &str) -> Option<&Vec<String>> {
        self.mappings.get(keyword)
    }

    /// Add a keyword mapping
    pub fn add_mapping(&mut self, keyword: String, apps: Vec<String>) {
        self.mappings.insert(keyword, apps);
    }

    /// Load from a TOML string
    pub fn from_toml(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut mapper = Self::new();
        let parsed: HashMap<String, Vec<String>> = toml::from_str(content)?;
        for (k, v) in parsed {
            mapper.add_mapping(k, v);
        }
        Ok(mapper)
    }
}
