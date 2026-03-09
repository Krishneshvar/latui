use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    Launch(String),
    Command(String),
}
