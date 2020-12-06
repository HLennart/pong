

// Options:
// Menu, running, stopped,
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum StateVariant {
    Menu,
    Running,
    Stopped,
    None,
}

impl Default for StateVariant {
    fn default() -> Self {
        Self::None
    }
}

