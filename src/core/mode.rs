use crate::core::item::Item;

pub trait Mode {

    fn name(&self) -> &str;

    fn load(&mut self);

    fn search(&mut self, query: &str) -> Vec<Item>;

    fn execute(&self, item: &Item);
}
