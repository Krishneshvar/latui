use std::collections::HashMap;

/// Manages semantic keyword mappings (e.g., "browser" -> ["firefox", "chrome"])
pub struct KeywordMapper {
    mappings: HashMap<String, Vec<String>>,
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
        // TODO: Parse embedded keywords.toml
        mapper
    }

    /// Check if a keyword matches an app name
    pub fn matches(&self, keyword: &str, app_name: &str) -> bool {
        if let Some(apps) = self.mappings.get(keyword) {
            let app_lower = app_name.to_lowercase();
            apps.iter().any(|a| app_lower.contains(a))
        } else {
            false
        }
    }

    /// Get all apps matching a keyword
    pub fn get_matches(&self, keyword: &str) -> Option<&Vec<String>> {
        self.mappings.get(keyword)
    }

    /// Add a keyword mapping
    pub fn add_mapping(&mut self, keyword: String, apps: Vec<String>) {
        self.mappings.insert(keyword, apps);
    }
}
