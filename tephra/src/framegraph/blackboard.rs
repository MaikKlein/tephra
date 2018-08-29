use anymap::AnyMap;
pub struct Blackboard {
    any_map: AnyMap,
}
impl Blackboard {
    pub fn new() -> Blackboard {
        Blackboard {
            any_map: AnyMap::new(),
        }
    }
    pub fn add<T: 'static>(&mut self, t: T) {
        self.any_map.insert(t);
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.any_map.get::<T>()
    }
}
