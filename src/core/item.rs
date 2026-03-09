use crate::core::action::Action;

#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub score: i64,
    pub action: Action,
}
