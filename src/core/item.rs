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

impl Item {
    pub fn new(id: impl Into<String>, title: impl Into<String>, search_text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            search_text: search_text.into(),
            description: None,
            icon: None,
            metadata: None,
        }
    }
}
