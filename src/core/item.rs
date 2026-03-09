use serde::{Serialize, Deserialize};

use crate::core::action::Action;

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,

    pub search_text: String,

    pub description: Option<String>,

    pub action: Action,
}
