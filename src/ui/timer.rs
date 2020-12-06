use core::sync::atomic::AtomicU8;

pub struct Timer {
    pub number: AtomicU8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            number: AtomicU8::new(0),
        }
    }
}