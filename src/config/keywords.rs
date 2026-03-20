use std::collections::HashMap;
use serde::Deserialize;
use crate::error::{LatuiError, ConfigError};

/// Manages semantic keyword mappings (e.g., "browser" -> ["firefox", "chrome"])
#[derive(Debug, Default)]
pub struct KeywordMapper {
    mappings: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct KeywordConfig {
    keywords: HashMap<String, Vec<String>>,
}

impl KeywordMapper {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Load default keywords from embedded TOML
    pub fn with_defaults() -> Self {
        let content = include_str!("keywords.toml");
        Self::from_toml(content).unwrap_or_else(|e| {
            tracing::error!("Failed to load bundled keywords: {}", e);
            Self::new()
        })
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
    pub fn from_toml(content: &str) -> Result<Self, LatuiError> {
        let mut mapper = Self::new();
        let parsed: KeywordConfig = toml::from_str(content)
            .map_err(|e| LatuiError::Config(ConfigError::Keywords(e)))?;
            
        for (k, v) in parsed.keywords {
            mapper.add_mapping(k, v);
        }
        Ok(mapper)
    }
}
