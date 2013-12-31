pub trait Component {
    fn set_id(&mut self, id: int);
    fn get_id(&self) -> int;
    fn get_name(&self) -> &str;
}

pub struct ComponentManager<'a> {
    priv components: ~[&'a mut Component],
    priv last_id: int,
}

impl<'a> ComponentManager<'a> {
    pub fn new() -> ComponentManager<'a> {
        ComponentManager {
            components: ~[],
            last_id: -1,
        }
    }
    pub fn add(&mut self, component: &'a mut Component) {
        self.last_id += 1;
        component.set_id(self.last_id);
        println!("new component add: {} ({})", component.get_name(), component.get_id());
        self.components.push(component);
    }
}
