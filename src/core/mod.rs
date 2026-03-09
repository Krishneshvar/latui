use crate::core::item::Item;

pub trait Mode {
    fn name(&self) -> &str;

    fn load(&mut self);

    fn items(&self) -> Vec<Item>;

    fn run(&self, item: &Item);
}
