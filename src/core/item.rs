use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,

    pub search_text: String,

    pub description: Option<String>,

    #[serde(default)]
    pub icon: Option<String>,

    /// Mode-specific metadata needed for execution
    pub metadata: Option<String>,
}
