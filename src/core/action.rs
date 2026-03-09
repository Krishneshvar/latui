#[derive(Clone)]
pub enum Action {
    Launch(String),
    Command(String),
    None,
}
