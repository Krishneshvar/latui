use crate::core::item::Item;

pub trait Mode {

    fn load(&mut self);

    fn search(&mut self, query: &str) -> Vec<Item>;

    fn execute(&mut self, item: &Item);
}
