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

    pub fn get<'a, T: 'static>(&self) -> Option<&T> {
        self.any_map.get::<T>()
    }

    pub fn get_mut<'a, T: 'static>(&mut self) -> Option<&mut T> {
        self.any_map.get_mut::<T>()
    }
}
