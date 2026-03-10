use crate::core::item::Item;

pub trait Mode {

    fn name(&self) -> &str;

    fn load(&mut self);

    fn search(&self, query: &str) -> Vec<Item>;

    fn execute(&self, item: &Item);
}
