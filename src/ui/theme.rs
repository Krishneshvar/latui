use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    // Add color fields as needed by the renderer
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "dark".to_string(),
        }
    }
}
