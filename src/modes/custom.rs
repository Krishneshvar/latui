use crate::config::settings::CustomModeConfig;
use crate::core::item::Item;
use crate::core::mode::Mode;
use crate::core::searchable_item::SearchableItem;
use crate::error::LatuiError;
use crate::search::engine::SearchEngine;

use serde::Deserialize;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct CustomItemDef {
    id: Option<String>,
    title: String,
    description: Option<String>,
    icon: Option<String>,
    metadata: Option<String>,
}

#[derive(Debug)]
pub struct CustomMode {
    pub id: String,
    pub config: CustomModeConfig,
    items: Vec<SearchableItem>,
    search_engine: SearchEngine,
    last_action_time: Option<Instant>,
}

impl CustomMode {
    pub fn new(id: String, config: CustomModeConfig) -> Self {
        Self {
            id,
            config,
            items: Vec::new(),
            search_engine: SearchEngine::new(),
            last_action_time: None,
        }
    }

    fn fetch_items(&mut self) -> Result<(), LatuiError> {
        if self.config.list_cmd.trim().is_empty() {
            return Ok(());
        }

        tracing::debug!("Fetching items for custom mode '{}' via: {}", self.id, self.config.list_cmd);
        
        // Use the shell to execute the command to allow full flexibility (pipes, env vars, etc.)
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        
        let output = Command::new(&shell)
            .arg("-c")
            .arg(&self.config.list_cmd)
            .output()
            .map_err(LatuiError::Io)?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Custom mode list_cmd failed: {}", err_msg);
            return Err(LatuiError::App(format!("Script failed: {}", err_msg)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let content = stdout.trim();

        if content.is_empty() {
            self.items.clear();
            return Ok(());
        }

        // Try JSON parsing first
        let parsed_items: Vec<Item> = if content.starts_with('[') {
            match serde_json::from_str::<Vec<CustomItemDef>>(content) {
                Ok(defs) => defs
                    .into_iter()
                    .enumerate()
                    .map(|(idx, def)| {
                        Item {
                            id: def.id.unwrap_or_else(|| format!("custom_{}_{}", self.id, idx)),
                            title: def.title.clone(),
                            search_text: def.title.to_lowercase(),
                            description: def.description,
                            icon: def.icon,
                            metadata: def.metadata,
                        }
                    })
                    .collect(),
                Err(e) => {
                    tracing::warn!("Failed to parse JSON for mode '{}': {}", self.id, e);
                    // Fallback to text lines if JSON fails despite starting with [
                    self.parse_text_lines(content)
                }
            }
        } else {
            self.parse_text_lines(content)
        };

        // Build searchable items
        self.items = parsed_items
            .into_iter()
            .map(|item| {
                SearchableItem::new(item.clone())
                    .with_field("title", &item.title, 10.0)
                    .with_field(
                        "description",
                        item.description.as_deref().unwrap_or(""),
                        5.0,
                    )
            })
            .collect();

        tracing::info!("Loaded {} items for custom mode '{}'", self.items.len(), self.id);

        Ok(())
    }

    fn parse_text_lines(&self, content: &str) -> Vec<Item> {
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(idx, line)| {
                Item {
                    id: format!("custom_{}_{}", self.id, idx),
                    title: line.to_string(),
                    search_text: line.to_lowercase(),
                    description: None,
                    icon: None,
                    metadata: Some(line.to_string()),
                }
            })
            .collect()
    }
}

impl Mode for CustomMode {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn icon(&self) -> &str {
        &self.config.icon
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn load(&mut self) -> Result<(), LatuiError> {
        self.fetch_items()
    }

    fn search(&mut self, query: &str) -> Vec<Item> {
        let q = query.trim();
        
        if q.is_empty() {
            // Return all items if no query
            return self.items.iter().map(|s| s.item.clone()).collect();
        }

        let mut results = self.search_engine.search_scored(q, &self.items);
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter().map(|(item, _)| item).collect()
    }

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Rate limiting
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500)
        {
            tracing::warn!("Rate limiting execution for: {}", item.title);
            return Ok(());
        }
        self.last_action_time = Some(Instant::now());

        if self.config.exec_cmd.trim().is_empty() {
            tracing::warn!("No exec_cmd configured for mode '{}'", self.id);
            return Ok(());
        }

        tracing::info!("Executing custom action for mode '{}': {}", self.id, item.title);

        let mut env_vars = vec![
            ("LATUI_ITEM_ID", item.id.as_str()),
            ("LATUI_ITEM_TITLE", item.title.as_str()),
        ];
        
        if let Some(desc) = &item.description {
            env_vars.push(("LATUI_ITEM_DESC", desc.as_str()));
        }
        
        if let Some(meta) = &item.metadata {
            env_vars.push(("LATUI_ITEM_METADATA", meta.as_str()));
        }

        crate::core::execution::ExecutionEngine::spawn_shell(&self.config.exec_cmd, &env_vars)
    }

    fn record_selection(&mut self, _query: &str, _item: &Item) {
        // Optional: Could integrate with frequency tracker here if needed
    }

    fn stays_open(&self) -> bool {
        self.config.stays_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_mode_json_parsing() {
        let config = CustomModeConfig {
            name: "Test".into(),
            icon: "X".into(),
            description: "Test".into(),
            list_cmd: "echo '[{\"id\":\"foo\", \"title\":\"Bar\"}]'".into(),
            exec_cmd: "".into(),
            stays_open: false,
        };
        let mut mode = CustomMode::new("test".into(), config);
        mode.load().unwrap();
        let items = mode.search("");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "foo");
        assert_eq!(items[0].title, "Bar");
    }

    #[test]
    fn test_custom_mode_text_fallback_parsing() {
        let config = CustomModeConfig {
            name: "Test".into(),
            icon: "X".into(),
            description: "Test".into(),
            list_cmd: "echo 'Line 1\nLine 2'".into(),
            exec_cmd: "".into(),
            stays_open: false,
        };
        let mut mode = CustomMode::new("test".into(), config);
        mode.load().unwrap();
        let items = mode.search("");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Line 1");
        assert_eq!(items[1].title, "Line 2");
    }
}
