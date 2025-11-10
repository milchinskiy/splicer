pub mod pane;
pub mod peer;
pub mod session;
pub mod window;
pub mod state;

#[derive(Default)]
pub struct IdAllocator {
    next: u64,
}
impl IdAllocator {
    pub fn allocate<T>(&mut self, ctor: fn(u64) -> Option<T>) -> T {
        loop {
            self.next += 1;
            if let Some(id) = ctor(self.next) {
                return id;
            }
        }
    }
}
